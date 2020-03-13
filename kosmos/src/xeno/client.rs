use crate::{client::UnixClient, utils::*};
use async_std::{os::unix::net::UnixStream, prelude::*, task};
use async_trait::async_trait;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait XenoHandler: Send + Sync + Clone {
    type In: DeserializeOwned + Send + Sync;
    type Out: Package;

    async fn handle(&self, input: Self::In) -> anyhow::Result<Option<Self::Out>>;
}

pub struct XenoClient<T: XenoHandler + 'static> {
    client: UnixClient,
    handler: T,
}

async fn recv_in<T: XenoHandler + 'static>(handler: &T, stream: &mut UnixStream) -> anyhow::Result<Status> {
    let input: ReadResult<T::In> = stream.unpack().await?;
    if let ReadResult::Continue(input) = input {
        if let Some(output) = handler.handle(input).await? {
            stream.send(&output).await?;
        }
    } else {
        return Ok(Status::Exit);
    }
    Ok(Status::Continue)
}

impl<T: XenoHandler> XenoClient<T> {
    pub fn new<N: Into<String>>(name: N, handler: T) -> Self {
        let client = UnixClient::new(name);
        Self { client, handler }
    }

    pub async fn regist(&mut self) -> anyhow::Result<()> {
        self.client.regist().await?;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let listener = self.client.listen().await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let handler = self.handler.clone();
            task::spawn(async move {
                let mut stream = stream.unwrap();
                loop {
                    match recv_in(&handler, &mut stream).await {
                        Ok(Status::Continue) => {}
                        Ok(Status::Exit) => break,
                        Err(e) => {
                            log::error!(target:"xeno client", "Xeno Error: {}", e);
                            break;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}
