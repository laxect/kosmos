use crate::planet::{self, Package};
use async_std::{io, os::unix::net, prelude::*, task, fs};
use async_trait::async_trait;
use std::time;

#[async_trait]
trait Server<T>
where
    T: io::Write + io::Read + Send + Sync + 'static + Unpin,
{
    fn set(&self, planet: &mut planet::Planet) -> anyhow::Result<()>;

    fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>>;

    async fn run(&self) -> anyhow::Result<()>;

    async fn handle(&self, stream: &mut T) -> anyhow::Result<()> {
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
                planet.update_name()?;
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

pub async fn link_init() -> anyhow::Result<()> {
    let path = "/tmp/kosmos/link";
    let _ = fs::create_dir_all(path).await;
    let mut dir = fs::read_dir(path).await?;
    while let Some(res) = dir.next().await {
        let entry = res?;
        fs::remove_file(entry.path()).await?;
    }
    Ok(())
}

#[derive(Clone)]
pub struct UnixSocketServer {
    name: String,
    name_map: sled::Db,
}

impl UnixSocketServer {
    pub fn new(name: String) -> anyhow::Result<Self> {
        let name_map = sled::open(["/tmp/kosmos/db/", &name].concat())?;
        Ok(Self { name, name_map })
    }

    pub fn with_custom_db_path(name: String, path: &str) -> anyhow::Result<Self> {
        let name_map = sled::open(path)?;
        Ok(Self { name, name_map })
    }
}

#[async_trait]
impl Server<net::UnixStream> for UnixSocketServer {
    fn set(&self, planet: &mut planet::Planet) -> anyhow::Result<()> {
        let name = planet.name();
        let binary_planet: Vec<u8> = bincode::serialize(planet)?;
        self.name_map.insert(name, binary_planet)?;
        Ok(())
    }

    fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>> {
        if let Some(binary_planet) = self.name_map.scan_prefix(name).values().nth(0) {
            let planet = bincode::deserialize(binary_planet?.as_ref())?;
            Ok(Some(planet))
        } else {
            Ok(None)
        }
    }

    async fn run(&self) -> anyhow::Result<()> {
        let listener = net::UnixListener::bind(["/tmp/kosmos/link/", &self.name].concat()).await?;
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

#[cfg(test)]
mod tests {
    use super::{super::*, *};
    use async_std::{io, task};

    #[test]
    fn set_get() ->anyhow::Result<()> {
        let server = UnixSocketServer::with_custom_db_path("test".to_owned(), "test")?;
        let mut planet = planet::Planet::new("test".to_owned(), planet::AirportKind::UnixSocket);
        planet.update_name()?;
        server.set(&mut planet)?;
        let back = server.get(planet.name())?;
        assert_eq!(back, Some(planet));
        Ok(())
    }

    #[test]
    fn system_test() {
        task::block_on(link_init()).unwrap();
        task::spawn(async {
            let server = UnixSocketServer::new("test".to_owned()).unwrap();
            server.run().await.unwrap();
        });
        task::spawn_blocking(async || {
            use async_std::os::unix::net;

            let mut stream = net::UnixStream::connect("/tmp/kosmos/link/test").await.unwrap();
            let req = planet::Request::Get("test".to_owned());
            let req = req.package().unwrap();
            stream.write(&req).await.unwrap();
            let mut len = [0u8; 4];
            stream.read_exact(&mut len).await.unwrap();
            let len: u32 = bincode::deserialize(&len).unwrap();
            let mut resp = vec![0u8; len as usize];
            stream
                .read_exact(&mut resp)
                .timeout(time::Duration::from_millis(100))
                .await
                .unwrap()
                .unwrap();
            let resp: planet::GetResponse = bincode::deserialize(&resp).expect("decode failed");
            assert_eq!(resp, planet::GetResponse::NotFound);
        });
    }

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
            self.handle(&mut cur)
                .timeout(time::Duration::from_millis(100))
                .await
                .unwrap()
                .unwrap();
            cur.set_position(now);
            let mut len = [0u8; 4];
            cur.read_exact(&mut len).await?;
            let len: u32 = bincode::deserialize(&len)?;
            let mut resp = vec![0u8; len as usize];
            cur.read_exact(&mut resp)
                .timeout(time::Duration::from_millis(100))
                .await
                .unwrap()
                .unwrap();
            let resp: planet::GetResponse = bincode::deserialize(&resp).expect("decode failed");
            assert_eq!(resp, planet::GetResponse::NotFound);
            Ok(())
        }
    }

    #[test]
    fn server_trait_handle() {
        let test_server = TestServer {};
        task::block_on(async move {
            test_server.run().await.unwrap();
        });
    }
}
