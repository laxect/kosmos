use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "/tmp/kosmos/db/aoi";

lazy_static! {
    static ref DB: sled::Db = { sled::open(DB_PATH).unwrap() };
}

#[derive(Clone)]
pub(crate) struct Store {
    inner: sled::Tree,
}

impl Store {
    pub(crate) fn new<V: AsRef<[u8]>>(name: V) -> anyhow::Result<Self> {
        let tree = DB.open_tree(name)?;
        let store = Self { inner: tree };
        Ok(store)
    }

    pub(crate) fn insert<V: AsRef<[u8]>, T: Serialize>(&self, key: V, val: &T) -> anyhow::Result<()> {
        let binary_val = bincode::serialize(val)?;
        self.inner.insert(key, binary_val)?;
        Ok(())
    }

    pub(crate) fn remove<V: AsRef<[u8]>>(&self, key: V) -> anyhow::Result<()> {
        self.inner.remove(key)?;
        Ok(())
    }

    pub(crate) fn clear(&self) -> anyhow::Result<()> {
        self.inner.clear()?;
        Ok(())
    }

    pub(crate) fn iter(&self) -> sled::Iter {
        self.inner.iter()
    }

    pub(crate) fn keys(&self) -> anyhow::Result<Vec<String>> {
        let mut keys = Vec::new();
        for key in self.iter().keys() {
            let key = key?;
            let key: String = String::from_utf8(Vec::from(key.as_ref()))?;
            keys.push(key);
        }
        Ok(keys)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub(crate) struct WatchTarget {}
