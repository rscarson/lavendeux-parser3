use crate::traits::SerializeToBytes;

use super::{
    number::Number, types::ValueType, CheckedArithmetic, CheckedBitwise, CheckedBoolean, ValueError,
};

/// Represents a primitive value.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Primitive {
    /// Represents a boolean value.
    Boolean(bool),

    /// Represents an integer value.
    Integer(i128),

    /// Represents a decimal value.
    Decimal(Number),

    /// Represents a string value.
    String(String),
}

impl Primitive {
    /// Returns the type of the primitive
    pub fn type_of(&self) -> ValueType {
        match self {
            Primitive::Boolean(_) => ValueType::Boolean,
            Primitive::Integer(_) => ValueType::Integer,
            Primitive::Decimal(_) => ValueType::Decimal,
            Primitive::String(_) => ValueType::String,
        }
    }

    /// Converts the primitive to a specific type
    pub fn into_type(self, value_type: ValueType) -> Option<Self> {
        match value_type {
            ValueType::Boolean => Some(self.as_boolean()),
            ValueType::Integer => self.as_integer(),
            ValueType::Decimal => self.as_decimal(),
            ValueType::String => Some(self.as_string()),

            ValueType::Numeric => match self {
                Primitive::Integer(_) | Primitive::Decimal(_) | Primitive::Boolean(_) => Some(self),
                _ => None,
            },

            ValueType::All | ValueType::Primitive => Some(self),

            _ => None,
        }
    }

    /// Converts the primitive to a boolean
    pub fn as_boolean(self) -> Self {
        match self {
            Primitive::Boolean(b) => Primitive::Boolean(b),
            Primitive::Integer(i) => Primitive::Boolean(i != 0),
            Primitive::Decimal(d) => Primitive::Boolean(!d.is_zero()),
            Primitive::String(s) => Primitive::Boolean(!s.is_empty()),
        }
    }

    /// Extracts the boolean value from the primitive
    pub fn into_boolean(self) -> bool {
        match self.as_boolean() {
            Primitive::Boolean(b) => b,
            _ => false,
        }
    }

    /// Converts the primitive to an integer
    pub fn as_integer(self) -> Option<Self> {
        match self {
            Primitive::Boolean(b) => Some(Primitive::Integer(b as i128)),
            Primitive::Integer(i) => Some(Primitive::Integer(i)),
            Primitive::Decimal(d) => Some(Primitive::Integer(d.try_into().ok()?)),
            Primitive::String(_) => None,
        }
    }

    /// Extracts the integer value from the primitive
    pub fn into_integer(self) -> Option<i128> {
        match self.as_integer() {
            Some(Primitive::Integer(i)) => Some(i),
            _ => None,
        }
    }

    /// Converts the primitive to a decimal
    pub fn as_decimal(self) -> Option<Self> {
        match self {
            Primitive::Boolean(b) => Some(Primitive::Decimal(Number::from(b as i128))),
            Primitive::Integer(i) => Some(Primitive::Decimal(Number::from(i))),
            Primitive::Decimal(d) => Some(Primitive::Decimal(d)),
            Primitive::String(_) => None,
        }
    }

    /// Converts the primitive to a string
    pub fn as_string(self) -> Self {
        match self {
            Primitive::Boolean(b) => Primitive::String(b.to_string()),
            Primitive::Integer(i) => Primitive::String(i.to_string()),
            Primitive::Decimal(d) => Primitive::String(d.to_string()),
            Primitive::String(s) => Primitive::String(s),
        }
    }

    /// Resolves the type of two primitives
    /// Priority: String > Decimal > Integer > Boolean
    pub fn resolve(self, other: Self) -> Option<(Self, Self)> {
        match (self, other) {
            (Primitive::String(s), other_) => Some((Primitive::String(s), other_.as_string())),
            (self_, Primitive::String(s)) => Some((self_.as_string(), Primitive::String(s))),

            (Primitive::Decimal(d), other_) => Some((Primitive::Decimal(d), other_.as_decimal()?)),
            (self_, Primitive::Decimal(d)) => Some((self_.as_decimal()?, Primitive::Decimal(d))),

            (Primitive::Integer(i), other_) => Some((Primitive::Integer(i), other_.as_integer()?)),
            (self_, Primitive::Integer(i)) => Some((self_.as_integer()?, Primitive::Integer(i))),

            (Primitive::Boolean(b), other_) => Some((Primitive::Boolean(b), other_.as_boolean())),
        }
    }
}

impl CheckedArithmetic for Primitive {
    fn checked_add(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a ^ b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_add(b).ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => {
                Ok(Primitive::Decimal(a.checked_add(b)?))
            }
            (Primitive::String(a), Primitive::String(b)) => {
                Ok(Primitive::String(format!("{}{}", a, b)))
            }

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_sub(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();

        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a ^ b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_sub(b).ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => {
                Ok(Primitive::Decimal(a.checked_sub(b)?))
            }

            (Primitive::String(a), Primitive::String(b)) => {
                Ok(Primitive::String(a.replace(&b, "")))
            }

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_mul(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a && b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_mul(b).ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => {
                Ok(Primitive::Decimal(a.checked_mul(b)?))
            }

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_div(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a && b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_div(b).ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => {
                Ok(Primitive::Decimal(a.checked_div(b)?))
            }

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_rem(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a && b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_rem(b).ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => {
                Ok(Primitive::Decimal(a.checked_rem(b)?))
            }

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_pow(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a && b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(
                a.checked_pow(b.try_into().ok().ok_or(ValueError::ArithmeticOverflow)?)
                    .ok_or(ValueError::ArithmeticOverflow)?,
            )),
            (Primitive::Decimal(a), Primitive::Decimal(b)) => Ok(Primitive::Decimal(
                a.checked_pow(b.try_into().ok().ok_or(ValueError::ArithmeticOverflow)?)?,
            )),

            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_neg(self) -> Result<Self, ValueError> {
        match self {
            Primitive::Boolean(b) => Ok(Primitive::Boolean(!b)),
            Primitive::Integer(i) => Ok(Primitive::Integer(-i)),
            Primitive::Decimal(d) => Ok(Primitive::Decimal(d.checked_neg()?)),
            Primitive::String(s) => Ok(Primitive::String(s.chars().rev().collect::<String>())),
        }
    }
}

impl CheckedBitwise for Primitive {
    fn checked_and(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a && b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(a & b)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_or(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a || b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(a | b)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_xor(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => Ok(Primitive::Boolean(a ^ b)),
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(a ^ b)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_shl(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(a << b)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_shr(self, other: Self) -> Result<Self, ValueError> {
        let (ta, tb) = (self.type_of(), other.type_of());
        let (a, b) = self
            .resolve(other)
            .ok_or(ValueError::TypeConversion(ta, tb))?;
        let t = a.type_of();
        match (a, b) {
            (Primitive::Integer(a), Primitive::Integer(b)) => Ok(Primitive::Integer(a >> b)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_not(self) -> Result<Self, ValueError> {
        let t = self.type_of();
        match self {
            Primitive::Boolean(b) => Ok(Primitive::Boolean(!b)),
            Primitive::Integer(i) => Ok(Primitive::Integer(!i)),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }
}

impl CheckedBoolean for Primitive {
    fn checked_logical_and(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = (self.into_boolean(), other.into_boolean());
        Ok(Primitive::Boolean(a && b))
    }

    fn checked_logical_or(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = (self.into_boolean(), other.into_boolean());
        Ok(Primitive::Boolean(a || b))
    }

    fn checked_logical_not(self) -> Result<Self, ValueError> {
        let a = self.into_boolean();
        Ok(Primitive::Boolean(!a))
    }

    fn checked_eq(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a == b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a == b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a == b,
            (Primitive::String(a), Primitive::String(b)) => a == b,
            _ => false,
        }))
    }

    fn checked_ne(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a != b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a != b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a != b,
            (Primitive::String(a), Primitive::String(b)) => a != b,
            _ => false,
        }))
    }

    fn checked_ge(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a >= b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a >= b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a >= b,
            (Primitive::String(a), Primitive::String(b)) => a >= b,
            _ => false,
        }))
    }

    fn checked_gt(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a > b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a > b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a > b,
            (Primitive::String(a), Primitive::String(b)) => a > b,
            _ => false,
        }))
    }

    fn checked_le(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a <= b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a <= b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a <= b,
            (Primitive::String(a), Primitive::String(b)) => a <= b,
            _ => false,
        }))
    }

    fn checked_lt(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other).unwrap();
        Ok(Primitive::Boolean(match (a, b) {
            (Primitive::Boolean(a), Primitive::Boolean(b)) => a < b,
            (Primitive::Integer(a), Primitive::Integer(b)) => a < b,
            (Primitive::Decimal(a), Primitive::Decimal(b)) => a < b,
            (Primitive::String(a), Primitive::String(b)) => a < b,
            _ => false,
        }))
    }

    fn checked_seq(self, other: Self) -> Result<Self, ValueError> {
        Ok(Primitive::Boolean(self == other))
    }

    fn checked_sne(self, other: Self) -> Result<Self, ValueError> {
        Ok(Primitive::Boolean(self != other))
    }
}

impl std::fmt::Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Boolean(b) => write!(f, "{}", b),
            Primitive::Integer(i) => write!(f, "{}", i),
            Primitive::Decimal(d) => write!(f, "{}", d),
            Primitive::String(s) => write!(f, "`{}`", s),
        }
    }
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Boolean(b) => write!(f, "{}", b),
            Primitive::Integer(i) => write!(f, "{}", i),
            Primitive::Decimal(d) => write!(f, "{}", d),
            Primitive::String(s) => write!(f, "{}", s),
        }
    }
}

impl SerializeToBytes for Primitive {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.type_of() as u8);
        match self {
            Primitive::Boolean(b) => bytes.push(b as u8),
            Primitive::Integer(i) => bytes.extend(i.serialize_into_bytes()),
            Primitive::Decimal(d) => bytes.extend(d.serialize_into_bytes()),
            Primitive::String(s) => bytes.extend(s.serialize_into_bytes()),
        }

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let ty = u8::deserialize_from_bytes(bytes)?;
        match ValueType::from_u8(ty) {
            Some(ValueType::Boolean) => {
                Ok(Primitive::Boolean(u8::deserialize_from_bytes(bytes)? != 0))
            }
            Some(ValueType::Integer) => {
                Ok(Primitive::Integer(i128::deserialize_from_bytes(bytes)?))
            }
            Some(ValueType::Decimal) => {
                Ok(Primitive::Decimal(Number::deserialize_from_bytes(bytes)?))
            }
            Some(ValueType::String) => {
                Ok(Primitive::String(String::deserialize_from_bytes(bytes)?))
            }

            _ => Err(crate::traits::ByteDecodeError::MalformedData(
                "Primitive".to_string(),
                "Invalid value type".to_string(),
            )),
        }
    }
}
