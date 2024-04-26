use crate::{
    value::Value,
    vm::{
        error::{RuntimeError, RuntimeErrorType},
        OpCode,
    },
};

use super::IOExt;

pub trait StackExt {
    fn push(&mut self, value: Value);
    fn pop(&mut self) -> Result<Value, RuntimeError>;

    fn op_push(&mut self) -> Result<(), RuntimeError>;
    fn op_pop(&mut self) -> Result<(), RuntimeError>;
    fn swap(&mut self) -> Result<(), RuntimeError>;
    fn dup(&mut self) -> Result<(), RuntimeError>;
}

impl StackExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack
            .pop()
            .ok_or(self.emit_err(RuntimeErrorType::StackEmpty(self.last_opcode)))
    }

    #[inline(always)]
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    #[inline(always)]
    fn op_push(&mut self) -> Result<(), RuntimeError> {
        let value = self.read_value()?;
        self.stack.push(value);
        Ok(())
    }

    #[inline(always)]
    fn op_pop(&mut self) -> Result<(), RuntimeError> {
        self.stack
            .pop()
            .ok_or(self.emit_err(RuntimeErrorType::StackEmpty(OpCode::POP)))?;
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
