use jagaimo_macro::jagaimo;

jagaimo! {
    #[
        no_help,
        no_version,
        nu_cmp,
        fish_cmp,
        root_name = "jagaimo_oishii_desu",
        // ignore_naming_conventions,
        // no_auto_alias,
        derives(Debug, Clone),
    ]

    o(add) = A
    s(remote) = rmt
    s(collections) = colls

    c { s(hosts) o(view) [ <i32> filter<String> colored query<String> ] }
    c { s(history) o(view) [ flatten encoding<String> max<u8> max<f64> ] }
    c { s(history) o(annotate) [ max<u8> verbose tags<Vec<String>> ] }
    c { [ <(String, f64)> size<Dimensions> show_all ] }
    c { s(collections) o(obfuscate) [ <std::fs::File> allocate  rand<f64> hash<String> fuzzing algorithm<String> ] }
    c { o(view) [ list_all theme<String> ] }
    c { s(collections) o(list) [ <Vec<f64>> long pipe_into_list_of<String> output_file<std::fs::File> ] }
    // t { s(colls) o(list) |_ base: String|
    //         { if base == "_" { auto as bool } else { base "BASE{base}" as Base } }
    // }
    // t { s(history) |use_max| { use 453 as u32} }
}

struct Dimensions {
    x: u8,
    y: u8,
}
