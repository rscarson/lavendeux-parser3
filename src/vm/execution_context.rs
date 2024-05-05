use crate::{compiler::DebugProfile, value::ValueType};

use super::error::{RuntimeError, RuntimeErrorType};

/// A contextual layer in the VM
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    code: Vec<u8>,
    pc: usize,
    debug_profile: Option<DebugProfile>,
    returns: ValueType,
}

impl ExecutionContext {
    pub fn new(code: Vec<u8>, debug_profile: Option<DebugProfile>, returns: ValueType) -> Self {
        Self {
            code,
            pc: 0,
            debug_profile,
            returns,
        }
    }

    /// Read the current context's bytecode
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Set the program counter to a new value
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    /// Read the current context's bytecode
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Read the current context's debug profile
    pub fn debug_profile(&self) -> Option<&DebugProfile> {
        self.debug_profile.as_ref()
    }

    /// Read the current context's return type
    /// Panics if the context does not have a return type.
    pub fn return_type(&self) -> ValueType {
        self.returns
    }

    /// Emit an error at the current position
    pub fn emit_err(&self, error: RuntimeErrorType) -> RuntimeError {
        let e = RuntimeError {
            error,
            pos: self.pc(),
            token: None,
            parent: None,
        };
        match &self.debug_profile() {
            Some(profile) => e.with_context(&profile),
            None => e,
        }
    }
}
