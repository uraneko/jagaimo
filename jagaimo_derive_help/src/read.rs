use std::collections::HashMap;

use syn::Ident;
use toml::Table;

use crate::styled::Styled;

use resolve_calling_crate::ResolveCrate;

pub struct RootHelp {
    pub links: Option<HashMap<String, String>>,
    pub description: String,
    pub spaces: Option<HashMap<String, String>>,
    pub ops: Option<HashMap<String, String>>,
    pub flags: Option<HashMap<String, String>>,
}

pub struct SpaceHelp {
    pub links: Option<HashMap<String, String>>,
    pub description: String,
    pub ops: Option<HashMap<String, String>>,
    pub flags: Option<HashMap<String, String>>,
}

pub struct OpHelp {
    pub links: Option<HashMap<String, String>>,
    pub description: String,
    pub flags: Option<HashMap<String, String>>,
}

pub trait ExtractHelp<T> {
    fn links(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn description(&self) -> String;

    fn spaces(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        None
    }

    fn extract(self) -> T;
}

// FIXME this needs to be in snake case
pub struct Excavator {
    toml: toml::Table,
    space: Option<Ident>,
    op: Option<Ident>,
}

impl Excavator {
    pub fn new(space: Option<Ident>, op: Option<Ident>, toml: toml::Table) -> Self {
        Self { toml, space, op }
    }
}

fn hashmap_from_table(table: Option<&Table>) -> Option<HashMap<String, String>> {
    if table.is_none() {
        return None;
    }

    Some(
        table
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.as_str().map(|v| v.to_owned())))
            .filter(|(_, v)| v.is_some())
            .map(|(k, v)| (k, v.unwrap()))
            .collect(),
    )
}

fn op_fallback_desc(space: &Ident, op: &Ident) -> String {
    format!("the {} operation from the {} namespace", op, space)
}

impl ExtractHelp<OpHelp> for Excavator {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let [space, op] = [
            self.space.as_ref().map(|s| s.to_string()).unwrap(),
            self.op.as_ref().map(|o| o.to_string()).unwrap(),
        ];

        let space = links.get(space);
        if let Some(tbl) = space {
            if let Some(op) = tbl.get(&op) {
                return hashmap_from_table(op.as_table());
            }
        } else {
            let fallback = links.get("_").map(|s| s.get(&op).map(|v| v.as_table()));
            if let Some(Some(t)) = fallback {
                return hashmap_from_table(t);
            }
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        let [space, op] = [
            self.space.as_ref().map(|s| s.to_string()).unwrap(),
            self.op.as_ref().map(|o| o.to_string()).unwrap(),
        ];

        if let Some(ops) = flags.get(space) {
            if let Some(op) = ops.get(&op) {
                return hashmap_from_table(op.as_table());
            }
        } else {
            let fallback = flags.get("_").map(|s| s.get(&op).map(|v| v.as_table()));
            if let Some(Some(t)) = fallback {
                return hashmap_from_table(t);
            }
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descriptions");
        if descs.is_none() {
            return op_fallback_desc(self.space.as_ref().unwrap(), self.op.as_ref().unwrap());
        }
        let descs = descs.unwrap();

        let [space, op] = [
            self.space.as_ref().map(|s| s.to_string()).unwrap(),
            self.op.as_ref().map(|o| o.to_string()).unwrap(),
        ];

        if let Some(ops) = descs.get(space) {
            if let Some(op) = ops.get(&op) {
                return op.as_str().map(|s| s.to_owned()).unwrap();
            }
        }

        descs
            .get("_")
            .map(|descs| {
                descs
                    .get(&op)
                    .map(|s| s.as_str().unwrap().to_owned())
                    .unwrap_or_else(|| {
                        op_fallback_desc(self.space.as_ref().unwrap(), self.op.as_ref().unwrap())
                    })
            })
            .unwrap_or_else(|| {
                op_fallback_desc(self.space.as_ref().unwrap(), self.op.as_ref().unwrap())
            })
    }

    fn extract(self) -> OpHelp {
        let description = <Excavator as ExtractHelp<OpHelp>>::description(&self);
        let flags = <Self as ExtractHelp<OpHelp>>::flags(&self);
        let links = <Self as ExtractHelp<OpHelp>>::links(&self);

        OpHelp {
            description,
            links,
            flags,
        }
    }
}

fn space_fallback_desc(space: &Ident) -> String {
    format!("the {} namespace", space)
}

impl<'a> ExtractHelp<SpaceHelp> for Excavator {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let space = self.space.as_ref().map(|s| s.to_string()).unwrap();

        if let Some(tbl) = links.get(space) {
            return hashmap_from_table(tbl.as_table());
        }

        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        let ops = self.toml.get("ops");
        if ops.is_none() {
            return None;
        }
        let ops = ops.unwrap();

        let space = self.space.as_ref().map(|s| s.to_string()).unwrap();
        if let Some(ops) = ops.get(space) {
            return hashmap_from_table(ops.as_table());
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        let space = self.space.as_ref().map(|s| s.to_string()).unwrap();

        if let Some(flags) = flags.get(space) {
            return hashmap_from_table(flags.as_table());
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descriptions");
        if descs.is_none() {
            return space_fallback_desc(self.space.as_ref().unwrap());
        }
        let descs = descs.unwrap();

        let space = self.space.as_ref().map(|s| s.to_string()).unwrap();

        if let Some(desc) = descs.get(space) {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        space_fallback_desc(self.space.as_ref().unwrap())
    }

    fn extract(self) -> SpaceHelp {
        let description = <Excavator as ExtractHelp<SpaceHelp>>::description(&self);
        let flags = <Self as ExtractHelp<SpaceHelp>>::flags(&self);
        let links = <Self as ExtractHelp<SpaceHelp>>::links(&self);
        let ops = <Self as ExtractHelp<SpaceHelp>>::ops(&self);

        SpaceHelp {
            ops,
            description,
            links,
            flags,
        }
    }
}

fn root_fallback_desc(space: &Ident) -> String {
    format!("the {} cli tool", space)
}

impl<'a> ExtractHelp<RootHelp> for Excavator {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        if let Some(tbl) = links.get("root") {
            return hashmap_from_table(tbl.as_table());
        }

        None
    }

    fn ops(&self) -> Option<HashMap<String, String>> {
        let ops = self.toml.get("ops");
        if ops.is_none() {
            return None;
        }
        let ops = ops.unwrap();

        if let Some(ops) = ops.get("root") {
            return hashmap_from_table(ops.as_table());
        }

        None
    }

    fn spaces(&self) -> Option<HashMap<String, String>> {
        if let Some(spaces) = self.toml.get("spaces") {
            return hashmap_from_table(spaces.as_table());
        }

        None
    }

    fn flags(&self) -> Option<HashMap<String, String>> {
        let flags = self.toml.get("flags");
        if flags.is_none() {
            return None;
        }
        let flags = flags.unwrap();

        if let Some(flags) = flags.get("root") {
            return hashmap_from_table(flags.as_table());
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descriptions");
        if descs.is_none() {
            return space_fallback_desc(self.space.as_ref().unwrap());
        }
        let descs = descs.unwrap();

        if let Some(desc) = descs.get("root") {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        root_fallback_desc(self.space.as_ref().unwrap())
    }

    fn extract(self) -> RootHelp {
        let description = <Excavator as ExtractHelp<RootHelp>>::description(&self);
        let flags = <Self as ExtractHelp<RootHelp>>::flags(&self);
        let links = <Self as ExtractHelp<RootHelp>>::links(&self);
        let ops = <Self as ExtractHelp<RootHelp>>::ops(&self);
        let spaces = <Self as ExtractHelp<RootHelp>>::spaces(&self);

        RootHelp {
            spaces,
            ops,
            description,
            links,
            flags,
        }
    }
}
