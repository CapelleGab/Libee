mod adapter;
mod error;
mod homebrew;
mod manifest;
mod npm;
mod runner;

pub use adapter::{Capabilities, PackageManager};
pub use error::ManagerError;
pub use homebrew::Homebrew;
pub use npm::Npm;
pub use runner::{CommandOutput, CommandRunner, SystemRunner};
