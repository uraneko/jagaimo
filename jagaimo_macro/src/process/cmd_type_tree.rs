use std::collections::HashMap;

use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{Ident, Type};

use super::capitalize_ident;
use super::tokenized_commands::{CommandToken, TokenizedCommand};
use crate::parse::flags::Flag;
use crate::parse::scope::Scope;
use crate::parse::{Attributes, CommandStack};

pub struct TypeTreeStack {
    attrs: Attributes,
    tt: TypeTree,
    cmds: Vec<TokenizedCommand>,
}

impl TypeTreeStack {
    pub fn from_cmd_stack(mut cs: CommandStack) -> Self {
        let cmds = cs.tokenize_commands();
        let attrs = cs.take_attrs();
        let name = Ident::new(attrs.root_name(), Span::call_site());
        let tt = TypeTree::from_commands(&name, &cmds);

        Self { attrs, cmds, tt }
    }

    pub fn root_name(&self) -> Ident {
        Ident::new(self.attrs.root_name(), Span::call_site())
    }

    pub fn commands(&self) -> &[TokenizedCommand] {
        &self.cmds
    }

    pub fn tt(&self) -> &TypeTree {
        &self.tt
    }

    pub fn generate_type_tree(&self) -> TS2 {
        let tt = self.tt();
        let (root, spaces, ops) = (
            self.generate_root_type(tt.root()),
            tt.spaces()
                .into_iter()
                .map(|space| self.generate_space_type(space)),
            tt.ops().into_iter().map(|op| Self::generate_op_type(op)),
        );

        quote! {
            #root

            #(#spaces)*

            #(#ops)*
        }
    }

    pub fn generate_root_type(&self, r: &GenerateRoot) -> TS2 {
        let ident = r.name();
        let variants = r.variants();
        let mut anon = TypeTree::generate_root_anon(self.commands())
            .map(|cmd| {
                let mut cmd = cmd.clone();
                TypeTree::make_anon_op(&mut cmd, ident.to_string());
                cmd
            })
            .map(|cmd| Self::generate_op_type(&TypeTree::generate_op(&cmd).unwrap()));

        if variants.len() == 1 {
            let variant = variants.into_iter().next().unwrap();

            quote! { struct #ident ( #variant );
                #anon
            }
        } else {
            let variants = variants.into_iter().map(|v| quote! { #v ( #v ) });
            quote! { enum #ident { #(#variants,)* }
                #anon
            }
        }
    }

    pub fn generate_space_type(&self, s: &GenerateSpace) -> TS2 {
        let ident = s.ident();
        let variants = s.variants();
        let mut anon = TypeTree::generate_space_anon(&ident, self.commands())
            .map(|cmd| {
                let mut cmd = cmd.clone();
                TypeTree::make_anon_op(&mut cmd, ident.to_string());
                cmd
            })
            .map(|cmd| Self::generate_op_type(&TypeTree::generate_op(&cmd).unwrap()));

        if variants.len() == 1 {
            let variant = variants.into_iter().next().unwrap();

            quote! { struct #ident ( #variant );
                #anon
            }
        } else {
            let variants = variants.into_iter().map(|v| quote! { #v ( #v ) });
            quote! { enum #ident { #(#variants,)* }
                #anon
            }
        }
    }

    pub fn generate_op_type(o: &GenerateOp) -> TS2 {
        let ident = o.ident();
        let params = o.params().map(|ty| quote! { params: #ty });
        let fields = o
            .fields()
            .map(|flags| {
                flags.into_iter().map(|f| match f {
                    Flag::Bool(i) => quote! { #i: bool  },
                    Flag::Parameterized(i, t) => quote! { #i: #t },
                })
            })
            .map(|iter| quote! { #(#iter,)* });

        quote! {
            struct #ident {
                #fields
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

impl GenerateSpace {
    fn ident(&self) -> &Ident {
        &self.ident
    }

    fn variants(&self) -> &[Ident] {
        &self.variants
    }
}

#[derive(Debug)]
pub struct GenerateOp {
    scope: Scope,
    fields: Option<Vec<Flag>>,
    params: Option<Type>,
    ident: Ident,
}

impl GenerateOp {
    fn scope(&self) -> &Scope {
        &self.scope
    }

    fn fields(&self) -> Option<&[Flag]> {
        self.fields.as_ref().map(|v| v.as_slice())
    }

    fn params(&self) -> Option<&Type> {
        self.params.as_ref()
    }

    fn ident(&self) -> &Ident {
        &self.ident
    }
}

#[derive(Debug)]
pub struct GenerateRoot {
    name: Ident,
    variants: Vec<Ident>,
}

impl GenerateRoot {
    fn name(&self) -> &Ident {
        &self.name
    }

    fn variants(&self) -> &[Ident] {
        &self.variants
    }
}

impl GenerateRoot {
    pub fn ident_to_string(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Debug)]
pub struct TypeTree {
    root: GenerateRoot,
    spaces: Vec<GenerateSpace>,
    ops: Vec<GenerateOp>,
}

impl TypeTree {
    pub fn spaces(&self) -> &[GenerateSpace] {
        &self.spaces
    }

    pub fn ops(&self) -> &[GenerateOp] {
        &self.ops
    }

    pub fn root(&self) -> &GenerateRoot {
        &self.root
    }
}

impl TypeTree {
    fn from_commands(name: &Ident, cmds: &[TokenizedCommand]) -> Self {
        let ops = Self::generate_ops(cmds);
        let spaces: Vec<_> = Self::generate_spaces(&ops, cmds).collect();
        let root = Self::generate_root(name, cmds, &ops, &spaces);
        Self { root, ops, spaces }
    }

    // generates all the op types
    fn generate_ops(cmds: &[TokenizedCommand]) -> Vec<GenerateOp> {
        cmds.into_iter()
            .map(|cmd| Self::generate_op(cmd))
            .filter(|go| go.is_some())
            .map(|go| go.unwrap())
            .collect()
    }

    // generates an op type from a command if the command has one
    fn generate_op(cmd: &TokenizedCommand) -> Option<GenerateOp> {
        if let Some(o) = cmd.op() {
            return Some(GenerateOp {
                scope: cmd
                    .space()
                    .map(|s| s.ident().unwrap())
                    .map(|s| Scope::Space(s.clone()))
                    .unwrap_or(Scope::Root),
                fields: cmd.flags2().map(|f| f.to_vec()),

                params: cmd.params().map(|p| p.ty().unwrap().clone()),

                ident: capitalize_ident(o.ident().unwrap()),
            });
        }
        None
    }

    fn generate_root_anon(cmds: &[TokenizedCommand]) -> Option<&TokenizedCommand> {
        cmds.into_iter()
            .find(|cmd| cmd.space().is_none() && cmd.op().is_none())
    }

    fn generate_root_ops(ops: &[GenerateOp]) -> impl Iterator<Item = &GenerateOp> {
        ops.into_iter().filter(|op| Scope::Root == op.scope)
    }

    fn generate_root(
        name: &Ident,
        cmds: &[TokenizedCommand],
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

    fn make_anon_op(cmd: &mut TokenizedCommand, ident: String) {
        let i = Ident::new(&(ident + "Anon"), Span::call_site());
        cmd.set_op(CommandToken::new_op(i));
    }

    fn generate_space_anon<'a>(
        ident: &'a Ident,
        cmds: &'a [TokenizedCommand],
    ) -> Option<&'a TokenizedCommand> {
        cmds.into_iter()
            .find(|cmd| cmd.space_matches(ident) && cmd.op().is_none())
    }

    // generates space types from the generated op types
    fn generate_spaces(
        ops: &[GenerateOp],
        cmds: &[TokenizedCommand],
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
