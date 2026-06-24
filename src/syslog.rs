// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use anyhow::{Result, Context};
use log::{debug, info, trace, warn};
use regex::Regex;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio_graceful_shutdown::{SubsystemHandle, SubsystemBuilder};

use crate::Config;
use crate::MessageData;
use crate::fl;
use crate::lang;

use once_cell::sync::Lazy;

static NODEINFO_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Update changed=\d+ user (.+)/([^,/]+), id=0x([0-9a-fA-F]+), channel=\d+").unwrap()
});

static HANDLE_RECEIVED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^handleReceived\(([^)]+)\) \((.*)\)$").unwrap()
});

static TEXT_MSG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)Received text msg from=0x([0-9a-fA-F]+), id=0x([0-9a-fA-F]+), msg=(.+)").unwrap()
});

static RANGE_TEST_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^seq \d+$").unwrap()
});

#[derive(Clone)]
struct NodeInfo {
    shortname: String,
    longname: String,
}

#[derive(Clone)]
struct ViaInfo {
    to: u32,
    ch: u32,
    snr: Option<f32>,
    rssi: Option<i32>,
    hop_lim: Option<u32>,
    hop_start: Option<u32>,
    fr: Option<u32>,
    is_mqtt: bool,
    timestamp: u64,
}

struct HandleInfo {
    vias: HashMap<String, ViaInfo>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn parse_syslog_message(text: &str) -> Result<(String, String), &'static str> {
    let mut cursor: usize = 0;

    // Skip PRI and version if present
    if text.starts_with('<') {
        if let Some(pri_end) = text.find(' ') {
            cursor = pri_end + 1;
        } else {
            return Err("Invalid PRI/version format");
        }
    }

    // Skip timestamp or NILVALUE
    if let Some(ts_end) = text[cursor..].find(' ').map(|i| i + cursor) {
        cursor = ts_end + 1;
    } else {
        return Err("Missing timestamp");
    }

    // Parse ident
    if let Some(ident_end) = text[cursor..].find(' ').map(|i| i + cursor) {
        let ident = text[cursor..ident_end].to_string();
        cursor = ident_end + 1;

        // Skip extra fields to message separator
        if let Some(msg_start) = text[cursor..].find(':').map(|i| i + cursor) {
            cursor = msg_start + 1;
            if text.as_bytes().get(cursor) == Some(&b' ') {
                cursor += 1;
            }

            let message = text[cursor..].trim_end_matches('\n').to_string();
            return Ok((ident, message));
        } else {
            return Err("Missing message separator");
        }
    } else {
        return Err("Missing ident");
    }
}

async fn parse_and_store_nodeinfo(
    message: &str,
    known_nodes: &Arc<Mutex<HashMap<u32, NodeInfo>>>,
) -> bool {
    if let Some(caps) = NODEINFO_RE.captures(message) {
        let longname = caps[1].to_string();
        let shortname = caps[2].to_string();
        let id = match u32::from_str_radix(&caps[3], 16) {
            Ok(id) => id,
            Err(_) => return false,
        };

        let mut nodes = known_nodes.lock().await;
        nodes.insert(
            id,
            NodeInfo {
                shortname: shortname.clone(),
                longname: longname.clone(),
            },
        );

        debug!("{}", fl!("processed-nodeinfo", longname = longname, shortname = shortname, id = format!("0x{:08x}", id)));
        return true;
    }

    false
}

async fn parse_and_store_handle_received(
    message: &str,
    ident: &str,
    handle_infos: &Arc<Mutex<HashMap<u32, HandleInfo>>>,
) -> bool {
    if let Some(caps) = HANDLE_RECEIVED_RE.captures(message) {
        let mut content = caps[2].to_string();
        content = content
            .replace(',', " ")
            .replace(" = ", "=")
            .replace("via ", "via=");

        let mut fields: HashMap<String, String> = HashMap::new();
        for pair in content.split_whitespace() {
            if let Some((k, v)) = pair.split_once('=') {
                fields.insert(k.to_string(), v.to_string());
            }
        }

        if fields.get("Portnum").map(|s| s.as_str()) != Some("1") {
            return true; // Not text, but handled
        }

        let id_str = match fields.get("id") {
            Some(s) => s.clone(),
            None => return false,
        };

        let id = match u32::from_str_radix(&id_str[2..], 16) {
            Ok(id) => id,
            Err(_) => return false,
        };

        let fr = fields
            .get("fr")
            .and_then(|s| u32::from_str_radix(&s[2..], 16).ok());
        let to = fields
            .get("to")
            .and_then(|s| u32::from_str_radix(&s[2..], 16).ok());
        let ch = fields
            .get("Ch")
            .and_then(|s| u32::from_str_radix(&s[2..], 16).ok())
            .unwrap_or(0);
        let snr = fields.get("rxSNR").and_then(|s| s.parse::<f32>().ok());
        let rssi = fields.get("rxRSSI").and_then(|s| s.parse::<i32>().ok());
        let hop_lim = fields.get("HopLim").and_then(|s| s.parse::<u32>().ok());
        let hop_start = fields.get("hopStart").and_then(|s| s.parse::<u32>().ok());
        let via_str = fields.get("via").cloned().unwrap_or_default();

        let is_mqtt = via_str == "MQTT";

        let mut handles = handle_infos.lock().await;
        let entry = handles.entry(id).or_insert(HandleInfo {
            vias: HashMap::new(),
        });

        entry.vias.insert(
            ident.to_string(),
            ViaInfo {
                to: to.unwrap_or(0),
                ch,
                snr,
                rssi,
                hop_lim,
                hop_start,
                fr,
                is_mqtt,
                timestamp: now(),
            },
        );

        debug!("{}", fl!("stored-handle-info",
            id = format!("0x{:08x}", id),
            via = ident,
            ch = ch,
            to = format!("0x{:08x}", to.unwrap_or(0)),
            is_mqtt = lang::localize_bool(is_mqtt)
        ));
        return true;
    }

    false
}

async fn parse_and_process_text_message<F, Fut>(
    message: &str,
    ident: &str,
    config: &Config,
    sender: &F,
    handle_infos: &Arc<Mutex<HashMap<u32, HandleInfo>>>,
    known_nodes: &Arc<Mutex<HashMap<u32, NodeInfo>>>,
) -> bool
where
    F: Fn(MessageData) -> Fut,
    Fut: Future<Output = ()>,
{
    if let Some(caps) = TEXT_MSG_RE.captures(message) {
        let from = match u32::from_str_radix(&caps[1], 16) {
            Ok(f) => f,
            Err(_) => return false,
        };

        let id = match u32::from_str_radix(&caps[2], 16) {
            Ok(i) => i,
            Err(_) => return false,
        };

        let text = caps[3].to_string();
        let from_hex = format!("0x{:08x}", from);
        let formatted_id = format!("0x{:08x}", id);

        info!("{}", fl!("received-text-msg", from = from_hex.as_str(), id = formatted_id.as_str(), text = text.as_str()));

        if RANGE_TEST_RE.is_match(&text) {
            debug!("{}", fl!("ignoring-range-test", from = from_hex, id = format!("0x{:08x}", id)));
            return true;
        }

        let handles = handle_infos.lock().await;
        let h = match handles.get(&id) {
            Some(h) => h,
            None => {
                warn!("{}", fl!("no-handle-info", id = format!("0x{:08x}", id)));
                return true;
            }
        };

        let via_key = ident.to_string();
        let via_info = match h.vias.get(&via_key) {
            Some(v) => v.clone(),
            None => {
                warn!("{}", fl!("no-via-info", id = format!("0x{:08x}", id), via = ident));
                return true;
            }
        };
        drop(handles); // release lock early

        if via_info.is_mqtt {
            debug!("{}", fl!("skipping-mqtt", id = format!("0x{:08x}", id)));
            return true;
        }

        let forward = if via_info.to == 0xffffffff {
            if let Some(ch) = config.channel {
                via_info.ch == ch
            } else {
                false
            }
        } else {
            config.dm && via_info.ch == 0
        };

        if !forward {
            info!("{}", fl!("ignoring-text-msg", id = format!("0x{:08x}", id), ch = via_info.ch, to = format!("0x{:08x}", via_info.to)));
            return true;
        }

        let snr = via_info.snr;
        let rssi = via_info.rssi;
        let hops_away = via_info.hop_start.zip(via_info.hop_lim).map(|(hs, hl)| hs.saturating_sub(hl) as i32);

        // Format and send to Telegram
        let mut from_name = known_nodes
            .lock()
            .await
            .get(&from)
            .map(|n| n.longname.clone())
            .unwrap_or(from_hex.clone());

        if from == 0 {
            let parts: Vec<&str> = ident.split('_').collect();
            let shortname = if parts.len() == 2 { parts[0] } else { "Unknown" };
            from_name = format!("{} (Local)", shortname);
        }

        let data = MessageData {
            from: from_name,
            via: ident.to_string(),
            text,
            snr,
            rssi,
            hops_away,
        };

        sender(data).await;
        return true;
    }

    false
}

async fn cleanup_subsystem(
    subsys: SubsystemHandle,
    handle_infos: Arc<Mutex<HashMap<u32, HandleInfo>>>,
) -> Result<()> {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                let current_time = now();
                let mut handles = handle_infos.lock().await;
                let mut to_remove: Vec<u32> = Vec::new();
                for (id, handle) in handles.iter_mut() {
                    handle.vias.retain(|_, via| current_time - via.timestamp <= 180);
                    if handle.vias.is_empty() {
                        to_remove.push(*id);
                    }
                }
                for id in to_remove {
                    handles.remove(&id);
                    trace!("{}", fl!("cleaned-stale-handle-info", id = format!("0x{:08x}", id)));
                }
            }
            _ = subsys.on_shutdown_requested() => {
                debug!("{}", fl!("cleanup-shutdown"));
                break;
            }
        }
    }
    Ok(())
}

pub async fn run_server<F>(
    subsys: SubsystemHandle,
    config: Config,
    sender: F,
) -> Result<()>
where
    F: Fn(MessageData) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    let known_nodes: Arc<Mutex<HashMap<u32, NodeInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let handle_infos: Arc<Mutex<HashMap<u32, HandleInfo>>> = Arc::new(Mutex::new(HashMap::new()));

    let addr = format!("{}:{}", config.syslog_host, config.syslog_port);
    let socket = UdpSocket::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind UDP socket on {}", addr))?;

    info!("{}", fl!("syslog-binding", addr = addr));

    let handle_infos_clone = handle_infos.clone();
    subsys.start(SubsystemBuilder::new(
        "cleanup-task",
        move |s| cleanup_subsystem(s, handle_infos_clone),
    ));

    let mut buf = [0; 1024];
    loop {
        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                match res {
                    Ok((len, peer)) => {
                        let msg = match String::from_utf8(buf[..len].to_vec()) {
                            Ok(m) => m,
                            Err(_) => {
                                warn!("{}", fl!("invalid-utf8", peer = peer.to_string()));
                                continue;
                            }
                        };

                        let (ident, message) = match parse_syslog_message(&msg) {
                            Ok(r) => r,
                            Err(err) => {
                                warn!("{}", fl!("failed-to-parse-syslog", error = err, raw = msg));
                                continue;
                            }
                        };

                        if parse_and_store_nodeinfo(&message, &known_nodes).await {
                            continue;
                        }

                        if parse_and_store_handle_received(&message, &ident, &handle_infos).await {
                            continue;
                        }

                        if parse_and_process_text_message(
                            &message,
                            &ident,
                            &config,
                            &sender,
                            &handle_infos,
                            &known_nodes,
                        )
                        .await
                        {
                            continue;
                        }

                        trace!("{}", fl!("unhandled-syslog", message = message));
                    }
                    Err(e) => {
                        warn!("{}", fl!("recv-error", error = e.to_string()));
                    }
                }
            }
            _ = subsys.on_shutdown_requested() => {
                debug!("{}", fl!("shutdown-signal-received"));
                break;
            }
        }
    }

    Ok(())
}
