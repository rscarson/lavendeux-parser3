use crate::lexer::Token;

/// Error during compilation.
#[derive(thiserror::Error, Debug, Clone)]
pub enum CompilerError {
    /// Error node found in the AST
    #[error("{0}")]
    Parser(#[from] crate::parser::ParserError),

    /// Error compiling a __syscalld call
    #[error("{0}\n= Not an opcode; {1}")]
    InvalidSyscallOpcode(Token<'static>, String),
}
