use crate::{
    lexer::{Category, Rule, Token},
    literals::LiteralError,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("{0}\n= Syntax error; Unrecognized token")]
    UnrecognizedToken(Token<'static>),

    #[error(
        "{found}\n= Syntax error; expected one of:\n= {}",
        Category::format_rules(expected)
    )]
    Syntax {
        expected: Vec<Rule>,
        found: Token<'static>,
    },

    #[error("{0}\n= Unreachable statement; Put this case before the default case")]
    UnreachableSwitchCase(Token<'static>),

    #[error("{0}\n= Conditionals are required to have an 'else' block.\n= If a value is not needed, use `else nil`")]
    MissingElse(Token<'static>),

    #[error("{0}\n= Could not assign to constant value")]
    AssignmentToConstant(Token<'static>),

    #[error("{0}\n= Not the name of a decorator; expected an identifier")]
    NotADecorator(Token<'static>),

    #[error("{0}\n= {1}")]
    InvalidLiteral(Token<'static>, LiteralError),
}
