use std::collections::HashMap;

use proc_macro2::Span;
use proc_macro2::TokenStream as TS2;
use quote::quote;
use syn::{Ident, Type};

use super::capitalize_ident;
use crate::parse::flags::Flag;
use crate::parse::{CommandRule, Rules};

impl Rules {
    pub fn generate_root(&self, name: &str) -> TS2 {
        let mut cmds = self.commands();
        let mut types: Vec<GenerateType> = vec![];

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

    pub fn generate_type(&self) {}
}

#[derive(Debug, PartialEq, Clone)]
pub enum GenerateType {
    Root {
        spaces: Option<Vec<Ident>>,
        direct: bool,
    },
    Space {
        ops: Option<Vec<Ident>>,
        direct: bool,
    },
    Operation {
        ident: Ident,
        params: Option<Type>,
        flags: Option<Vec<Flag>>,
    },
    Direct {
        params: Option<Type>,
        flags: Option<Vec<Flag>>,
    },
}

impl GenerateType {
    fn direct(&mut self, b: bool) {
        match self {
            Self::Root { direct, .. } | Self::Space { direct, .. } => *direct = b,
            _ => (),
        }
    }

    fn push_op(&mut self, op: &Ident) {
        if let Self::Space { ops, .. } = self {
            ops.as_mut().unwrap().push(op.clone());
        }
    }

    fn set_direct(self, b: bool) -> Self {
        match self {
            Self::Root { mut direct, .. } | Self::Space { mut direct, .. } => direct = b,
            _ => (),
        }

        self
    }

    fn new_space() -> Self {
        Self::Space {
            ops: None,
            direct: false,
        }
    }
}
