use async_std::io;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait ReadEx: io::Read + io::ReadExt + Unpin + Send {
    async fn get_len(&mut self) -> anyhow::Result<u32> {
        let mut len = [0u8; 4];
        self.read_exact(&mut len).await?;
        let len = bincode::deserialize(&len)?;
        Ok(len)
    }

    async fn get_obj<T: DeserializeOwned>(&mut self, len: u32) -> anyhow::Result<T> {
        let mut buffer = vec![0u8; len as usize];
        self.read_exact(buffer.as_mut()).await?;
        let obj = bincode::deserialize(&buffer)?;
        Ok(obj)
    }
}

impl<T: io::Write + io::Read + Send + Sync + 'static + Unpin> ReadEx for T {}

pub trait Package: Serialize + Clone {
    fn package(&self) -> anyhow::Result<Vec<u8>> {
        let mut binary_self = bincode::serialize(&self)?;
        let len: u32 = binary_self.len() as u32;
        let mut pkg = bincode::serialize(&len)?;
        pkg.append(&mut binary_self);
        Ok(pkg)
    }
}

impl<T: Serialize + Clone> Package for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::{io::Cursor, prelude::*};

    #[async_std::test]
    async fn read_ex() -> anyhow::Result<()> {
        let mut cur = Cursor::new(Vec::<u8>::new());
        let input = "test".to_owned();
        let binary_input = input.package()?;
        cur.write(&binary_input).await?;
        cur.set_position(0);
        let len = cur.get_len().await?;
        let obj: String = cur.get_obj(len).await?;
        assert_eq!(obj, input);
        Ok(())
    }
}
