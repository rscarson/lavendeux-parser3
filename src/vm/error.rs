//! Error types for the virtual machine
//! Contains RuntimeError - the actual error wrapper
//! and RuntimeErrorType - the different types of errors that can occur
use super::OpCode;
use crate::{compiler::DebugProfile, lexer::Token, traits::IntoOwned, value::ValueType};

/// The different types of errors that can occur during execution
#[rustfmt::skip]
#[derive(thiserror::Error, Debug, Clone)]
pub enum RuntimeErrorType {
    //
    // This category of errors deals with issues in the bytecode
    // They are not caused by the user's code, but by a bug in the compiler
    //

    /// Bytecode ended unexpectedly
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= Unexpected end of bytecode while reading {0:?}")]
    UnexpectedEnd(OpCode),

    /// Attempted to pop from an empty stack
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= {0:?} attempted to pop a value from an empty stack")]
    StackEmpty(OpCode),

    /// Encountered an invalid opcode during execution
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= Invalid bytecode; {0:02X} is not an opcode")]
    InvalidOpcode(u8),

    /// Encountered an invalid typecode during execution
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= Invalid bytecode; {0:02X} is not a type")]
    InvalidType(u8),

    /// Error occured due to bad bytecode
    #[error(
        "An issue occurred with the Lavendeux VM - this is a bug\n= {0:?} attempted to read malformed bytecode:\n= {1}"
    )]
    Decode(OpCode, crate::traits::ByteDecodeError),

    //
    // This category of errors deals with issues in the user's code
    //

    /// Attempted to modify a read-only value
    #[error("Cannot assign a constant\n= To assign a value use an identifier, like `a`, `name_2`, or `my_variable`")]
    NotAReference,

    /// Attempt to build a range with invalid values
    #[error("Cannot build a range of {0} values\n= Expected integers (0..10) or single-character strings ('a'..'z')")]
    InvalidValuesForRange(ValueType),

    /// Attempt to build a range with invalid strings
    #[error("Invalid strings for range.\n= Expected single characters")]
    InvalidStringsForRange,

    /// Failed attempt to index into a value
    #[error("Collection does not contain that index")]
    IndexingValue,

    /// Attempt to index into a value using the wrong type
    #[error("Invalid type for indexing")]
    IndexingType,

    /// Attempt to index into a value that is not indexable
    #[error("Cannot index into type; Expected string, array, or object")]
    IndexingBaseType,

    /// Attempted to index into an empty collection
    #[error("Collection is empty")]
    IteratorEmpty,

    /// Attempted to call a non-function
    #[error("No such function exists")]
    UndefinedFunction,

    /// Attempted to call a function with the wrong arguments
    #[error("Arguments incorrect;\n= {0}")]
    IncorrectFunctionArgs(String),

    /// Error occurred during operation on a value
    #[error("{0}")]
    Value(crate::value::ValueError),
}

/// The error wrapper for runtime errors
#[derive(Debug, Clone)]
pub struct RuntimeError {
    /// The type of error that occurred
    pub error: RuntimeErrorType,

    /// The position in the bytecode where the error occurred
    pub pos: usize,

    /// The token that caused the error (if a debug profile is available)
    pub token: Option<Token<'static>>,

    /// The parent error that caused this error (if any)
    pub parent: Option<Box<RuntimeError>>,
}
impl std::error::Error for RuntimeError {}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(token) = &self.token {
            write!(f, "{}\n= ", token)?;
        }
        write!(f, "{}", self.error)
    }
}

impl RuntimeError {
    /// Add debug information to the error
    pub fn with_context(self, debug_profile: &DebugProfile<'_>) -> Self {
        let token = debug_profile
            .current_token(self.pos)
            .map(|t| t.clone().into_owned());
        Self { token, ..self }
    }
}