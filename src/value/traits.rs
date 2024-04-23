use super::ValueError;

/// Trait for checked arithmetic operations.
pub trait CheckedArithmetic
where
    Self: Sized,
{
    /// Add two values together.
    fn checked_add(self, other: Self) -> Result<Self, ValueError>;

    /// Subtract one value from another.
    fn checked_sub(self, other: Self) -> Result<Self, ValueError>;

    /// Multiply two values together.
    fn checked_mul(self, other: Self) -> Result<Self, ValueError>;

    /// Divide one value by another.
    fn checked_div(self, other: Self) -> Result<Self, ValueError>;

    /// Calculate the remainder of one value divided by another.
    fn checked_rem(self, other: Self) -> Result<Self, ValueError>;

    /// Calculate the power of one value to another.
    fn checked_pow(self, other: Self) -> Result<Self, ValueError>;

    /// Calculate the square root of the value.
    fn checked_neg(self) -> Result<Self, ValueError>;
}

/// Trait for checked bitwise operations.
pub trait CheckedBitwise
where
    Self: Sized,
{
    /// Perform a bitwise left shift operation.
    fn checked_shl(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a bitwise right shift operation.
    fn checked_shr(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a bitwise and operation.
    fn checked_and(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a bitwise or operation.
    fn checked_or(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a bitwise xor operation.
    fn checked_xor(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a bitwise not operation.
    fn checked_not(self) -> Result<Self, ValueError>;
}

/// Trait for checked boolean operations.
pub trait CheckedBoolean
where
    Self: Sized,
{
    /// Perform a logical and operation.
    fn checked_logical_and(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a logical or operation.
    fn checked_logical_or(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a logical not operation.
    fn checked_logical_not(self) -> Result<Self, ValueError>;

    /// Perform an equality check.
    fn checked_eq(self, other: Self) -> Result<Self, ValueError>;

    /// Perform an inequality check.
    fn checked_ne(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a greater than check.
    fn checked_gt(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a greater than or equal check.
    fn checked_ge(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a less than check.
    fn checked_lt(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a less than or equal check.
    fn checked_le(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a strict equality check.
    fn checked_seq(self, other: Self) -> Result<Self, ValueError>;

    /// Perform a strict inequality check.
    fn checked_sne(self, other: Self) -> Result<Self, ValueError>;
}

/// Trait for checked matching operations.
pub trait CheckedMatching
where
    Self: Sized,
{
    /// Check if the value matches the other value.
    fn checked_matches(self, other: Self) -> Result<Self, ValueError>;

    /// Check if the value contains the other value.
    fn checked_contains(self, other: Self) -> Result<Self, ValueError>;

    /// Check if the value starts with the other value.
    fn checked_starts_with(self, other: Self) -> Result<Self, ValueError>;

    /// Check if the value ends with the other value.
    fn checked_ends_with(self, other: Self) -> Result<Self, ValueError>;

    /// Check a value against a regex pattern.
    /// The formatting callback is used to format regex pattern before calling
    fn checked_regex<F>(
        value: &str,
        pattern: &str,
        formatting_callback: F,
    ) -> Result<Self, ValueError>
    where
        F: Fn(String) -> String;
}
