pub mod parser;

// #[derive(IntoCLI)]
struct SomeCLITool {}

// the derive
struct IntoCLI {
    cli: SomeCLITool,
    graph: CLIGraph,
}

struct CLIGraph {}

// this calls the help function when $ cmd help is used in the shell
// behind the hood, the attribute macro call registers this function to the help cli action with no
// arguments
// #[nikujaga::cmd(help)]
fn help() {}

// cli action takes no args and always calls the help function with a false value for the with_colors arg
// #[nikujaga::cmd(help with_colors=false)]
// fn help(with_colors: bool) {}

// cli action is help, it takes 1 arg --with-colors of value true or false
// #[nikujaga::cmd(help [--with-colors -c c = true | false])]
fn help_colors() {}

// cli action is help, it takes 1 arg --with-colors of any string value
// #[nikujaga::cmd(help [--with-colors -c c = string])]
// fn help_colors() {}

// #[nikujaga::cmd(help { with_colors=[--with-colors -c c]} )]
// fn help(with_colors: bool) {}
