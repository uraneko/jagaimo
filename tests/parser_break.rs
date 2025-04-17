use nikujaga::parse_str;
use nikujaga::{CLICall, CLICommand, CLIOption};

#[test]
#[should_panic]
fn break_on_successive_args1() {
    // cli command: $ cat a.txt b.txt
    let in_call = "a.txt b.txt";

    let out_call = CLICall {
        cmd: CLICommand::Command("".into()),
        opts: vec![
            CLIOption::Option("a.txt".into()),
            CLIOption::Option("b.txt".into()),
        ],
    };

    // parse the command into a CLICall
    let call = parse_str(in_call).unwrap();
    println!("the parsed command is: {:?}", call);

    assert_eq!(call, out_call);
}

#[test]
#[should_panic]
fn break_on_successive_args2() {
    // cli command: $ git config --global user.email aaa@bbb.ccc
    let in_call = "config --global user.email aaa@bbb.ccc";

    let out_call = CLICall {
        cmd: CLICommand::Command("config".to_string()),
        opts: vec![
            CLIOption::Option("--global".into()),
            CLIOption::OptionWithArg {
                opt: "user.email".into(),
                arg: "aaa@bbb.ccc".into(),
            },
        ],
    };

    // parse the command into a CLICall
    let call = parse_str(in_call).unwrap();
    println!("the parsed command is: {:?}", call);

    assert_eq!(call, out_call);
}
