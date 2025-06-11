use proc_macro2::Span;
use quote::ToTokens;
use quote::quote;
use syn::Error;
use syn::Result as SRes;
use syn::{Attribute, Expr, ExprLit, Ident, Meta};
use syn::{MetaList, MetaNameValue, Path};

use resolve_calling_crate::ResolveCrate;
use toml::Table;

use crate::read::{Excavator, OpHelp, RootHelp, SpaceHelp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HelpAttrs {
    root: bool,
    space: Option<String>,
    at: String,
}

impl Default for HelpAttrs {
    fn default() -> Self {
        Self {
            at: String::from("help.toml"),
            space: None,
            root: false,
        }
    }
}

impl HelpAttrs {
    pub fn new(attrs: Vec<Attribute>) -> SRes<Self> {
        let mut res = Self::default();
        let mut attrs = attrs.into_iter();

        while let Some(a) = attrs.next() {
            let meta = a.meta;
            if let Ok(ml) = meta.require_list() {
                res.space = Some(parse_space(ml)?);
            } else if let Ok(mnv) = meta.require_name_value() {
                res.at = parse_at(mnv)?;
            } else if let Ok(p) = meta.require_path_only() {
                res.root = parse_root(p)?;
            }
        }

        if res.root && res.space.is_some() {
            return Err(Error::new(
                Span::call_site(),
                "attr conflict; found root attr indicating root namespace and space attr for operation dont mix",
            ));
        }

        Ok(res)
    }

    // matches the help type that excavator should be called on later
    pub fn help_type(&self) -> HelpHelp {
        if self.root {
            HelpHelp::Root
        } else if self.space.is_some() {
            HelpHelp::Operation
        } else {
            HelpHelp::Space
        }
    }

    // reads the toml file found at self.at
    pub fn read_help(&self) -> Table {
        std::fs::read_to_string(ResolveCrate::new().into_string_suffixed(&self.at))
            .unwrap()
            .parse::<Table>()
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpHelp {
    Root,
    Space,
    Operation,
}

pub fn parse_at(mnv: &MetaNameValue) -> SRes<String> {
    if !mnv.path.is_ident("at") {
        return Err(Error::new(
            Span::call_site(),
            "expected at, got unrecognized attr meta name value",
        ));
    }

    let mut s = mnv.value.to_token_stream().to_string();
    s.remove(s.len() - 1);
    s.remove(0);

    Ok(s)
}

pub fn parse_root(p: &Path) -> SRes<bool> {
    // might be better to error out in case the value is not root
    // since that would be an invalid attr value
    if !p.is_ident("root") {
        return Err(Error::new(
            Span::call_site(),
            "expected root, got unrecognizable attr path value",
        ));
    }

    Ok(true)
}

pub fn parse_space(ml: &MetaList) -> SRes<String> {
    if !ml.path.is_ident("space") {
        return Err(Error::new(
            Span::call_site(),
            "expected space, got unrecognized attr metalist path",
        ));
    }

    ml.parse_args().map(|el: ExprLit| {
        let mut s = el.to_token_stream().to_string();
        s.remove(s.len() - 1);
        s.remove(0);

        s
    })
}

pub struct GenerateExcavator {
    ident: Ident,
    toml: Table,
    help_type: HelpHelp,
    space: Option<String>,
}

impl GenerateExcavator {
    pub fn new(ident: Ident, ha: HelpAttrs) -> Self {
        Self {
            help_type: ha.help_type(),
            toml: ha.read_help(),
            ident,
            space: ha.space,
        }
    }

    pub fn excavator(self) -> Excavator {
        Excavator::new(
            self.space.map(|s| Ident::new(&s, Span::call_site())),
            Some(self.ident),
            self.toml,
        )
    }
}

// NOTE for now just repeat same operation
// TODO use an http server to read, parse then store the data of the toml
// every derive macro call sends a get request to the server with
// a specific (root/space/op) endpoint with its params, the server in turn looks for the asked for
// data and removes it from the toml data, giving it away to the derive
// derive then uses that data to do its thing
