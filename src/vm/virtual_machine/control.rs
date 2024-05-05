use crate::{
    value::{Value, ValueError},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, StackExt};

pub trait ControlExt {
    fn jump(&mut self) -> Result<(), RuntimeError>;
    fn jump_if<F>(&mut self, f: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<bool, ValueError>;

    fn op_jump_if_false(&mut self) -> Result<(), RuntimeError>;
    fn op_jump_if_true(&mut self) -> Result<(), RuntimeError>;
    fn op_jump_if_empty(&mut self) -> Result<(), RuntimeError>;
    fn op_jump_if_not_empty(&mut self) -> Result<(), RuntimeError>;

    fn jump_to(&mut self, pos: usize) -> Result<(), RuntimeError>;
}

impl ControlExt for super::VirtualMachine {
    #[inline(always)]
    fn jump(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()?;
        self.jump_to(pos as usize)
    }

    fn jump_if<F>(&mut self, f: F) -> Result<(), RuntimeError>
    where
        F: Fn(Value) -> Result<bool, ValueError>,
    {
        let pos = self.read_u64()?;
        let value = self.pop_value()?;
        if f(value).map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))? {
            self.jump_to(pos as usize)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn op_jump_if_false(&mut self) -> Result<(), RuntimeError> {
        self.jump_if(|v| v.cast_boolean().map(|b| !b))
    }

    #[inline(always)]
    fn op_jump_if_true(&mut self) -> Result<(), RuntimeError> {
        self.jump_if(|v| v.cast_boolean())
    }

    #[inline(always)]
    fn op_jump_if_not_empty(&mut self) -> Result<(), RuntimeError> {
        self.jump_if(|v| Ok(v.len() > 0))
    }

    #[inline(always)]
    fn op_jump_if_empty(&mut self) -> Result<(), RuntimeError> {
        self.jump_if(|v| Ok(v.len() == 0))
    }

    fn jump_to(&mut self, pos: usize) -> Result<(), RuntimeError> {
        self.context_mut().set_pc(pos);
        Ok(())
    }
}
