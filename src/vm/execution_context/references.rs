use crate::{
    value::{Reference, Value},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, StackExt};

pub trait RefExt {
    fn resolve_reference(&mut self, value: Value) -> Result<Value, RuntimeError>;

    fn consume_reference(&mut self) -> Result<(), RuntimeError>;
    fn read_reference(&mut self) -> Result<(), RuntimeError>;
    fn write_reference(&mut self) -> Result<(), RuntimeError>;
    fn delete_reference(&mut self) -> Result<(), RuntimeError>;
}

impl RefExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn resolve_reference(&mut self, value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::Reference(mut reference) => {
                reference
                    .resolve(&mut self.mem)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                reference
                    .value(&mut self.mem)
                    .map(|v| v.clone())
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))
            }
            _ => Ok(value),
        }
    }

    #[inline(always)]
    fn consume_reference(&mut self) -> Result<(), RuntimeError> {
        let reference = self.pop()?;
        let value = self.resolve_reference(reference)?;
        self.push(value);
        Ok(())
    }

    #[inline(always)]
    fn read_reference(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let reference = Reference::Unresolved(name_hash);
        self.push(Value::Reference(reference));
        Ok(())
    }

    #[inline(always)]
    fn write_reference(&mut self) -> Result<(), RuntimeError> {
        let reference = self.pop()?;
        let value = self.pop()?;
        match reference {
            Value::Reference(mut reference) => {
                let value = self.resolve_reference(value)?;
                reference
                    .write(&mut self.mem, value.clone())
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                self.push(Value::Reference(reference));
            }
            _ => return Err(self.emit_err(RuntimeErrorType::NotAReference)),
        }

        Ok(())
    }

    fn delete_reference(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        match value {
            Value::Reference(mut reference) => {
                reference
                    .resolve(&mut self.mem)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                let value = reference
                    .delete(&mut self.mem)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                self.push(value);
            }
            _ => return Err(self.emit_err(RuntimeErrorType::NotAReference)),
        }

        Ok(())
    }
}
