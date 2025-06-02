pub trait Help {
    // impl this if the type has a description to be printed out on call to help
    fn description(&self) -> Option<String> {
        None
    }

    // impl this if the type has links to be printed out on call to help
    fn links(&self) -> Option<Vec<String>> {
        None
    }

    // returns the formatted help message
    fn help(&self) -> String;

    fn format(&self) -> String;

    // generates the USAGE ...
    // instructions from the type's values
    fn usage(&self) -> String;

    // resolves colors for different sections of the help message
    fn colors(&self, target: &str) -> Color {
        match target {
            "main" => Color("\x1b[1;38;2;182;213;132m"),
            "sub" => Color("\x1b[1;38;2;192;122;113m"),
            "reset" => Color("\x1b[0m"),
            _ => Color::default(),
        }
    }
}

pub struct Color(&'static str);

// TODO different color
impl Default for Color {
    fn default() -> Self {
        Color("\x1b[1;38;2;192;122;113m")
    }
}

// impl Help {
//     pub fn help() -> Help {
//         Help {
//             desc: "base 64, 45, 32, 16 encoding/decoding",
//             usage: "mkr [COMMAND] [OPTIONS] [ARGS]",
//             commands: vec![
//                 Cmd {
//                     item: "encode",
//                     aliases: vec![", e"],
//                     desc: "encode given string with the given base encoding",
//                     options: vec![],
//                 },
//                 Cmd {
//                     item: "decode",
//                     aliases: vec![", d"],
//                     desc: "decode given string from the given base encoding",
//                     options: vec![],
//                 },
//                 Cmd {
//                     item: "convert",
//                     aliases: vec![", c"],
//                     desc: "takes an encoded string and transforms it to a different encoding",
//                     options: vec![
//                         Opt {
//                             item: "--src",
//                             alias: ", -s",
//                             desc: "base encoding of the input string to be converted",
//                         },
//                         Opt {
//                             item: "--dest",
//                             alias: ", -d",
//                             desc: "base encoding of the output string to be generated from source base encoded string",
//                         },
//                     ],
//                 },
//                 Cmd {
//                     item: "deduce",
//                     aliases: vec![],
//                     desc: "tries to deduce the given string's base encoding, may get it wrong",
//                     options: vec![],
//                 },
//             ],
//             options: vec![
//                 Opt {
//                     item: "--base",
//                     alias: ", -b",
//                     desc: "base of the encoding/decoding to be applied, used in commands that only need one base option",
//                 },
//                 Opt {
//                     item: "--input",
//                     alias: ", -i",
//                     desc: "input string to be processed",
//                 },
//             ],
//         }
//     }
//
//     fn options(&self) -> Vec<Opt> {
//         self.options
//             .clone()
//             .into_iter()
//             .chain(
//                 self.commands
//                     .iter()
//                     .map(|c| c.options.clone())
//                     .flatten()
//                     .collect::<Vec<Opt>>(),
//             )
//             .collect()
//     }
//
//     fn format(&self) -> String {
//         format!(
//             "{}\n\n{SUP_S}Usage:{RESET_S} {SUB_S}{}{RESET_S}\n\n{SUP_S}Options:{RESET_S} \n{}\n\n{SUP_S}Commands:{RESET_S} \n{}",
//             self.desc,
//             self.usage,
//             self.options()
//                 .into_iter()
//                 .map(|o| {
//                     let delim = o.item.len() + o.alias.len();
//                     let spaces = " ".repeat(18 - delim);
//
//                     format!(
//                         "  {SUB_S}{}{}{RESET_S}{}{}\n",
//                         o.item, o.alias, spaces, o.desc
//                     )
//                 })
//                 .collect::<String>(),
//             self.commands
//                 .iter()
//                 .map(|c| {
//                     let aliases = if c.aliases.is_empty() {
//                         ""
//                     } else {
//                         &c.aliases.join(", ")
//                     };
//                     let delim = c.item.len() + aliases.len();
//                     let spaces = " ".repeat(18 - delim);
//
//                     format!(
//                         "  {SUB_S}{}{}{RESET_S}{}{}\n",
//                         c.item, aliases, spaces, c.desc
//                     )
//                 })
//                 .collect::<String>()
//         )
//     }
// }
