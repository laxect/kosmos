use crate::bot;
use async_std::{os::unix::net, prelude::*, task};
use kosmos::{client, utils::*};
use lazy_static::lazy_static;
use std::env;
use telegram_types::bot::types;
use yukikaze::Request;

lazy_static! {
    static ref TOKEN: String = env::var("KOSMOS_TG_TOKEN").unwrap();
    static ref CHANNEL: types::ChatId = {
        let channel_id = env::var("KOSMOS_TG_CHANNEL").unwrap();
        let chat_id: i64 = channel_id.parse().unwrap();
        types::ChatId(chat_id)
    };
}

pub(crate) struct Postamt {
    kosmos: client::UnixClient,
}

impl Default for Postamt {
    fn default() -> Self {
        Self::new("yukikaze")
    }
}

pub(crate) async fn handle(stream: &mut net::UnixStream) -> anyhow::Result<()> {
    let len = stream.get_len().await?;
    let req: Request = stream.get_obj(len).await?;
    let bot = bot::Bot::new(&TOKEN);
    let text = req.to_owned();
    bot.send_message(text, *CHANNEL).await?;
    Ok(())
}

impl Postamt {
    fn new<T: Into<String>>(name: T) -> Postamt {
        let name = name.into();
        Postamt {
            kosmos: client::UnixClient::new(name),
        }
    }

    pub(crate) async fn regist(&mut self) -> anyhow::Result<()> {
        self.kosmos.regist().await?;
        Ok(())
    }

    pub(crate) async fn listen(&self) -> anyhow::Result<()> {
        let listener = self.kosmos.listen().await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let mut stream = stream?;
            task::spawn(async move {
                loop {
                    match handle(&mut stream).await {
                        Ok(_) => continue,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}
