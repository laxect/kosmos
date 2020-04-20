pub mod store {
    use once_cell::sync::{Lazy, OnceCell};
    use serde::{de::DeserializeOwned, Serialize};
    use sled::Db;

    static DB_NAME: OnceCell<String> = OnceCell::new();
    static DB: Lazy<Db> = Lazy::new(|| {
        let path = DB_NAME.get().expect("db path not set");
        sled::open(&path).unwrap()
    });

    pub fn db_init<P: Into<String>>(path: P) {
        DB_NAME.set(path.into()).unwrap();
    }

    pub fn set<T: Serialize>(key: &str, val: &T) -> anyhow::Result<()> {
        let key = bincode::serialize(key)?;
        let val = bincode::serialize(val)?;
        DB.insert(key, val)?;
        Ok(())
    }

    pub fn get<T: DeserializeOwned>(key: &str) -> anyhow::Result<Option<T>> {
        let key = bincode::serialize(key)?;
        let val = DB.get(key)?;
        if let Some(val) = val {
            Ok(Some(bincode::deserialize(&val[..])?))
        } else {
            Ok(None)
        }
    }

    pub fn clear() -> anyhow::Result<()> {
        DB.clear()?;
        Ok(())
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use once_cell::sync::Lazy;
        use std::fs;

        static SET_UP: Lazy<bool> = Lazy::new(|| {
            fs::create_dir_all("./test").ok();
            true
        });

        #[test]
        fn set_and_get() -> anyhow::Result<()> {
            if *SET_UP {
                log::info!("test set up");
            }
            db_init("./test/set_and_get");
            let key = String::from("key");
            let val = String::from("val");
            set(&key, &val)?;
            assert_eq!(Some(val), get(&key)?, "set and get failed");
            clear()?;
            let val: Option<String> = get(&key)?;
            assert_eq!(None, val, "clear failed");
            Ok(())
        }
    }
}
