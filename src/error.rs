/// Error type for the language
/// Encapsulates all possible errors during lexing, parsing, compiling, and running
#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    /// Errors during tokenization; Mostly unrecognized tokens
    #[error("{0}")]
    Lexer(#[from] crate::lexer::LexerError),

    /// Errors during parsing; Syntax errors
    #[error("{0}")]
    Parser(#[from] crate::parser::ParserError),

    /// Errors during compilation; Fairly uncommon, mostly stdlib issues
    #[error("{0}")]
    Compiler(#[from] crate::compiler::CompilerError),

    /// Errors during execution; Type errors, overflows etc
    #[error("{0}")]
    Runtime(#[from] crate::vm::error::RuntimeError),
}
