mod adapter;
mod error;
mod runner;

pub use adapter::{Capabilities, PackageManager};
pub use error::ManagerError;
pub use runner::{CommandOutput, CommandRunner, SystemRunner};
