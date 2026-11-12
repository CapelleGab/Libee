use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::de::IgnoredAny;

/// The parts of a Node `package.json` Libee cares about. Shared by npm, pnpm and Yarn.
#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    bin: Option<Bin>,
    #[serde(default)]
    dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Bin {
    Named(BTreeMap<String, String>),
    Single(IgnoredAny),
}

impl Manifest {
    /// A missing or broken manifest must not abort the whole scan.
    pub fn read(package_dir: &Path) -> Option<Self> {
        let content = fs::read_to_string(package_dir.join("package.json")).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn binaries(&self, package_name: &str) -> Vec<String> {
        match &self.bin {
            Some(Bin::Named(bins)) => bins.keys().cloned().collect(),
            Some(Bin::Single(_)) => {
                let unscoped = package_name.rsplit('/').next().unwrap_or(package_name);
                vec![unscoped.to_owned()]
            }
            None => Vec::new(),
        }
    }

    pub fn dependencies(&self) -> Vec<String> {
        self.dependencies.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Manifest;

    fn manifest(json: &str) -> Manifest {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("package.json"), json).unwrap();
        Manifest::read(dir.path()).unwrap()
    }

    #[test]
    fn reads_named_binaries_and_dependencies() {
        let manifest = manifest(
            r#"{ "bin": { "tsc": "bin/tsc", "tsserver": "bin/tsserver" },
                 "dependencies": { "semver": "^7.0.0" } }"#,
        );

        assert_eq!(manifest.binaries("typescript"), ["tsc", "tsserver"]);
        assert_eq!(manifest.dependencies(), ["semver"]);
    }

    #[test]
    fn single_binary_is_named_after_the_unscoped_package() {
        let manifest = manifest(r#"{ "bin": "./cli.js" }"#);

        assert_eq!(manifest.binaries("@angular/cli"), ["cli"]);
    }

    #[test]
    fn missing_or_invalid_manifest_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Manifest::read(dir.path()).is_none());

        fs::write(dir.path().join("package.json"), "{ not json").unwrap();
        assert!(Manifest::read(dir.path()).is_none());
    }
}
