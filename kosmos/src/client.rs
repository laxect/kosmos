use crate::{planet, utils::*};
use async_std::{os::unix::net, prelude::*};

const KOSMOS_SERVER: &str = "/tmp/kosmos/link/kosmos";

pub enum Stream {
    UnixSocket(net::UnixStream),
}

#[derive(Clone, Debug)]
pub struct UnixCLient {
    name: String,
}

impl UnixCLient {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub async fn regist(&self) -> anyhow::Result<()> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let me = planet::Planet::new(self.name.clone(), planet::AirportKind::UnixSocket);
        let req = planet::Request::Regist(me);
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        // parser response
        let len = stream.get_len().await?;
        let resp: planet::RegistResponse = stream.get_obj(len).await?;
        match resp {
            planet::RegistResponse::Success(_) => {}
            planet::RegistResponse::Fail(e) => {
                eprintln!("Regist failed: {}", e);
            }
        }
        Ok(())
    }

    async fn resolve(&self, name: String) -> anyhow::Result<Option<planet::Planet>> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let req = planet::Request::Get(name);
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        let len = stream.get_len().await?;
        let resp: planet::GetResponse = stream.get_obj(len).await?;
        match resp {
            planet::GetResponse::Get(planet) => Ok(Some(planet)),
            _ => Ok(None),
        }
    }

    async fn ping(&self, name: String) -> anyhow::Result<()> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let req = planet::Request::Ping(name);
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        Ok(())
    }

    pub async fn connent(&self, name: String) -> anyhow::Result<Stream> {
        let planet = self.resolve(name.clone()).await?;
        if let Some(planet) = planet {
            if planet.is_unix_socket() {
                if let Ok(stream) = net::UnixStream::connect(planet.name()).await {
                    return Ok(Stream::UnixSocket(stream));
                } else {
                    self.ping(name).await?;
                    return Err(anyhow::Error::msg("can not connect."));
                }
            } else {
                unimplemented!();
            }
        }
        Err(anyhow::Error::msg("can not resolve target planet"))
    }

    // must regist first
    pub async fn listen(&self) -> anyhow::Result<net::UnixStream> {
        let stream = net::UnixStream::connect(["/tmp/kosmos/link/", self.name.as_ref()].concat()).await?;
        Ok(stream)
    }
}
