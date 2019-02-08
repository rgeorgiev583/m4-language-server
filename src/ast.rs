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
