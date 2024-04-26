use super::ValueType;

/// An error that occurs during value operations
#[rustfmt::skip]
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ValueError {
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

    //
    // Errors during casting and arithmetic operations
    //

    /// Caused by operations overflowing a type
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    /// Caused by using an operator on a type for which it is not defined
    #[error("Operator not valid for {0}")]
    InvalidOperationForType(ValueType),

    /// Caused by converting a value to a type that is not supported
    #[error("Cannot resolve values of type {0} and {1}")]
    TypeConversion(ValueType, ValueType),

    //
    // Errors during indexing
    //

    /// Caused by attempting to access a value index that does not exist
    #[error("Key not found in object")]
    KeyNotFound,

    /// Caused by attempting to access a value index that does not exist
    #[error("Index out of bounds")]
    IndexOutOfBounds,

    /// Caused by attempting to index into a value that does not support indexing
    #[error("Cannot index into {0}")]
    CannotIndexInto(ValueType),

    /// Caused by attempting to use a value as an index that is not supported
    #[error("Cannot use {0} as an index")]
    CannotIndexUsing(ValueType),

    /// Caused by attempting to alter a value that is read-only
    #[error("Specified index is read-only")]
    ReadOnlyIndexing,

    //
    // Errors during regex operations
    //

    /// An error caused by attempting to parse a regex literal
    /// Occurs during regex pattern compilation
    #[error("`{0}` is not a valid regex flag")]
    InvalidRegexFlag(char),

    /// An error caused by attempting to parse a regex literal
    /// Occurs during regex pattern compilation
    #[error("Invalid regex literal")]
    RegexError(#[from] regex::Error),
}
