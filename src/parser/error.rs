use crate::{
    lexer::{Category, Rule, Token},
    literals::LiteralError,
};

/// Error during parsing.
#[derive(thiserror::Error, Debug, Clone)]
pub enum ParserError {
    /// Syntax error.
    #[error(
        "{found}\n= Syntax error: Unexpected {}, expected one of:\n= {}",
        Category::from_rule(found.rule()).unwrap_or(Category::EOI),
        Category::format_rules(expected)
    )]
    Syntax {
        /// Expected rules.
        expected: Vec<Rule>,

        /// Found token.
        found: Token<'static>,
    },

    /// Invalid literal value.
    #[error("{0}\n= {1}")]
    InvalidLiteral(Token<'static>, LiteralError),

    /// Invalid type.
    #[error("{0}\n= Not a type; expected one of [bool, int, float, string, array, object, range, function, primitive, numeric, collection, all]")]
    InvalidType(Token<'static>),

    /// Cannot cast to type.
    #[error(
        "{0}\n= Cannot cast to type; expected one of [bool, int, float, string, array, object]"
    )]
    CannotCastToType(Token<'static>),

    /// Unreachable switch case.
    #[error("{0}\n= Unreachable statement; Put this case before the default case")]
    UnreachableSwitchCase(Token<'static>),

    /// Missing default case.
    #[error("{0}\n= Switch statements are required to have a default case ( _ => BLOCK )")]
    MissingDefaultCase(Token<'static>),

    /// Missing else block.
    #[error("{0}\n= Conditionals are required to have an 'else' block.\n= If a value is not needed, use `else nil`")]
    MissingElse(Token<'static>),

    /// Missing function return value.
    #[error("{0}\n= Function must return a value; return a value, or `nil`")]
    MustReturnAValue(Token<'static>),

    /// Assignment to constant value.
    #[error("{0}\n= Could not assign to constant value")]
    AssignmentToConstant(Token<'static>),

    /// Not a decorator.
    #[error("{0}\n= Not the name of a decorator; expected an identifier")]
    NotADecorator(Token<'static>),

    /// Invalid decorator signature.
    #[error("{0}\n= @decorator functions must accept a single argument")]
    DecoratorSignature(Token<'static>),

    /// Your function is silly and you are silly
    /// please stop
    #[error("{0}\n= Function definition is silly. Please limit your arguments to 255.")]
    TooManyArguments(Token<'static>),
}

impl ParserError {
    /// Get the token that caused the error.
    pub fn token(&self) -> &Token<'static> {
        match self {
            ParserError::InvalidLiteral(token, _) => token,
            ParserError::Syntax { found, .. } => found,
            ParserError::InvalidType(token) => token,
            ParserError::CannotCastToType(token) => token,
            ParserError::UnreachableSwitchCase(token) => token,
            ParserError::MissingDefaultCase(token) => token,
            ParserError::MissingElse(token) => token,
            ParserError::MustReturnAValue(token) => token,
            ParserError::AssignmentToConstant(token) => token,
            ParserError::NotADecorator(token) => token,
            ParserError::DecoratorSignature(token) => token,
            ParserError::TooManyArguments(token) => token,
        }
    }
}
