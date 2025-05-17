use nikujaga::parse_str;
use nikujaga::{CLICall, CLICommand, CLIOption};

#[test]
#[should_panic]
fn break_me1() {
    // cli command: $ cat a.txt b.txt
    let in_call = "a.txt b.txt";

    let out_call = CLICall {
        cmd: CLICommand::None,
        opts: vec![
            CLIOption::Param("a.txt".into()),
            CLIOption::Param("b.txt".into()),
        ],
    };

    // parse the command into a CLICall
    let call = parse_str(in_call).unwrap();

    assert_eq!(call, out_call);
}

// TODO there is no way for the lexer to handle this type of
// command for now
// support will be through attribute macros
// in this form when calling the parser
// ```rust
// #[nikujaga_heretical(OptionWithArg, *.*, *.*@*, 2, config)]
// let parsed = parse(input);
// ```
// as seen above, the macro takes at least 3 tokens/items
// the first is a variant of the CLIOption enum
// this tells the macro what the next items represent
// the second is the form of the heretical option value;
// in this case text.other
// the third is the form of the argument that the heretic option takes
// here it is some.text@other
// since OptionWithArg was passed at first, we know that only 1 argument will follow the
// heretic option
// fourth is the number of times an option of this heretical form is supposed to appear
// fifth is the command it could appear under. Here, it is config
// if the cli uses scoped commands (e.g., gh label create)
// then a scope should be passed before the command
// once implemented, this heretic macro should allow the lexer
// to handle special syntax opts correctly
//
// TODO add attr macro support for single case keywords
// i.e., normally if the command has no options
// the the lexer concludes that the command form is
// $ cli_tool arg1 arg2 ... argn
// #[nikujaga_once(Scope(scp), Command(cmd), Options(), Args(*))]
// with this macro, the lexer can understand that the command is actually
// $ cli_tool scp cmd arg1 arg2 argn
// only if the scope token == scp && the command token == cmd
#[test]
#[should_panic]
fn break_me2() {
    // cli command: $ git config --global user.email aaa@bbb.ccc
    let in_call = "config --global user.email aaa@bbb.ccc";

    let out_call = CLICall {
        cmd: CLICommand::Command("config".to_string()),
        opts: vec![
            CLIOption::Param("--global".into()),
            CLIOption::OptionWithArg {
                opt: "user.email".into(),
                arg: "aaa@bbb.ccc".into(),
            },
        ],
    };

    // parse the command into a CLICall
    let call = parse_str(in_call).unwrap();

    assert_eq!(call, out_call);
}
