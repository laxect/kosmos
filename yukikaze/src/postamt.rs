use crate::bot;
use async_std::{os::unix::net, prelude::*, task};
use kosmos::{
    client,
    utils::*,
    xeno::client::{XenoClient, XenoHandler},
};
use telegram_types::bot::types;
use yukikaze::{Ask, Request};

/// kosmos to tg
#[derive(Copy, Clone)]
pub(crate) struct KosmosHandler {}

#[async_trait::async_trait]
impl XenoHandler for KosmosHandler {
    type In = Request;
    type Out = ();

    async fn handle(&self, input: Self::In) -> anyhow::Result<Option<Self::Out>> {
        let text = input.to_owned();
        bot::send_to_default(text).await?;
        Ok(None)
    }
}

pub(crate) fn kosmos_to_tg() -> XenoClient<KosmosHandler> {
    let handler = KosmosHandler {};
    XenoClient::new("yukikaze", handler)
}

/// Tg to kosmos
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

async fn handle(stream: &mut net::UnixStream, chat_id: types::ChatId) -> anyhow::Result<Status> {
    let input: ReadResult<Request> = stream.unpack().await?;
    if let ReadResult::Continue(input) = input {
        let text = input.to_owned();
        bot::send_message(text, chat_id).await?;
    } else {
        return Ok(Status::Exit);
    }
    Ok(Status::Continue)
}

impl Postamt {
    fn new<T: Into<String>>(name: T) -> Postamt {
        let name = name.into();
        Postamt {
            kosmos: client::UnixClient::new(name),
        }
    }

    pub(crate) async fn incoming(&self) -> anyhow::Result<()> {
        let updates = bot::get_updates().await?;
        let mut u = updates.into_iter();
        while let Some(types::Message {
            text: Some(text),
            chat: box types::Chat { id: chat_id, .. },
            ..
        }) = u.next()
        {
            println!("{}", &text);
            let words: Vec<String> = text.splitn(2, ' ').map(|x| x.to_owned()).collect();
            if let [cmd, target] = words.as_slice() {
                if target == "yukikaze" {
                    let cmd = cmd.clone();
                    task::spawn(async { handle_cmd(cmd) });
                } else {
                    let ask = Ask::new(cmd.clone());
                    let two_sec: std::time::Duration = std::time::Duration::from_secs(2);
                    let mut stream = self.kosmos.connect_until_success(target).timeout(two_sec).await??;
                    task::spawn(async move {
                        if let Err(e) = stream.send(&ask).await {
                            log::error!("ask send failed: {}", e);
                        } else {
                            loop {
                                match handle(&mut stream, chat_id).await {
                                    Err(e) => {
                                        println!("{}", e);
                                        break;
                                    }
                                    Ok(Status::Continue) => continue,
                                    Ok(Status::Exit) => break,
                                }
                            }
                        }
                    });
                }
            }
        }
        Ok(())
    }
}
