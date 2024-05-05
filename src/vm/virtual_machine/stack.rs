use crate::{
    value::Value,
    vm::{
        error::{RuntimeError, RuntimeErrorType},
        value_source::ValueSource,
    },
};

use super::IOExt;

pub trait StackExt {
    fn pop_value(&mut self) -> Result<Value, RuntimeError>;
    fn push_value(&mut self, value: Value);

    fn push(&mut self, value: ValueSource);
    fn pop(&mut self) -> Result<ValueSource, RuntimeError>;

    fn op_push(&mut self) -> Result<(), RuntimeError>;
    fn op_pop(&mut self) -> Result<(), RuntimeError>;
    fn swap(&mut self) -> Result<(), RuntimeError>;
    fn dup(&mut self) -> Result<(), RuntimeError>;
}

impl StackExt for super::VirtualMachine {
    #[inline(always)]
    fn pop_value(&mut self) -> Result<Value, RuntimeError> {
        self.pop()?
            .into_value(&mut self.mem)
            .map_err(|e| self.emit_err(e))
    }

    #[inline(always)]
    fn push_value(&mut self, value: Value) {
        self.push(ValueSource::Literal(value));
    }

    #[inline(always)]
    fn pop(&mut self) -> Result<ValueSource, RuntimeError> {
        self.mem
            .pop_blank()
            .ok_or_else(|| self.emit_err(RuntimeErrorType::StackEmpty(self.last_opcode)))
    }

    #[inline(always)]
    fn push(&mut self, value: ValueSource) {
        self.mem.push_blank(value);
    }

    #[inline(always)]
    fn op_push(&mut self) -> Result<(), RuntimeError> {
        let value = self.read_value()?;
        self.push(ValueSource::Literal(value));
        Ok(())
    }

    #[inline(always)]
    fn op_pop(&mut self) -> Result<(), RuntimeError> {
        self.pop()?;
        Ok(())
    }

    #[inline(always)]
    fn swap(&mut self) -> Result<(), RuntimeError> {
        let value1 = self.pop()?;
        let value2 = self.pop()?;
        self.push(value1);
        self.push(value2);
        Ok(())
    }

    #[inline(always)]
    fn dup(&mut self) -> Result<(), RuntimeError> {
        let value = self.pop()?;
        self.push(value.clone());
        self.push(value);
        Ok(())
    }
}
