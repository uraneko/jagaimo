use crate::parse::Aliased;
use std::collections::HashMap;

// TODO the help struct and all its children types
// can be generated from the raw command tree
// then toml parse is called on the help.toml file into the Help struct
// the help messages are generated for those types
// if this approach is taken
// then need a Help trait to be implemented on all the types inside Help struct as well as the Help
// struct itself

pub struct Help {
    main: Main,
    links: Links,
    tokens: HashMap<String, String>,
    spaces: Spaces,
    operations: Operations,
    flags: Flags,
}

struct Links {
    issue_tracker: Option<String>,
    src_code: Option<String>,
    website: Option<String>,
    extra: Option<HashMap<String, String>>,
}

struct Main {
    decription: Option<String>,
}

struct Spaces {}

struct Operations {}

struct Flags {}
