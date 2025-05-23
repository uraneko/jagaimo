use nikujaga_macro::nikujaga;

// DOCS
// given a cli command of unknown structure
// plus a set of lexer rules
// the lexer transforms it into a sequence of tokens
// given a random sequence of tokens
// plus a set of paresr rules
// the parser transforms the tokens into a cli command struct; useful to the end user
//
// sequence of tokens is easily defined
// token can only be one of a finite set of token variants
//
// 2 ways
// -
// set rules by executable
// then parse by scope/action
// this gives very specific structs/enums
// -
// set rules by executable
// parse for general Command struct
// this gives generic struct values; works for eeverything
//
// implement both ways

// struct ShortFormat;

// internal implementation details
// struct Parser<ShortFormat> {
//     data: std::marker::PhantomData<ShortFormat>,
// }

// DOCS
// make it so that commands get lexed then parsed to a format then parsed to command representation
// command repr can go to any parsed format and be derived from any parsed format
// lex and parse traits wil be key, with their rules implementations

// conditional rules:
// is the command scoped? does it have args? does it have flags? an action? ...
//
// categorical rules:
// this token is always an action when it is the 1st or 2nd token
// e.g., create, remove, edit, view are always actions
//
// transofrmative rules:
// transform some tokens into other tokens
// --base _ => --auto

// Nikuajaga! {
//     scopes: [],
//     actions: [],
// }

// only commands that are defined and derived can be parsed
// otherwise parser returns an error
//
// #[derive(Parse)]
// enum Remote {
//  Add(RemoteAdd)
//  Remove(RemoteRemove),
//  Edit(RemoteEdit),
//  View(RemoteView),
// }
//
// #[derive(Parse)]
// struct RemoteAdd {
//  uri: URI,
//  name: String,
//  mirror: Mirror
// }
//
// #[derive(Parse)]
// struct URI (String)
//
// #[derive(Parse)]
// enum Mirror {
//  Push,
//  Fetch,
//  All,
// }
// this allows parsing of git remote add ... commands
// but not any other git remote <operation> ... commands
// nor any other git <region> ... commands

/// Description can be either
/// inline: inside every item of the macro struct
/// embedded: comes after items are defined. lists all items descriptions
/// json: inside a json file
///
/// the macro is not a real struct,
/// you only need pass it what your cli needs
///
/// if your cli commands dont use regions
/// then skip the regions field
///
/// likewise for operations
///
/// you can use the macro rules to specify:
/// * allowed sub scopes of a scope
/// -> ops { !region [add, remove, view] }
/// <- no region, cli operations are add, remove and view,
/// <- it is a compiler/expand error not to specify the rules necessary for the defined ops
/// you can do the same to specify regions or flags
/// * allowed parameter type for a scope
/// -> params { operation(add) [String, u32] }
/// <- all add operations under any scope can only take either a String or a u32
/// <- any other params are a compiler/expand error
/// specific rules always shadow general rules
/// add transform rule -> help => --help | --base _ => --auto | yes => true, no => false
//
// HOWTO:
// * define all the possible rules
// * define all the possible flags (AliasWhenPossible is a flag), flags apply to rule types
// (r.g., a flag for regions rule)
// * define all the possible attributes (#[no_help] is an attribute), attrs apply to the macro
// * define the valid syntax of all the rules, flags and attributes
// * parse all the rules, flags and attrs into their asts
// * evaluate all the macro attributes
// * evaluate all the rules and their flags
// * build the CLI tool struct type
// - use the name of the crate unless a name is specified in an attr
// * implement the Lex and Parse traits based on the rules asts
// <- this gives us the parser
// * implement the Command trait
// <- this gives us the cli help and version commands
// plus a default impl of command(self) which can only call version, help or panic
// the user then needs to implement their own command method for the generated type
// * if their attributes are set, then generate fish and nushell completion files
//
// TODO: add support for auto gen of fish shell and nushell completions (completion.fish/nu files)
// for the macro created cli tool
// nikujaga! {
//     semantics: Vec<Semantics>,
//     values: Vec::from([
//         meta: Semantics::OPS,
//         functional: Semantics::BARE
//     ]),
//
//     regions: Regions {
//         syntax: Syntax,
//         values: Vec<String>,
//
//     },
//     // AliasWhenPossible
//     // vec![
//     //  collections, remote, databases, history, help <- help is auto generated
//     // ]
//     operations: Operations {
//         syntax: Syntax,
//         values: Vec<Scoped { scope: Scope,  values: Vec<String> }>,
//     },
//     // syntax UpperShort LowerLong | Snake
//     // <- the 2 syntaxes means that user of cli can pass
//     // either cli collections list ...
//     // or     cli collections l ...
//     // both would be valid
//     // vec![
//     //  { collections [list, add, remove, edit, view] }
//     //  same as { region(collections) [list, add, remove, edit, view] }
//     //  <- operations of the collections region
//     //  { [reset, reload, meta, version <- version is auto added here ] }
//     //  same as { !region [reset, reload, meta, version ] }
//     //  <- operations that apply directly to cli tool
//     //  ...
//     // ]
//     flags: Flags {
//         syntax: Syntax,
//         values: Vec<Scoped{ scope: Scope, values: Vec<String> }>,
//     },
//     // syntax LowerLong | Kebab Snake | AliasWhenPossible
//     // all flags chars have to be in lower case
//     // *  both kebab and snake cases work
//     // -> both --my-flag and --my_flag are valid
//     // *  both lowerlong and lowershort work
//     // -> this makes both --my-flag --my_flag valid
//     // flag --my-flag wont have an alias
//     // but  --flag can have -f or -F aliases as a short form
//     // as long as other flags didnt already take both aliases
//     // then --flag wont have any aliases either
//     // vec![
//     //  { operation(add) [ git ] }, <- for all scopes that end with add
//     //  { r(colls) o(add) [ filter-existing ] }
//     // <- for the root -> r(colls) -> o(add) scope
//     // overwrites the more general o(add) rule
//     //  { [ delim, glob ] } <- for the root scope
//     // ]
//     params: Params {
//         values: Vec<Scoped{ scope: Scope, values: Vec<String> }>,
//     },
//     // NoDirectParams <- turns off scope params = all params must come from a flag
//     // vec![
//     //  { operation(add) [String] },
//     //  all add operations no matter what region (including !region) they are called on
//     //  can only take a single string parameter
//     //  * this is valid:     cli region add "my string value"
//     //  * also valid:        cli add "my string value"
//     //  * this is not valid: cli region add 765
//     //  * also not valid:    cli add "some str" "and then" "another"
//     //  { ! [u8, String, String+u8] },
//     //  this makes this valid:      cli "str"
//     //  as well as this:            cli 23
//     //  while this is invalid:      cli 434 <- 434 > u8::MAX
//     //  this is also valid:         cli 123 "string" <- due to the last rule String+u8
//     //  params are interchangeable: cli "string" 123
//     //  { region(collections) operation(add) Flag(source) [Vec<String>, Path] },
//     //  rule for the same operation add that is defined above for all uses
//     //  this smaller scoped rule takes precedence over the more general rule
//     //  so this:  cli collections add "this" "and that" <- is valid
//     //  but this: cli collections add "str" <- is not valid
//     //  (cant use general rule, when specific rule exists)
//     //  { operation(add) flag(git) [URI] }
//     //  the previously defined git flag of the add operation can only take a param of
//     //  the URI type
//     // ]
//     descriptions: Descriptions,
//
//     // TODO use rust's type system to check validity of passed parameters
//     //
// }

nikujaga! {
    r { [ collections, tags, history, algos ] }
    o { collections [ list, add, remove, edit, view ] }
    f { r(collections) o(list) [ max, query, tag ] }
    p { r(history) o(view) [ u8, (String, bool), (String, Vec<i32>, char) ] }
}

// add trait Command {
// command(&self) -> ??? {} <- needs default impl in the macro
// help(&self) -> ???; <- implement in the macro
// version(&self) -> ???; <- implement in the macro
// help and version could have default impls that are used when auto gen is turned off
// }

// DOCS core concepts are regions, operations, flags and parameters
// definition of a cli tool
//
// semantic representation
// a single cli tool commands can be either:
// cli region operation flags parameters
// cli operation flags parameters
// cli flags parameters
// a cli tool may make use of more than one of these
//
// syntatic representation
// given a hypothetical region 'external-databases, it can be written as:
// Syntax{ Capital, Kebab } External-databases
// Syntax{ Title, Kebab } External-Databases
// Syntax{ Capital, Snake } External_databases
// Syntax{ Title, Snake } External_Databases
// Syntax{ Normal, Kebab } external-databases
// Syntax{ Normal, Snake } external_databases
// Syntax { Short, Snake } ex_db
// ...
// a flag can be represented in many different ways
// --some-flag
// --some_flag
// -F
// -f
//
//
// functional representation

// DOCS
// 2 ways this could go
// 1 /
// decide default parser behavior
// implement it
//
// 2 /
// implement various parser behaviors
// impl default parser from them
//
// + /
// users use default if enough
// else extend behavior with macros

// NOTE default behaviour
// no scope
// yes action
// flag followed by arg(s) is an opt
// help command is auto generated
// version command is auto generated

// default behaviour is no scope
// override it
// #[scope]
//
// default behaviour is action
// override it
// #[no_action]
//
// when --global is found
// it is treated as a flag
// even when arguments succeed it
// #[flags(--global, --max)]
//
// if you find remote then we have a scope and no action behavior
// #[command(scope = remote, action = false)]
//
// lexer will output Str(yes) tokens into Bool(true), same for no
// #[bool(yes = true, no = false)] // lexer
//
// lexer will output Int(0) tokens into Bool(false), same for 1
// #[bool(0 = false, 1 = true)] // lexer
//
// alias for scope | action | flag | opt
// NOTE low priority
// #[alias(action = help | H | h)] // lexer
//
// bind command syntax
// NOTE low priority
// #[syntax(action = Syntax::SingleHyphenUpperCase)]
//
// replace expressions with others
// NOTE low priority
// #[replace(opt = (--base, _), flag = --auto)] // lexer
//
// dont auto generate a help command
// NOTE low priority
// need to write the help gen logic first
// #[no_help]
//
// dont auto generate a version command
// NOTE same as help
// #[no_version]
//
// #[derive(nikujaga)]
// struct ExampleParser {}
