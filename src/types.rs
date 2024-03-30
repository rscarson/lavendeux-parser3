use crate::IntoOwned;
use std::collections::HashMap;

mod currency;

pub use currency::Currency;

#[derive(Debug, Clone)]
pub enum Primitive {
    Int(i128),
    Float(f64),
    String(String),
    Bool(bool),
    Currency(Currency),
    Array(Vec<Primitive>),
    Object(HashMap<Primitive, Primitive>),
}

#[derive(Debug, Clone)]
pub enum Value<'source> {
    Primitive(Primitive),
    Function(Vec<String>, crate::parser::Node<'source>),
    Reference,
}

impl IntoOwned for Value<'_> {
    type Owned = Value<'static>;
    fn into_owned(self) -> Self::Owned {
        match self {
            Self::Primitive(p) => Self::Owned::Primitive(p),
            Self::Function(args, body) => Self::Owned::Function(args, body.into_owned()),
            Self::Reference => Self::Owned::Reference,
        }
    }
}
