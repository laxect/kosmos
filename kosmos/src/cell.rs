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

#[async_trait]
impl<S, N, L> KosmosCell<S, N> for Cell<S, N, L>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<S, N>,
{
    fn set_source(&mut self, source: sync::Receiver<S>) -> Result<(), CellError> {
        if self.source.is_some() {
            Err(CellError::SourceExist)?;
        }
        self.source = Some(source);
        Ok(())
    }

    fn set_next(&mut self, next: sync::Sender<N>) -> Result<(), CellError> {
        if self.next.is_some() {
            Err(CellError::NextExist)?;
        }
        self.next = Some(next);
        Ok(())
    }

    async fn recv(&self) -> Option<S> {
        self.source.as_ref().unwrap().recv().await
    }

    async fn send(&self, pkg: N) {
        self.next.as_ref().map(async move |s| s.send(pkg).await);
    }

    fn check_set(&self) -> Result<(), CellError> {
        if self.source.is_none() {
            Err(CellError::SourceNotExist)?;
        }
        if self.next.is_none() {
            Err(CellError::NextNotExist)?;
        }
        Ok(())
    }

    async fn lambda(&self, input: S) -> anyhow::Result<N> {
        self.lambda.run(input).await
    }
}

#[async_trait]
pub trait KosmosCell<S, N>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    Self: Sized + Sync + Send + 'static,
{
    fn set_source(&mut self, source: sync::Receiver<S>) -> Result<(), CellError>;

    fn set_next(&mut self, next: sync::Sender<N>) -> Result<(), CellError>;

    async fn recv(&self) -> Option<S>;

    async fn send(&self, pkg: N);

    async fn lambda(&self, input: S) -> anyhow::Result<N>;

    fn link_to<ON, OL, C>(&mut self, other: &mut C) -> Result<(), CellError>
    where
        C: KosmosCell<N, ON>,
        ON: Serialize + DeserializeOwned + Send + Sync + 'static,
        OL: Lambda<N, ON>,
    {
        let (s, r) = sync::channel(20);
        self.set_next(s)?;
        other.set_source(r)?;
        Ok(())
    }

    fn check_set(&self) -> Result<(), CellError>;

    async fn run(&self) -> Result<(), CellError> {
        self.check_set()?;
        let input = self.recv().await;
        if let Some(i) = input {
            match self.lambda(i).await {
                Ok(n) => self.send(n).await,
                Err(e) => {
                    log::error!("Cell Error: {}", e);
                    Err(CellError::from(e))?;
                }
            }
        }
        Ok(())
    }

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

#[async_trait]
pub trait Lambda<S, N>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
    N: Serialize + DeserializeOwned + Send + Sync + 'static,
    Self: Sync + Sized + Clone + Send + 'static,
{
    async fn run(&self, input: S) -> anyhow::Result<N>;
}

pub struct NullCell<S>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    source: Option<sync::Receiver<S>>,
    next: Option<sync::Sender<()>>,
}

#[async_trait]
impl<S> KosmosCell<S, ()> for NullCell<S>
where
    S: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    fn set_source(&mut self, source: sync::Receiver<S>) -> Result<(), CellError> {
        if self.source.is_some() {
            Err(CellError::SourceExist)?;
        }
        self.source = Some(source);
        Ok(())
    }

    fn set_next(&mut self, next: sync::Sender<()>) -> Result<(), CellError> {
        if self.next.is_some() {
            Err(CellError::NextExist)?;
        }
        self.next = Some(next);
        Ok(())
    }

    async fn recv(&self) -> Option<S> {
        self.source.as_ref().unwrap().recv().await
    }

    async fn send(&self, _pkg: ()) {
        // Null Cell never send
    }

    fn check_set(&self) -> Result<(), CellError> {
        if self.source.is_none() {
            Err(CellError::SourceNotExist)?;
        }
        // null Cell can have no next
        Ok(())
    }

    async fn lambda(&self, _input: S) -> anyhow::Result<()> {
        log::trace!("Null lambda.");
        Ok(())
    }
}
