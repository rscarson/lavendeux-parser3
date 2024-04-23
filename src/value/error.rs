use super::ValueType;

/// An error that occurs during value operations
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ValueError {
    /// Caused by operations overflowing a type
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,

    /// Caused by using an operator on a type for which it is not defined
    #[error("Operator not valid for {0}")]
    InvalidOperationForType(ValueType),

    /// Caused by converting a value to a type that is not supported
    #[error("Cannot resolve values of type {0} and {1}")]
    TypeConversion(ValueType, ValueType),

    /// Caused by attempting to access a value by reference without resolving it
    #[error("Unresolved reference. This is probably a bug.")]
    UnresolvedReference,

    /// Caused by attempting to access a value by reference that has been deleted
    #[error("Invalid reference. Has the value been deleted?")]
    BadReference,

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

    /// An error caused by attempting to parse a regex literal
    /// Occurs during regex pattern compilation
    #[error("`{0}` is not a valid regex flag")]
    InvalidRegexFlag(char),

    /// An error caused by attempting to parse a regex literal
    /// Occurs during regex pattern compilation
    #[error("Invalid regex literal")]
    RegexError(#[from] regex::Error),
}
