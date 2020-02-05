use crate::planet::{self, Package};
use async_std::{io, prelude::*};
use async_trait::async_trait;
pub use domain_socker_server::DomainSocketServer;
use std::time;

#[async_trait]
trait Server<T>
where
    T: io::Write + io::Read + Send + Sync + 'static + Unpin,
{
    fn set(&self, planet: &mut planet::Planet) -> anyhow::Result<()>;

    fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>>;

    async fn run(&self) -> anyhow::Result<()>;

    async fn handle(&self, stream:&mut T) -> anyhow::Result<()> {
        let mut len = [0u8; 4];
        stream.read_exact(&mut len).await?;
        let len: u32 = bincode::deserialize(&len)?;
        let mut request = vec![0u8; len as usize];
        stream
            .read_exact(request.as_mut())
            .timeout(time::Duration::from_millis(500))
            .await??;
        let request: planet::Request = bincode::deserialize(request.as_ref())?;
        match request {
            planet::Request::Get(name) => {
                let response = match self.get(name)? {
                    Some(planet) => planet::GetResponse::Get(planet),
                    None => planet::GetResponse::NotFound,
                };
                let pkg = response.package()?;
                stream.write(pkg.as_ref()).await?;
            }
            planet::Request::Ping(_name) => {
                unimplemented!();
            }
            planet::Request::Regist(mut planet) => {
                let response = match self.set(&mut planet) {
                    Ok(_) => planet::RegistResponse::Success(planet.name()),
                    Err(e) => planet::RegistResponse::Fail(e.to_string()),
                };
                let pkg = response.package()?;
                stream.write(pkg.as_ref()).await?;
            }
        }
        Ok(())
    }
}

mod udp_server {}

mod domain_socker_server {
    use super::{async_trait, planet};
    use async_std::{os::unix::net, prelude::*, task};

    #[derive(Clone)]
    pub struct DomainSocketServer {
        name: String,
        name_map: sled::Db,
    }

    impl DomainSocketServer {
        pub fn new(name: String) -> anyhow::Result<Self> {
            let name_map = sled::open(["/tmp/kosmos/", &name].concat())?;
            Ok(Self { name, name_map })
        }
    }

    #[async_trait]
    impl super::Server<net::UnixStream> for DomainSocketServer {
        fn set(&self, planet: &mut planet::Planet) -> anyhow::Result<()> {
            let name = planet.name();
            let binary_planet: Vec<u8> = bincode::serialize(planet)?;
            self.name_map.insert(name, binary_planet)?;
            Ok(())
        }

        fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>> {
            if let Some(binary_planet) = self.name_map.get(name)? {
                let planet = bincode::deserialize(binary_planet.as_ref())?;
                Ok(Some(planet))
            } else {
                Ok(None)
            }
        }

        async fn run(&self) -> anyhow::Result<()> {
            let listener = net::UnixListener::bind(["/tmp/", &self.name].concat()).await?;
            let mut incoming = listener.incoming();
            while let Some(stream) = incoming.next().await {
                let mut stream = stream?;
                let server = self.clone();
                task::spawn(async move {
                    if let Err(e) = server.handle(&mut stream).await {
                        eprintln!("Server failed on {}", e);
                    }
                });
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::io;
    use async_std::task;

    struct TestServer {}

    #[async_trait]
    impl Server<io::Cursor<Vec<u8>>> for TestServer {
        fn set(&self, _planet: &mut planet::Planet) -> anyhow::Result<()> {
            Ok(())
        }

        fn get(&self, _name: String) -> anyhow::Result<Option<planet::Planet>> {
            Ok(None)
        }

        // run test
        async fn run(&self) -> anyhow::Result<()> {
            let mut cur = io::Cursor::new(Vec::<u8>::new());
            let req = planet::Request::Get("test".to_owned());
            let req = req.package()?;
            cur.write(req.as_ref()).await.unwrap();
            let now = cur.position();
            cur.set_position(0);
            self.handle(&mut cur).timeout(time::Duration::from_millis(100)).await.unwrap().unwrap();
            cur.set_position(now);
            let mut len = [0u8; 4];
            cur.read_exact(&mut len).await?;
            let len: u32 = bincode::deserialize(&len)?;
            let mut resp = vec![0u8; len as usize];
            cur.read_exact(&mut resp).timeout(time::Duration::from_millis(100)).await.unwrap().unwrap();
            let resp: planet::GetResponse = bincode::deserialize(&resp).expect("decode failed");
            assert_eq!(resp, planet::GetResponse::NotFound);
            Ok(())
        }
    }

    #[test]
    fn server_trait_test() {
        let test_server = TestServer {};
        task::block_on(async move {
            test_server.run().await.unwrap();
        });
    }
}
