// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use teloxide::{prelude::*, Bot};

use crate::Config;

pub fn init_bot(config: &Config) -> Bot {
    Bot::new(config.bot_token.clone())
}

pub async fn send_message(
    bot: &Bot,
    chat_id: &str,
    message: &str,
) -> Result<(), teloxide::RequestError> {
    // bot.send_message(ChatId(chat_id.parse::<i64>().map_err(|_| teloxide::RequestError::InvalidJson { raw: "Invalid chat ID".to_string() })?), message)
    //     .await?;
    Ok(())
}
