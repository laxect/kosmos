use crate::{request, store};
use async_std::{os::unix::net, prelude::*, task};
use kosmos::{client::UnixClient, utils::*};
use structopt::StructOpt;
use yukikaze::{Ask, Request, EXIT};

const NODE: &str = "青い";

fn yukikaze_msg(msg: String) -> anyhow::Result<Vec<u8>> {
    let req = Request::post(msg, NODE.into());
    req.package()
}

fn format_keys(keys: Vec<String>, mut top: String) -> anyhow::Result<Vec<u8>> {
    top.push('\n');
    for s in keys.iter() {
        top.push_str("  -");
        top.push_str(s);
        top.push('\n');
    }
    if top.ends_with("\n\n") {
        top.pop();
    }
    let msg = yukikaze_msg(top);
    msg
}

#[derive(Clone)]
pub(crate) struct Postamt {
    kosmos: UnixClient,
}

async fn handle(stream: &mut net::UnixStream) -> anyhow::Result<Status> {
    println!("handle");
    let len = stream.get_len().await?;
    dbg!(len);
    if len == 0 {
        return Ok(Status::Exit);
    }
    let Ask(msg) = stream.get_obj(len).await?;
    dbg!(&msg);
    let mut head = String::from("aoi ");
    head.push_str(&msg);
    dbg!(&head);
    let args = head.split_whitespace();
    let req = request::Request::from_iter_safe(args)?;
    let namespace = req.namespace();
    let item = req.inner();
    let tree = store::Store::new(namespace)?;
    match item {
        request::Item::Add { name: args } => {
            for key in args.into_iter() {
                let val = store::WatchTarget {};
                tree.insert(key, &val)?;
            }
            let keys = tree.keys()?;
            let top = String::from("key added");
            let resp = format_keys(keys, top)?;
            stream.write(&resp).await?;
        }
        request::Item::Remove { name: args } => {
            for key in args.into_iter() {
                tree.remove(key)?;
            }
            let keys = tree.keys()?;
            let top = String::from("key removed");
            let resp = format_keys(keys, top)?;
            stream.write(&resp).await?;
        }
        request::Item::Clear => {
            tree.clear()?;
            let msg = String::from("key list cleard");
            let resp = yukikaze_msg(msg)?;
            stream.write(&resp).await?;
        }
        request::Item::List => {
            let keys = tree.keys()?;
            let top = String::from("list key");
            let resp = format_keys(keys, top)?;
            stream.write(&resp).await?;
        }
    }
    stream.write(&EXIT).await?;
    Ok(Status::Exit)
}

impl Postamt {
    pub(crate) fn new<T: Into<String>>(name: T) -> Postamt {
        Postamt {
            kosmos: UnixClient::new(name.into()),
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
                        Ok(Status::Continue) => {},
                        Ok(Status::Exit) => break,
                        Err(e) => {
                            println!("{}", e);
                            println!("{}", e.backtrace());
                            break;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

impl Default for Postamt {
    fn default() -> Postamt {
        Postamt::new("aoi")
    }
}
