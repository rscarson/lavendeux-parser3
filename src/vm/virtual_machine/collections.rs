use std::collections::HashMap;

use crate::{
    traits::SafeVecAlloc,
    value::{Value, ValueType},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, StackExt};

pub trait CollectionExt {
    fn op_make_array(&mut self) -> Result<(), RuntimeError>;
    fn op_make_object(&mut self) -> Result<(), RuntimeError>;
    fn op_make_range(&mut self) -> Result<(), RuntimeError>;

    fn op_push_array(&mut self) -> Result<(), RuntimeError>;
    fn op_push_object(&mut self) -> Result<(), RuntimeError>;
}

impl CollectionExt for super::VirtualMachine {
    #[inline(always)]
    fn op_make_array(&mut self) -> Result<(), RuntimeError> {
        let n = self.read_u64()? as usize;
        let mut values =
            Vec::safe_alloc(n).map_err(|e| self.emit_err(RuntimeErrorType::MemoryAllocation(e)))?;
        for _ in 0..n {
            let value = self.pop_value()?;
            values.push(value);
        }
        self.push_value(Value::Array(values));
        Ok(())
    }

    #[inline(always)]
    fn op_make_object(&mut self) -> Result<(), RuntimeError> {
        let n = self.read_u64()? as usize;
        let mut values = HashMap::safe_alloc(n)
            .map_err(|e| self.emit_err(RuntimeErrorType::MemoryAllocation(e)))?;
        for _ in 0..n {
            let key = self.pop_value()?;
            let key = key
                .cast_primitive()
                .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

            let value = self.pop_value()?;
            values.insert(key, value);
        }
        self.push_value(Value::Object(values));
        Ok(())
    }

    #[inline(always)]
    fn op_make_range(&mut self) -> Result<(), RuntimeError> {
        let end = self.pop_value()?;
        let start = self.pop_value()?;
        let (start, end) = start
            .resolve(end)
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        let types = start.type_of();
        match types {
            ValueType::Integer => {
                let start = start
                    .cast_integer()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                let end = end
                    .cast_integer()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                if start > end {
                    self.push_value(Value::Range(end..start))
                } else {
                    self.push_value(Value::Range(start..end));
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
                    self.push_value(Value::Array(crange));
                } else {
                    return Err(self.emit_err(RuntimeErrorType::InvalidStringsForRange));
                }
            }
            _ => return Err(self.emit_err(RuntimeErrorType::InvalidValuesForRange(types))),
        }
        Ok(())
    }

    #[inline(always)]
    fn op_push_array(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;

        let array = self.pop_value()?;

        let mut array = array
            .cast_array()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
        array.push(value);
        self.push_value(Value::Array(array));

        Ok(())
    }

    #[inline(always)]
    fn op_push_object(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;

        let key = self.pop_value()?;
        let key = key
            .cast_primitive()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        let object = self.pop_value()?;

        let mut object = object
            .cast_object()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
        object.insert(key, value);
        self.push_value(Value::Object(object));

        Ok(())
    }
}
