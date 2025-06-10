use jagaimo_derive_help::Help;

#[derive(Debug, Clone, Help)]
struct Examplary {
    one: u8,
    _2: String,
    三: EvenMoreSo,
}

#[derive(Debug, Clone, Help)]
enum EvenMoreSo {
    First(bool),
    Second(String),
    Third(Vec<u8>),
}

fn main() {}
