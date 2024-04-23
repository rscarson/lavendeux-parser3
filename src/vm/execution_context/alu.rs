use crate::{
    value::Value,
    vm::error::{RuntimeError, RuntimeErrorType},
};

use crate::vm::execution_context::RefExt;

use super::StackExt;

pub trait ALUExt {
    fn op_unary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<Value, crate::value::ValueError>;

    fn op_binary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value, Value) -> Result<Value, crate::value::ValueError>;
}

impl ALUExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn op_unary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<Value, crate::value::ValueError>,
    {
        let a = self.pop()?;
        let a = self.resolve_reference(a)?;
        self.push(handler(a).map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?);
        Ok(())
    }

    #[inline(always)]
    fn op_binary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value, Value) -> Result<Value, crate::value::ValueError>,
    {
        let a = self.pop()?;
        let a = self.resolve_reference(a)?;

        let b = self.pop()?;
        let b = self.resolve_reference(b)?;

        self.push(handler(b, a).map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?);
        Ok(())
    }
}
