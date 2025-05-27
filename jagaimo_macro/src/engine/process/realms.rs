use super::super::{Flag, FlagsRule, OperationsRule, ParamsRule, RealmsRule, RuleBook};
use super::{
    super::{Span, capitalize_ident, capitalize_ident_to_string},
    Ident, TS2, Type, ident_to_variant, quote,
};

impl RuleBook {
    // returns the operations that can be used directly on this realm
    fn realm_ops(&self, r: &Ident) -> impl Iterator<Item = &Ident> {
        self.ops()
            .into_iter()
            .filter(|ops| ops.scope().matches_realm(r))
            .map(|ops| ops.ops())
            .flatten()
    }

    // returns the flags that can be used directly on this realm
    fn realm_flags(&self, r: &Ident) -> impl Iterator<Item = &Flag> {
        self.flags()
            .into_iter()
            .filter(|flags| flags.scope().matches_realm(r))
            .map(|flags| flags.flags())
            .flatten()
    }

    // returns the params that can be used directly on this realm
    fn realm_params(&self, r: &Ident) -> impl Iterator<Item = &Type> {
        self.params()
            .into_iter()
            .filter(|params| params.scope().matches_realm(r))
            .map(|params| params.params())
            .flatten()
    }

    // would also need to check if
    // realm can run commands ope-less
    // and add a variant for it then
    // RealmNameNoOperation
    pub fn generate_realms(&self) -> Vec<TS2> {
        self.realms_vec()
            .into_iter()
            .map(|r| {
                let ops = self
                    .realm_ops(&r)
                    .map(|o| ident_to_variant(&capitalize_ident(&o)));
                if self.realm_flags(&r).count() + self.realm_params(&r).count() > 0 {
                    let s = capitalize_ident_to_string(r) + "Bare";
                    let i = Ident::new(&s, Span::call_site());

                    let ops = ops.chain([ident_to_variant(&i)]);

                    return quote! {
                        enum #r {
                            #(#ops,)*
                        }
                    };
                }

                // TODO the derives
                quote! {
                    enum #r {
                        #(#ops,)*
                    }
                }
            })
            .collect::<Vec<TS2>>()
    }
}
