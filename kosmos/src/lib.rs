#![feature(async_closure, fn_traits)]
pub use async_trait::async_trait;

pub mod cell;
pub mod prelude {
    pub use super::cell::*;
}
