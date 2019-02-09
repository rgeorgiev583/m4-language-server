use std::env::args;
use std::fs::File;
use std::io::Read;

pub use m4_language_server::parser;

#[derive(Debug)]
enum Action {
    None,
    DumpAst,
    PrintMacroDefinition(String),
    PrintMacroInvocations(String),
    RenameMacro(String, String),
}

fn process_input<T: Read>(mut input: T, action: &Action) {
    let mut input_str = String::new();
    input.read_to_string(&mut input_str).unwrap();
    let mut source = parser::source(input_str.as_str()).unwrap();

    match action {
        Action::DumpAst => println!("{:?}", source),
        Action::PrintMacroDefinition(macro_name) => {
            if let Some(macro_definition) = source.get_macro_definition(macro_name.as_str()) {
                println!("{}", macro_definition);
            } else {
                println!("macro definition not found");
            }
        }
        Action::PrintMacroInvocations(macro_name) => {
            let macro_invocations = source.get_macro_invocations(macro_name.as_str());
            println!("found a total of {} invocations:", macro_invocations.len());
            for invocation in macro_invocations {
                println!("{}", invocation);
            }
        }
        Action::RenameMacro(macro_name, new_macro_name) => {
            source.rename_macro(macro_name.as_str(), new_macro_name.as_str());
            println!("{}", source);
        }
        _ => eprintln!("error"),
    }
}

fn main() {
    let mut args = args().skip(1);
    if let Some(action_str) = args.next() {
        let action = match action_str.as_str() {
            "dump-ast" => Action::DumpAst,
            "print-macro-definition" => {
                let macro_name = args.next().unwrap();
                Action::PrintMacroDefinition(macro_name)
            }
            "print-macro-invocations" => {
                let macro_name = args.next().unwrap();
                Action::PrintMacroInvocations(macro_name)
            }
            "rename-macro" => {
                let macro_name = args.next().unwrap();
                let new_macro_name = args.next().unwrap();
                Action::RenameMacro(macro_name, new_macro_name)
            }
            _ => Action::None,
        };
        if args.len() > 0 {
            args.for_each(|filename| process_input(File::open(filename).unwrap(), &action))
        };
    }
}
