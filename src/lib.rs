use std::fmt::{Display, Formatter, Result};

pub mod ast {
    #[derive(Debug)]
    pub struct Source {
        pub tokens: Vec<Token>,
    }

    #[derive(Debug)]
    pub enum Token {
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
        pub args: Vec<Source>,
    }
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/m4.rs"));
}

pub use ast::*;

impl Display for Source {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for token in self.tokens.iter() {
            write!(f, "{}", token)?
        }
        Ok(())
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Token::Syntax(syntax) => write!(f, "{}", syntax),
            Token::LiteralString(string) => write!(f, "{}", string),
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
            if let Some(Token::Syntax(syntax)) = first_arg.tokens.first() {
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
    false
}

fn is_definition<'a>(invocation: &'a MacroInvocationToken, macro_name: &str) -> bool {
    is_first_arg_of_invocation_with_name(invocation, "define", macro_name)
}

fn is_undefinition<'a>(invocation: &'a MacroInvocationToken, macro_name: &str) -> bool {
    is_first_arg_of_invocation_with_name(invocation, "undefine", macro_name)
}

impl Source {
    pub fn get_macro_definition<'a>(
        &'a self,
        macro_name: &str,
    ) -> Option<&'a MacroInvocationToken> {
        for token in self.tokens.iter() {
            if let Token::Syntax(syntax) = token {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_definition(invocation, macro_name) {
                        return Some(invocation);
                    }
                }
            }
        }
        None
    }

    pub fn get_macro_invocations<'a>(&'a self, macro_name: &str) -> Vec<&'a MacroInvocationToken> {
        let mut invocations = vec![];
        let mut is_defined = false;
        for token in self.tokens.iter() {
            if let Token::Syntax(syntax) = token {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_defined && invocation.name == macro_name {
                        invocations.push(invocation);
                    } else if is_definition(&invocation, macro_name) {
                        is_defined = true;
                    } else if is_undefinition(&invocation, macro_name) {
                        is_defined = false;
                    }
                }
            }
        }
        invocations
    }

    pub fn rename_macro<'a>(&'a mut self, macro_name: &str, new_macro_name: &str) {
        let mut is_defined = false;
        for token in self.tokens.iter_mut() {
            if let Token::Syntax(syntax) = token {
                if let SyntaxToken::MacroInvocation(invocation) = syntax {
                    if is_defined && invocation.name == macro_name {
                        invocation.name = new_macro_name.to_string();
                    } else if invocation.name == "define" {
                        if let Some(first_arg) = invocation.args.first_mut() {
                            if let Some(first_token) = first_arg.tokens.first_mut() {
                                if let Token::Syntax(syntax) = first_token {
                                    match syntax {
                                        SyntaxToken::MacroInvocation(inner_invocation) => {
                                            if inner_invocation.name == macro_name {
                                                inner_invocation.name = new_macro_name.to_string();
                                                is_defined = true;
                                            }
                                        }
                                        SyntaxToken::QuotedString(quoted_string) => {
                                            if quoted_string == macro_name {
                                                *quoted_string = new_macro_name.to_string();
                                                is_defined = true;
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
    }
}
