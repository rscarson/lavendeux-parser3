//! # Value
//! The value type used by the language
//! Contains the main value type and all subtypes
use crate::traits::SerializeToBytes;
use std::collections::HashMap;

mod error;
pub use error::ValueError;

mod function;
pub use function::*;

mod number;
pub use number::{Number, NumberSymbol};

mod primitive;
pub use primitive::Primitive;

mod traits;
pub use traits::*;

mod types;
pub use types::ValueType;

mod indexing;
pub use indexing::*;

/// Represents a value in Lavendeux
/// This is the main data structure used by the language
#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    /// Represents a single primitive value
    Primitive(Primitive),

    /// Represents an array of values of any types
    Array(Vec<Value>),

    /// Represents an object, which is a map of keys to values
    /// Keys are always primitives
    Object(HashMap<Primitive, Value>),

    /// Represents a range of integers
    Range(std::ops::Range<i128>),

    /// Represents a function, which can be called
    Function(Function),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(o) => o,
            None => {
                // Fall back to type ordering
                // Primitive < Array < Object < Range < Function
                match (self, other) {
                    (Value::Primitive(_), _) => std::cmp::Ordering::Less,
                    (_, Value::Primitive(_)) => std::cmp::Ordering::Greater,

                    (Value::Array(_), _) => std::cmp::Ordering::Less,
                    (_, Value::Array(_)) => std::cmp::Ordering::Greater,

                    (Value::Object(_), _) => std::cmp::Ordering::Less,
                    (_, Value::Object(_)) => std::cmp::Ordering::Greater,

                    (Value::Range(_), _) => std::cmp::Ordering::Less,
                    (_, Value::Range(_)) => std::cmp::Ordering::Greater,

                    (Value::Function(_), _) => std::cmp::Ordering::Greater,
                }
            }
        }
    }
}

impl Value {
    /// Cast a value to a type
    pub fn cast(self, typename: ValueType) -> Result<Self, ValueError> {
        let own_type = self.type_of();
        if self.is_a(typename) {
            Ok(self)
        } else {
            match typename {
                ValueType::Boolean => self
                    .as_boolean()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),
                ValueType::Integer => self
                    .as_integer()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),
                ValueType::Decimal => self
                    .as_decimal()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),
                ValueType::String => self
                    .as_string()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),
                ValueType::Array => self
                    .as_array()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),
                ValueType::Object => self
                    .as_object()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),

                ValueType::Range => self
                    .as_range()
                    .ok_or_else(|| ValueError::TypeConversion(own_type, typename)),

                ValueType::Primitive => self.cast_primitive().map(Value::Primitive),
                ValueType::Numeric => self
                    .cast_decimal()
                    .map(|n| Value::Primitive(Primitive::Decimal(n))),
                ValueType::Collection => self.cast_array().map(Value::Array),

                ValueType::All => Ok(self),

                _ => Err(ValueError::TypeConversion(own_type, typename)),
            }
        }
    }

    /// Returns the length of the value
    /// For arrays, objects, and strings, this is the number of elements
    /// For ranges, this is the difference between the start and end
    /// For primitives, this is always 1
    pub fn len(&self) -> i128 {
        match self {
            Value::Array(a) => a.len() as i128,
            Value::Object(o) => o.len() as i128,
            Value::Range(r) => (r.end - r.start) as i128,
            Value::Primitive(Primitive::String(s)) => s.chars().count() as i128,
            _ => 1,
        }
    }

    /// Checks if the value is of a certain type
    pub fn is_a(&self, typename: ValueType) -> bool {
        let a = self.type_of();
        match (a, typename) {
            (
                ValueType::Integer | ValueType::Decimal | ValueType::Boolean | ValueType::String,
                ValueType::Primitive,
            ) => true,

            (
                ValueType::String | ValueType::Array | ValueType::Object | ValueType::Range,
                ValueType::Collection,
            ) => true,

            (ValueType::Boolean | ValueType::Integer | ValueType::Decimal, ValueType::Numeric) => {
                true
            }

            (_, ValueType::All) => true,

            _ if a == typename => true,

            _ => false,
        }
    }

    /// Returns the type of the value
    pub fn type_of(&self) -> ValueType {
        match self {
            Value::Primitive(p) => p.type_of(),
            Value::Function(_) => ValueType::Function,

            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
            Value::Range(_) => ValueType::Range,
        }
    }

    /// Resolves two values into a common type
    pub fn resolve(self, other: Self) -> Result<(Self, Self), ValueError> {
        let (mut ta, mut tb) = (self.type_of(), other.type_of());
        if ta == tb {
            Ok((self, other))
        } else {
            if matches!(
                ta,
                ValueType::Boolean | ValueType::Integer | ValueType::Decimal | ValueType::String
            ) {
                ta = ValueType::Primitive;
            }
            if matches!(
                tb,
                ValueType::Boolean | ValueType::Integer | ValueType::Decimal | ValueType::String
            ) {
                tb = ValueType::Primitive;
            }

            match (ta, tb) {
                (ValueType::Primitive, ValueType::Primitive) => {
                    if let (Value::Primitive(p1), Value::Primitive(p2)) = (self, other) {
                        let (t1, t2) = (p1.type_of(), p2.type_of());
                        let (p1, p2) = p1
                            .resolve(p2)
                            .ok_or_else(|| ValueError::TypeConversion(t1, t2))?;
                        Ok((Value::Primitive(p1), Value::Primitive(p2)))
                    } else {
                        unreachable!("Both values are primitives")
                    }
                }
                (ValueType::Primitive, ValueType::Array) => Ok((
                    self.as_array()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                    other,
                )),
                (ValueType::Primitive, ValueType::Object) => Ok((
                    self.as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                    other,
                )),

                (ValueType::Array, ValueType::Primitive) => Ok((
                    self,
                    other
                        .as_array()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                )),
                (ValueType::Object, ValueType::Primitive) => Ok((
                    self,
                    other
                        .as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                )),

                (ValueType::Array, ValueType::Object) => Ok((
                    self.as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                    other,
                )),
                (ValueType::Object, ValueType::Array) => Ok((
                    self,
                    other
                        .as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                )),

                (ValueType::Array, ValueType::Range) => Ok((
                    self,
                    other
                        .as_array()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                )),
                (ValueType::Object, ValueType::Range) => Ok((
                    self,
                    other
                        .as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                )),
                (ValueType::Range, ValueType::Array) => Ok((
                    self.as_array()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                    other,
                )),
                (ValueType::Range, ValueType::Object) => Ok((
                    self.as_object()
                        .ok_or_else(|| ValueError::TypeConversion(ta, tb))?,
                    other,
                )),

                (ValueType::Array, ValueType::Array) => Ok((self, other)),
                (ValueType::Object, ValueType::Object) => Ok((self, other)),
                (ValueType::Range, ValueType::Range) => Ok((self, other)),

                _ => Err(ValueError::TypeConversion(ta, tb)),
            }
        }
    }

    /// Turns the value into a primitive, if possible
    pub fn cast_primitive(self) -> Result<Primitive, ValueError> {
        match self {
            Value::Primitive(p) => Ok(p),
            _ => Err(ValueError::TypeConversion(
                self.type_of(),
                ValueType::Primitive,
            )),
        }
    }

    /// Turns the value into a primitive, if possible
    pub fn cast_decimal(self) -> Result<Number, ValueError> {
        let own_type = self.type_of();
        match self {
            Value::Primitive(p) => match p.as_decimal() {
                Some(Primitive::Decimal(n)) => Ok(n),
                _ => Err(ValueError::TypeConversion(own_type, ValueType::Decimal)),
            },
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Decimal)),
        }
    }

    /// Creates a new boolean value
    pub fn boolean(value: bool) -> Self {
        Value::Primitive(Primitive::Boolean(value))
    }

    /// Turns the value into a boolean, if possible
    pub fn as_boolean(self) -> Option<Self> {
        match self {
            Value::Primitive(p) => Some(Value::Primitive(p.as_boolean())),
            Value::Array(a) => Some(Value::Primitive(Primitive::Boolean(!a.is_empty()))),
            Value::Object(o) => Some(Value::Primitive(Primitive::Boolean(!o.is_empty()))),
            Value::Range(r) => Some(Value::Primitive(Primitive::Boolean(r.start >= r.end))),
            _ => None,
        }
    }

    /// Returns the truth value of the value
    pub fn cast_boolean(self) -> Result<bool, ValueError> {
        let own_type = self.type_of();
        match self.as_boolean() {
            Some(Value::Primitive(Primitive::Boolean(i))) => Ok(i),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Boolean)),
        }
    }

    /// Creates a new integer value
    pub fn integer(value: i128) -> Self {
        Value::Primitive(Primitive::Integer(value))
    }

    /// Turns the value into an integer, if possible
    pub fn as_integer(self) -> Option<Self> {
        match self {
            Value::Primitive(p) => p.as_integer().map(Value::Primitive),
            _ => None,
        }
    }

    /// Turns the value into an integer, if possible
    pub fn cast_integer(self) -> Result<i128, ValueError> {
        let own_type = self.type_of();
        match self.as_integer() {
            Some(Value::Primitive(Primitive::Integer(i))) => Ok(i),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Integer)),
        }
    }

    /// Creates a new decimal value
    pub fn decimal(value: Number) -> Self {
        Value::Primitive(Primitive::Decimal(value))
    }

    /// Turns the value into a decimal, if possible
    pub fn as_decimal(self) -> Option<Self> {
        match self {
            Value::Primitive(p) => p.as_decimal().map(Value::Primitive),
            _ => None,
        }
    }

    /// Creates a new string value
    pub fn string(value: String) -> Self {
        Value::Primitive(Primitive::String(value))
    }

    /// Turns the value into a string, if possible
    pub fn as_string(self) -> Option<Self> {
        match self {
            Value::Primitive(p) => Some(Value::Primitive(p.as_string())),
            Value::Array(a) => Some(Value::Primitive(Primitive::String(format!("{:?}", a)))),
            Value::Object(o) => Some(Value::Primitive(Primitive::String(format!("{:?}", o)))),
            Value::Range(r) => Some(Value::Primitive(Primitive::String(format!("{:?}", r)))),
            Value::Function(f) => Some(Value::Primitive(Primitive::String(f.docs.signature))),
        }
    }

    /// Turns the value into a string, if possible
    pub fn cast_string(self) -> Result<String, ValueError> {
        let own_type = self.type_of();
        match self.as_string() {
            Some(Value::Primitive(Primitive::String(s))) => Ok(s),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::String)),
        }
    }

    /// Turns the value into an array, if possible
    pub fn as_array(self) -> Option<Self> {
        match self.type_of() {
            ValueType::Integer
            | ValueType::Decimal
            | ValueType::String
            | ValueType::Boolean
            | ValueType::Primitive => Some(Value::Array([self].to_vec())),
            ValueType::Array => Some(self),

            _ => match self {
                Value::Object(o) => Some(Value::Array(o.into_values().collect::<Vec<_>>())),

                Value::Range(r) => Some(Value::Array(
                    r.map(|i| Value::Primitive(Primitive::Integer(i)))
                        .collect::<Vec<_>>(),
                )),

                _ => None,
            },
        }
    }

    /// Turns the value into an array, if possible
    pub fn cast_array(self) -> Result<Vec<Value>, ValueError> {
        let own_type = self.type_of();
        match self.as_array() {
            Some(Value::Array(a)) => Ok(a),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Array)),
        }
    }

    /// Turns the value into an object, if possible
    pub fn as_object(self) -> Option<Self> {
        match self {
            Value::Primitive(p) => Some(Value::Object(
                [(Primitive::Integer(0), Value::Primitive(p))]
                    .into_iter()
                    .collect(),
            )),

            Value::Array(a) => Some(Value::Object(
                a.into_iter()
                    .enumerate()
                    .map(|(i, v)| (Primitive::Integer(i as i128), v))
                    .collect(),
            )),

            Value::Object(_) => Some(self),

            Value::Range(r) => Some(Value::Object(
                r.into_iter()
                    .enumerate()
                    .map(|(i, v)| (Primitive::Integer(i as i128), Value::integer(v)))
                    .collect(),
            )),

            Value::Function(f) => Some(Value::Object(f.docs.into_hashmap())),
        }
    }

    /// Returns an object representation of the value, if possible
    pub fn cast_object(self) -> Result<HashMap<Primitive, Value>, ValueError> {
        let own_type = self.type_of();
        match self.as_object() {
            Some(Value::Object(o)) => Ok(o),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Object)),
        }
    }

    pub fn as_range(self) -> Option<Self> {
        match self {
            Value::Range(_) => Some(self),
            Value::Primitive(p) => p.into_integer().map(|i| Value::Range(i..i + 1)),
            _ => None,
        }
    }

    pub fn into_range(self) -> Result<std::ops::Range<i128>, ValueError> {
        let own_type = self.type_of();
        match self.as_range() {
            Some(Value::Range(r)) => Ok(r),
            _ => Err(ValueError::TypeConversion(own_type, ValueType::Range)),
        }
    }

    /// Sort this value
    /// For arrays, this sorts the array in place
    /// For all other types, this does nothing as by definition they are already sorted
    /// (length 1, or unordered)
    pub fn sort(&mut self) {
        match self {
            Value::Array(a) => a.sort_by(|a, b| a.cmp(b)),
            _ => {}
        }
    }
}

impl CheckedArithmetic for Value {
    fn checked_add(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_add(b).map(Value::Primitive),
            (Value::Array(mut a), Value::Array(mut b)) => {
                a.append(&mut b);
                Ok(Value::Array(a))
            }
            (Value::Object(mut a), Value::Object(b)) => {
                for (k, v) in b.into_iter() {
                    a.insert(k, v);
                }
                Ok(Value::Object(a))
            }
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_sub(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_sub(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_mul(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_mul(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_div(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_div(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_rem(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_rem(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_pow(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_pow(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_neg(self) -> Result<Self, ValueError> {
        let t = self.type_of();
        match self {
            Value::Primitive(p) => p.checked_neg().map(Value::Primitive),
            Value::Array(a) => Ok(Value::Array(a.into_iter().rev().collect())),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }
}

impl CheckedBitwise for Value {
    fn checked_and(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_and(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_or(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_or(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_xor(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_xor(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_shl(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_shl(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_shr(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_shr(b).map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_not(self) -> Result<Self, ValueError> {
        let t = self.type_of();
        match self {
            Value::Primitive(p) => p.checked_not().map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }
}

impl CheckedBoolean for Value {
    fn checked_logical_and(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => {
                a.checked_logical_and(b).map(Value::Primitive)
            }
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_logical_or(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        let t = a.type_of();
        match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => {
                a.checked_logical_or(b).map(Value::Primitive)
            }
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_logical_not(self) -> Result<Self, ValueError> {
        let t = self.type_of();
        match self {
            Value::Primitive(p) => p.checked_logical_not().map(Value::Primitive),
            _ => Err(ValueError::InvalidOperationForType(t)),
        }
    }

    fn checked_eq(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_eq(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a == b),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a == b),
            (Value::Range(a), Value::Range(b)) => Value::boolean(a == b),
            _ => Value::boolean(false),
        })
    }

    fn checked_ne(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_ne(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a != b),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a != b),
            (Value::Range(a), Value::Range(b)) => Value::boolean(a != b),
            _ => Value::boolean(false),
        })
    }

    fn checked_gt(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_gt(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a.len() > b.len()),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a.len() > b.len()),
            (Value::Range(a), Value::Range(b)) => {
                Value::boolean((a.end - a.start) > (b.end - b.start))
            }
            _ => Value::boolean(false),
        })
    }

    fn checked_ge(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_ge(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a.len() >= b.len()),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a.len() >= b.len()),
            (Value::Range(a), Value::Range(b)) => {
                Value::boolean((a.end - a.start) >= (b.end - b.start))
            }
            _ => Value::boolean(false),
        })
    }

    fn checked_lt(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_lt(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a.len() < b.len()),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a.len() < b.len()),
            (Value::Range(a), Value::Range(b)) => {
                Value::boolean((a.end - a.start) < (b.end - b.start))
            }
            _ => Value::boolean(false),
        })
    }

    fn checked_le(self, other: Self) -> Result<Self, ValueError> {
        let (a, b) = self.resolve(other)?;
        Ok(match (a, b) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_le(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a.len() <= b.len()),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a.len() <= b.len()),
            (Value::Range(a), Value::Range(b)) => {
                Value::boolean((a.end - a.start) <= (b.end - b.start))
            }
            _ => Value::boolean(false),
        })
    }

    fn checked_seq(self, other: Self) -> Result<Self, ValueError> {
        Ok(match (self, other) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_seq(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a == b),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a == b),
            (Value::Range(a), Value::Range(b)) => Value::boolean(a == b),
            _ => Value::boolean(false),
        })
    }

    fn checked_sne(self, other: Self) -> Result<Self, ValueError> {
        Ok(match (self, other) {
            (Value::Primitive(a), Value::Primitive(b)) => a.checked_sne(b).map(Value::Primitive)?,
            (Value::Array(a), Value::Array(b)) => Value::boolean(a == b),
            (Value::Object(a), Value::Object(b)) => Value::boolean(a == b),
            (Value::Range(a), Value::Range(b)) => Value::boolean(a == b),
            _ => Value::boolean(false),
        })
    }
}

impl CheckedMatching for Value {
    fn checked_matches(self, other: Self) -> Result<Self, ValueError> {
        match self.resolve(other)? {
            (Value::Primitive(Primitive::String(a)), Value::Primitive(Primitive::String(b))) => {
                Value::checked_regex(&a, &b, |mut s| {
                    if !s.starts_with('^') {
                        s.insert_str(0, "^");
                    }
                    if !s.ends_with('$') {
                        s.push('$');
                    }
                    s
                })
            }
            (a, b) => a.checked_eq(b),
        }
    }

    fn checked_contains(self, other: Self) -> Result<Self, ValueError> {
        Ok(match self {
            Value::Range(v) => Value::boolean(v.contains(&other.cast_integer()?)),
            Value::Array(v) => Value::boolean(v.contains(&other)),
            Value::Object(v) => Value::boolean(v.contains_key(&other.cast_primitive()?)),

            Value::Primitive(Primitive::String(a)) => {
                let b = other.cast_string()?;
                Value::checked_regex(&a, &b, |s| s)?
            }

            _ => return Err(ValueError::InvalidOperationForType(self.type_of())),
        })
    }

    fn checked_starts_with(self, other: Self) -> Result<Self, ValueError> {
        match self.resolve(other)? {
            (Value::Primitive(Primitive::String(a)), Value::Primitive(Primitive::String(b))) => {
                Value::checked_regex(&a, &b, |mut s| {
                    if !s.starts_with('^') {
                        s.insert_str(0, "^");
                    }
                    s
                })
            }

            (Value::Array(a), Value::Array(b)) => Ok(Value::boolean(a.starts_with(&b))),
            (Value::Range(a), Value::Range(b)) => {
                Ok(Value::boolean(b.start == a.start && b.end <= a.end))
            }

            (a, _) => Err(ValueError::InvalidOperationForType(a.type_of())),
        }
    }

    fn checked_ends_with(self, other: Self) -> Result<Self, ValueError> {
        match self.resolve(other)? {
            (Value::Primitive(Primitive::String(a)), Value::Primitive(Primitive::String(b))) => {
                Value::checked_regex(&a, &b, |mut s| {
                    if !s.ends_with('$') {
                        s.push('$');
                    }
                    s
                })
            }

            (Value::Array(a), Value::Array(b)) => Ok(Value::boolean(a.ends_with(&b))),
            (Value::Range(a), Value::Range(b)) => {
                Ok(Value::boolean(a.end == b.end && b.start >= a.start))
            }

            (a, _) => Err(ValueError::InvalidOperationForType(a.type_of())),
        }
    }

    fn checked_regex<F>(
        value: &str,
        pattern: &str,
        formatting_callback: F,
    ) -> Result<Self, ValueError>
    where
        F: Fn(String) -> String,
    {
        let pattern = convert_string_to_pattern(pattern, formatting_callback)?;
        Ok(Value::boolean(pattern.is_match(value)))
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Primitive(p) => write!(f, "{:?}", p)?,
            Value::Function(fnc) => write!(f, "{}", fnc.docs.signature)?,

            Value::Array(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, "]")?;
            }

            Value::Object(o) => {
                write!(f, "{{")?;
                for (k, v) in o.iter() {
                    write!(f, "{:?}: {:?}, ", k, v)?;
                }
                write!(f, "}}")?;
            }

            Value::Range(r) => {
                write!(f, "{}..{}", r.start, r.end)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Primitive(v) => write!(f, "{}", v)?,
            Value::Function(v) => write!(f, "{}", v.docs.signature)?,
            Value::Array(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|v| format!("{v:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
            Value::Object(v) => write!(
                f,
                "{{{}}}",
                v.iter()
                    .map(|(k, v)| format!("{k:?}: {v:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?,
            Value::Range(v) => write!(f, "{}..{}", v.start, v.end)?,
        }

        Ok(())
    }
}

impl SerializeToBytes for Value {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = vec![];

        match self {
            Value::Primitive(p) => {
                bytes.push(ValueType::Primitive as u8);
                bytes.extend(p.serialize_into_bytes());
            }

            Value::Function(f) => {
                bytes.push(ValueType::Function as u8);
                bytes.extend(f.serialize_into_bytes());
            }

            Value::Array(a) => {
                bytes.push(ValueType::Array as u8);
                bytes.extend(a.serialize_into_bytes());
            }

            Value::Object(o) => {
                bytes.push(ValueType::Object as u8);
                bytes.extend(o.len().serialize_into_bytes());
                for (k, v) in o {
                    bytes.extend(k.serialize_into_bytes());
                    bytes.extend(v.serialize_into_bytes());
                }
            }

            Value::Range(r) => {
                bytes.push(ValueType::Range as u8);
                bytes.extend(r.start.serialize_into_bytes());
                bytes.extend(r.end.serialize_into_bytes());
            }
        }

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let ty = u8::deserialize_from_bytes(bytes)?;
        match ValueType::from_u8(ty) {
            Some(ValueType::Primitive) => {
                Ok(Value::Primitive(Primitive::deserialize_from_bytes(bytes)?))
            }

            Some(ValueType::Function) => {
                Ok(Value::Function(Function::deserialize_from_bytes(bytes)?))
            }
            Some(ValueType::Array) => Ok(Value::Array(Vec::deserialize_from_bytes(bytes)?)),
            Some(ValueType::Object) => {
                let len = usize::deserialize_from_bytes(bytes)?;
                let mut o = HashMap::new();
                for _ in 0..len {
                    let k = Primitive::deserialize_from_bytes(bytes)?;
                    let v = Value::deserialize_from_bytes(bytes)?;
                    o.insert(k, v);
                }
                Ok(Value::Object(o))
            }
            Some(ValueType::Range) => {
                let start = i128::deserialize_from_bytes(bytes)?;
                let end = i128::deserialize_from_bytes(bytes)?;
                Ok(Value::Range(start..end))
            }

            _ => Err(crate::traits::ByteDecodeError::MalformedData(
                "Value".to_string(),
                "Invalid value type".to_string(),
            )),
        }
    }
}

fn convert_string_to_pattern<F>(
    input: &str,
    formatting_callback: F,
) -> Result<regex::Regex, ValueError>
where
    F: Fn(String) -> String,
{
    const FLAG_INREGEX: usize = 0b0001;
    const FLAG_INFLAGS: usize = 0b0010;
    const FLAG_ESCAPE: usize = 0b0100;

    let mut pattern = String::new();
    let mut flags = Vec::new();
    let mut state = 0;
    for char in input.chars() {
        if state & FLAG_ESCAPE != 0 {
            state &= !FLAG_ESCAPE;
            pattern.push(char);
            continue;
        }

        match char {
            '/' if state & FLAG_INREGEX == 0 => {
                state |= FLAG_INREGEX;
            }
            '/' => {
                state &= !FLAG_INREGEX;
                state |= FLAG_INFLAGS;
            }
            '\\' => {
                state |= FLAG_ESCAPE;
                pattern.push(char);
            }
            _ if state & FLAG_INFLAGS != 0 => {
                flags.push(char);
            }
            _ => {
                pattern.push(char);
            }
        }
    }

    // Catch the case where the string starts with a / but doesn't end with one
    if state & FLAG_INREGEX != 0 {
        pattern = input.to_string();
    }

    pattern = formatting_callback(pattern);

    let mut regex = regex::RegexBuilder::new(&pattern);
    for flag in flags {
        match flag {
            'i' => regex.case_insensitive(true),
            'm' => regex.multi_line(true),
            's' => regex.dot_matches_new_line(true),
            'U' => regex.swap_greed(true),
            'u' => regex.unicode(true),
            'x' => regex.ignore_whitespace(true),
            _ => {
                return Err(ValueError::InvalidRegexFlag(flag));
            }
        };
    }

    Ok(regex.build()?)
}
