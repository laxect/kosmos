use crate::target;
use async_std::{os::unix::net, prelude::*, task};
use kosmos::{client, utils::*};
use once_cell::sync as cell;
use yukikaze::{Ask, Request};

static NODE: cell::Lazy<String> = cell::Lazy::new(|| String::from("Github_watcher"));

async fn handle(stream: &mut net::UnixStream) -> anyhow::Result<ReadResult<()>> {
    if let ReadResult::Continue(Ask(ask)) = stream.unpack().await? {
        let resp = target::cmd(ask)?;
        let req = Request::post(resp, NODE.clone());
        let bin_req = req.package()?;
        stream.write(&bin_req).await?;
    } else {
        return Ok(ReadResult::Exit);
    }
    Ok(ReadResult::Continue(()))
}

pub(crate) async fn listen() -> anyhow::Result<()> {
    let mut client = client::UnixClient::new(String::from("github_release"));
    client.regist().await?;
    let listener = client.listen().await?;
    let mut incoming = listener.incoming();
    while let Some(s) = incoming.next().await {
        let mut s = s?;
        task::spawn(async move {
            loop {
                match handle(&mut s).await {
                    Ok(ReadResult::Continue(())) => continue,
                    Ok(ReadResult::Exit) => break,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        eprintln!("  {}", e.backtrace());
                        break;
                    }
                }
            }
        });
    }
    Ok(())
}
