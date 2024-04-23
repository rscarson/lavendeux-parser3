use crate::{
    value::Value,
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, StackExt};

pub trait RefExt {
    fn resolve_reference(&mut self, value: Value) -> Result<Value, RuntimeError>;

    fn read_reference(&mut self) -> Result<(), RuntimeError>;
    fn write_reference(&mut self) -> Result<(), RuntimeError>;
    fn delete_reference(&mut self) -> Result<(), RuntimeError>;
}

impl RefExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn resolve_reference(&mut self, value: Value) -> Result<Value, RuntimeError> {
        value
            .reference_read(&mut self.mem)
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))
    }

    #[inline(always)]
    fn read_reference(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let value = self
            .mem
            .get_ref(name_hash)
            .ok_or_else(|| self.emit_err(RuntimeErrorType::NameError))?;
        self.push(Value::Reference(value, vec![]));
        Ok(())
    }

    #[inline(always)]
    fn write_reference(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        let reference = self.pop()?;
        match reference {
            Value::Reference(slotref, idxpath) => {
                if idxpath.is_empty() {
                    slotref.set(&mut self.mem, value);
                } else if idxpath.len() == 1 {
                    let idx = idxpath.iter().next().unwrap();
                    let reference = match slotref.get_mut(&mut self.mem) {
                        Some(value) => value,
                        None => return Err(self.emit_err(RuntimeErrorType::BadReference)),
                    };
                    reference
                        .set_index(idx.clone(), value)
                        .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                    self.push(Value::Reference(slotref, idxpath));
                } else {
                    let target_idx = idxpath.last().unwrap();
                    let mut reference = match slotref.get_mut(&mut self.mem) {
                        Some(value) => value,
                        None => return Err(self.emit_err(RuntimeErrorType::BadReference)),
                    };
                    for idx in &idxpath[..idxpath.len() - 1] {
                        reference = match reference.mut_index(idx) {
                            Ok(value) => value,
                            Err(e) => return Err(self.emit_err(RuntimeErrorType::Value(e))),
                        }
                    }
                    reference
                        .set_index(target_idx.clone(), value)
                        .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                }
            }
            _ => return Err(self.emit_err(RuntimeErrorType::BadReference)),
        }

        Ok(())
    }

    fn delete_reference(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        match value {
            Value::Reference(slotref, mut idxpath) => {
                if idxpath.is_empty() {
                    slotref
                        .delete(&mut self.mem)
                        .ok_or(self.emit_err(RuntimeErrorType::BadReference))?;
                } else if idxpath.len() == 1 {
                    let idx = idxpath.into_iter().next().unwrap();
                    let value = match slotref.get_mut(&mut self.mem) {
                        Some(value) => value,
                        None => return Err(self.emit_err(RuntimeErrorType::BadReference)),
                    };
                    let value = value
                        .delete_index(idx)
                        .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                    self.push(value);
                } else {
                    let target_idx = idxpath.pop().unwrap();
                    let mut value = match slotref.get_mut(&mut self.mem) {
                        Some(value) => value,
                        None => return Err(self.emit_err(RuntimeErrorType::BadReference)),
                    };
                    for idx in idxpath {
                        value = match value.mut_index(&idx) {
                            Ok(value) => value,
                            Err(e) => return Err(self.emit_err(RuntimeErrorType::Value(e))),
                        };
                    }
                    let value = value
                        .delete_index(target_idx)
                        .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                    self.push(value);
                }
            }
            _ => return Err(self.emit_err(RuntimeErrorType::BadReference)),
        }

        Ok(())
    }
}
