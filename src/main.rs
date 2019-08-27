#![feature(exact_size_is_empty)]

use std::env::args;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, stdin, Read};

pub use m4_language_server::parser::{self, ParseError};

#[derive(Debug)]
enum Error {
    RuntimeError(String),
    ParseError(ParseError),
    IoError(io::Error),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ParseError(error) => Some(error),
            Error::IoError(error) => Some(error),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::RuntimeError(description) => write!(f, "runtime error: {}", description),
            Error::ParseError(error) => write!(f, "parse error: {}", error),
            Error::IoError(error) => write!(f, "I/O error: {}", error),
        }
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error::ParseError(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IoError(error)
    }
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Action {
    None,
    DumpAst,
    PrintMacroDefinitions(String),
    PrintMacroInvocations(String),
    RenameMacro(String, String),
}

fn print_underlined_title(title: &str) {
    println!("{}", title);
    println!(
        "{}",
        std::iter::repeat("=").take(title.len()).collect::<String>()
    );
}

fn process_input<T: Read>(filename: &str, mut input: T, action: &Action) -> Result<()> {
    let mut input_str = String::new();
    input.read_to_string(&mut input_str)?;
    let mut input_ast = parser::parse(input_str.as_str())?;
    match action {
        Action::DumpAst => {
            if filename != "" {
                let title = format!("AST of file `{}`:", filename);
                print_underlined_title(title.as_str());
            }
            println!("{:?}", input_ast);
        }
        Action::PrintMacroDefinitions(macro_name) => {
            let macro_definitions = input_ast.get_macro_definitions(macro_name.as_str());
            print!(
                "found a total of {} (re)definitions of the `{}` macro",
                macro_definitions.len(),
                macro_name
            );
            if filename != "" {
                print!(" in file `{}`", filename);
            }
            if !macro_definitions.is_empty() {
                print!(":");
            }
            println!();
            for definition in macro_definitions {
                println!("* `{}` at offset {}", definition, definition.offset);
            }
        }
        Action::PrintMacroInvocations(macro_name) => {
            let macro_invocations = input_ast.get_macro_invocations(macro_name.as_str());
            print!(
                "found a total of {} invocations of the `{}` macro",
                macro_invocations.len(),
                macro_name,
            );
            if filename != "" {
                print!(" in file `{}`", filename);
            }
            if !macro_invocations.is_empty() {
                print!(":");
            }
            println!();
            for invocation in macro_invocations {
                println!("* `{}` at offset {}", invocation, invocation.offset);
            }
        }
        Action::RenameMacro(macro_name, new_macro_name) => {
            if filename != "" {
                let title = format!(
                    "contents of file `{}` after renaming of the `{}` macro to `{}`:",
                    filename, macro_name, new_macro_name
                );
                print_underlined_title(title.as_str());
            }
            input_ast.rename_macro(macro_name.as_str(), new_macro_name.as_str());
            println!("{}", input_ast);
        }
        _ => {}
    }
    println!();
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args().skip(1);
    let action_str = args
        .next()
        .ok_or(Error::RuntimeError("no subcommand specified".to_string()))?;
    let action = match action_str.as_str() {
        "dump-ast" => Action::DumpAst,
        "print-macro-definitions" => {
            let macro_name = args
                .next()
                .ok_or(Error::RuntimeError("no macro name specified".to_string()))?;
            Action::PrintMacroDefinitions(macro_name)
        }
        "print-macro-invocations" => {
            let macro_name = args
                .next()
                .ok_or(Error::RuntimeError("no macro name specified".to_string()))?;
            Action::PrintMacroInvocations(macro_name)
        }
        "rename-macro" => {
            let macro_name = args.next().ok_or(Error::RuntimeError(
                "no old macro name specified".to_string(),
            ))?;
            let new_macro_name = args.next().ok_or(Error::RuntimeError(
                "no new macro name specified".to_string(),
            ))?;
            Action::RenameMacro(macro_name, new_macro_name)
        }
        _ => Action::None,
    };
    if args.is_empty() {
        let stdin = stdin();
        let input = stdin.lock();
        process_input("", input, &action)
    } else {
        args.try_for_each(|filename| {
            process_input(filename.as_str(), File::open(filename.as_str())?, &action)
        })
    }
}
