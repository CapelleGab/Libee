use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ManagerKind {
    Homebrew,
    Npm,
    Pnpm,
    Yarn,
}

impl ManagerKind {
    pub const ALL: [Self; 4] = [Self::Homebrew, Self::Npm, Self::Pnpm, Self::Yarn];

    pub fn name(self) -> &'static str {
        match self {
            Self::Homebrew => "Homebrew",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "Yarn",
        }
    }

    pub fn executable(self) -> &'static str {
        match self {
            Self::Homebrew => "brew",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Yarn => "yarn",
        }
    }
}

impl fmt::Display for ManagerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A package manager found on the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manager {
    pub kind: ManagerKind,
    pub version: String,
    pub executable: PathBuf,
    /// Where the manager stores what it installs, e.g. `$(brew --prefix)` or the npm global root.
    pub install_root: Option<PathBuf>,
    pub config_files: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::ManagerKind;

    #[test]
    fn every_kind_has_a_distinct_executable() {
        let mut executables: Vec<_> = ManagerKind::ALL
            .iter()
            .map(|kind| kind.executable())
            .collect();
        executables.sort_unstable();
        executables.dedup();

        assert_eq!(executables.len(), ManagerKind::ALL.len());
    }

    #[test]
    fn displays_the_human_name() {
        assert_eq!(ManagerKind::Homebrew.to_string(), "Homebrew");
        assert_eq!(ManagerKind::Pnpm.to_string(), "pnpm");
    }
}
