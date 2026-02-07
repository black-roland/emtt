// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use log::{debug, warn};
use reqwest::{Client, header};
use serde_json::to_string;

use crate::{fl, MessageData};

pub async fn send_message(client: &Client, url: &str, data: &MessageData) {
    let json = match to_string(data) {
        Ok(j) => j,
        Err(e) => {
            let error_msg: String = e.to_string();
            warn!("{}", fl!("failed-to-render", error = error_msg));
            return;
        }
    };

    let json_for_log = json.clone();

    match client
        .post(url)
        .header(header::CONTENT_TYPE, "application/json")
        .body(json)
        .send()
        .await
    {
        Ok(_) => {
            debug!(
                "{}",
                fl!(
                    "forwarded-to-webhook",
                    from = data.from.clone(),  // Clone because `from` is String and macro may take by value
                    message = json_for_log
                )
            );
        }
        Err(err) => {
            let err_msg: String = err.to_string();
            warn!(
                "{}\n{}",
                fl!("failed-to-send-webhook", error = err_msg),
                fl!("message-content", content = json_for_log)
            );
        }
    }
}
