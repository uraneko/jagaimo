use std::collections::{HashMap, HashSet};

use syn::Ident;
use toml::{Table, Value};

use resolve_calling_crate::ResolveCrate;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Styled {
    content: String,
    color: Color,
    effects: HashSet<Effect>,
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    #[default]
    White = 7,
}

use std::mem::transmute;

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 => unsafe { transmute::<u8, Self>(value) },
            _ => Self::default(),
        }
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        match &value.to_lowercase()[..] {
            "red" | "r" | "1" => Self::Red,
            "green" | "g" | "grn" | "2" => Self::Green,
            "yellow" | "y" | "ylw" | "ylo" | "3" => Self::Yellow,
            "blue" | "blu" | "ble" | "4" => Self::Blue,
            "mgnt" | "magenta" | "mgt" | "mgn" | "mag" | "magen" | "magnt" | "5" => Self::Magenta,
            "cyan" | "c" | "cyn" | "can" | "yan" | "6" => Self::Cyan,
            "white" | "w" | "wht" | "whit" | "wte" | "7" => Self::White,
            "black" | "blk" | "blac" | "blck" | "0" | "bl" => Self::Black,
            _ => Self::default(),
        }
    }
}

impl From<&Color> for &str {
    fn from(value: &Color) -> &'static str {
        match value {
            Color::Black => "30",
            Color::Red => "31",
            Color::Green => "32",
            Color::Yellow => "33",
            Color::Blue => "34",
            Color::Magenta => "35",
            Color::Cyan => "36",
            Color::White => "37",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Effect {
    #[default]
    Clear = 0,
    Bold = 1,
    Italic = 3,
    Underline = 4,
    StrikeThrough = 9,
}

impl From<u8> for Effect {
    fn from(value: u8) -> Self {
        match value {
            0 | 1 | 3 | 4 | 9 => unsafe { transmute::<u8, Self>(value) },
            _ => Self::default(),
        }
    }
}

impl From<&str> for Effect {
    fn from(value: &str) -> Self {
        match &value.to_lowercase()[..] {
            "bold" | "b" | "bld" | "bol" | "1" => Self::Bold,
            "clr" | "c" | "clear" | "0" => Self::Clear,
            "strikethrough" | "st" | "strike_through" | "strk_throu" | "stkthr" | "9" => {
                Self::StrikeThrough
            }
            "underline" | "udrln" | "ul" | "under" | "undln" | "undline" | "4" => Self::Underline,
            "italic" | "itlc" | "i" | "it" | "itl" | "3" => Self::Italic,
            _ => Self::default(),
        }
    }
}

impl From<&Effect> for &str {
    fn from(value: &Effect) -> &'static str {
        match value {
            Effect::Bold => "1",
            Effect::Clear => "0",
            Effect::StrikeThrough => "9",
            Effect::Italic => "3",
            Effect::Underline => "4",
        }
    }
}

impl Styled {
    pub fn color(mut self, clr: impl Into<Color>) -> Self {
        self.color = clr.into();

        self
    }

    fn color_as_str(&self) -> &str {
        (&self.color).into()
    }

    fn effects_to_string(&self) -> String {
        self.effects
            .iter()
            .map(|e| e.into())
            .fold(String::new(), |acc, e| acc + ";" + e)
    }

    pub fn effect(mut self, efct: impl Into<Effect>) -> Self {
        self.effects.insert(efct.into());

        self
    }

    pub fn effects<T, I>(mut self, efcts: T) -> Self
    where
        T: IntoIterator<Item = I>,
        I: Into<Effect>,
    {
        efcts.into_iter().for_each(|e| {
            self.effects.insert(e.into());
        });

        self
    }

    pub fn remove(mut self, efct: impl Into<Effect>) -> Self {
        self.effects.remove(&efct.into());

        self
    }

    pub fn fmt(&self, content: impl AsRef<str>) -> String {
        format!(
            "\x1b[{}{}m{}\x1b[0m",
            self.color_as_str(),
            self.effects_to_string(),
            content.as_ref()
        )
    }
}

fn gen_delim(k: &mut String) {
    let len = k.len();
    let delim = 20 - len;
    k.push_str(&(0..delim).into_iter().map(|_| ' ').collect::<String>());
}

fn format_table(t: HashMap<String, String>, stl: &mut Styled) -> String {
    t.into_iter()
        .map(|(k, v)| (stl.fmt(k), v))
        .map(|(mut k, v)| {
            gen_delim(&mut k);
            k + &v
        })
        .fold(String::new(), |acc, l| acc + &l + "\n")
}

pub trait Help {
    // formats the entire help message of self
    // and returns it
    fn help() -> String {
        let stls = Self::styles();
        let hdr = stls.get("hdr").unwrap();
        let desc = Self::description();
        let links = Self::links(&stls).unwrap_or("".into());
        let usage = Self::usage(&stls);
        let spaces = Self::spaces(&stls)
            .map(|s| hdr.fmt("Spaces:") + &s)
            .unwrap_or("".into());
        let ops = Self::ops(&stls)
            .map(|s| hdr.fmt("Operations:") + &s)
            .unwrap_or("".into());
        let flags = Self::flags(&stls)
            .map(|s| hdr.fmt("Flags:") + &s)
            .unwrap_or("".into());

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}",
            desc, links, usage, flags, ops, spaces
        )
    }

    fn styles() -> HashMap<&'static str, Styled> {
        HashMap::from([
            ("base", Styled::default().color("y").effects(["b", "ul"])),
            ("flag", Styled::default().color("r").effect("b")),
            ("op", Styled::default().color("g").effect("b")),
            ("space", Styled::default().color("blue").effect("bold")),
            ("clear", Styled::default().effect("clear")),
            ("hdr", Styled::default().color("mgnt").effects(["b", "itl"])),
        ])
    }

    // returns the usage string of this scope
    fn usage(stls: &HashMap<&str, Styled>) -> String;

    // returns the url links provided in self's help message
    fn links(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn flags(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn ops(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    fn spaces(stls: &HashMap<&str, Styled>) -> Option<String> {
        None
    }

    // returns the description of this scope, if any
    // if none then defaults to returning self.named_scope()
    fn description() -> String;

    // returns the cli tool version
    // returns option because version only makes sense on the root namespace
    fn version() -> Option<String> {
        None
    }
}

pub fn read_help() -> toml::Table {
    ResolveCrate::new().read_help().into_table()
}

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
pub struct Excavator<'a, T> {
    toml: &'a toml::Table,
    space: Option<&'a Ident>,
    op: Option<&'a Ident>,
    _data: std::marker::PhantomData<T>,
}

impl<'a, T> Excavator<'a, T> {
    pub fn new(space: Option<&'a Ident>, op: Option<&'a Ident>, toml: &'a toml::Table) -> Self {
        Self {
            toml,
            space,
            op,
            _data: std::marker::PhantomData::<T>,
        }
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

impl<'a> ExtractHelp<OpHelp> for Excavator<'a, OpHelp> {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let [space, op] = [
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
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
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
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
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return op_fallback_desc(self.space.unwrap(), self.op.unwrap());
        }
        let descs = descs.unwrap();

        let [space, op] = [
            self.space.map(|s| s.to_string()).unwrap(),
            self.op.map(|o| o.to_string()).unwrap(),
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
                    .unwrap_or_else(|| op_fallback_desc(self.space.unwrap(), self.op.unwrap()))
            })
            .unwrap_or_else(|| op_fallback_desc(self.space.unwrap(), self.op.unwrap()))
    }

    fn extract(self) -> OpHelp {
        let description = self.description();
        let flags = self.flags();
        let links = self.links();

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

impl<'a> ExtractHelp<SpaceHelp> for Excavator<'a, SpaceHelp> {
    fn links(&self) -> Option<HashMap<String, String>> {
        let links = self.toml.get("links");
        if links.is_none() {
            return None;
        }
        let links = links.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();

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

        let space = self.space.map(|s| s.to_string()).unwrap();
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

        let space = self.space.map(|s| s.to_string()).unwrap();

        if let Some(flags) = flags.get(space) {
            return hashmap_from_table(flags.as_table());
        }

        None
    }

    fn description(&self) -> String {
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return space_fallback_desc(self.space.unwrap());
        }
        let descs = descs.unwrap();

        let space = self.space.map(|s| s.to_string()).unwrap();

        if let Some(desc) = descs.get(space) {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        space_fallback_desc(self.space.unwrap())
    }

    fn extract(self) -> SpaceHelp {
        let description = self.description();
        let flags = self.flags();
        let links = self.links();
        let ops = self.ops();

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

impl<'a> ExtractHelp<RootHelp> for Excavator<'a, RootHelp> {
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
        let descs = self.toml.get("descripts");
        if descs.is_none() {
            return space_fallback_desc(self.space.unwrap());
        }
        let descs = descs.unwrap();

        if let Some(desc) = descs.get("root") {
            return desc.as_str().map(|s| s.to_owned()).unwrap();
        }

        root_fallback_desc(self.space.unwrap())
    }

    fn extract(self) -> RootHelp {
        let description = self.description();
        let flags = self.flags();
        let links = self.links();
        let ops = self.ops();
        let spaces = self.spaces();

        RootHelp {
            spaces,
            ops,
            description,
            links,
            flags,
        }
    }
}
