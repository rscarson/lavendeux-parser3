use fpdec::{CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, Decimal, Round};

use crate::traits::{ByteDecodeError, SerializeToBytes};

use super::{CheckedArithmetic, ValueError};

/// A symbol that can be attached to a number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberSymbol {
    /// A prefix symbol that is placed before the number
    Prefix(String),

    /// A suffix symbol that is placed after the number
    Suffix(String),
}

/// A fixed-point number that can be used in calculations.
/// The number is represented as a Decimal with a symbol and precision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Number {
    value: fpdec::Decimal,
    symbol: Option<NumberSymbol>,
    precision: Option<i8>,
}

impl Number {
    /// Create a new Number with the given value, symbol, and precision.
    pub fn new(value: fpdec::Decimal, symbol: Option<NumberSymbol>, precision: Option<i8>) -> Self {
        let mut i = Self {
            value,
            symbol,
            precision,
        };
        i.update_precision();
        i
    }

    /// Update the precision of the number.
    pub fn update_precision(&mut self) {
        if let Some(precision) = self.precision {
            self.value = self.value.checked_round(precision).unwrap();
        }
    }

    /// Return true if the number is zero.
    pub fn is_zero(&self) -> bool {
        self.value.eq_zero()
    }

    /// Resolve the precision and symbol of two numbers.
    pub fn resolve(self, other: Self) -> (Self, Self) {
        let (v1, s1, p1) = self.decompose();
        let (v2, s2, p2) = other.decompose();

        let p = p1.max(p2);
        let s = match (s1, s2) {
            (Some(s1), Some(s2)) if s1 != s2 => None,
            (Some(s1), _) => Some(s1),
            (None, Some(s2)) => Some(s2),
            (None, None) => None,
        };

        (Self::new(v1, s.clone(), p), Self::new(v2, s, p))
    }

    /// Decompose the number into its components.
    pub fn decompose(self) -> (fpdec::Decimal, Option<NumberSymbol>, Option<i8>) {
        (self.value, self.symbol, self.precision)
    }

    /// Convert the number into a float
    pub fn into_f64(self) -> f64 {
        self.value.try_into().unwrap()
    }

    /// Create a new Number from a float
    pub fn from_f64(value: f64) -> Result<Self, ValueError> {
        Ok(Self::new(
            value
                .try_into()
                .ok()
                .ok_or(ValueError::ArithmeticOverflow)?,
            None,
            None,
        ))
    }

    /// Consume the value, returning one rounded to the given precision.
    pub fn round(self, precision: i8) -> Result<Self, ValueError> {
        Ok(Self::new(
            self.value
                .checked_round(precision)
                .ok_or(ValueError::ArithmeticOverflow)?,
            self.symbol,
            self.precision,
        ))
    }

    /// The value pi
    pub fn pi() -> Self {
        Self::new(std::f64::consts::PI.try_into().unwrap(), None, None)
    }

    /// The value e
    pub fn e() -> Self {
        Self::new(std::f64::consts::E.try_into().unwrap(), None, None)
    }

    /// The value tau
    pub fn tau() -> Self {
        Self::new(std::f64::consts::TAU.try_into().unwrap(), None, None)
    }
}

impl TryInto<i128> for Number {
    type Error = fpdec::TryFromDecimalError;

    fn try_into(self) -> Result<i128, Self::Error> {
        self.value.try_into()
    }
}

impl From<bool> for Number {
    fn from(value: bool) -> Self {
        Self::new(
            match value {
                true => fpdec::Decimal::ONE,
                false => fpdec::Decimal::ZERO,
            },
            None,
            None,
        )
    }
}

impl From<i128> for Number {
    fn from(value: i128) -> Self {
        Self::new(value.into(), None, None)
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.symbol {
            Some(NumberSymbol::Prefix(prefix)) => write!(f, "{}{}", prefix, self.value),
            Some(NumberSymbol::Suffix(suffix)) => write!(f, "{}{}", self.value, suffix),
            None => write!(f, "{}", self.value),
        }
    }
}

impl CheckedArithmetic for Number {
    fn checked_add(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        Ok(Self::new(
            v1.checked_add(&v2).ok_or(ValueError::ArithmeticOverflow)?,
            symbol,
            precision,
        ))
    }

    fn checked_sub(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        Ok(Self::new(
            v1.checked_sub(&v2).ok_or(ValueError::ArithmeticOverflow)?,
            symbol,
            precision,
        ))
    }

    fn checked_mul(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        Ok(Self::new(
            v1.checked_mul(&v2).ok_or(ValueError::ArithmeticOverflow)?,
            symbol,
            precision,
        ))
    }

    fn checked_div(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        Ok(Self::new(
            v1.checked_div(&v2).ok_or(ValueError::ArithmeticOverflow)?,
            symbol,
            precision,
        ))
    }

    fn checked_rem(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        Ok(Self::new(
            v1.checked_rem(&v2).ok_or(ValueError::ArithmeticOverflow)?,
            symbol,
            precision,
        ))
    }

    fn checked_pow(self, other: Self) -> Result<Self, ValueError> {
        let (self_, other_) = self.resolve(other);
        let (v1, _, _) = self_.decompose();
        let (v2, symbol, precision) = other_.decompose();

        let v1: f64 = v1.try_into().ok().ok_or(ValueError::ArithmeticOverflow)?;
        let v2: f64 = v2.try_into().ok().ok_or(ValueError::ArithmeticOverflow)?;
        let v = v1
            .powf(v2)
            .try_into()
            .ok()
            .ok_or(ValueError::ArithmeticOverflow)?;
        Ok(Self::new(v, symbol, precision))
    }

    fn checked_neg(self) -> Result<Self, ValueError> {
        let (v, symbol, precision) = self.decompose();
        let m: Decimal = (-1).into();
        let v = v.checked_mul(m).ok_or(ValueError::ArithmeticOverflow)?;
        Ok(Self::new(v, symbol, precision))
    }
}

impl SerializeToBytes for Number {
    fn serialize_into_bytes(self) -> Vec<u8> {
        // Decompose the inner fpdec
        let (coeef, frac) = (self.value.coefficient(), self.value.n_frac_digits());
        let mut bytes = coeef.serialize_into_bytes();
        bytes.push(frac);

        // Serialize the symbol
        match self.symbol {
            None => {
                bytes.push(0x00);
            }
            Some(NumberSymbol::Prefix(prefix)) => {
                bytes.push(0x01);
                bytes.extend(prefix.serialize_into_bytes());
            }
            Some(NumberSymbol::Suffix(suffix)) => {
                bytes.push(0x02);
                bytes.extend(suffix.serialize_into_bytes());
            }
        }

        // Serialize the precision
        bytes.extend(self.precision.serialize_into_bytes());

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let coeef = i128::deserialize_from_bytes(bytes)?;
        let frac = u8::deserialize_from_bytes(bytes)?;

        let symbol_type = bytes
            .next()
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("Number".to_string()))?;
        let symbol = match symbol_type {
            0x00 => None,
            0x01 => {
                let prefix = String::deserialize_from_bytes(bytes)?;
                Some(NumberSymbol::Prefix(prefix))
            }
            0x02 => {
                let suffix = String::deserialize_from_bytes(bytes)?;
                Some(NumberSymbol::Suffix(suffix))
            }
            _ => {
                return Err(ByteDecodeError::MalformedData(
                    "Number".to_string(),
                    "Invalid symbol type".to_string(),
                ))
            }
        };

        let precision = Option::<i8>::deserialize_from_bytes(bytes)?;

        Ok(Self::new(Decimal::new_raw(coeef, frac), symbol, precision))
    }
}
