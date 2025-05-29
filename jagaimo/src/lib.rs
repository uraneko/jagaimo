use jagaimo_macro::jagaimo;

jagaimo! {
    #[
        no_help,
        no_version,
        branch_off_root,
        nu_cmp,
        fish_cmp,
        root_name = "jagaimo",
        ignore_naming_conventions,
        // no_auto_alias,
        disable_derives(Debug, Clone)
    ]

    aliases {
        o(add) = A,
        s(remote) = rmt,
        s(collections) = colls,
    }

    c { s(history) o(view, list) [ (i32), filter(String), include, query(String) ] }
    c { [ ((String, f64)), size(Dimensions), show_all ] }
    // t { s(colls) o(list) |_, base: String|
    //         { if base == "_" { auto as bool } else { base "BASE{base}" as Base } }
    // }
    // t { s(history) |use_max| { use 453 as u32} }
}
