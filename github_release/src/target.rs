use once_cell::sync as cell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Target(String, String);

impl Target {
    pub(crate) fn get_user(&self) -> &str {
        &self.0
    }

    pub(crate) fn get_repo(&self) -> &str {
        &self.1
    }

    pub(crate) fn new<R: ToString, U: ToString>(user: U, repo: R) -> Self {
        Self(user.to_string(), repo.to_string())
    }
}

impl ToString for Target {
    fn to_string(&self) -> String {
        format!("{}/{}", self.0, self.1)
    }
}

static STORE: cell::Lazy<sled::Db> = cell::Lazy::new(|| sled::open("/tmp/kosmos/db/github_release").unwrap());

const BEHOLDER: [u8; 0] = [0u8; 0];

pub(crate) fn add(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    STORE.insert(bin_target, &BEHOLDER)?;
    Ok(())
}

pub(crate) fn remove(target: &Target) -> anyhow::Result<()> {
    let bin_target = bincode::serialize(target)?;
    STORE.remove(bin_target)?;
    Ok(())
}

pub(crate) fn list() -> anyhow::Result<Vec<Target>> {
    let mut res = Vec::new();
    for key in STORE.iter().keys() {
        let key = key?;
        let key = bincode::deserialize(&key)?;
        res.push(key);
    }
    Ok(res)
}

fn cmd(input: String) -> anyhow::Result<()> {

    Ok(())
}
