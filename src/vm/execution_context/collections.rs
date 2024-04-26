use std::collections::HashMap;

use crate::{
    value::{Value, ValueType},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, RefExt, StackExt};

pub trait CollectionExt {
    fn make_array(&mut self) -> Result<(), RuntimeError>;
    fn make_object(&mut self) -> Result<(), RuntimeError>;
    fn make_range(&mut self) -> Result<(), RuntimeError>;

    fn push_array(&mut self) -> Result<(), RuntimeError>;
    fn push_object(&mut self) -> Result<(), RuntimeError>;
}

impl CollectionExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn make_array(&mut self) -> Result<(), RuntimeError> {
        let n = self.read_u64()? as usize;
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            let value = self.pop()?;
            let value = self.resolve_reference(value)?;
            values.push(value);
        }
        self.push(Value::Array(values));
        Ok(())
    }

    #[inline(always)]
    fn make_object(&mut self) -> Result<(), RuntimeError> {
        let n = self.read_u64()? as usize;
        let mut values = HashMap::with_capacity(n);
        for _ in 0..n {
            let key = self.pop()?;
            let key = self.resolve_reference(key)?;
            let key = key
                .cast_primitive()
                .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

            let value = self.pop()?;
            let value = self.resolve_reference(value)?;
            values.insert(key, value);
        }
        self.push(Value::Object(values));
        Ok(())
    }

    #[inline(always)]
    fn make_range(&mut self) -> Result<(), RuntimeError> {
        let end = self.pop()?;
        let start = self.pop()?;

        let start = self.resolve_reference(start)?;
        let end = self.resolve_reference(end)?;

        let (start, end) = start
            .resolve(end)
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        let types = start
            .type_of(None)
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        match types {
            ValueType::Integer => {
                let start = start
                    .cast_integer()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                let end = end
                    .cast_integer()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                if start > end {
                    self.push(Value::Range(end..start))
                } else {
                    self.push(Value::Range(start..end));
                }
            }
            ValueType::String => {
                let start = start
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                let end = end
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                if start.len() == 1 && end.len() == 1 {
                    let start = start.chars().next().unwrap();
                    let end = end.chars().next().unwrap();
                    let crange = if start > end {
                        (end..=start)
                            .map(|c| Value::string(c.to_string()))
                            .collect()
                    } else {
                        (start..=end)
                            .map(|c| Value::string(c.to_string()))
                            .collect()
                    };
                    self.push(Value::Array(crange));
                } else {
                    return Err(self.emit_err(RuntimeErrorType::InvalidStringsForRange));
                }
            }
            _ => return Err(self.emit_err(RuntimeErrorType::InvalidValuesForRange(types))),
        }
        Ok(())
    }

    #[inline(always)]
    fn push_array(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let value = self.resolve_reference(value)?;

        let array = self.pop()?;
        let array = self.resolve_reference(array)?;

        let mut array = array
            .cast_array()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
        array.push(value);
        self.push(Value::Array(array));

        Ok(())
    }

    #[inline(always)]
    fn push_object(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let value = self.resolve_reference(value)?;

        let key = self.pop()?;
        let key = self.resolve_reference(key)?;
        let key = key
            .cast_primitive()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        let object = self.pop()?;
        let object = self.resolve_reference(object)?;

        let mut object = object
            .cast_object()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
        object.insert(key, value);
        self.push(Value::Object(object));

        Ok(())
    }
}
