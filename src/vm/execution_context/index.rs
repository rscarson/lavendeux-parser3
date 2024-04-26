use std::{collections::HashMap, ops::Range};

use crate::{
    value::{Primitive, Value},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{RefExt, StackExt};

pub type Map = HashMap<Primitive, Value>;

pub trait IndexExt {
    fn index_into(&mut self) -> Result<(), RuntimeError>;
}

impl IndexExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn index_into(&mut self) -> Result<(), RuntimeError> {
        // Ok this one is a bit of a workhorse
        // It's used for indexing, references, and assignments
        // First we pop [base, index] off the stack
        let index = self.pop()?;
        let base = self.pop()?;

        let index = self.resolve_reference(index)?;

        // Now we branch on whether the base is a reference
        match base {
            Value::Reference(mut reference) => {
                // This is the easiest cast, we just add the new index to the ref path
                // The actual work of checking the reference happens when the value is used
                reference
                    .resolve(&mut self.mem)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                reference
                    .add_index(index)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                self.push(Value::Reference(reference));
            }

            Value::Primitive(Primitive::String(string)) => {
                // If the base is a string, we're getting some chars
                self.push(match index {
                    Value::Range(range) => {
                        idx_string_by_range(string, range).map_err(|e| self.emit_err(e))?
                    }
                    Value::Array(array) => {
                        idx_string_by_arr(string, array).map_err(|e| self.emit_err(e))?
                    }
                    Value::Primitive(p) => {
                        idx_string_by_val(string, p).map_err(|e| self.emit_err(e))?
                    }
                    _ => return Err(self.emit_err(RuntimeErrorType::IndexingType)),
                });
            }

            Value::Range(range) => {
                // If the base is a range, we're getting a set of values
                match index {
                    Value::Array(array) => {
                        self.push(idx_range_by_arr(range, array).map_err(|e| self.emit_err(e))?);
                    }
                    Value::Range(index_rng) => {
                        self.push(
                            idx_range_by_range(range, index_rng).map_err(|e| self.emit_err(e))?,
                        );
                    }
                    Value::Primitive(p) => {
                        self.push(idx_range_by_val(range, p).map_err(|e| self.emit_err(e))?);
                    }

                    _ => return Err(self.emit_err(RuntimeErrorType::IndexingType)),
                }
            }

            Value::Array(array) => {
                // If the base is an array, we're getting a value
                match index {
                    Value::Range(range) => {
                        self.push(idx_arr_by_range(array, range).map_err(|e| self.emit_err(e))?);
                    }
                    Value::Array(index_arr) => {
                        self.push(idx_arr_by_arr(array, index_arr).map_err(|e| self.emit_err(e))?);
                    }
                    Value::Primitive(p) => {
                        self.push(idx_arr_by_val(array, p).map_err(|e| self.emit_err(e))?);
                    }
                    _ => return Err(self.emit_err(RuntimeErrorType::IndexingType)),
                }
            }

            Value::Object(object) => {
                // If the base is an object, we're getting a value
                match index {
                    Value::Range(range) => {
                        self.push(idx_obj_by_range(object, range).map_err(|e| self.emit_err(e))?);
                    }
                    Value::Array(index_arr) => {
                        self.push(idx_obj_by_arr(object, index_arr).map_err(|e| self.emit_err(e))?);
                    }
                    Value::Primitive(p) => {
                        self.push(idx_obj_by_val(object, p).map_err(|e| self.emit_err(e))?);
                    }
                    _ => return Err(self.emit_err(RuntimeErrorType::IndexingType)),
                }
            }

            _ => return Err(self.emit_err(RuntimeErrorType::IndexingBaseType)),
        }

        Ok(())
    }
}

#[inline(always)]
fn idx_string_by_val(base: String, index: Primitive) -> Result<Value, RuntimeErrorType> {
    match index {
        Primitive::Integer(mut i) => {
            if i < 0 {
                i = base.len() as i128 + i;
            }

            let c = base
                .chars()
                .nth(i as usize)
                .ok_or(RuntimeErrorType::IndexingValue)?;
            Ok(Value::string(c.to_string()))
        }
        _ => Err(RuntimeErrorType::IndexingType),
    }
}
#[inline(always)]
fn idx_string_by_arr(base: String, index: Vec<Value>) -> Result<Value, RuntimeErrorType> {
    let indices = index
        .into_iter()
        .map(|v| v.cast_integer().map_err(RuntimeErrorType::Value))
        .collect::<Result<Vec<_>, _>>()?;
    let s = indices
        .iter()
        .map(|i| {
            let i = if *i < 0 { base.len() as i128 + i } else { *i };
            base.chars()
                .nth(i as usize)
                .ok_or(RuntimeErrorType::IndexingValue)
        })
        .collect::<Result<String, RuntimeErrorType>>()?;
    Ok(Value::string(s))
}
#[inline(always)]
fn idx_string_by_range(base: String, index: Range<i128>) -> Result<Value, RuntimeErrorType> {
    let s = base
        .chars()
        .enumerate()
        .filter(|(i, _)| index.contains(&(*i as i128)))
        .map(|(_, c)| c)
        .collect();
    Ok(Value::string(s))
}

#[inline(always)]
fn idx_range_by_val(base: Range<i128>, index: Primitive) -> Result<Value, RuntimeErrorType> {
    match index {
        Primitive::Integer(i) => {
            let n = if i < 0 { base.end + i } else { base.start + i };

            if base.contains(&n) {
                Ok(Value::integer(n))
            } else {
                Err(RuntimeErrorType::IndexingValue)
            }
        }
        _ => Err(RuntimeErrorType::IndexingType),
    }
}
#[inline(always)]
fn idx_range_by_arr(base: Range<i128>, index: Vec<Value>) -> Result<Value, RuntimeErrorType> {
    let indices = index
        .into_iter()
        .map(|v| v.cast_integer().map_err(RuntimeErrorType::Value))
        .collect::<Result<Vec<_>, _>>()?;
    let values = indices
        .iter()
        .map(|i| {
            let n = if *i < 0 { base.end + i } else { base.start + i };
            if base.contains(&n) {
                Ok(Value::integer(n))
            } else {
                Err(RuntimeErrorType::IndexingValue)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(values))
}
#[inline(always)]
fn idx_range_by_range(base: Range<i128>, index: Range<i128>) -> Result<Value, RuntimeErrorType> {
    if base.start > index.end || base.end < index.start {
        Err(RuntimeErrorType::IndexingValue)
    } else {
        let start = base.start.max(index.start);
        let end = base.end.min(index.end);
        Ok(Value::Range(start..end))
    }
}

#[inline(always)]
fn idx_arr_by_val(base: Vec<Value>, index: Primitive) -> Result<Value, RuntimeErrorType> {
    match index {
        Primitive::Integer(mut i) => {
            if i < 0 {
                i = base.len() as i128 + i;
            }
            let value = base
                .into_iter()
                .nth(i as usize)
                .ok_or(RuntimeErrorType::IndexingValue)?;
            Ok(value)
        }
        _ => Err(RuntimeErrorType::IndexingType),
    }
}
#[inline(always)]
fn idx_arr_by_arr(base: Vec<Value>, index: Vec<Value>) -> Result<Value, RuntimeErrorType> {
    let mut indices = index
        .into_iter()
        .map(|v| {
            let mut i = v.cast_integer().map_err(RuntimeErrorType::Value)?;
            if i < 0 {
                i = base.len() as i128 + i;
            }
            Ok::<_, RuntimeErrorType>(i)
        })
        .collect::<Result<Vec<_>, _>>()?;
    indices.sort();
    let mut values = vec![];
    let mut iter = base.into_iter();
    let mut b = 0;
    for i in indices.into_iter() {
        values.push(
            iter.nth((i - b) as usize)
                .ok_or(RuntimeErrorType::IndexingValue)?,
        );
        b += i;
    }
    Ok(Value::Array(values))
}
#[inline(always)]
fn idx_arr_by_range(base: Vec<Value>, index: Range<i128>) -> Result<Value, RuntimeErrorType> {
    let values = base
        .into_iter()
        .enumerate()
        .filter(|(i, _)| index.contains(&(*i as i128)))
        .map(|(_, v)| v)
        .collect();
    Ok(Value::Array(values))
}

#[inline(always)]
fn idx_obj_by_val(mut base: Map, index: Primitive) -> Result<Value, RuntimeErrorType> {
    base.remove(&index).ok_or(RuntimeErrorType::IndexingValue)
}
#[inline(always)]
fn idx_obj_by_arr(mut base: Map, index: Vec<Value>) -> Result<Value, RuntimeErrorType> {
    let indices = index
        .into_iter()
        .map(|v| v.cast_primitive().map_err(RuntimeErrorType::Value))
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = vec![];
    for i in indices.iter() {
        values.push(base.remove(i).ok_or(RuntimeErrorType::IndexingValue)?);
    }
    Ok(Value::Array(values))
}
#[inline(always)]
fn idx_obj_by_range(base: Map, index: Range<i128>) -> Result<Value, RuntimeErrorType> {
    let values = base
        .into_iter()
        .filter(|(k, _)| {
            if let Primitive::Integer(i) = k {
                index.contains(i)
            } else {
                false
            }
        })
        .map(|(_, v)| v)
        .collect();
    Ok(Value::Array(values))
}
