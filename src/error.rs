use crate::tokenizer::{Category, Rule, Token};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "{found}\n= Syntax error; expected one of:\n= {}",
        Category::format_rules(expected)
    )]
    Syntax {
        expected: Vec<Rule>,
        found: Token<'static>,
    },

    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,

    #[error("{0}\n= Syntax error; Unrecognized token")]
    UnrecognizedToken(Token<'static>),

    #[error("{0}\n= Unreachable statement; Put this case before the default case")]
    UnreachableSwitchCase(Token<'static>),

    #[error("{0}\n= Conditionals are required to have an 'else' block.\n= If a value is not needed, use `else nil`")]
    MissingElse(Token<'static>),

    #[error("{0}\n= Could not assign to constant value")]
    AssignmentToConstant(Token<'static>),

    #[error("{0}\n= Not the name of a decorator; expected an identifier")]
    NotADecorator(Token<'static>),

    #[error("{0}\n= Invalid float literal")]
    InvalidFloatLiteral(Token<'static>),

    #[error("{0}\n= Invalid integer literal")]
    InvalidIntLiteral(Token<'static>),
}
