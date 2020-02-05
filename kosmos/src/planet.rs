use serde::{Deserialize, Serialize};
use std::{cmp::PartialEq, time};

fn get_random_str() -> anyhow::Result<String> {
    let timestamp = time::SystemTime::now()
        .duration_since(time::SystemTime::UNIX_EPOCH)?
        .as_secs();
    let randint: u32 = rand::random();
    let fmt = format!("{}-{}", timestamp, randint);
    Ok(base64::encode(fmt.as_bytes()))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Copy)]
pub(crate) enum AirportKind {
    UnixSocket,
    DomainName,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Planet {
    airport_kind: AirportKind,
    name: String,
}

impl Planet {
    pub(crate) fn new(name: String, airport_kind: AirportKind) -> Self {
        Planet { name, airport_kind }
    }

    pub(crate) fn update_name(&mut self) -> anyhow::Result<()> {
        if !self.name.contains('/') {
            self.name.push('/');
            self.name.push_str(get_random_str()?.as_str());
        }
        Ok(())
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum Request {
    Get(String),
    Regist(Planet),
    // heart beat
    Ping(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum GetResponse {
    NotFound,
    NotAvaliable,
    Get(Planet),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum RegistResponse {
    Success(String),
    Fail(String),
}

// ping response
type Pong = u32;

pub trait Package: Serialize + Clone {
    fn package(self) -> anyhow::Result<Vec<u8>> {
        let mut binary_self = bincode::serialize(&self)?;
        let len: u32 = binary_self.len() as u32;
        let mut pkg = bincode::serialize(&len)?;
        pkg.append(&mut binary_self);
        Ok(pkg)
    }
}

impl<T: Serialize + Clone> Package for T {}
