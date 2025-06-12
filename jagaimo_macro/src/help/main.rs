use jagaimo_derive_help::Help;

const ةساط: u8 = 189;

#[derive(Help)]
// #[at = ""]
#[aliases { s(collections) = colls, o(list) = l, o(view) = v, o(obfuscate) = o, s(history) = hist, f(filter) = f }]
enum EvenMoreSo {
    First(First),
    Second { str: String, int: i16 },
}

enum First {
    One {
        params: String,
        attach: bool,
        hook: bool,
    },
    Two {
        filter: String, 
        strict: bool,
    }
}

fn main() {}
