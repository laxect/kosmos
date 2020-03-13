use async_std::sync::Mutex;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
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

static TOKEN: Lazy<String> = Lazy::new(|| std::env::var("KOSMOS_TG_TOKEN").unwrap());
static CHANNEL: Lazy<types::ChatId> = Lazy::new(|| {
    let channel_id = std::env::var("KOSMOS_TG_CHANNEL").unwrap();
    types::ChatId(channel_id.parse().unwrap())
});
static FIRST: OnceCell<()> = OnceCell::new();
static UPDATE_ID: Lazy<Mutex<Cell<i64>>> = Lazy::new(|| Mutex::new(Cell::new(0)));

fn is_first() -> bool {
    if FIRST.get().is_none() {
        FIRST.set(()).unwrap();
        true
    } else {
        false
    }
}

fn action(action: &str) -> String {
    format!("https://api.telegram.org/bot{}/{}", *TOKEN, action)
}

pub async fn get_updates() -> anyhow::Result<Vec<types::Message>> {
    let cell = UPDATE_ID.lock().await;
    let uri = action("getUpdates");
    let get_updates: GetUpdates = surf::get(uri).recv_json().await.into_bot_result()?;
    if !get_updates.ok {
        return Err(anyhow::Error::msg("tg api error"));
    }
    let mut updates = Vec::new();
    for u in get_updates.result.into_iter() {
        let types::UpdateId(update_id) = u.update_id;
        if update_id > cell.get() {
            cell.set(update_id);
            if is_first() {
                continue;
            }
            if let types::UpdateContent::Message(message) = u.content {
                updates.push(message);
            }
        }
    }
    Ok(updates)
}

#[allow(dead_code)]
pub async fn text_reply<T: AsRef<str>>(message: types::Message, text: T) -> anyhow::Result<()> {
    let chat_id = methods::ChatTarget::Id(message.chat.id);
    let reply_to = message.message_id;
    let send_message = methods::SendMessage::new(chat_id, text.as_ref()).reply(reply_to);
    let uri = action("sendMessage");
    surf::post(uri).body_json(&send_message)?.await.into_bot_result()?;
    Ok(())
}

pub(crate) async fn send_message<T: Into<String>>(text: T, chat_id: types::ChatId) -> anyhow::Result<()> {
    let text = text.into();
    log::info!("Send: {:?} - {}", &chat_id, &text);
    let chat_id = methods::ChatTarget::Id(chat_id);
    let send_message = methods::SendMessage::new(chat_id, text);
    let uri = action("sendMessage");
    surf::post(uri).body_json(&send_message)?.await.into_bot_result()?;
    Ok(())
}

pub(crate) async fn send_to_default<T: Into<String>>(text: T) -> anyhow::Result<()> {
    send_message(text, *CHANNEL).await
}
