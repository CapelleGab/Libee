use std::path::PathBuf;
use std::time::SystemTime;

use super::ManagerKind;
use crate::shared::Bytes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    Global,
    User,
    System,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Status {
    Active,
    Outdated,
    /// The same package is installed by another manager too.
    Duplicate,
    /// Files left behind with no manager claiming them.
    Orphan,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub manager: ManagerKind,
    pub scope: Scope,
    pub path: PathBuf,
    pub size: Option<Bytes>,
    /// Commands put on the PATH by this package, e.g. `tsc` for `typescript`.
    pub binaries: Vec<String>,
    pub dependencies: Vec<String>,
    pub installed_at: Option<SystemTime>,
    pub status: Status,
}

impl Package {
    pub fn new(manager: ManagerKind, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            manager,
            scope: Scope::Global,
            path: PathBuf::new(),
            size: None,
            binaries: Vec::new(),
            dependencies: Vec::new(),
            installed_at: None,
            status: Status::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Package, Scope, Status};
    use crate::models::ManagerKind;

    #[test]
    fn new_package_defaults_to_global_and_unknown() {
        let package = Package::new(ManagerKind::Npm, "typescript", "5.6.3");

        assert_eq!(package.name, "typescript");
        assert_eq!(package.scope, Scope::Global);
        assert_eq!(package.status, Status::Unknown);
        assert!(package.size.is_none());
    }
}
