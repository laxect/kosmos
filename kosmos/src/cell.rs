use crate::async_trait;
use async_std::{sync, task};
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
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    source: Option<sync::Receiver<S>>,
    next: Option<sync::Sender<N>>,
    lambda: Box<L>,
}

impl<S, N, L> Cell<S, N, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    pub fn new(lambda: L) -> Self {
        Self {
            source: None,
            next: None,
            lambda: Box::new(lambda),
        }
    }
}

impl<S, N, L> SourceCell<N> for Cell<S, N, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    fn set_next(&mut self, send: sync::Sender<N>) -> Result<(), CellError> {
        if self.next.is_some() {
            Err(CellError::NextExist)?;
        }
        self.next = Some(send);
        Ok(())
    }
}

impl<S, N, L> NextCell<S> for Cell<S, N, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    fn set_source(&mut self, recv: sync::Receiver<S>) -> Result<(), CellError> {
        if self.source.is_some() {
            Err(CellError::SourceExist)?;
        }
        self.source = Some(recv);
        Ok(())
    }
}

#[async_trait]
impl<S, N, L> KosmosCell<S, N> for Cell<S, N, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    fn check_set(&self) -> Result<(), CellError> {
        if self.source.is_none() {
            Err(CellError::SourceNotExist)
        } else if self.next.is_none() {
            Err(CellError::NextNotExist)
        } else {
            Ok(())
        }
    }

    async fn run(&self) -> Result<(), CellError> {
        self.check_set()?;
        if let Some(input) = self.source.as_ref().unwrap().recv().await {
            match self.lambda.call((input,)) {
                Ok(out) => {
                    self.next.as_ref().unwrap().send(out).await;
                }
                Err(e) => {
                    log::error!("Cell Lambda failed: {}", e);
                    Err(e)?;
                }
            }
        }
        Ok(())
    }
}

pub struct TailCell<S, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, ()>,
{
    source: Option<sync::Receiver<S>>,
    lambda: Box<L>,
}

impl<S, L> NextCell<S> for TailCell<S, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, ()>,
{
    fn set_source(&mut self, recv: sync::Receiver<S>) -> Result<(), CellError> {
        if self.source.is_some() {
            Err(CellError::SourceExist)?;
        }
        self.source = Some(recv);
        Ok(())
    }
}

#[async_trait]
impl<S, L> KosmosCell<S, ()> for TailCell<S, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, ()>,
{
    fn check_set(&self) -> Result<(), CellError> {
        if self.source.is_none() {
            Err(CellError::SourceNotExist)
        } else {
            Ok(())
        }
    }

    async fn run(&self) -> Result<(), CellError> {
        self.check_set()?;
        if let Some(input) = self.source.as_ref().unwrap().recv().await {
            match self.lambda.call((input,)) {
                Ok(()) => {}
                Err(e) => {
                    log::error!("Cell Lambda Failed: {}", e);
                    Err(e)?;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait KosmosCell<S, N>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    Self: Sized + Sync + Send + 'static,
{
    fn check_set(&self) -> Result<(), CellError>;

    async fn run(&self) -> Result<(), CellError>;

    async fn spawn_loop(self) -> task::JoinHandle<()> {
        task::spawn(async move {
            loop {
                log::trace!("loop start.");
                self.run().await.unwrap();
                log::trace!("loop ended.");
            }
        })
    }
}

pub trait SourceCell<N>
where
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn set_next(&mut self, send: sync::Sender<N>) -> Result<(), CellError>;

    fn link_to(&mut self, other: &mut dyn NextCell<N>) -> Result<(), CellError> {
        let (s, r) = sync::channel(42);
        self.set_next(s)?;
        other.set_source(r)?;
        Ok(())
    }
}

pub trait NextCell<S>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn set_source(&mut self, recv: sync::Receiver<S>) -> Result<(), CellError>;
}

#[async_trait]
pub trait Lambda<S, N>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    Self: Sync + Sized + Clone + Send + 'static + Fn(S) -> anyhow::Result<N>,
{
}

impl<S, N, F> Lambda<S, N> for F
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    F: Sync + Sized + Clone + Send + 'static + Fn(S) -> anyhow::Result<N>,
{
}
