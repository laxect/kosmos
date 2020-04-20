use crate::{async_trait, cell::*};
use async_std::{sync, task};
use serde::{de::DeserializeOwned, Serialize};
use std::time;

pub struct CronCell<Output, L, T>
where
    Output: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<(), Output>,
    T: Fn() -> time::Duration,
{
    next: Option<sync::Sender<Output>>,
    lambda: Box<L>,
    scheduler: Box<T>,
}

impl<Output, L, T> CronCell<Output, L, T>
where
    Output: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<(), Output>,
    T: Fn() -> time::Duration,
{
    pub fn new(lambda: L, scheduler: T) -> Self {
        Self {
            next: None,
            lambda: Box::new(lambda),
            scheduler: Box::new(scheduler),
        }
    }
}

impl<Output, L, T> SourceCell<Output> for CronCell<Output, L, T>
where
    Output: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<(), Output>,
    T: Fn() -> time::Duration,
{
    fn set_next(&mut self, next: sync::Sender<Output>) -> Result<(), CellError> {
        if self.next.is_some() {
            Err(CellError::NextExist)?;
        }
        self.next = Some(next);
        Ok(())
    }
}

#[async_trait]
impl<Output, L, T> KosmosCell<(), Output> for CronCell<Output, L, T>
where
    Output: Serialize + DeserializeOwned + Send + Sync + 'static,
    L: Lambda<(), Output>,
    T: Send + Sync + 'static + Fn() -> time::Duration,
{
    fn check_set(&self) -> Result<(), CellError> {
        if self.next.is_none() {
            Err(CellError::NextNotExist)
        } else {
            Ok(())
        }
    }

    async fn run(&self) -> Result<(), CellError> {
        self.check_set()?;
        match self.lambda.call(((),)) {
            Ok(output) => {
                self.next.as_ref().unwrap().send(output).await;
            }
            Err(e) => {
                log::error!("Cell Lambda Failed: {}", e);
                Err(e)?;
            }
        }
        Ok(())
    }

    async fn spawn_loop(self) -> task::JoinHandle<()> {
        task::spawn(async move {
            loop {
                log::trace!("loop start.");
                let dur = self.scheduler.call(());
                task::sleep(dur).await;
                self.run().await.unwrap();
                log::trace!("loop ended.");
            }
        })
    }
}
