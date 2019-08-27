use std::fmt::{Display, Formatter, Result};

pub mod ast {
    #[derive(Debug)]
    pub struct TokenStream {
        pub tokens: Vec<Token>,
    }

    #[derive(Debug)]
    pub struct Token {
        pub content: BaseToken,
        pub offset: usize,
    }

    #[derive(Debug)]
    pub enum BaseToken {
        Syntax(SyntaxToken),
        LiteralString(String),
    }

    #[derive(Debug)]
    pub enum SyntaxToken {
        MacroInvocation(MacroInvocationToken),
        QuotedString(String),
        Comment(String),
    }

    #[derive(Debug)]
    pub struct MacroInvocationToken {
        pub name: String,
        pub args: Vec<TokenStream>,
    }
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/m4.rs"));
}

pub use ast::*;

impl Display for TokenStream {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for token in self.tokens.iter() {
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.content)
    }
}

impl Display for BaseToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            BaseToken::Syntax(syntax) => write!(f, "{}", syntax),
            BaseToken::LiteralString(string) => write!(f, "{}", string),
        }
    }
}

impl Display for SyntaxToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            SyntaxToken::MacroInvocation(invocation) => write!(f, "{}", invocation),
            SyntaxToken::QuotedString(invocation) => write!(f, "`{}'", invocation),
            SyntaxToken::Comment(comment) => write!(f, "#{}\n", comment),
        }
    }
}

impl Display for MacroInvocationToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.name)?;
        if let Some((last_arg, most_args)) = self.args.split_last() {
            write!(f, "(")?;
            for arg in most_args {
                write!(f, "{},", arg)?;
            }
            write!(f, "{})", last_arg)?;
        }
        Ok(())
    }
}

fn is_first_arg_of_invocation_with_name<'a>(
    invocation: &'a MacroInvocationToken,
    invocation_name: &str,
    first_arg_name: &str,
) -> bool {
    if invocation.name == invocation_name {
        if let Some(first_arg) = invocation.args.first() {
            if let Some(token) = first_arg.tokens.first() {
                if let BaseToken::Syntax(syntax) = &token.content {
                    match syntax {
                        SyntaxToken::MacroInvocation(inner_invocation) => {
                            if inner_invocation.name == first_arg_name {
                                return true;
                            }
                        }
                        SyntaxToken::QuotedString(quoted_string) => {
                            if quoted_string == first_arg_name {
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    false
}

fn is_definition<'a>(invocation: &'a MacroInvocationToken, macro_name: &str) -> bool {
    is_first_arg_of_invocation_with_name(invocation, "define", macro_name)
}

fn is_undefinition<'a>(invocation: &'a MacroInvocationToken, macro_name: &str) -> bool {
    is_first_arg_of_invocation_with_name(invocation, "undefine", macro_name)
}

impl TokenStream {
    pub fn get_macro_definitions<'a>(&'a self, macro_name: &str) -> Vec<&'a Token> {
        let mut definitions = vec![];
        for token in self.tokens.iter() {
            if let BaseToken::Syntax(syntax) = &token.content {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_definition(invocation, macro_name) {
                        definitions.push(token);
                    }
                }
            }
        }
        definitions
    }

    pub fn get_macro_invocations<'a>(&'a self, macro_name: &str) -> Option<Vec<&'a Token>> {
        let mut invocations = vec![];
        let mut is_defined = false;
        let mut does_definition_exist = false;
        for token in self.tokens.iter() {
            if let BaseToken::Syntax(syntax) = &token.content {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_defined && invocation.name == macro_name {
                        invocations.push(token);
                    } else if is_definition(invocation, macro_name) {
                        is_defined = true;
                        does_definition_exist = true;
                    } else if is_undefinition(invocation, macro_name) {
                        is_defined = false;
                    }
                }
            }
        }
        if does_definition_exist {
            Some(invocations)
        } else {
            None
        }
    }

    pub fn rename_macro<'a>(&'a mut self, macro_name: &str, new_macro_name: &str) -> bool {
        let mut is_defined = false;
        let mut does_definition_exist = false;
        for token in self.tokens.iter_mut() {
            if let BaseToken::Syntax(syntax) = &mut token.content {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_defined && invocation.name == macro_name {
                        invocation.name = new_macro_name.to_string();
                    } else if invocation.name == "define" {
                        if let Some(first_arg) = invocation.args.first_mut() {
                            if let Some(first_token) = first_arg.tokens.first_mut() {
                                if let BaseToken::Syntax(syntax) = &mut first_token.content {
                                    match syntax {
                                        SyntaxToken::MacroInvocation(inner_invocation) => {
                                            if inner_invocation.name == macro_name {
                                                inner_invocation.name = new_macro_name.to_string();
                                                is_defined = true;
                                                does_definition_exist = true;
                                            }
                                        }
                                        SyntaxToken::QuotedString(quoted_string) => {
                                            if quoted_string == macro_name {
                                                *quoted_string = new_macro_name.to_string();
                                                is_defined = true;
                                                does_definition_exist = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    } else if is_undefinition(invocation, macro_name) {
                        is_defined = false;
                    }
                }
            }
        }
        does_definition_exist
    }
}
