use crate::{planet, utils::*};
use async_std::{fs, io, os::unix::net, prelude::*, task};
use async_trait::async_trait;
use std::time;

#[async_trait]
trait Server<T>
where
    T: io::Write + io::Read + Send + Sync + 'static + Unpin,
{
    fn expire(&self, name: String) -> anyhow::Result<()>;

    fn set(&self, planet: &mut planet::Planet) -> anyhow::Result<()>;

    fn get(&self, name: String) -> anyhow::Result<Option<planet::Planet>>;

    async fn run(&self) -> anyhow::Result<()>;

    async fn handle(&self, stream: &mut T) -> anyhow::Result<Status> {
        let len = stream.get_len().await?;
        if len == 0 {
            return Ok(Status::Exit);
        }
        let expired = time::Duration::from_millis(500);
        let req: planet::Request = stream.get_obj(len).timeout(expired).await??;
        match req {
            planet::Request::Get(name) => {
                let response = match self.get(name)? {
                    Some(planet) => planet::GetResponse::Get(planet),
                    None => planet::GetResponse::NotFound,
                };
                let pkg = response.package()?;
                stream.write(pkg.as_ref()).await?;
            }
            planet::Request::Ping(name) => {
                if let Ok(_ping_stream) = net::UnixStream::connect(["/tmp/kosmos/link/", name.as_ref()].concat()).await
                {
                    // do nothing
                    let resp = planet::Pong {};
                    let resp = resp.package()?;
                    stream.write(&resp).await?;
                } else {
                    self.expire(name)?;
                }
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
        Ok(Status::Continue)
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
    pub fn new<T: Into<String>>(name: T) -> anyhow::Result<Self> {
        let name = name.into();
        let name_map = sled::open(["/tmp/kosmos/db/", &name].concat())?;
        Ok(Self { name, name_map })
    }

    pub fn with_custom_db_path(name: String, path: &str) -> anyhow::Result<Self> {
        let name_map = sled::open(path)?;
        Ok(Self { name, name_map })
    }

    pub async fn listen(&self) -> anyhow::Result<()> {
        self.run().await?;
        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        self.name_map.clear()?;
        Ok(())
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
        if let Some(binary_planet) = self.name_map.scan_prefix(name).values().next() {
            let planet = bincode::deserialize(binary_planet?.as_ref())?;
            Ok(Some(planet))
        } else {
            Ok(None)
        }
    }

    fn expire(&self, name: String) -> anyhow::Result<()> {
        self.name_map.remove(name)?;
        Ok(())
    }

    async fn run(&self) -> anyhow::Result<()> {
        let listener = net::UnixListener::bind(["/tmp/kosmos/link/", &self.name].concat()).await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let mut stream = stream?;
            let server = self.clone();
            task::spawn(async move {
                loop {
                    match server.handle(&mut stream).await {
                        Ok(Status::Continue) => {}
                        Ok(Status::Exit) => break,
                        Err(e) => {
                            log::error!(target: "kosmos server", "Server failed on: {}", e);
                            break;
                        }
                    }
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
    use std::sync::Once;

    static ONCE: Once = Once::new();

    #[test]
    fn set_get() -> anyhow::Result<()> {
        let server = UnixSocketServer::with_custom_db_path("test_set_get".to_owned(), "test/set_get")?;
        let mut planet = planet::Planet::new("test_set_get".to_owned(), planet::AirportKind::UnixSocket);
        planet.update_name()?;
        server.set(&mut planet)?;
        let back = server.get(planet.name())?;
        assert_eq!(back, Some(planet));
        Ok(())
    }

    #[test]
    fn unix_get() {
        ONCE.call_once(|| {
            task::block_on(link_init()).unwrap();
        });
        task::spawn(async {
            let server = UnixSocketServer::with_custom_db_path("test_unix_get".to_owned(), "test/unix_get").unwrap();
            server.run().await.unwrap();
        });
        task::spawn_blocking(async || {
            use async_std::os::unix::net;

            let mut stream = net::UnixStream::connect("/tmp/kosmos/link/test_unix_get")
                .await
                .unwrap();
            let req = planet::Request::Get("test".to_owned());
            let req = req.package().unwrap();
            stream.write(&req).await.unwrap();
            let len = stream.get_len().await.unwrap();
            let expired = time::Duration::from_millis(100);
            let resp: planet::GetResponse = stream.get_obj(len).timeout(expired).await.unwrap().unwrap();
            assert_eq!(resp, planet::GetResponse::NotFound);
        });
    }

    #[async_std::test]
    async fn unix_regist_and_resolve() -> anyhow::Result<()> {
        ONCE.call_once(|| {
            task::block_on(link_init()).unwrap();
        });
        task::spawn(async {
            // clear db
            {
                let db = sled::open("test/test_regist_and_resolve").unwrap();
                db.clear().unwrap();
            }
            let server = UnixSocketServer::with_custom_db_path(
                "test_regist_and_resolve".to_owned(),
                "test/test_regist_and_resolve",
            )
            .unwrap();
            server.listen().await.unwrap();
        });
        task::sleep(time::Duration::from_millis(500)).await;
        let mut stream = net::UnixStream::connect("/tmp/kosmos/link/test_regist_and_resolve").await?;
        let test_planet = planet::Planet::new("test".to_owned(), planet::AirportKind::UnixSocket);
        let req = planet::Request::Regist(test_planet);
        let req = req.package()?;
        stream.write(&req).await?;
        let len = stream.get_len().await?;
        let resp: planet::RegistResponse = stream.get_obj(len).await?;
        if let planet::RegistResponse::Success(name) = resp {
            let req = planet::Request::Get("test".to_owned());
            let req = req.package()?;
            stream.write(&req).await?;
            let len = stream.get_len().await?;
            let resp: planet::GetResponse = stream.get_obj(len).await?;
            if let planet::GetResponse::Get(planet) = resp {
                assert_eq!(planet.name(), name);
            } else {
                panic!("Can not get from server")
            }
        } else {
            panic!("Regist not success")
        }
        Ok(())
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

        fn expire(&self, _name: String) -> anyhow::Result<()> {
            Ok(())
        }

        // run test
        async fn run(&self) -> anyhow::Result<()> {
            let expired = time::Duration::from_millis(100);
            let mut cur = io::Cursor::new(Vec::<u8>::new());
            let req = planet::Request::Get("test".to_owned());
            let req = req.package()?;
            cur.write(req.as_ref()).await.unwrap();
            let now = cur.position();
            cur.set_position(0);
            self.handle(&mut cur).timeout(expired).await.unwrap().unwrap();
            cur.set_position(now);
            let len = cur.get_len().await.unwrap();
            let resp: planet::GetResponse = cur.get_obj(len).timeout(expired).await.unwrap().unwrap();
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
