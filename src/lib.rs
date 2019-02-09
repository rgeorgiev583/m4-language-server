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
        //QuotedString(String),
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
