use super::ManagerError;
use crate::models::{Cache, Manager, ManagerKind, Package};

/// What an adapter can actually do, so the UI never offers an action the
/// manager cannot perform.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub packages: bool,
    pub cache: bool,
    pub uninstall: bool,
    pub dry_run: bool,
}

pub trait PackageManager: Send + Sync {
    fn kind(&self) -> ManagerKind;

    fn capabilities(&self) -> Capabilities;

    /// `Ok(None)` when the manager is not installed: that is not an error.
    fn detect(&self) -> Result<Option<Manager>, ManagerError>;

    fn packages(&self) -> Result<Vec<Package>, ManagerError>;

    fn cache(&self) -> Result<Option<Cache>, ManagerError>;
}

#[cfg(test)]
mod tests {
    use super::{Capabilities, PackageManager};
    use crate::managers::ManagerError;
    use crate::models::{Cache, Manager, ManagerKind, Package};

    struct Absent;

    impl PackageManager for Absent {
        fn kind(&self) -> ManagerKind {
            ManagerKind::Yarn
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities::default()
        }

        fn detect(&self) -> Result<Option<Manager>, ManagerError> {
            Ok(None)
        }

        fn packages(&self) -> Result<Vec<Package>, ManagerError> {
            Ok(Vec::new())
        }

        fn cache(&self) -> Result<Option<Cache>, ManagerError> {
            Ok(None)
        }
    }

    #[test]
    fn adapters_can_be_used_as_trait_objects() {
        let adapters: Vec<Box<dyn PackageManager>> = vec![Box::new(Absent)];

        let detected: Vec<_> = adapters
            .iter()
            .filter_map(|adapter| adapter.detect().ok().flatten())
            .collect();

        assert!(detected.is_empty());
        assert_eq!(adapters[0].kind(), ManagerKind::Yarn);
        assert!(!adapters[0].capabilities().uninstall);
    }
}
