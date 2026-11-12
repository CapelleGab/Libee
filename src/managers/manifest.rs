use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::de::IgnoredAny;

/// The parts of a Node `package.json` Libee cares about. Shared by npm, pnpm and Yarn.
#[derive(Debug, Default, Deserialize)]
pub struct Manifest {
    pub description: Option<String>,
    #[serde(default, deserialize_with = "homepage")]
    pub homepage: Option<String>,
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

/// Some manifests put an object or a list here instead of a URL; ignore those.
fn homepage<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    Ok(match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(url) => Some(url),
        _ => None,
    })
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
            r#"{ "description": "TypeScript compiler", "homepage": "https://www.typescriptlang.org",
                 "bin": { "tsc": "bin/tsc", "tsserver": "bin/tsserver" },
                 "dependencies": { "semver": "^7.0.0" } }"#,
        );

        assert_eq!(manifest.description.as_deref(), Some("TypeScript compiler"));
        assert_eq!(
            manifest.homepage.as_deref(),
            Some("https://www.typescriptlang.org")
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
    fn ignores_non_string_homepage() {
        let manifest = manifest(r#"{ "homepage": { "url": "x" } }"#);

        assert!(manifest.homepage.is_none());
    }

    #[test]
    fn missing_or_invalid_manifest_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(Manifest::read(dir.path()).is_none());

        fs::write(dir.path().join("package.json"), "{ not json").unwrap();
        assert!(Manifest::read(dir.path()).is_none());
    }
}
