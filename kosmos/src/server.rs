use crate::planet;
use async_std::{io, prelude::*};
use async_trait::async_trait;
pub use domain_socker_server::DomainSocketServer;
use std::time;

#[async_trait]
trait Server<T>
where
    T: io::Write + io::Read + Send + Sync + 'static + Unpin,
{
    fn set(&self, planet: planet::Planet) -> anyhow::Result<()>;

    fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>>;

    async fn run(&self) -> anyhow::Result<()>;

    async fn handle(&self, mut stream: T) -> anyhow::Result<()> {
        let mut len = [0u8; 4];
        stream.read_exact(&mut len).await?;
        let len: u32 = bincode::deserialize(&len)?;
        let mut request = vec![0u8; len as usize];
        let mut handle = stream.take(len as u64);
        handle
            .read_exact(request.as_mut())
            .timeout(time::Duration::from_millis(500))
            .await??;
        let request: planet::Request = bincode::deserialize(request.as_ref())?;
        match request {
            planet::Request::Get(name) => {
                unimplemented!();
            }
            planet::Request::Ping(name) => {
                unimplemented!();
            }
            planet::Request::Regist(planet) => {
                unimplemented!();
            }
        }
        Ok(())
    }
}

mod udp_server {}

mod domain_socker_server {
    use super::{async_trait, planet};
    use async_std::{os::unix::net, prelude::*};

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
        fn set(&self, planet: planet::Planet) -> anyhow::Result<()> {
            let name = planet.name();
            let binary_planet: Vec<u8> = bincode::serialize(&planet)?;
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
                let stream = stream?;
                let server = self.clone();
                async_std::task::spawn(async move {
                    if let Err(e) = server.handle(stream).await {
                        eprintln!("Server failed on {}", e);
                    }
                });
            }
            Ok(())
        }
    }
}
