use crate::bot;
use async_std::{os::unix::net, prelude::*, task};
use kosmos::{client, utils::*};
use lazy_static::lazy_static;
use std::env;
use telegram_types::bot::types;
use yukikaze::{Ask, Request};

lazy_static! {
    static ref TOKEN: String = env::var("KOSMOS_TG_TOKEN").unwrap();
    static ref CHANNEL: types::ChatId = {
        let channel_id = env::var("KOSMOS_TG_CHANNEL").unwrap();
        let chat_id: i64 = channel_id.parse().unwrap();
        types::ChatId(chat_id)
    };
}

async fn handle_cmd<T: Into<String>>(cmd: T) -> anyhow::Result<()> {
    // have no func now
    println!("{}", cmd.into());
    Ok(())
}

#[derive(Clone)]
pub(crate) struct Postamt {
    kosmos: client::UnixClient,
}

impl Default for Postamt {
    fn default() -> Self {
        Self::new("yukikaze")
    }
}

pub(crate) async fn handle(stream: &mut net::UnixStream) -> anyhow::Result<Status> {
    let len = stream.get_len().await?;
    if len == 0 {
        return Ok(Status::Exit);
    }
    let req: Request = stream.get_obj(len).await?;
    let bot = bot::Bot::new(&TOKEN);
    let text = req.to_owned();
    bot.send_message(text, *CHANNEL).await?;
    Ok(Status::Continue)
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
                        Ok(Status::Continue) => continue,
                        Ok(Status::Exit) => break,
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

    pub(crate) async fn incoming(&self, bot: Option<bot::Bot>) -> anyhow::Result<bot::Bot> {
        let mut bot = bot.unwrap_or_else(|| bot::Bot::new(&TOKEN));
        let updates = bot.get_updates().await.unwrap();
        let mut u = updates.into_iter();
        while let Some(types::Message { text: Some(text), .. }) = u.next() {
            println!("{}", &text);
            let mut words: Vec<String> = text.splitn(2, ' ').map(|x| x.to_owned()).collect();
            let cmd = words.pop().ok_or_else(|| anyhow::Error::msg("expect a command"))?;
            let target = words.pop().ok_or_else(|| anyhow::Error::msg("expect a target"))?;
            if target == "yukikaze" {
                task::spawn(async { handle_cmd(cmd) });
            } else {
                let ask = Ask::new(cmd);
                let ask = ask.package()?;
                let mut stream = self.kosmos.connect_until_success(target).await?;
                task::spawn(async move || -> anyhow::Result<()> {
                    stream.write(&ask).await?;
                    while handle(&mut stream).await?.is_continue() {}
                    Ok(())
                }());
            }
        }
        Ok(bot)
    }
}
