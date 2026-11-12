use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Deserialize;

use super::{Capabilities, CommandRunner, ManagerError, PackageManager, SystemRunner};
use crate::models::{Cache, Manager, ManagerKind, Package, PackageKind, Status};
use crate::shared::CommandSpec;

pub struct Homebrew<R = SystemRunner> {
    runner: R,
}

impl Homebrew {
    pub fn new() -> Self {
        Self::with_runner(SystemRunner)
    }
}

impl Default for Homebrew {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: CommandRunner> Homebrew<R> {
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }

    fn command(args: &[&str]) -> CommandSpec {
        CommandSpec::new(ManagerKind::Homebrew.executable()).args(args.iter().copied())
    }

    fn query(&self, args: &[&str]) -> Result<String, ManagerError> {
        Ok(self.runner.stdout(&Self::command(args))?.trim().to_owned())
    }

    fn prefix(&self) -> Result<PathBuf, ManagerError> {
        self.query(&["--prefix"]).map(PathBuf::from)
    }

    fn installed(&self) -> Result<Installed, ManagerError> {
        let command = Self::command(&["info", "--json=v2", "--installed"]);
        let stdout = self.runner.stdout(&command)?;

        serde_json::from_str(&stdout).map_err(|error| ManagerError::UnexpectedOutput {
            command: command.to_string(),
            reason: error.to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct Installed {
    #[serde(default)]
    formulae: Vec<Formula>,
    #[serde(default)]
    casks: Vec<Cask>,
}

#[derive(Debug, Deserialize)]
struct Formula {
    name: String,
    desc: Option<String>,
    homepage: Option<String>,
    #[serde(default)]
    installed: Vec<Keg>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    outdated: bool,
    linked_keg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Keg {
    version: String,
    time: Option<u64>,
    #[serde(default)]
    installed_on_request: bool,
}

#[derive(Debug, Deserialize)]
struct Cask {
    token: String,
    #[serde(default)]
    name: Vec<String>,
    desc: Option<String>,
    homepage: Option<String>,
    installed: Option<String>,
    installed_time: Option<u64>,
    #[serde(default)]
    outdated: bool,
}

impl Formula {
    fn into_package(self, prefix: &Path) -> Option<Package> {
        let keg = match &self.linked_keg {
            Some(linked) => self.installed.iter().find(|keg| &keg.version == linked),
            None => self.installed.last(),
        }?;

        let path = prefix.join("Cellar").join(&self.name).join(&keg.version);
        let mut package = Package::new(ManagerKind::Homebrew, &self.name, &keg.version);
        package.binaries = binaries_in(&path.join("bin"));
        package.installed_at = keg.time.map(timestamp);
        package.explicit = Some(keg.installed_on_request);
        package.status = status(self.outdated);
        package.description = self.desc;
        package.homepage = self.homepage;
        package.dependencies = self.dependencies;
        package.path = path;
        Some(package)
    }
}

impl Cask {
    fn into_package(self, prefix: &Path) -> Option<Package> {
        let version = self.installed?;
        let path = prefix.join("Caskroom").join(&self.token).join(&version);

        let mut package = Package::new(ManagerKind::Homebrew, &self.token, version);
        package.kind = PackageKind::App;
        package.installed_at = self.installed_time.map(timestamp);
        package.explicit = Some(true);
        package.status = status(self.outdated);
        package.description = self.desc.or_else(|| self.name.into_iter().next());
        package.homepage = self.homepage;
        package.path = path;
        Some(package)
    }
}

fn status(outdated: bool) -> Status {
    if outdated {
        Status::Outdated
    } else {
        Status::Active
    }
}

fn timestamp(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn binaries_in(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort_unstable();
    names
}

impl<R: CommandRunner> PackageManager for Homebrew<R> {
    fn kind(&self) -> ManagerKind {
        ManagerKind::Homebrew
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            packages: true,
            cache: true,
            uninstall: true,
            dry_run: false,
        }
    }

    fn detect(&self) -> Result<Option<Manager>, ManagerError> {
        let banner = match self.query(&["--version"]) {
            Ok(banner) => banner,
            Err(ManagerError::NotInstalled(_)) => return Ok(None),
            Err(error) => return Err(error),
        };

        let version = banner
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("Homebrew "))
            .and_then(|version| version.split('-').next())
            .ok_or_else(|| ManagerError::UnexpectedOutput {
                command: Self::command(&["--version"]).to_string(),
                reason: format!("no version in {banner:?}"),
            })?;

        Ok(Some(Manager {
            kind: ManagerKind::Homebrew,
            version: version.to_owned(),
            executable: which::which(ManagerKind::Homebrew.executable())
                .unwrap_or_else(|_| PathBuf::from(ManagerKind::Homebrew.executable())),
            install_root: self.prefix().ok(),
            config_files: Vec::new(),
        }))
    }

    fn packages(&self) -> Result<Vec<Package>, ManagerError> {
        let prefix = self.prefix()?;
        let installed = self.installed()?;

        let formulae = installed
            .formulae
            .into_iter()
            .filter_map(|formula| formula.into_package(&prefix));
        let casks = installed
            .casks
            .into_iter()
            .filter_map(|cask| cask.into_package(&prefix));

        Ok(formulae.chain(casks).collect())
    }

    fn cache(&self) -> Result<Option<Cache>, ManagerError> {
        Ok(Some(Cache {
            manager: ManagerKind::Homebrew,
            location: PathBuf::from(self.query(&["--cache"])?),
            size: None,
            cleanup: Some(Self::command(&["cleanup", "--prune=all"])),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, UNIX_EPOCH};

    use super::Homebrew;
    use crate::managers::runner::fake::FakeRunner;
    use crate::managers::{ManagerError, PackageManager};
    use crate::models::{PackageKind, Status};

    const INSTALLED: &str = r#"{
      "formulae": [
        {
          "name": "ripgrep",
          "desc": "Search tool like grep and The Silver Searcher",
          "homepage": "https://github.com/BurntSushi/ripgrep",
          "installed": [
            { "version": "14.1.0", "time": 1700000000, "installed_on_request": true },
            { "version": "14.1.1", "time": 1780000000, "installed_on_request": true }
          ],
          "dependencies": ["pcre2"],
          "linked_keg": "14.1.1",
          "outdated": false
        },
        {
          "name": "pcre2",
          "desc": "Perl compatible regular expressions library",
          "installed": [{ "version": "10.44", "time": 1780000000, "installed_on_request": false }],
          "dependencies": [],
          "linked_keg": null,
          "outdated": true
        },
        { "name": "ghost", "installed": [], "dependencies": [] }
      ],
      "casks": [
        {
          "token": "visual-studio-code",
          "name": ["Microsoft Visual Studio Code"],
          "desc": null,
          "homepage": "https://code.visualstudio.com/",
          "installed": "1.95.3",
          "installed_time": 1790000000,
          "outdated": false
        }
      ]
    }"#;

    fn brew(prefix: &Path) -> Homebrew<FakeRunner> {
        let runner = FakeRunner::default()
            .with("brew --version", "Homebrew 7.0.13-77-g8679d30\n")
            .with("brew --prefix", &format!("{}\n", prefix.display()))
            .with("brew --cache", "/Users/dev/Library/Caches/Homebrew\n")
            .with("brew info --json=v2 --installed", INSTALLED);
        Homebrew::with_runner(runner)
    }

    #[test]
    fn missing_brew_is_not_detected() {
        assert!(
            Homebrew::with_runner(FakeRunner::default())
                .detect()
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn detects_clean_version_and_prefix() {
        let manager = brew(Path::new("/opt/homebrew")).detect().unwrap().unwrap();

        assert_eq!(manager.version, "7.0.13");
        assert_eq!(manager.install_root.unwrap(), Path::new("/opt/homebrew"));
    }

    #[test]
    fn rejects_unknown_version_banner() {
        let runner = FakeRunner::default().with("brew --version", "Linuxbrew?\n");

        let error = Homebrew::with_runner(runner).detect().unwrap_err();

        assert!(matches!(error, ManagerError::UnexpectedOutput { .. }));
    }

    #[test]
    fn lists_formulae_with_linked_version_and_binaries() {
        let prefix = tempfile::tempdir().unwrap();
        let bin = prefix.path().join("Cellar/ripgrep/14.1.1/bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("rg"), "").unwrap();

        let packages = brew(prefix.path()).packages().unwrap();
        let ripgrep = &packages[0];

        assert_eq!(ripgrep.version, "14.1.1");
        assert_eq!(ripgrep.binaries, ["rg"]);
        assert_eq!(ripgrep.dependencies, ["pcre2"]);
        assert_eq!(ripgrep.explicit, Some(true));
        assert_eq!(ripgrep.status, Status::Active);
        assert_eq!(
            ripgrep.installed_at,
            Some(UNIX_EPOCH + Duration::from_secs(1_780_000_000))
        );
        assert_eq!(ripgrep.path, prefix.path().join("Cellar/ripgrep/14.1.1"));
    }

    #[test]
    fn marks_dependencies_and_outdated_formulae() {
        let packages = brew(Path::new("/opt/homebrew")).packages().unwrap();
        let pcre2 = &packages[1];

        assert_eq!(pcre2.explicit, Some(false));
        assert_eq!(pcre2.status, Status::Outdated);
    }

    #[test]
    fn skips_formulae_without_installed_keg() {
        let packages = brew(Path::new("/opt/homebrew")).packages().unwrap();

        assert!(packages.iter().all(|package| package.name != "ghost"));
    }

    #[test]
    fn lists_casks_as_apps() {
        let packages = brew(Path::new("/opt/homebrew")).packages().unwrap();
        let vscode = packages.last().unwrap();

        assert_eq!(vscode.name, "visual-studio-code");
        assert_eq!(vscode.kind, PackageKind::App);
        assert_eq!(
            vscode.description.as_deref(),
            Some("Microsoft Visual Studio Code")
        );
        assert_eq!(
            vscode.path,
            Path::new("/opt/homebrew/Caskroom/visual-studio-code/1.95.3")
        );
    }

    #[test]
    fn exposes_cache_with_cleanup_command() {
        let cache = brew(Path::new("/opt/homebrew")).cache().unwrap().unwrap();

        assert_eq!(
            cache.location,
            Path::new("/Users/dev/Library/Caches/Homebrew")
        );
        assert_eq!(
            cache.cleanup.unwrap().to_string(),
            "brew cleanup --prune=all"
        );
    }
}
