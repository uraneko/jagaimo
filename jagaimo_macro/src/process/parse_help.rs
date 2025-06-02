use syn::Ident;
use toml::Table;

use crate::resolve_crate::{ReadHelp, ResolveCrate};

pub fn read_help() -> toml::Table {
    let data = ResolveCrate::new().read_help();

    data.into_table()
}

pub fn extract_space_description(ident: &str, descrs: &Table) {
    let description = descrs
        .get(ident)
        .map(|d| d.as_str().unwrap())
        .unwrap_or(&("The ".to_owned() + ident + " cli tool"));
}

pub fn extract_root_description(ident: &str, descrs: &Table) {
    let description = descrs
        .get("root")
        .map(|d| d.as_str().unwrap())
        .unwrap_or(&("The ".to_owned() + ident + " space"));
}

pub fn extract_op_description(s: &str, o: &str, descrs: &Table) {
    let description = descrs
        .get(s)
        .map(|s| s.get(o).map(|op| op.as_str().unwrap()))
        .unwrap_or(Some(("The ".to_owned() + o + " operation").as_str()));
}

pub fn extract_space_help(s: &Ident) {}

use super::cmd_type_tree::GenerateRoot;

pub fn extract_from_tables() {
    let t = read_help();
    let [d, l, s, o, f] = [
        t.get("descriptions")
            .map(|m| m.as_table().unwrap())
            .unwrap(),
        t.get("links").map(|m| m.as_table().unwrap()).unwrap(),
        t.get("spaces").map(|m| m.as_table().unwrap()).unwrap(),
        t.get("operations").map(|m| m.as_table().unwrap()).unwrap(),
        t.get("flags").map(|m| m.as_table().unwrap()).unwrap(),
    ];

    let root = extract_root_help("jagaimo", d, l, s, o, f);

    // let spaces = generate_spaces.map extract_space_help(spc, d,l,s,o,f);
    // let ops = generate_op.map extract_op_help(op, d,l,s,o,f)
}

pub fn extract_root_help(
    ident: &str,
    descr: &Table,
    links: &Table,
    spaces: &Table,
    ops: &Table,
    flags: &Table,
) {
    let d = descr
        .get("root")
        .map(|d| d.as_str().unwrap())
        .unwrap_or(ident);
    println!("{}\n\nLinks:", d);

    let l = links.get("root").map(|m| m.as_table().unwrap().into_iter());
    l.unwrap().for_each(|(k, v)| println!("{} {}", k, v));
    println!("\nSpaces:");

    spaces
        .into_iter()
        .for_each(|(k, v)| println!("{} {}", k, v));
    println!("\nOperations:");

    let o = ops.get("root").map(|m| m.as_table().unwrap().into_iter());
    o.unwrap().for_each(|(k, v)| println!("{} {}", k, v));
    println!("\nFlags:");

    let f = flags.get("root").map(|m| m.as_table().unwrap().into_iter());
    f.unwrap().for_each(|(k, v)| println!("{} {}", k, v));
}
