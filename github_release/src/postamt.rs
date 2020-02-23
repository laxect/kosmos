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

static POST_CLIENT: cell::Lazy<client::UnixClient> = cell::Lazy::new(|| client::UnixClient::new("github_release"));

pub(crate) async fn post(target: &str, version: &str) -> anyhow::Result<()> {
    let msg = format!("{} release {}.", target, version);
    let req = Request::post(msg, String::from("Github_watcher"));
    let req = req.package()?;
    let sec_2 = std::time::Duration::from_secs(2);
    let mut stream = POST_CLIENT.connect_until_success("yukikaze").timeout(sec_2).await??;
    stream.write(&req).await?;
    let exit = [0u8; 4];
    stream.write(&exit).await?;
    Ok(())
}
