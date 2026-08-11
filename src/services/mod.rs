//! Serviços de aplicação.

mod config;
mod normalize;
mod package_manager;
mod search;

pub use config::*;
pub use normalize::*;
pub use package_manager::*;
pub use search::*;
