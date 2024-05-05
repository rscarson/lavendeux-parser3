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

    /// Error occurred due to bad type in bytecode
    /// Probably stack corruption
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= {0:?} attempted to read a value of the wrong type:\n= Expected {1}")]
    BadType(OpCode, ValueType),

    /// A reference points to another reference
    /// This is a bug in the compiler
    #[error("An issue occurred with the Lavendeux VM - this is a bug\n= Possible circular reference detected")]
    NestedReference,

    //
    // This category of errors deals with issues in the user's code
    //

    /// Attempted to allocate too much memory
    #[error("{0}")]
    MemoryAllocation(#[from] std::collections::TryReserveError),
    
    //
    // Errors during reference resolution
    //

    /// Caused by attempting to resolve a reference that is not in memory
    #[error("Variable not defined\n= You can assign a value with `name = ...`")]
    HashNotFound,

    /// Caused by attempting to pull a value from a slot that is no longer valid
    /// This should never happen, I think? It's a bug if it does probably
    /// It would mean we leaked a reference without resolving it
    #[error("A value reference is invalid.\n= This is likely a bug in the Lavendeux VM.")]
    SlotRefInvalid,

    /// Caused by attempting to use a reference that is not resolved
    /// This is always a bug
    #[error("A reference was used before it was resolved.\n= This is a bug in the Lavendeux VM.\n= Error occurred in `Reference::{0}()`")]
    ReferenceNotResolved(String),

    /// Attempted to delete a constant value
    #[error("Cannot delete a constant value\n= To delete a value use an identifier, like `a`, `name_2`, or `my_variable`")]
    DeleteLiteral,

    /// Attempted to write to a constant value
    #[error("Cannot write to a constant value\n= To write to a value use an identifier, like `a`, `name_2`, or `my_variable`")]
    SetLiteral,

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

    //
    // Errors during function calls
    //

    /// Error during function call
    #[error("In function")]
    Function,

    /// Attempted to call a non-function
    #[error("No such function exists")]
    UndefinedFunction,
    
    /// Attempted to call a function with the wrong arguments
    #[error("Number of arguments incorrect;\n= {0}")]
    IncorrectFunctionArgCount(String),
    
    /// Attempted to call a function with the wrong arguments
    #[error("Argument type incorrect;\n= Argument {index} expected `{expected}`, found `{provided}`\n= {signature}")]
    IncorrectFunctionArg{
        /// The signature of the function
        signature: String,

        /// The expected type
        expected: ValueType,

        /// The provided type
        provided: ValueType,

        /// The index of the argument
        index: usize,
    },

    /// Call to THRW, or stdlib::throw
    #[error("{0}")]
    Custom(String),

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
        if let Some(parent) = &self.parent {
            write!(f, "{}\n", parent)?;
        }
        if let Some(token) = &self.token {
            write!(f, "{}\n", token)?;
        }
        write!(f, "= {}", self.error)
    }
}

impl RuntimeError {
    /// Add debug information to the error
    pub fn with_context(self, debug_profile: &DebugProfile) -> Self {
        let token = debug_profile
            .current_token(self.pos)
            .map(|t| t.clone().into_owned());
        Self { token, ..self }
    }
}
