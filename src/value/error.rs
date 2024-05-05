use super::ValueType;

/// An error that occurs during value operations
#[rustfmt::skip]
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ValueError {
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

    /// Caused by attempting to use the bigliest memory
    #[error("{0}")]
    MemoryAllocation(#[from] std::collections::TryReserveError),

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
    #[error("Cannot use {0} as an index into {1} values")]
    CannotIndexUsing(ValueType, ValueType),

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
