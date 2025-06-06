use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::{Ident, Type};

use super::{AliasedToken, TokenizedCommand};
use crate::input::Flag;

#[derive(Debug, Clone)]
pub struct TypeTree<'a> {
    root: RootType<'a>,
    spaces: Vec<SpaceType<'a>>,
    ops: Vec<OpType<'a>>,
}

// TODO when bare resolution is handled after parsing macro input
// operation name conflicts should also be handled
// i.e., given 2 command rules
// s(history) o(view) ....
// s(collections) o(view) ...
// s(hosts) o(view)
//
// 3 possible cases
// * all view ops have different contexts
// -> each op is renamed to space_name_op_name
// <- this way, they are treated as different ops in type tree generation
// * all view ops have the same contexts
// -> no additional work is done
// <- all 3 ops are treated as the same op in type tree generation
// * most (2) of the 3 ops have the same context
// -> the 2 that  have the same context are left as is
// and the unique one is renamed to space_name_op_name
// <- this way name conflicts are avoided in the type tree

// TODO need special handling for fully bare scope
// -> if space == op
// -> space is a redundant op name
// there is no space but an op for the root space
//
// TODO spaceless operations all belong to root space
//
// WARN space and op names are always there because every command needs a space and op names/ids for
// its states to be reprensented in the type tree
//  but when figuring the space and op of a command
//  the valid check is to use is_bare method on scope tokens
impl<'a> TypeTree<'a> {
    pub fn new(tcmd: Vec<TokenizedCommand<'a>>, root_name: &str) -> Self {
        let mut iter = tcmd.into_iter();
        let mut spaces: HashMap<&Ident, SpaceType<'_>> = HashMap::new();
        let mut ops: HashMap<&Ident, OpType<'_>> = HashMap::new();
        while let Some(cmd) = iter.next() {
            let space = cmd.space_cloned();
            let ident = space.ident().unwrap();
            let op = cmd.op_cloned();

            if !spaces.contains(ident) {
                spaces.register(space);
            }
            spaces.update(ident, op);

            let ident = cmd.op().ident().unwrap();
            ops.register(cmd.op_cloned());

            if let Some(flags) = cmd.flags_cloned() {
                flags.into_iter().for_each(|f| ops.update(ident, f))
            };

            if let Some(params) = cmd.params_cloned() {
                ops.update(ident, params);
            }
            // match cmd {
            //     // command is a space operation
            //     // some space's named command
            //     tc if tc.is_space_op() => {}
            //     // command is a space bare command
            //     // some space's bare command
            //     tc if tc.is_space() => {}
            //     // command is an bare operation command
            //     // root bare command
            //     tc if tc.is_op() => {}
            //     // command is somethig else, unreachable
            //     _ => panic!(),
            // }
        }

        let root_name = Ident::new(root_name, Span::call_site());
        let root_variants = spaces.clone().into_keys().collect();
        let root = RootType {
            ident: root_name,
            variants: root_variants,
        };

        let spaces = spaces.into_values().collect();
        let ops = ops.into_values().collect();

        Self { root, spaces, ops }
    }

    pub fn render(self) -> TS2 {
        let root = self.root.render();
        let spaces = self.spaces.into_iter().map(|s| s.render());
        let ops = self.ops.into_iter().map(|o| o.render());

        quote! {
            #root

            #(#spaces)*

            #(#ops)*
        }
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

    fn update(&mut self, ident: &Ident, token: AliasedToken<'a>) {
        if token.is_space() || token.is_op() {
            // TODO error
            return;
        }

        self.get_mut(ident).map(|ot| {
            if token.is_flag() {
                ot.insert_flag(token);
            } else {
                ot.set_params(token)
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootType<'a> {
    ident: Ident,
    variants: Vec<&'a Ident>,
}

impl RootType<'_> {
    fn render(self) -> TS2 {
        let ident = self.ident;
        let mut variants = self.variants;

        if variants.len() == 1 {
            let variant = variants.pop().unwrap();
            quote! {
                struct #ident (#variant);
            }
        } else {
            quote! {
                enum #ident {
                    #(#variants,)*
                }
            }
        }
    }
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

    fn render(self) -> TS2 {
        println!("1");
        let ident = self.ident.ident().unwrap();
        let mut variants = self.variants.into_iter().map(|atok| atok.ident().unwrap());

        if variants.len() == 1 {
            let variant = variants.next().unwrap();
            println!("1");
            quote! {
                struct #ident (#variant);
            }
        } else {
            quote! {
                enum #ident {
                    #(#variants,)*
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpType<'a> {
    ident: AliasedToken<'a>,
    fields: HashSet<AliasedToken<'a>>,
    params: Option<AliasedToken<'a>>,
}

impl<'a> OpType<'a> {
    fn insert_flag(&mut self, field: AliasedToken<'a>) {
        _ = self.fields.insert(field);
    }

    fn set_params(&mut self, params: AliasedToken<'a>) {
        _ = self.params = Some(params);
    }

    fn render(self) -> TS2 {
        println!("2");
        let ident = self.ident.ident().unwrap();
        let fields = self.fields.into_iter().map(|f| {
            println!("{}", f);
            let flag = f.flag().unwrap();
            let ident = flag.ident();
            let ty = flag
                .ty()
                .map(|ty| quote! { #ty })
                .unwrap_or_else(|| quote! { bool });

            quote! { #ident: #ty  }
        });

        let params = self
            .params
            .map(|p| p.ty().unwrap())
            .map(|ty| quote! { params: #ty });

        println!("2");
        quote! {
            struct #ident {
                #(#fields,)*
                #params
            }
        }
    }
}
