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
    fn jump_if_not_empty(&mut self) -> Result<(), RuntimeError>;

    fn is_empty(&mut self) -> Result<bool, RuntimeError>;
}

impl ControlExt for super::ExecutionContext<'_> {
    #[inline(always)]
    fn jump(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()?;
        self.set_pc(pos as usize);
        Ok(())
    }

    #[inline(always)]
    fn jump_if_false(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()?;

        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let truthy = value
            .cast_boolean()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        if !truthy {
            self.set_pc(pos as usize);
            Ok(())
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn jump_if_true(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()?;

        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        let truthy = value
            .cast_boolean()
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        if truthy {
            self.set_pc(pos as usize);
            Ok(())
        } else {
            Ok(())
        }
    }

    #[inline(always)]
    fn jump_if_not_empty(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()? as usize;
        if !self.is_empty()? {
            self.set_pc(pos);
        }
        Ok(())
    }

    #[inline(always)]
    fn jump_if_empty(&mut self) -> Result<(), RuntimeError> {
        let pos = self.read_u64()? as usize;
        if self.is_empty()? {
            self.set_pc(pos);
        }
        Ok(())
    }

    #[inline(always)]
    fn is_empty(&mut self) -> Result<bool, RuntimeError> {
        let value = self.pop()?;
        let value = self.resolve_reference(value)?;
        Ok(match value {
            Value::Primitive(_) | Value::Function(_) | Value::Reference(..) => false,
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            Value::Range(r) => r.is_empty(),
        })
    }
}
