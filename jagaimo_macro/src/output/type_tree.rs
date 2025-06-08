use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream as TS2};
use quote::quote;
use syn::{Ident, Type};

use super::{AliasedToken, TokenizedCommand};
use crate::input::Flag;

// TODO instead of fixing operations naming conflicts
// should use a module for every space
// that way op names can be duplicated
// and generated code structure would be clearer

#[derive(Debug, Clone)]
pub struct TypeTree<'a> {
    root: RootType<'a>,
}

// WARN space and op names are always there because every command needs a space and op names/ids for
// its states to be reprensented in the type tree
//  but when figuring the space and op of a command
//  the valid check is to use is_bare method on scope tokens
impl<'a> TypeTree<'a> {
    pub fn new(tcmd: Vec<TokenizedCommand<'a>>, root_name: &str) -> Self {
        let mut iter = tcmd.into_iter();
        let mut spaces: HashMap<&Ident, SpaceType<'_>> = HashMap::new();
        while let Some(cmd) = iter.next() {
            let space = cmd.space_cloned();
            let ident = space.ident().unwrap();

            if !spaces.contains(ident) {
                spaces.register(space);
            }

            let mut op = OpType::new(cmd.op_cloned());

            if let Some(flags) = cmd.flags_cloned() {
                flags.into_iter().for_each(|f| op.insert_flag(f))
            };

            if let Some(params) = cmd.params_cloned() {
                op.set_params(params);
            }

            // This inserts a full op type into a space with ident Ident
            spaces.update(ident, op);
        }

        let root_ident = Ident::new(root_name, Span::call_site());

        let root_ops = spaces.remove(&root_ident);
        let root_direct_op = root_ops.clone().map(|spc| spc.direct_op);
        let root_ops = root_ops.map(|spc| spc.ops);

        let root_spaces = spaces.into_values().collect::<Vec<SpaceType>>();

        let root = RootType {
            ident: root_ident,
            spaces: root_spaces,
            ops: root_ops.unwrap_or(Vec::new()),
            direct_op: root_direct_op.unwrap_or(None),
        };

        Self { root }
    }

    pub fn render(self) -> TS2 {
        let root = self.root.render();

        quote! {
            #root
        }
    }
}

trait TypeTreeExt<'a, T, C>
where
    T: 'a,
    C: 'a,
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
    fn update(&mut self, ident: &Ident, variant: C);
}

impl<'a> TypeTreeExt<'a, SpaceType<'a>, OpType<'a>> for HashMap<&'a Ident, SpaceType<'a>> {
    fn contains(&self, ident: &Ident) -> bool {
        self.contains_key(ident)
    }

    fn register(&mut self, token: AliasedToken<'a>) {
        let i = token.ident().unwrap();
        _ = self.insert(
            i,
            SpaceType {
                token,
                direct_op: None,
                ops: Vec::new(),
            },
        );
    }

    fn update(&mut self, ident: &Ident, op: OpType<'a>) {
        self.get_mut(ident).map(|st| {
            if op
                .token
                .ident()
                .map(|i| i.to_string().ends_with("DirectOp"))
                .unwrap()
            {
                st.set_direct(op);
            } else {
                st.insert(op);
            }
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootType<'a> {
    ident: Ident,
    spaces: Vec<SpaceType<'a>>,
    ops: Vec<OpType<'a>>,
    direct_op: Option<OpType<'a>>,
}

impl RootType<'_> {
    fn render(self) -> TS2 {
        let ident = self.ident;
        let count =
            self.spaces.len() + self.ops.len() + if self.direct_op.is_some() { 1 } else { 0 };

        let root = if count == 1 {
            if let Some(op) = self.direct_op {
                let op = op.render_fields();

                println!("0-0");
                return quote! {
                    struct #ident {
                        #op
                    }
                };
            } else if self.spaces.len() == 1 {
                let [module, field] = self
                    .spaces
                    .into_iter()
                    .next()
                    .map(|s| {
                        [s.clone().render(), {
                            let module = s.module_name();
                            let ident = s.token.ident().unwrap();

                            quote! {
                                #module: #module :: #ident
                            }
                        }]
                    })
                    .unwrap();

                println!("0-1");
                return quote! {
                    pub struct #ident {
                        #field
                    }

                    #module
                };
            } else {
                let op = self
                    .ops
                    .into_iter()
                    .next()
                    .map(|op| op.render_fields())
                    .unwrap();

                println!("0-2");
                return quote! {
                    pub struct #ident {
                        #op
                    }
                };
            }
        } else {
            let direct_variant = self
                .direct_op
                .clone()
                .map(|op| (op.token.ident(), op.render_fields()))
                .map(|(i, f)| quote! { #i { #f }})
                .inspect(|ts| println!(">>>>>>>>\n\n{}\n\n<<<<<<<<<", ts));
            let direct = self.direct_op.map(|op| op.render());

            let op_variants = self
                .ops
                .clone()
                .into_iter()
                .map(|op| op.token.ident())
                .map(|i| quote! { #i ( #i ) });
            let ops = self.ops.into_iter().map(|op| op.render());

            let space_variants = self
                .spaces
                .clone()
                .into_iter()
                .map(|scp| scp.render_variant());
            let spaces = self
                .spaces
                .into_iter()
                .map(|spc| spc.render())
                .inspect(|ts| println!("{}", ts));

            println!("1");
            return quote! {
                pub enum #ident {
                    #(#space_variants,)*
                    #(#op_variants,)*
                    #direct_variant
                }

                #(#ops)*

                #(#spaces)*
            };
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpaceType<'a> {
    token: AliasedToken<'a>,
    ops: Vec<OpType<'a>>,
    direct_op: Option<OpType<'a>>,
}

impl<'a> SpaceType<'a> {
    fn insert(&mut self, op: OpType<'a>) {
        _ = self.ops.push(op);
    }

    fn set_direct(&mut self, op: OpType<'a>) {
        self.direct_op = Some(op);
    }

    fn is_space_direct(&self) -> bool {
        self.token.is_direct()
    }

    fn is_op_direct(&self) -> bool {
        self.token.is_root() && self.direct_op.is_some()
    }

    fn module_name(&self) -> Ident {
        let s = self.token.ident().unwrap().to_string();
        Ident::new(
            &s.chars()
                .enumerate()
                .map(|(i, c)| {
                    if c.is_ascii_uppercase() {
                        if i == 0 {
                            String::from(c.to_ascii_lowercase())
                        } else {
                            let mut s = String::from("_");
                            s.push(c.to_ascii_lowercase());
                            s
                        }
                    } else {
                        String::from(c)
                    }
                })
                .fold(String::new(), |acc, s| acc + &s),
            Span::call_site(),
        )
    }

    fn render_variant(self) -> TS2 {
        let module_name = self.module_name();
        let mut ident = self.token.ident().unwrap().clone();

        if self.ops.len() == 1 {
            ident = Ident::new(
                &(ident.to_string()
                    + &self
                        .ops
                        .into_iter()
                        .next()
                        .unwrap()
                        .token
                        .ident()
                        .unwrap()
                        .to_string()),
                Span::call_site(),
            );
        }

        quote! {
            #ident ( #module_name :: #ident )
        }
    }

    fn render(self) -> TS2 {
        let ident = self.token.ident().unwrap();
        let mod_ident = self.module_name();

        let op_count = if self.direct_op.is_some() { 1 } else { 0 } + self.ops.len();
        let space_type = if op_count == 1 {
            let (op, ident) = if let Some(op) = self.direct_op {
                (op, ident.clone())
            } else {
                let op = self.ops.into_iter().next().unwrap();
                let ident = Ident::new(
                    &(ident.to_string() + &op.token.ident().unwrap().to_string()),
                    Span::call_site(),
                );
                (op, ident)
            };
            let fields = op.render_fields();
            // TODO let ident = ident + op ident ;
            quote! {
                pub struct #ident {
                    #fields
                }
            }
        } else {
            let direct_variant = self
                .direct_op
                .clone()
                .map(|op| op.token.ident())
                .map(|i| quote! { #i ( #i )});
            let direct = self.direct_op.map(|op| op.render());

            let op_variants = self
                .ops
                .clone()
                .into_iter()
                .map(|op| op.token.ident())
                .map(|i| quote! { #i ( #i ) });
            let ops = self.ops.into_iter().map(|op| op.render());

            quote! {
                pub enum #ident {
                    #(#op_variants,)*
                    #direct_variant

                }

                    #(#ops)*

                    #direct

            }
        };

        quote! {
            pub mod #mod_ident {
                #space_type
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpType<'a> {
    token: AliasedToken<'a>,
    fields: Vec<AliasedToken<'a>>,
    params: Option<AliasedToken<'a>>,
}

impl<'a> OpType<'a> {
    fn new(atok: AliasedToken<'a>) -> Self {
        Self {
            token: atok,
            fields: Vec::new(),
            params: None,
        }
    }

    fn insert_flag(&mut self, field: AliasedToken<'a>) {
        _ = self.fields.push(field);
    }

    fn set_params(&mut self, params: AliasedToken<'a>) {
        _ = self.params = Some(params);
    }

    fn render_fields(self) -> TS2 {
        let fields = self.fields.into_iter().map(|atok| {
            let f = atok.flag().unwrap();
            let ident = f.ident();
            let fallback = Type::Verbatim("bool".parse().unwrap());
            let ty = f.ty().unwrap_or(&fallback);

            quote! { #ident: #ty  }
        });
        let params = self.params.map(|atok| {
            let ty = atok.ty();
            quote! { params: #ty }
        });

        quote! {
            #(#fields,)*
            #params
        }
    }

    fn render(self) -> TS2 {
        let ident = self.token.ident();
        let fields = self.fields.into_iter().map(|atok| {
            let f = atok.flag().unwrap();
            let ident = f.ident();
            let fallback = Type::Verbatim("bool".parse().unwrap());
            let ty = f.ty().unwrap_or(&fallback);

            quote! { #ident: #ty  }
        });
        let params = self.params.map(|atok| {
            let ty = atok.ty();
            quote! { params: #ty }
        });

        quote! {
            pub struct #ident {
                #(#fields,)*
                #params
            }
        }
    }
}
