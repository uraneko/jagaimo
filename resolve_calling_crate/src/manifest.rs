#![cfg(feature = "manifest")]
use crate::ResolveCrate;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ReadManifest {
    manifest: String,
}

impl From<&ResolveCrate> for ReadManifest {
    fn from(resolver: &ResolveCrate) -> Self {
        Self {
            manifest: std::fs::read_to_string(resolver.to_string_suffixed("Cargo.toml")).unwrap(),
        }
    }
}

impl From<ResolveCrate> for ReadManifest {
    fn from(resolver: ResolveCrate) -> Self {
        Self {
            manifest: std::fs::read_to_string(resolver.into_string_suffixed("Cargo.toml")).unwrap(),
        }
    }
}

impl ReadManifest {
    pub fn crate_name_version(&self) -> [String; 2] {
        let mut iter = self
            .manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());

        // NOTE could do collect -> try_into
        // but "name =" may exist for every  [[bin]] [[test]] or [[example]]
        // and there is no guarentee that there would be no second version key somewhere
        [iter.next().unwrap(), iter.next().unwrap()]
    }

    pub fn crate_name(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("name = "))
            .map(|l| &l[8..l.len() - 1])
            .unwrap()
    }

    pub fn crate_version(&self) -> &str {
        self.manifest
            .lines()
            .find(|l| l.starts_with("version = "))
            .map(|l| &l[10..l.len() - 1])
            .unwrap()
    }

    pub fn as_str(&self) -> &str {
        &self.manifest
    }

    pub fn to_string(&self) -> String {
        self.manifest.clone()
    }
}

impl From<ReadManifest> for String {
    fn from(value: ReadManifest) -> String {
        value.manifest
    }
}

impl std::fmt::Display for ReadManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.manifest)
    }
}
