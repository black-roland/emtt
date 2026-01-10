// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use teloxide::{prelude::*, Bot};
use teloxide::types::ChatId;

use crate::Config;

pub fn init_bot(config: &Config) -> Bot {
    Bot::new(config.bot_token.clone())
}

pub async fn send_message(
    bot: &Bot,
    chat_id: i64,
    message: &str,
) -> Result<(), teloxide::RequestError> {
    bot.send_message(ChatId(chat_id), message).await?;
    Ok(())
}
