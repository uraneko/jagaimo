use std::collections::HashMap;

use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{Ident, Type};

use super::capitalize_ident;
use crate::parse::flags::Flag;
use crate::parse::scope::Scope;
use crate::parse::{CommandRule, Rules};

impl Rules {
    pub fn generate_root(&self, name: &str) -> TS2 {
        let mut cmds = self.commands();

        let mut space_variants = self.space_variants();
        let mut non_space_variants = self.non_space_variants();

        let space_len = space_variants.len();
        let non_space_len = non_space_variants.len();
        let variant_count = space_len + non_space_len;

        let mut variants = space_variants
            .into_iter()
            .map(|v| Ident::new(&v, Span::call_site()))
            .chain(non_space_variants)
            .map(|i| {
                let i = capitalize_ident(&i);
                quote! {
                    #i (#i)
                }
            });

        let name = Ident::new(name, Span::call_site());
        match variant_count {
            1 => {
                let ident = variants.next().unwrap();
                quote! { struct #name ( #ident ) }
            }
            len => quote! { enum #name { #(#variants,)* } },
        }
    }

    pub fn generate_type_tree(&self, name: &Ident) -> TS2 {
        let tt = TypeTree::from_commands(name, self.commands());
        let (root, spaces, ops) = (
            self.generate_root_type(tt.root),
            tt.spaces
                .into_iter()
                .map(|space| self.generate_space_type(space)),
            tt.ops.into_iter().map(|op| Self::generate_op_type(op)),
        );

        quote! {
            #root

            #(#spaces)*

            #(#ops)*
        }
    }

    pub fn generate_root_type(&self, r: GenerateRoot) -> TS2 {
        let ident = r.name;
        let variants = r.variants;
        let mut anon = TypeTree::generate_root_anon(self.commands())
            .map(|cr| {
                let mut cr = cr.clone();
                TypeTree::make_anon_op(&mut cr, ident.to_string());
                cr
            })
            .map(|cr| Self::generate_op_type(TypeTree::generate_op(&cr).unwrap()));

        if variants.len() == 1 {
            let variant = variants.into_iter().next().unwrap();

            quote! { struct #ident ( #variant )
                #anon
            }
        } else {
            let variants = variants.into_iter().map(|v| quote! { #v ( #v ) });
            quote! { enum #ident { #(#variants,)* }
                #anon
            }
        }
    }

    pub fn generate_space_type(&self, s: GenerateSpace) -> TS2 {
        let ident = s.ident;
        let variants = s.variants;
        let mut anon = TypeTree::generate_space_anon(&ident, self.commands())
            .map(|cr| {
                let mut cr = cr.clone();
                TypeTree::make_anon_op(&mut cr, ident.to_string());
                cr
            })
            .map(|cr| Self::generate_op_type(TypeTree::generate_op(&cr).unwrap()));

        if variants.len() == 1 {
            let variant = variants.into_iter().next().unwrap();

            quote! { struct #ident ( #variant )
                #anon
            }
        } else {
            let variants = variants.into_iter().map(|v| quote! { #v ( #v ) });
            quote! { enum #ident { #(#variants,)* }
                #anon
            }
        }
    }

    pub fn generate_op_type(o: GenerateOp) -> TS2 {
        let ident = o.ident;
        let params = o.params.map(|ty| quote! { params: #ty });
        let fields = o
            .fields
            .map(|flags| {
                flags.into_iter().map(|f| match f {
                    Flag::Bool(i) => quote! { #i: bool  },
                    Flag::Parameterized(i, t) => quote! { #i: #t },
                })
            })
            .map(|iter| quote! { #(#iter,)* });

        quote! {
            struct #ident {
                #fields,
                #params

            }
        }
    }
}

#[derive(Debug)]
pub struct GenerateSpace {
    ident: Ident,
    variants: Vec<Ident>,
}

#[derive(Debug)]
pub struct GenerateOp {
    scope: Scope,
    fields: Option<Vec<Flag>>,
    params: Option<Type>,
    ident: Ident,
}

#[derive(Debug)]
pub struct GenerateRoot {
    name: Ident,
    variants: Vec<Ident>,
}

#[derive(Debug)]
pub struct TypeTree {
    root: GenerateRoot,
    spaces: Vec<GenerateSpace>,
    ops: Vec<GenerateOp>,
}

impl TypeTree {
    fn from_commands(name: &Ident, cmds: &[CommandRule]) -> Self {
        let ops = Self::generate_ops(cmds);
        let spaces: Vec<_> = Self::generate_spaces(&ops, cmds).collect();
        let root = Self::generate_root(name, cmds, &ops, &spaces);
        Self { root, ops, spaces }
    }

    // generates all the op types
    fn generate_ops(cmds: &[CommandRule]) -> Vec<GenerateOp> {
        cmds.into_iter()
            .map(|cmd| Self::generate_op(cmd))
            .filter(|go| go.is_some())
            .map(|go| go.unwrap())
            .collect()
    }

    // generates an op type from a command if the command has one
    fn generate_op(cmd: &CommandRule) -> Option<GenerateOp> {
        if let Some(o) = cmd.op() {
            return Some(GenerateOp {
                scope: cmd
                    .space()
                    .map(|s| Scope::Space(s.clone()))
                    .unwrap_or(Scope::Root),
                fields: cmd.flags().map(|f| f.to_vec()),
                params: cmd.params().cloned(),
                ident: capitalize_ident(o),
            });
        }

        None
    }

    fn generate_root_anon(cmds: &[CommandRule]) -> Option<&CommandRule> {
        cmds.into_iter()
            .find(|cr| cr.space().is_none() && cr.op().is_none())
    }

    fn generate_root_ops(ops: &[GenerateOp]) -> impl Iterator<Item = &GenerateOp> {
        ops.into_iter().filter(|op| Scope::Root == op.scope)
    }

    fn generate_root(
        name: &Ident,
        cmds: &[CommandRule],
        ops: &[GenerateOp],
        spaces: &[GenerateSpace],
    ) -> GenerateRoot {
        let anon = Self::generate_root_anon(cmds).map(|_| {
            Ident::new(
                &(capitalize_ident(name).to_string() + "Anon"),
                Span::call_site(),
            )
        });
        let spaces = spaces.into_iter().map(|s| capitalize_ident(&s.ident));
        let ops = Self::generate_root_ops(ops).map(|o| capitalize_ident(&o.ident));
        let variants = ops.chain(spaces).chain(anon).collect();

        GenerateRoot {
            name: capitalize_ident(name),
            variants,
        }
    }

    fn make_anon_op(cmd: &mut CommandRule, ident: String) {
        let i = Ident::new(&(ident + "Anon"), Span::call_site());
        cmd.set_op(i);
    }

    fn generate_space_anon<'a>(
        ident: &'a Ident,
        cmds: &'a [CommandRule],
    ) -> Option<&'a CommandRule> {
        cmds.into_iter()
            .find(|cr| cr.space() == Some(ident) && cr.op().is_none())
    }

    // generates space types from the generated op types
    fn generate_spaces(
        ops: &[GenerateOp],
        cmds: &[CommandRule],
    ) -> impl Iterator<Item = GenerateSpace> {
        ops.chunk_by(|a, b| a.scope == b.scope).map(|s| {
            let ident = capitalize_ident(&s[0].scope.space().unwrap().clone());
            let anon = Self::generate_space_anon(&ident, cmds)
                .map(|_| Ident::new(&format!("{}Anon", &ident), Span::call_site()));

            GenerateSpace {
                ident,
                variants: s
                    .into_iter()
                    .map(|o| capitalize_ident(&o.ident))
                    .chain(anon)
                    .collect(),
            }
        })
    }
}

// generate type tree
// impl From<TokenizedCommand> for each type in the the TypeTree
// parse help file
// impl Help for each type in the TypeTree
