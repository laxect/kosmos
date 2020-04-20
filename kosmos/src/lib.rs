#![feature(async_closure, fn_traits)]
pub use async_trait::async_trait;
pub use cell::*;
pub use cron::CronCell;
pub use surf;

mod cell;
mod cron;
pub mod plugin;

pub mod prelude {
    pub use crate::{cell::*, plugin::store, surf};
}
