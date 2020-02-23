use async_std::io;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum ReadResult<T> {
    Exit,
    Continue(T),
}

impl<T> From<ReadResult<T>> for Status {
    fn from(rr: ReadResult<T>) -> Status {
        match rr {
            ReadResult::Exit => Status::Exit,
            ReadResult::Continue(_) => Status::Continue,
        }
    }
}

const C_EXIT: Status = Status::Exit;
const C_CONTINUE: Status = Status::Continue;

impl<T> Deref for ReadResult<T> {
    type Target = Status;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadResult::Exit => &C_EXIT,
            ReadResult::Continue(_) => &C_CONTINUE,
        }
    }
}

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

    async fn unpack<T: DeserializeOwned>(&mut self) -> anyhow::Result<ReadResult<T>> {
        let len = self.get_len().await?;
        if len == 0 {
            return Ok(ReadResult::Exit);
        }
        let obj = self.get_obj(len).await?;
        Ok(ReadResult::Continue(obj))
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

#[derive(Clone, Copy, Debug)]
pub enum Status {
    Continue,
    Exit,
}

impl Status {
    pub fn is_exit(self) -> bool {
        match self {
            Self::Exit => true,
            _ => false,
        }
    }

    pub fn is_continue(self) -> bool {
        match self {
            Self::Continue => true,
            _ => false,
        }
    }
}
