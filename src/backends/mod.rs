//! Backends de pacotes.

mod trait_backend;
pub mod apt;
pub mod dnf;
pub mod snap;
pub mod flatpak;
pub mod appimage;
pub mod manual;
pub mod mock;

pub use trait_backend::*;
