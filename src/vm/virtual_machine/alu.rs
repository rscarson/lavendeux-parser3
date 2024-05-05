use crate::{
    value::Value,
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::StackExt;

pub trait ALUExt {
    /// Perform a unary value operation on the top of the stack.
    /// The result is pushed back onto the stack.
    fn op_unary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<Value, crate::value::ValueError>;

    /// Perform a binary value operation on the top two values of the stack.
    /// The result is pushed back onto the stack.
    fn op_binary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value, Value) -> Result<Value, crate::value::ValueError>;
}

impl ALUExt for super::VirtualMachine {
    #[inline(always)]
    fn op_unary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<Value, crate::value::ValueError>,
    {
        let a = self.pop_value()?;
        self.push_value(handler(a).map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?);
        Ok(())
    }

    #[inline(always)]
    fn op_binary<F>(&mut self, handler: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value, Value) -> Result<Value, crate::value::ValueError>,
    {
        let a = self.pop_value()?;
        let b = self.pop_value()?;
        self.push_value(handler(b, a).map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?);
        Ok(())
    }
}
