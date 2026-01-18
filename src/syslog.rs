// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use log::{debug, info, warn};
use regex::Regex;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use crate::Config;
use super::MessageData;

// Import the fl! macro from the crate root (which re-exports it from lang)
use crate::fl;

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
    timestamp: u64,
}

struct HandleInfo {
    is_mqtt: bool,
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
    handle_infos: &Arc<Mutex<HashMap<u32, HandleInfo>>>,
    known_nodes: &Arc<Mutex<HashMap<u32, NodeInfo>>>,
) -> bool {
    let re = Regex::new(r"Update changed=\d+ user (.+)/([^,/]+), id=0x([0-9a-fA-F]+), channel=\d+").unwrap();

    if let Some(caps) = re.captures(message) {
        let longname = caps[1].to_string();
        let shortname = caps[2].to_string();
        let id = match u32::from_str_radix(&caps[3], 16) {
            Ok(id) => id,
            Err(_) => return false,
        };

        let handles = handle_infos.lock().await;
        if let Some(h) = handles.get(&id) {
            if h.is_mqtt {
                debug!("Skipping MQTT-forwarded nodeinfo for node_id: 0x{:08x}", id);
                return true;
            }
        }
        drop(handles);

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
    let re = Regex::new(r"^handleReceived\(([^)]+)\) \((.*)\)$").unwrap();

    if let Some(caps) = re.captures(message) {
        // h_type = &caps[1]; not used
        let mut content = caps[2].to_string();
        content = content.replace(',', " ").replace(" = ", "=");

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
            is_mqtt: false,
            vias: HashMap::new(),
        });

        entry.is_mqtt = is_mqtt;
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
                timestamp: now(),
            },
        );

        debug!(
            "Stored handle info for text msg id: 0x{:08x}, via: {}, ch: {}, is_mqtt: {}",
            id, ident, ch, is_mqtt
        );
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
    let re = Regex::new(r"Received text msg from=0x([0-9a-fA-F]+), id=0x([0-9a-fA-F]+), msg=(.+)").unwrap();

    if let Some(caps) = re.captures(message) {
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

        let range_test_re = Regex::new(r"^seq \d+$").unwrap();
        if range_test_re.is_match(&text) {
            debug!("{}", fl!("ignoring-range-test", from = from_hex, id = format!("0x{:08x}", id)));
            return true;
        }

        let mut handles = handle_infos.lock().await;
        let h = match handles.get_mut(&id) {
            Some(h) => h,
            None => {
                warn!("{}", fl!("no-handle-info", id = format!("0x{:08x}", id)));
                return true;
            }
        };

        let via_key = ident.to_string();
        let via_info = match h.vias.remove(&via_key) {
            Some(v) => v,
            None => {
                warn!("{}", fl!("no-via-info", id = format!("0x{:08x}", id), via = ident));
                if h.vias.is_empty() {
                    handles.remove(&id);
                }
                return true;
            }
        };

        if now() - via_info.timestamp > 180 {
            warn!("{}", fl!("stale-handle-info", id = format!("0x{:08x}", id)));
            if h.vias.is_empty() {
                handles.remove(&id);
            }
            return true;
        }

        if h.is_mqtt {
            debug!("{}", fl!("skipping-mqtt", id = format!("0x{:08x}", id)));
            if h.vias.is_empty() {
                handles.remove(&id);
            }
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
            if h.vias.is_empty() {
                handles.remove(&id);
            }
            return true;
        }

        let snr = via_info.snr;
        let rssi = via_info.rssi;
        let hops_away = via_info.hop_start.zip(via_info.hop_lim).map(|(hs, hl)| hs.saturating_sub(hl) as i32);

        if h.vias.is_empty() {
            handles.remove(&id);
        }

        drop(handles);

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

pub async fn run_server<F>(config: &Config, sender: F)
where
    F: Fn(MessageData) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static,
{
    let known_nodes: Arc<Mutex<HashMap<u32, NodeInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let handle_infos: Arc<Mutex<HashMap<u32, HandleInfo>>> = Arc::new(Mutex::new(HashMap::new()));

    let addr = format!("{}:{}", config.syslog_host, config.syslog_port);
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to bind UDP socket on {}: {}", addr, e);
            return;
        }
    };

    info!("{}", fl!("syslog-binding", addr = addr));

    // Spawn periodic cleanup task
    let handle_infos_clone = handle_infos.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // Every 1 minute
            let current_time = now();
            let mut handles = handle_infos_clone.lock().await;
            let mut to_remove: Vec<u32> = Vec::new();
            for (id, handle) in handles.iter_mut() {
                handle.vias.retain(|_, via| current_time - via.timestamp <= 180);
                if handle.vias.is_empty() {
                    to_remove.push(*id);
                }
            }
            for id in to_remove {
                handles.remove(&id);
                debug!("Cleaned up stale handle info for id: 0x{:08x}", id);
            }
        }
    });

    let mut buf = [0; 1024];
    loop {
        let (len, peer) = match socket.recv_from(&mut buf).await {
            Ok((l, p)) => (l, p),
            Err(e) => {
                warn!("{}", fl!("recv-error", error = e.to_string()));
                continue;
            }
        };

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

        debug!(
            "Syslog message: ident: {}, ip: {}, message: {}",
            ident, peer, message
        );

        if parse_and_store_nodeinfo(&message, &handle_infos, &known_nodes).await {
            continue;
        }

        if parse_and_store_handle_received(&message, &ident, &handle_infos).await {
            continue;
        }

        if parse_and_process_text_message(
            &message,
            &ident,
            config,
            &sender,
            &handle_infos,
            &known_nodes,
        )
        .await
        {
            continue;
        }

        // If none matched, log verbose
        debug!("{}", fl!("unhandled-syslog", message = message));
    }
}
