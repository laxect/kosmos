use serde::{Deserialize, Serialize};
use telegram_types::bot::{methods, types};

#[derive(thiserror::Error, Debug)]
enum BotError {
    #[error("surf error: {0}")]
    SurfError(#[from] surf::Exception),
}

trait IntoBotResult<T> {
    fn into_bot_result(self) -> Result<T, BotError>;
}

impl<T> IntoBotResult<T> for Result<T, surf::Exception> {
    fn into_bot_result(self) -> Result<T, BotError> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
struct GetUpdates {
    pub ok: bool,
    pub result: Vec<types::Update>,
}

#[derive(Clone, Copy)]
pub struct Bot {
    token: &'static str,
    update_id: i64,
    first: bool,
}

impl Bot {
    pub fn new(token: &'static str) -> Bot {
        Bot {
            token,
            update_id: 0,
            first: true,
        }
    }

    fn action(&self, action: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.token, action)
    }

    pub async fn get_updates(&mut self) -> anyhow::Result<Vec<types::Message>> {
        let uri = self.action("getUpdates");
        let get_updates: GetUpdates = surf::get(uri).recv_json().await.into_bot_result()?;
        if !get_updates.ok {
            return Err(anyhow::Error::msg("tg api error"));
        }
        let mut updates = Vec::new();
        for u in get_updates.result.into_iter() {
            let types::UpdateId(update_id) = u.update_id;
            if update_id > self.update_id {
                self.update_id = update_id;
                if self.first {
                    continue;
                }
                if let types::UpdateContent::Message(message) = u.content {
                    updates.push(message);
                }
            }
        }
        self.first = false;
        Ok(updates)
    }

    #[allow(dead_code)]
    pub async fn text_reply<T: Into<String>>(&self, message: types::Message, text: T) -> anyhow::Result<()> {
        let chat_id = methods::ChatTarget::Id(message.chat.id);
        let reply_to = message.message_id;
        let send_message = methods::SendMessage::new(chat_id, text.into()).reply(reply_to);
        let uri = self.action("sendMessage");
        surf::post(uri).body_json(&send_message)?.await.into_bot_result()?;
        Ok(())
    }

    pub async fn send_message<T: Into<String>>(&self, text: T, chat_id: types::ChatId) -> anyhow::Result<()> {
        let chat_id = methods::ChatTarget::Id(chat_id);
        let send_message = methods::SendMessage::new(chat_id, text.into());
        let uri = self.action("sendMessage");
        surf::post(uri).body_json(&send_message)?.await.into_bot_result()?;
        Ok(())
    }
}
