use crate::async_trait;
use async_std::sync;
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CellError {
    #[error("Alreay has a source")]
    SourceExist,
    #[error("Alreay has a next")]
    NextExist,
    #[error("No source set")]
    SourceNotExist,
    #[error("No next set")]
    NextNotExist,
    #[error("Lambda error")]
    LambdaFailed(#[from] anyhow::Error),
}

pub struct Cell<S, N, L>
where
    S: Serialize + DeserializeOwned,
    N: Serialize + DeserializeOwned,
    L: Lambda<S, N>,
{
    source: Option<sync::Receiver<S>>,
    next: Option<sync::Sender<N>>,
    lambda: Box<L>,
}

impl<S, N, L> Cell<S, N, L>
where
    S: Serialize + DeserializeOwned,
    N: Serialize + DeserializeOwned,
    L: Lambda<S, N>,
{
    pub fn new(lambda: L) -> Self {
        Self {
            source: None,
            next: None,
            lambda: Box::new(lambda),
        }
    }

    pub fn link_to<ON, OL>(&mut self, other: &mut Cell<N, ON, OL>) -> Result<(), CellError>
    where
        ON: Serialize + DeserializeOwned,
        OL: Lambda<N, ON>,
    {
        let (s, r) = sync::channel(20);
        if self.next.is_some() {
            return Err(CellError::NextExist);
        }
        if other.source.is_some() {
            return Err(CellError::SourceExist);
        }
        self.next = Some(s);
        other.source = Some(r);
        Ok(())
    }

    fn check_set(&self) -> Result<(), CellError> {
        if self.source.is_none() {
            return Err(CellError::SourceNotExist);
        }
        if self.next.is_none() {
            return Err(CellError::NextNotExist);
        }
        Ok(())
    }

    pub async fn run(&self) -> Result<(), CellError> {
        self.check_set()?;
        let input = self.source.as_ref().unwrap().recv().await;
        if let Some(i) = input {
            match self.lambda.run(i).await {
                Ok(n) => self.next.as_ref().unwrap().send(n).await,
                Err(e) => {
                    log::error!("Cell Error: {}", e);
                    return Err(CellError::from(e));
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait Lambda<S, N>
where
    S: Serialize + DeserializeOwned,
    N: Serialize + DeserializeOwned,
    Self: Sync + Sized + Clone + Send,
{
    async fn run(&self, input: S) -> anyhow::Result<N>;
}
