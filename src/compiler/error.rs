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

    /// Error compiling an __include call
    #[error("{0}\n= Invalid value for include;\n= include() expects a string literal")]
    InvalidInclude(Token<'static>),

    /// Error compiling an __include call
    #[error("{0}\n= Could not read `{1}`")]
    FileNotFound(Token<'static>, String),

    /// Error compiling an __include call
    #[error("{1}\n= In include:\n{0}")]
    IncludeError(Token<'static>, Box<crate::Error>),

    /// Error compiling a special function call
    #[error("{0}\n= {1}() expects {2} arguments, found {3}")]
    InvalidArgumentCount(Token<'static>, String, usize, usize),
}
