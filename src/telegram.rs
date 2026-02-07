// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use teloxide::{prelude::*, types::{ChatId, ParseMode}};

pub fn init_bot(token: String) -> Bot {
    Bot::new(token)
}

pub async fn send_message(
    bot: &Bot,
    chat_id: i64,
    message: &str,
    parse_mode: Option<ParseMode>,
) -> Result<(), teloxide::RequestError> {
    let mut req = bot.send_message(ChatId(chat_id), message);
    if let Some(pm) = parse_mode {
        req = req.parse_mode(pm);
    }
    req.await?;
    Ok(())
}
