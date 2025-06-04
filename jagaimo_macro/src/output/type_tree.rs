use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use syn::{Ident, Type};

use super::{AliasedToken, TokenizedCommand};
use crate::input::Flag;

#[derive(Debug, Clone)]
pub struct TypeTree<'a> {
    root: RootType<'a>,
    spaces: HashSet<SpaceType<'a>>,
    ops: HashSet<OpType<'a>>,
}

impl TypeTree<'_> {
    fn new(tcmd: Vec<TokenizedCommand>) -> Self {
        let mut iter = tcmd.into_iter();
        let mut spaces: HashMap<&Ident, SpaceType<'_>> = HashMap::new();
        let mut ops: HashMap<&Ident, OpType<'_>> = HashMap::new();
        while let Some(cmd) = iter.next() {
            match cmd {
                // command is a space operation
                tc if tc.is_space_op() => {
                    let space = tc.space().unwrap();
                    let ident = space.ident().unwrap();
                    let op = tc.op().unwrap();

                    if !spaces.contains(ident) {
                        spaces.register(space);
                    }
                    spaces.update(ident, op);
                }
                // command is a space anonymous command
                tc if tc.is_space() => {
                    let space = tc.space().unwrap();
                    let ident = space.ident().unwrap();
                    // let op = nameless_ident(&ident);

                    if !spaces.contains(ident) {
                        spaces.register(space);
                    }
                    // spaces.update(ident, op);
                }
                // command is an anonymous operation command
                tc if tc.is_op() => {}
                // command is somethig else, unreachable
                _ => panic!(),
            }
        }

        todo!()
    }
}

trait TypeTreeExt<'a, T>
where
    T: 'a,
{
    fn contains(&self, ident: &Ident) -> bool;

    // it should be an error to attempt register a new
    // value only to find that it already existed in the map
    //
    // this function only creates new space types and sets their idents
    // pushing any variants to the SpaceType is out of the scope of this function
    fn register(&mut self, ident: AliasedToken<'a>);

    // it should be an error to attempt an update to an existing
    // value only to find that it didnt already exist in the map
    //
    // this function only inserts new variants into the given ident's
    // spacetype
    fn update(&mut self, ident: &Ident, variant: AliasedToken<'a>);
}

impl<'a> TypeTreeExt<'a, SpaceType<'a>> for HashMap<&'a Ident, SpaceType<'a>> {
    fn contains(&self, ident: &Ident) -> bool {
        self.contains_key(ident)
    }

    fn register(&mut self, ident: AliasedToken<'a>) {
        let i = ident.ident().unwrap();
        _ = self.insert(
            i,
            SpaceType {
                ident,
                variants: HashSet::new(),
            },
        );
    }

    fn update(&mut self, ident: &Ident, variant: AliasedToken<'a>) {
        self.get_mut(ident).map(|st| st.insert(variant));
    }
}

impl<'a> TypeTreeExt<'a, OpType<'a>> for HashMap<&'a Ident, OpType<'a>> {
    fn contains(&self, ident: &Ident) -> bool {
        self.contains_key(ident)
    }

    fn register(&mut self, ident: AliasedToken<'a>) {
        let i = ident.ident().unwrap();
        _ = self.insert(
            i,
            OpType {
                ident,
                fields: HashSet::new(),
                params: None,
            },
        );
    }

    fn update(&mut self, ident: &Ident, variant: AliasedToken<'a>) {
        self.get_mut(ident).map(|ot| ot.insert(variant));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootType<'a> {
    ident: AliasedToken<'a>,
    variants: HashSet<AliasedToken<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceType<'a> {
    ident: AliasedToken<'a>,
    variants: HashSet<AliasedToken<'a>>,
}

impl<'a> SpaceType<'a> {
    fn insert(&mut self, variant: AliasedToken<'a>) {
        _ = self.variants.insert(variant);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpType<'a> {
    ident: AliasedToken<'a>,
    fields: HashSet<AliasedToken<'a>>,
    params: Option<AliasedToken<'a>>,
}

impl<'a> OpType<'a> {
    fn insert(&mut self, field: AliasedToken<'a>) {
        _ = self.fields.insert(field);
    }

    fn set_params(&mut self, params: AliasedToken<'a>) {
        _ = self.params = Some(params);
    }
}
