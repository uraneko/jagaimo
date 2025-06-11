use jagaimo_derive_help::Help;

// #[derive(Help)]
// struct Examplary {
//     one: u8,
//     _2: String,
//     三: EvenMoreSo,
// }

#[derive(Help)]
// #[at = ""]
enum EvenMoreSo {
    First(First),
    Second { str: String, int: i16 },
}

#[derive(Help)]
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
