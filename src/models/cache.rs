use std::path::PathBuf;

use super::ManagerKind;
use crate::shared::{Bytes, CommandSpec};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cache {
    pub manager: ManagerKind,
    pub location: PathBuf,
    pub size: Option<Bytes>,
    /// The manager's own cleanup command; `None` when it offers none.
    pub cleanup: Option<CommandSpec>,
}
