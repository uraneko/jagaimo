pub(crate) fn read_manifest() -> [String; 2] {
    std::fs::read_to_string("Cargo.toml")
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("name = ") || l.starts_with("version = "))
        .map(|l| l.to_owned())
        .collect::<Vec<String>>()
        .try_into()
        .unwrap()
}

pub(crate) fn crate_name<T: Iterator<Item = String>>(lines: &mut T) -> Option<String> {
    lines.next()
}

pub(crate) fn crate_version<T: Iterator<Item = String>>(lines: &mut T) -> Option<String> {
    lines.next()
}
