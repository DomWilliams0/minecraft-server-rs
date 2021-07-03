pub mod connection;

pub mod config;
pub mod error;
pub mod game;
pub mod packet;
pub mod server;

pub(crate) mod prelude {
    pub use async_std::io::prelude::*;
    pub use async_std::io::Result as IoResult;
    pub use log::*;

    pub use async_trait::async_trait;

    pub use crate::connection::{McRead, McStream, McWrite};
    pub use crate::error::{McError, McResult};
}
