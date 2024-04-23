use crate::{
    value::Value,
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, RefExt, StackExt};

pub trait MemExt {
    fn write_memory(&mut self) -> Result<(), RuntimeError>;
    fn read_memory(&mut self) -> Result<(), RuntimeError>;
    fn delete_memory(&mut self) -> Result<(), RuntimeError>;
}

impl MemExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn read_memory(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let value = self
            .mem
            .read(name_hash)
            .ok_or(self.emit_err(RuntimeErrorType::NameError))?;
        self.push(value.clone());
        Ok(())
    }

    #[inline(always)]
    fn write_memory(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let value = self.mem.write(name_hash, value);
        let value = Value::Reference(value, vec![]);
        self.push(value);
        Ok(())
    }

    #[inline(always)]
    fn delete_memory(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        match self.mem.delete(name_hash) {
            Some(value) => self.push(value),
            None => return Err(self.emit_err(RuntimeErrorType::NameError)),
        }
        Ok(())
    }
}
