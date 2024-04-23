use crate::{
    value::Value,
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, RefExt, StackExt};

pub trait ControlExt {
    fn jump(&mut self) -> Result<(), RuntimeError>;
    fn jump_if_false(&mut self) -> Result<(), RuntimeError>;
    fn jump_if_true(&mut self) -> Result<(), RuntimeError>;
    fn jump_if_empty(&mut self) -> Result<(), RuntimeError>;

    fn jump_by(&mut self, offset: usize) -> Result<(), RuntimeError>;
}

impl ControlExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn jump(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_i32()?;
        self.jump_by(offset as usize)
    }

    #[inline(always)]
    fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_i32()?;

        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let truthy = value
            .cast_boolean()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        if !truthy {
            self.jump_by(offset as usize)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn jump_if_true(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_i32()?;

        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let truthy = value
            .cast_boolean()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        if truthy {
            self.jump_by(offset as usize)
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn jump_if_empty(&mut self) -> Result<(), RuntimeError> {
        let offset = self.read_i32()? as usize;
        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let is_empty = match value {
            Value::Primitive(_) | Value::Function(_) | Value::Reference(..) => false,
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            Value::Range(r) => r.is_empty(),
        };
        if is_empty {
            self.set_pc(
                self.pc()
                    .checked_add(offset)
                    .ok_or(self.emit_err(RuntimeErrorType::UnexpectedEnd(self.last_opcode)))?,
            );
        }
        Ok(())
    }

    #[inline(always)]
    fn jump_by(&mut self, offset: usize) -> Result<(), RuntimeError> {
        self.set_pc(
            self.pc()
                .checked_add(offset)
                .ok_or(self.emit_err(RuntimeErrorType::UnexpectedEnd(self.last_opcode)))?,
        );
        Ok(())
    }
}
