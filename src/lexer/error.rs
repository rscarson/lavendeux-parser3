use super::Token;

/// Error during lexing.
#[derive(thiserror::Error, Debug, Clone)]
pub enum LexerError {
    /// Encountered an unexpected token.
    #[error("| {}\n= Unrecognized token", .0.slice())]
    UnrecognizedToken(Token<'static>),
}
