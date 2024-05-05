use crate::traits::SafeVecAlloc;

use super::{Primitive, Value, ValueError, ValueType};

pub enum ValueIndexResult<'a> {
    Mutable(&'a mut Value),
    Immutable(&'a Value),
    Owned(Value),
}
impl ValueIndexResult<'_> {
    pub fn into_value(self) -> Value {
        match self {
            ValueIndexResult::Mutable(value) => value.clone(),
            ValueIndexResult::Immutable(value) => value.clone(),
            ValueIndexResult::Owned(value) => value,
        }
    }

    pub fn value(&self) -> &Value {
        match self {
            ValueIndexResult::Mutable(value) => value,
            ValueIndexResult::Immutable(value) => value,
            ValueIndexResult::Owned(value) => value,
        }
    }
}

pub trait IndexingExt {
    fn ref_index(&self, index: Value) -> Result<ValueIndexResult<'_>, ValueError>;
    fn mut_index(&mut self, index: Value) -> Result<ValueIndexResult<'_>, ValueError>;
    fn into_index(self, index: Value) -> Result<Value, ValueError>;
    fn delete_index(&mut self, index: Value) -> Result<Value, ValueError>;
    fn set_index(&mut self, index: Value, value: Value) -> Result<(), ValueError>;
}

impl IndexingExt for ValueIndexResult<'_> {
    fn ref_index(&self, index: Value) -> Result<ValueIndexResult<'_>, ValueError> {
        match self {
            ValueIndexResult::Mutable(value) => value.ref_index(index),
            ValueIndexResult::Immutable(value) => value.ref_index(index),
            ValueIndexResult::Owned(value) => value.ref_index(index),
        }
    }

    fn mut_index(&mut self, index: Value) -> Result<ValueIndexResult<'_>, ValueError> {
        match self {
            ValueIndexResult::Mutable(value) => value.mut_index(index),
            ValueIndexResult::Immutable(_) => Err(ValueError::ReadOnlyIndexing),
            ValueIndexResult::Owned(value) => value.mut_index(index),
        }
    }

    fn into_index(self, index: Value) -> Result<Value, ValueError> {
        match self {
            ValueIndexResult::Mutable(value) => value.clone().into_index(index),
            ValueIndexResult::Immutable(value) => value.clone().into_index(index),
            ValueIndexResult::Owned(value) => value.into_index(index),
        }
    }

    fn delete_index(&mut self, index: Value) -> Result<Value, ValueError> {
        match self {
            ValueIndexResult::Mutable(value) => value.delete_index(index),
            ValueIndexResult::Immutable(_) => Err(ValueError::ReadOnlyIndexing),
            ValueIndexResult::Owned(value) => value.delete_index(index),
        }
    }

    fn set_index(&mut self, index: Value, value: Value) -> Result<(), ValueError> {
        match self {
            ValueIndexResult::Mutable(base) => base.set_index(index, value),
            ValueIndexResult::Immutable(_) => Err(ValueError::ReadOnlyIndexing),
            ValueIndexResult::Owned(base) => base.set_index(index, value),
        }
    }
}

impl IndexingExt for Value {
    fn ref_index(&self, index: Value) -> Result<ValueIndexResult<'_>, ValueError> {
        let own_type = self.type_of();
        match self {
            Value::Array(a) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > a.len() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                match a.get(indices) {
                    Some(slice) if slice.len() == 1 => Ok(ValueIndexResult::Immutable(&slice[0])),
                    Some(slice) => Ok(ValueIndexResult::Owned(Value::Array(slice.to_vec()))),
                    None => Err(ValueError::IndexOutOfBounds),
                }
            }

            Value::Object(o) => match index {
                Value::Primitive(index) => match o.get(&index) {
                    Some(value) => Ok(ValueIndexResult::Immutable(value)),
                    None => Err(ValueError::KeyNotFound),
                },
                _ => Err(ValueError::CannotIndexUsing(
                    ValueType::Object,
                    index.type_of(),
                )),
            },

            Value::Primitive(Primitive::String(s)) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > s.chars().count() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                let s = s
                    .chars()
                    .skip(indices.start)
                    .take(indices.end - indices.start)
                    .collect::<String>();
                Ok(ValueIndexResult::Owned(Value::string(s)))
            }

            Value::Range(r) => {
                let r_len = r.end - r.start;
                let indices = index.into_range()?;
                if indices.start > r_len || indices.end > r_len {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let start = r.start + indices.start;
                let end = r.start + indices.end;
                match end - start {
                    1 => Ok(ValueIndexResult::Owned(Value::integer(start))),
                    _ => Ok(ValueIndexResult::Owned(Value::Range(start..end))),
                }
            }

            _ => Err(ValueError::CannotIndexInto(own_type)),
        }
    }

    fn mut_index(&mut self, index: Value) -> Result<ValueIndexResult<'_>, ValueError> {
        match self {
            Value::Array(a) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > a.len() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                match a.get_mut(indices) {
                    Some(slice) if slice.len() == 1 => Ok(ValueIndexResult::Mutable(&mut slice[0])),
                    Some(slice) => Ok(ValueIndexResult::Owned(Value::Array(slice.to_vec()))),
                    None => Err(ValueError::IndexOutOfBounds),
                }
            }

            Value::Object(o) => match index {
                Value::Primitive(index) => match o.get_mut(&index) {
                    Some(value) => Ok(ValueIndexResult::Mutable(value)),
                    None => Err(ValueError::KeyNotFound),
                },
                _ => Err(ValueError::CannotIndexUsing(
                    ValueType::Object,
                    index.type_of(),
                )),
            },

            _ => self.ref_index(index),
        }
    }

    fn into_index(self, index: Value) -> Result<Value, ValueError> {
        match self {
            Value::Array(mut a) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > a.len() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                let mut result = a.drain(indices).collect::<Vec<_>>();
                match result.len() {
                    1 => Ok(result.pop().unwrap()),
                    _ => Ok(Value::Array(result)),
                }
            }

            Value::Object(mut o) => match index {
                Value::Primitive(index) => match o.remove(&index) {
                    Some(value) => Ok(value),
                    None => Err(ValueError::KeyNotFound),
                },
                _ => Err(ValueError::CannotIndexUsing(
                    ValueType::Object,
                    index.type_of(),
                )),
            },

            _ => self.ref_index(index).map(|r| r.into_value()),
        }
    }

    fn set_index(&mut self, index: Value, value: Value) -> Result<(), ValueError> {
        match self {
            Value::Array(a) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > a.len() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                match indices.len() {
                    1 => a[indices.start as usize] = value,
                    _ => {
                        a.drain(indices.clone());
                        let mut a2 = Vec::safe_alloc(a.len() + indices.len())?;
                        a2.extend_from_slice(&a[..indices.start as usize]);
                        a2.extend(value.cast_array()?);
                        a2.extend_from_slice(&a[indices.end as usize..]);

                        *self = Value::Array(a2);
                    }
                }
                Ok(())
            }

            Value::Object(o) => match index {
                Value::Primitive(index) => {
                    o.insert(index, value);
                    Ok(())
                }
                _ => Err(ValueError::CannotIndexUsing(
                    ValueType::Object,
                    index.type_of(),
                )),
            },

            Value::Primitive(Primitive::String(s)) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > s.chars().count() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                let s = s
                    .chars()
                    .take(indices.start)
                    .chain(value.cast_string()?.chars())
                    .chain(s.chars().skip(indices.end))
                    .collect::<String>();
                *self = Value::string(s);
                Ok(())
            }

            _ => Err(ValueError::CannotIndexInto(self.type_of())),
        }
    }

    fn delete_index(&mut self, index: Value) -> Result<Value, ValueError> {
        match self {
            Value::Array(a) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > a.len() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                let mut result = a.drain(indices).collect::<Vec<_>>();
                match result.len() {
                    1 => Ok(result.pop().unwrap()),
                    _ => Ok(Value::Array(result)),
                }
            }

            Value::Object(o) => match index {
                Value::Primitive(index) => match o.remove(&index) {
                    Some(value) => Ok(value),
                    None => Err(ValueError::KeyNotFound),
                },
                _ => Err(ValueError::CannotIndexUsing(
                    ValueType::Object,
                    index.type_of(),
                )),
            },

            Value::Primitive(Primitive::String(s)) => {
                let indices = index.into_range()?;
                if indices.start < 0 || indices.end > s.chars().count() as i128 {
                    return Err(ValueError::IndexOutOfBounds);
                }

                let indices = (indices.start as usize)..(indices.end as usize);
                let result = s[indices.clone()].to_string();
                *s = s
                    .chars()
                    .take(indices.start)
                    .chain(s.chars().skip(indices.end))
                    .collect::<String>();
                Ok(Value::string(result))
            }

            Value::Range(r) => {
                let indices = index.into_range()?;
                if indices.start == 0 && indices.end < r.end {
                    // If the range starts with 0 and end is < r.end, then we can just set r.start to end
                    r.start = indices.end;
                    Ok(Value::Range(r.start..indices.end))
                } else if indices.start > r.start && indices.end == r.end {
                    // If the range starts inside the range and ends at the end of the range, then we can just set r.end to start
                    r.end = indices.start;
                    Ok(Value::Range(indices.start..r.end))
                } else {
                    Err(ValueError::IndexOutOfBounds)
                }
            }

            _ => Err(ValueError::CannotIndexInto(self.type_of())),
        }
    }
}
