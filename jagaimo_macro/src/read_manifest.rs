use std::env::{current_dir as cwd, var};
use std::fs::read_to_string;

// this is crate wide public so that
// if I needed to get the manifest and reuse it multiple times,
// i would use this with ref returning fns
pub(crate) struct CrateResolver {
    // running in workspace or not
    // is_workspace: bool,
    // name of the crate being compiled
    // who: String,
    // manifest of the compile_who crate
    manifest: String,
}

impl CrateResolver {
    fn resolve() -> Self {
        let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
        if manifest.contains("[workspace]") {
            let compile_who = std::env::var("CARGO_CRATE_NAME").unwrap();
            let manifest = std::fs::read_to_string(compile_who + "/Cargo.toml").unwrap();
            Self { manifest }
        } else {
            Self { manifest }
        }
    }

    fn into_manifest(self) -> String {
        self.manifest
    }

    fn as_manifest(&self) -> &str {
        &self.manifest
    }
}

pub(crate) struct ManifestReader;

impl ManifestReader {
    fn crate_name_version() -> [String; 2] {
        let manifest = CrateResolver::resolve().into_manifest();
        let mut iter = manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());

        [iter.next().unwrap(), iter.next().unwrap()]
    }

    pub(crate) fn crate_name() -> String {
        let manifest = CrateResolver::resolve().into_manifest();
        let mut iter = manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());

        iter.next().unwrap()
    }

    pub(crate) fn crate_version() -> String {
        let manifest = CrateResolver::resolve().into_manifest();
        let mut iter = manifest
            .lines()
            .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
            .map(|l| l.to_owned());
        iter.next();

        iter.next().unwrap()
    }
}
