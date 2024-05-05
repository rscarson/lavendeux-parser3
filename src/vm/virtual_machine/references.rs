use crate::vm::{
    error::{RuntimeError, RuntimeErrorType},
    value_source::ValueSource,
};

use super::{IOExt, StackExt};

pub trait RefExt {
    fn verify_reference(&self) -> Result<(), RuntimeError>;
    fn consume_reference(&mut self) -> Result<(), RuntimeError>;
    fn read_reference(&mut self) -> Result<(), RuntimeError>;
    fn write_reference(&mut self) -> Result<(), RuntimeError>;
    fn delete_reference(&mut self) -> Result<(), RuntimeError>;
}

impl RefExt for super::VirtualMachine {
    #[inline(always)]
    fn verify_reference(&self) -> Result<(), RuntimeError> {
        let reference = self
            .mem
            .peek_blank()
            .ok_or_else(|| self.emit_err(RuntimeErrorType::StackEmpty(self.last_opcode)))?;
        reference
            .clone()
            .into_value(&self.mem)
            .map(|_| ())
            .map_err(|e| self.emit_err(e))?;
        Ok(())
    }

    #[inline(always)]
    fn consume_reference(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop_value()?;
        self.push_value(value);
        Ok(())
    }

    #[inline(always)]
    fn read_reference(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        self.push(ValueSource::unresolved(name_hash));
        Ok(())
    }

    #[inline(always)]
    fn write_reference(&mut self) -> Result<(), RuntimeError> {
        let mut reference = self.pop()?;
        let value = self.pop_value()?;

        reference
            .ref_set(value.clone(), &mut self.mem)
            .map_err(|e| self.emit_err(e))?;

        self.push_value(value);
        Ok(())
    }

    fn delete_reference(&mut self) -> Result<(), RuntimeError> {
        let reference = self.pop()?;
        let value = reference
            .delete(&mut self.mem)
            .map_err(|e| self.emit_err(e))?;
        self.push_value(value);
        Ok(())
    }
}
