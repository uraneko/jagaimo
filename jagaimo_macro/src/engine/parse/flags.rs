use super::{
    Ident, IdentExt, Parse, ParseResult, ParseStream, Scope, Token, Type, braced, bracketed,
    parenthesized,
};
use crate::engine::{discriminant, dummy_ident, dummy_type};

#[derive(Debug)]
pub struct FlagsRule {
    scope: Scope,
    flags: Vec<Flag>,
}

#[derive(Debug)]
pub enum Flag {
    Bool(Ident),
    Parameterized(Ident, Type),
    Params(Ident, Vec<Type>),
}

#[derive(Debug, Default)]
pub struct FlagsVec {
    flags: Vec<Flag>,
}

impl Parse for Flag {
    fn parse(s: ParseStream<'_>) -> ParseResult<Self> {
        let ident = Ident::parse(s)?;
        if s.is_empty() {
            return Ok(Self::Bool(ident));
        }

        let content;
        _ = parenthesized!(content in s);
        let mut tys = content
            .parse_terminated(Type::parse, Token![,])?
            .into_iter()
            .collect::<Vec<Type>>();
        Ok(match tys.len() {
            0 => Self::Bool(ident),
            1 => Self::Parameterized(ident, tys.pop().unwrap()),
            _ => Self::Params(ident, tys),
        })
    }
}

impl FlagsVec {
    fn push(&mut self, f: ParseResult<Flag>) {
        if let Ok(f) = f {
            self.flags.push(f);

            return;
        }

        println!("flag was an error {:?}", f);
    }

    fn flags(self) -> Vec<Flag> {
        self.flags
    }
}

impl Parse for FlagsVec {
    fn parse(s: ParseStream) -> ParseResult<Self> {
        let mut f = FlagsVec::default();
        f.push(Flag::parse(s));

        while !s.is_empty() {
            _ = <Token![,]>::parse(s)?;

            f.push(Flag::parse(s));
        }

        Ok(f)
    }
}

impl Parse for FlagsRule {
    fn parse(stream: ParseStream) -> ParseResult<Self> {
        let mut content;
        let brace = braced!(content in stream);

        let mut idents: Vec<Ident> = vec![];
        let mut scope = vec![];
        let mut temp;
        loop {
            if content.peek(Ident::peek_any) {
                scope.extend([Ident::parse(&content)?, {
                    let _paren = parenthesized!(temp in content);

                    Ident::parse(&temp)?
                }]);
            } else {
                break;
            }
        }
        let bracket = bracketed!(content in content);
        let arr = FlagsVec::parse(&content)?.flags;

        Ok(Self {
            scope: scope.into(),
            flags: arr,
        })
    }
}

impl FlagsRule {
    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn flags(&self) -> &[Flag] {
        &self.flags
    }
}

impl Flag {
    pub fn is_bool(&self) -> bool {
        discriminant(self) == discriminant(&Self::Bool(dummy_ident()))
    }

    pub fn is_param(&self) -> bool {
        discriminant(self) == discriminant(&Self::Parameterized(dummy_ident(), dummy_type()))
    }

    pub fn is_params(&self) -> bool {
        discriminant(self) == discriminant(&Self::Params(dummy_ident(), vec![]))
    }
}
