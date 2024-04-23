/// Represents the type of a value.
/// Can be a concrete type, a group of types, or all types.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ValueType {
    Boolean = 0b0001_0001,
    Integer = 0b0001_0010,
    Decimal = 0b0001_0100,
    String = 0b0001_1000,
    Array = 0b010_0001,
    Object = 0b0010_0010,
    Range = 0b0010_0100,
    Function = 0b0100_0001,

    Primitive = 0b0001_1111,
    Numeric = 0b0001_0111,
    Collection = 0b0010_0111,
    All = 0xFF,
}

impl ValueType {
    /// Parses a `ValueType` from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "bool" => Some(ValueType::Boolean),
            "int" => Some(ValueType::Integer),
            "float" => Some(ValueType::Decimal),
            "string" => Some(ValueType::String),
            "array" => Some(ValueType::Array),
            "object" => Some(ValueType::Object),
            "range" => Some(ValueType::Range),
            "function" => Some(ValueType::Function),
            "primitive" => Some(ValueType::Primitive),
            "numeric" => Some(ValueType::Numeric),
            "collection" => Some(ValueType::Collection),
            "all" => Some(ValueType::All),
            _ => None,
        }
    }

    /// Parses a `ValueType` from a byte.
    pub fn from_u8(value: u8) -> Option<Self> {
        Some(match value {
            0b0001_0001 => ValueType::Boolean,
            0b0001_0010 => ValueType::Integer,
            0b0001_0100 => ValueType::Decimal,
            0b0001_1000 => ValueType::String,
            0b010_0001 => ValueType::Array,
            0b0010_0010 => ValueType::Object,
            0b0010_0100 => ValueType::Range,
            0b0100_0001 => ValueType::Function,
            0b0001_1111 => ValueType::Primitive,
            0b0001_0111 => ValueType::Numeric,
            0b0010_0111 => ValueType::Collection,
            0xFF => ValueType::All,
            _ => return None,
        })
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Boolean => write!(f, "bool"),
            ValueType::Integer => write!(f, "int"),
            ValueType::Decimal => write!(f, "float"),
            ValueType::String => write!(f, "string"),
            ValueType::Array => write!(f, "array"),
            ValueType::Object => write!(f, "object"),
            ValueType::Range => write!(f, "range"),
            ValueType::Function => write!(f, "function"),
            ValueType::Primitive => write!(f, "primitive"),
            ValueType::Numeric => write!(f, "numeric"),
            ValueType::Collection => write!(f, "collection"),
            ValueType::All => write!(f, "all"),
        }
    }
}
