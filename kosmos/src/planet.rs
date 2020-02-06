use serde::{Deserialize, Serialize};
use std::{cmp::PartialEq, matches, time};

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

    pub(crate) fn is_unix_socket(&self) -> bool {
        matches!(self.airport_kind, AirportKind::UnixSocket)
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
