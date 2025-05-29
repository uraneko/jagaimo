use proc_macro2::Span;
use syn::Ident;
use syn::parse::{Parse, ParseStream, Result as ParseResult};

use crate::process::dummy_ident;

use std::mem::discriminant;

#[derive(Debug, Default, PartialEq)]
pub enum Scope {
    #[default]
    Root,
    Space(Ident),
    SpaceOperation {
        space: Ident,
        op: Ident,
    },
    Operation(Ident),
}

impl Scope {
    // returns option of the ident of space only if self is of the space variant
    // otherwise returns None
    pub fn space(&self) -> Option<&Ident> {
        let Scope::Space(i) = self else {
            return None;
        };

        Some(i)
    }

    pub fn space_op(&self) -> Option<[&Ident; 2]> {
        let Scope::SpaceOperation { space, op } = self else {
            return None;
        };

        Some([space, op])
    }

    pub fn op(&self) -> Option<&Ident> {
        let Scope::Operation(i) = self else {
            return None;
        };

        Some(i)
    }

    pub fn is_root(&self) -> bool {
        self == &Scope::Root
    }

    pub fn is_space(&self) -> bool {
        discriminant(self) == discriminant(&Self::Space(dummy_ident()))
    }

    pub fn is_space_op(&self) -> bool {
        discriminant(self)
            == discriminant(&Self::SpaceOperation {
                space: dummy_ident(),
                op: dummy_ident(),
            })
    }

    pub fn is_op(&self) -> bool {
        discriminant(self) == discriminant(&Self::Operation(dummy_ident()))
    }

    pub fn matches_space(&self, r: &Ident) -> bool {
        if let Some(space) = self.space() {
            return space == r;
        }

        false
    }

    pub fn matches_space_op(&self, ro: &[&Ident; 2]) -> bool {
        if let Some([r, o]) = self.space_op() {
            return r == ro[0] && o == ro[1];
        }

        false
    }

    pub fn matches_op(&self, o: &Ident) -> bool {
        if let Some(op) = self.op() {
            return op == o;
        }

        false
    }
}

impl From<Vec<Ident>> for Scope {
    fn from(mut value: Vec<Ident>) -> Self {
        match value.len() {
            0 => Self::Root,
            2 => {
                let item = value.pop().unwrap();
                if value[0] == Ident::new("r", Span::call_site()) {
                    Self::Space(item)
                } else {
                    Self::Operation(item)
                }
            }
            4 => Self::SpaceOperation {
                space: value.remove(1),
                op: value.pop().unwrap(),
            },
            _ => panic!("scope cant take only: 0, 2 or 4 idents"),
        }
    }
}
