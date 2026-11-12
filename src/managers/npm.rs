use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use super::manifest::Manifest;
use super::{Capabilities, CommandRunner, ManagerError, PackageManager, SystemRunner};
use crate::models::{Cache, Manager, ManagerKind, Package};
use crate::shared::CommandSpec;

pub struct Npm<R = SystemRunner> {
    runner: R,
}

impl Npm {
    pub fn new() -> Self {
        Self::with_runner(SystemRunner)
    }
}

impl Default for Npm {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: CommandRunner> Npm<R> {
    pub fn with_runner(runner: R) -> Self {
        Self { runner }
    }

    fn command(args: &[&str]) -> CommandSpec {
        CommandSpec::new(ManagerKind::Npm.executable()).args(args.iter().copied())
    }

    fn query(&self, args: &[&str]) -> Result<String, ManagerError> {
        Ok(self.runner.stdout(&Self::command(args))?.trim().to_owned())
    }

    fn global_root(&self) -> Result<PathBuf, ManagerError> {
        self.query(&["root", "-g"]).map(PathBuf::from)
    }

    /// `npm ls` exits with an error when the tree has problems (missing peers,
    /// invalid versions) but still prints the full JSON, so the output is kept.
    fn list_global(&self) -> Result<GlobalList, ManagerError> {
        let command = Self::command(&["ls", "-g", "--json", "--depth=0"]);
        let output = self.runner.run(&command)?;

        if output.stdout.trim().is_empty() {
            return Err(ManagerError::CommandFailed {
                command: command.to_string(),
                code: output.code,
                stderr: output.stderr.trim().to_owned(),
            });
        }

        serde_json::from_str(&output.stdout).map_err(|error| ManagerError::UnexpectedOutput {
            command: command.to_string(),
            reason: error.to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct GlobalList {
    #[serde(default)]
    dependencies: BTreeMap<String, GlobalEntry>,
}

#[derive(Debug, Deserialize)]
struct GlobalEntry {
    version: Option<String>,
}

impl<R: CommandRunner> PackageManager for Npm<R> {
    fn kind(&self) -> ManagerKind {
        ManagerKind::Npm
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            packages: true,
            cache: true,
            uninstall: true,
            dry_run: true,
        }
    }

    fn detect(&self) -> Result<Option<Manager>, ManagerError> {
        let version = match self.query(&["--version"]) {
            Ok(version) => version,
            Err(ManagerError::NotInstalled(_)) => return Ok(None),
            Err(error) => return Err(error),
        };

        let config_files = self
            .query(&["config", "get", "userconfig"])
            .map(PathBuf::from)
            .into_iter()
            .filter(|path| path.is_file())
            .collect();

        Ok(Some(Manager {
            kind: ManagerKind::Npm,
            version,
            executable: which::which(ManagerKind::Npm.executable())
                .unwrap_or_else(|_| PathBuf::from(ManagerKind::Npm.executable())),
            install_root: self.global_root().ok(),
            config_files,
        }))
    }

    fn packages(&self) -> Result<Vec<Package>, ManagerError> {
        let root = self.global_root()?;
        let list = self.list_global()?;

        Ok(list
            .dependencies
            .into_iter()
            .map(|(name, entry)| {
                let path = root.join(&name);
                let manifest = Manifest::read(&path).unwrap_or_default();

                let mut package =
                    Package::new(ManagerKind::Npm, &name, entry.version.unwrap_or_default());
                package.binaries = manifest.binaries(&name);
                package.dependencies = manifest.dependencies();
                package.installed_at = fs::metadata(&path).and_then(|meta| meta.created()).ok();
                package.path = path;
                package
            })
            .collect())
    }

    fn cache(&self) -> Result<Option<Cache>, ManagerError> {
        let location = PathBuf::from(self.query(&["config", "get", "cache"])?);

        Ok(Some(Cache {
            manager: ManagerKind::Npm,
            location,
            size: None,
            cleanup: Some(Self::command(&["cache", "clean", "--force"])),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::Npm;
    use crate::managers::runner::fake::FakeRunner;
    use crate::managers::{ManagerError, PackageManager};

    const LS_GLOBAL: &str = r#"{
      "name": "lib",
      "dependencies": {
        "@angular/cli": { "version": "19.0.0", "overridden": false },
        "corepack": { "version": "0.36.0", "overridden": false },
        "typescript": { "version": "7.0.2", "overridden": false }
      }
    }"#;

    fn npm_with_root(root: &Path) -> Npm<FakeRunner> {
        let runner = FakeRunner::default()
            .with("npm --version", "11.19.0\n")
            .with("npm root -g", &format!("{}\n", root.display()))
            .with("npm config get userconfig", "/nowhere/.npmrc\n")
            .with("npm config get cache", "/home/dev/.npm\n")
            .with("npm ls -g --json --depth=0", LS_GLOBAL);
        Npm::with_runner(runner)
    }

    #[test]
    fn missing_npm_is_not_detected() {
        let npm = Npm::with_runner(FakeRunner::default());

        assert!(npm.detect().unwrap().is_none());
    }

    #[test]
    fn detects_version_and_global_root() {
        let npm = npm_with_root(Path::new("/home/dev/.nvm/lib/node_modules"));

        let manager = npm.detect().unwrap().unwrap();

        assert_eq!(manager.version, "11.19.0");
        assert_eq!(
            manager.install_root.unwrap(),
            Path::new("/home/dev/.nvm/lib/node_modules")
        );
        assert!(manager.config_files.is_empty());
    }

    #[test]
    fn lists_global_packages_with_manifest_details() {
        let root = tempfile::tempdir().unwrap();
        let typescript = root.path().join("typescript");
        fs::create_dir_all(&typescript).unwrap();
        fs::write(
            typescript.join("package.json"),
            r#"{ "bin": { "tsc": "bin/tsc" }, "dependencies": { "semver": "^7" } }"#,
        )
        .unwrap();

        let packages = npm_with_root(root.path()).packages().unwrap();

        let names: Vec<_> = packages
            .iter()
            .map(|package| package.name.as_str())
            .collect();
        assert_eq!(names, ["@angular/cli", "corepack", "typescript"]);

        let typescript = &packages[2];
        assert_eq!(typescript.version, "7.0.2");
        assert_eq!(typescript.binaries, ["tsc"]);
        assert_eq!(typescript.dependencies, ["semver"]);
        assert_eq!(typescript.path, root.path().join("typescript"));
        assert!(typescript.installed_at.is_some());

        assert!(packages[0].binaries.is_empty(), "no manifest on disk");
    }

    #[test]
    fn keeps_the_list_when_npm_reports_tree_problems() {
        let mut runner = FakeRunner::default().with("npm root -g", "/tmp/libee-none\n");
        runner =
            runner.failing_with_stdout("npm ls -g --json --depth=0", 1, LS_GLOBAL, "ELSPROBLEMS");

        let packages = Npm::with_runner(runner).packages().unwrap();

        assert_eq!(packages.len(), 3);
    }

    #[test]
    fn rejects_unexpected_output() {
        let runner = FakeRunner::default()
            .with("npm root -g", "/tmp\n")
            .with("npm ls -g --json --depth=0", "not json");

        let error = Npm::with_runner(runner).packages().unwrap_err();

        assert!(matches!(error, ManagerError::UnexpectedOutput { .. }));
    }

    #[test]
    fn exposes_cache_with_cleanup_command() {
        let cache = npm_with_root(Path::new("/tmp")).cache().unwrap().unwrap();

        assert_eq!(cache.location, Path::new("/home/dev/.npm"));
        assert_eq!(
            cache.cleanup.unwrap().to_string(),
            "npm cache clean --force"
        );
    }
}
