use syn::parse::{Parse, ParseStream, Result as PRes};
use syn::token::Bracket;

pub mod attrs;
pub mod flags;
pub mod rules;
pub mod scope;

pub use attrs::Attrs;
pub use flags::Flag;
pub use rules::Rules;
pub use scope::AliasScope;

pub type MacroInput = (Attrs, Rules);

pub fn parse_attrs_rules(stream: ParseStream) -> PRes<MacroInput> {
    Ok((Attrs::parse(stream)?, Rules::parse(stream)?))
}
