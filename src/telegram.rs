// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//use teloxide::Bot;
//use teloxide::prelude::*;
use crate::Config;

pub fn init_bot(config: &Config) {
    //Bot::new(&config.bot_token)
}

pub async fn send_message(chat_id: &str, message: &str) {
    //bot.send_message(chat_id.to_string(), message).await?;
    //Ok(())
}
