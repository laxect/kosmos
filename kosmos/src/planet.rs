use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum AirportKind {
    DomainSocket(String),
    DomainName(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Planet {
    airport_kind: AirportKind,
    name: String,
}

impl Planet {
    pub(crate) fn new(name: String, airport_kind: AirportKind) -> Self {
        Planet { name, airport_kind }
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum Request {
    Get(String),
    Regist(Planet),
    // heart beat
    Ping(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum GetResponse {
    NotFound,
    NotAvaliable,
    Get(Planet),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum RegistResponse {
    Success(String),
    Fail(String),
}

// ping response
type Pong = u32;

trait Package: Serialize + Clone {
    fn package(self) -> anyhow::Result<(u32, Vec<u8>)> {
        let binary_self: Vec<u8> = bincode::serialize(&self)?;
        Ok((binary_self.len() as u32, binary_self))
    }
}

impl<T: Serialize + Clone> Package for T {}
