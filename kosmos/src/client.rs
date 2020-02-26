use crate::{planet, utils::*};
use async_std::{os::unix::net, prelude::*};

const KOSMOS_SERVER: &str = "/tmp/kosmos/link/kosmos";
const EXIT: [u8; 4] = [0u8; 4];
const CAN_NOT_CONNECT: &str = "can not connect";

#[derive(Clone, Debug)]
pub struct UnixClient {
    name: String,
}

impl UnixClient {
    pub fn new<T: Into<String>>(name: T) -> Self {
        let name = name.into();
        Self { name }
    }

    pub async fn regist(&mut self) -> anyhow::Result<String> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let me = planet::Planet::new(self.name.clone(), planet::AirportKind::UnixSocket);
        let req = planet::Request::Regist(me);
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        // parser response
        let len = stream.get_len().await?;
        let resp: planet::RegistResponse = stream.get_obj(len).await?;
        stream.write(&EXIT).await?;
        match resp {
            planet::RegistResponse::Success(new_name) => {
                self.name = new_name.clone();
                Ok(new_name)
            }
            planet::RegistResponse::Fail(e) => Err(anyhow::Error::msg(e)),
        }
    }

    async fn resolve(&self, name: &str) -> anyhow::Result<Option<planet::Planet>> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let req = planet::Request::Get(name.to_owned());
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        let len = stream.get_len().await?;
        let resp: planet::GetResponse = stream.get_obj(len).await?;
        stream.write(&EXIT).await?;
        match resp {
            planet::GetResponse::Get(planet) => Ok(Some(planet)),
            _ => Ok(None),
        }
    }

    async fn ping(&self, name: &str) -> anyhow::Result<()> {
        let mut stream = net::UnixStream::connect(KOSMOS_SERVER).await?;
        let req = planet::Request::Ping(name.to_owned());
        let req = req.package()?;
        stream.write(req.as_ref()).await?;
        stream.write(&EXIT).await?;
        Ok(())
    }

    pub async fn connect<T: Into<String>>(&self, name: T) -> anyhow::Result<net::UnixStream> {
        let name = name.into();
        let planet = self.resolve(&name).await?;
        if let Some(planet) = planet {
            if planet.is_unix_socket() {
                if let Ok(stream) = net::UnixStream::connect(["/tmp/kosmos/link/", &planet.name()].concat()).await {
                    return Ok(stream);
                } else {
                    self.ping(&planet.name()).await?;
                    return Err(anyhow::Error::msg(CAN_NOT_CONNECT));
                }
            } else {
                unimplemented!();
            }
        }
        Err(anyhow::Error::msg("can not resolve target planet"))
    }

    pub async fn connect_until_success<T: Into<String>>(&self, name: T) -> anyhow::Result<net::UnixStream> {
        let name = name.into();
        loop {
            match self.connect(&name).await {
                Err(e) => {
                    if e.to_string() == CAN_NOT_CONNECT {
                        continue;
                    } else {
                        break Err(e);
                    }
                }
                Ok(stream) => {
                    break Ok(stream);
                }
            }
        }
    }

    pub async fn send_once<N: Into<String>, T: Package>(&self, name: N, pkg: &T) -> anyhow::Result<()> {
        let mut stream = self.connect_until_success(name).await?;
        stream.send(pkg).await?;
        stream.exit().await?;
        Ok(())
    }

    // must regist first
    pub async fn listen(&self) -> anyhow::Result<net::UnixListener> {
        println!("name - {}", self.name);
        let stream = net::UnixListener::bind(["/tmp/kosmos/link/", self.name.as_ref()].concat()).await?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Error;

    #[test]
    fn ensure_anyhow_msg_error() {
        const MSG: &str = "test";
        let e = Error::msg(MSG);
        assert_eq!(e.to_string(), MSG);
    }
}
