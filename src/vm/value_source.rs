use crate::value::Value;

use super::{
    error::RuntimeErrorType,
    memory_manager::{MemoryManager, SlotRef},
};

/// A source for a value
pub enum ValueSource {
    /// A reference that has not been resolved yet
    /// Or does not exist in memory
    Unresolved(u64),

    /// A reference that has been resolved
    /// Contains the SlotRef to the value
    /// And a path of indexes to traverse to get to the value
    Resolved(SlotRef, Vec<Value>),

    /// A constant value, that is not stored in memory
    Literal(Value),
}

impl ValueSource {
    /// Attempt to get a reference to the value that this source points to
    pub fn value<'mem>(
        &'mem self,
        mem: &'mem MemoryManager,
    ) -> Result<&'mem Value, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.read(*name_hash) {
                Some(value) => Ok(value),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, idxpath) => {
                let mut value = slotref.get(mem).ok_or(RuntimeErrorType::SlotRefInvalid)?;
                for idx in idxpath {
                    value = value
                        .get_index(idx.clone())
                        .map_err(RuntimeErrorType::Value)?;
                }
                Ok(value)
            }

            Self::Literal(value) => Ok(value),
        }
    }

    /// Attempt to get a mutable reference to the value that this source points to
    pub fn value_mut<'mem>(
        &'mem mut self,
        mem: &'mem mut MemoryManager,
    ) -> Result<&'mem mut Value, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.read_mut(*name_hash) {
                Some(value) => Ok(value),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, idxpath) => {
                let mut value = slotref
                    .get_mut(mem)
                    .ok_or(RuntimeErrorType::SlotRefInvalid)?;
                for idx in idxpath {
                    value = value.mut_index(&idx).map_err(RuntimeErrorType::Value)?;
                }
                Ok(value)
            }

            Self::Literal(value) => Ok(value),
        }
    }

    /// Consume this source and return a new one with the value as a literal
    pub fn into_literal(self, mem: &mut MemoryManager) -> Result<Self, RuntimeErrorType> {
        if matches!(self, Self::Literal(_)) {
            Ok(self)
        } else {
            Ok(Self::Literal(self.value(mem)?.clone()))
        }
    }

    /// Consume this source and return a new one with the value as a resolved reference
    /// This has no effect if the source is a literal
    /// If already resolved, will verify the reference is still valid
    pub fn into_resolved(self, mem: &mut MemoryManager) -> Result<Self, RuntimeErrorType> {
        if let Self::Unresolved(name_hash) = self {
            match mem.get_ref(name_hash) {
                Some(value) => Ok(Self::Resolved(value, vec![])),
                None => Err(RuntimeErrorType::HashNotFound),
            }
        } else {
            self.value(mem)?;
            Ok(self)
        }
    }

    /// Attempt to delete the value that this source points to
    /// This will fail if the source is a literal
    pub fn delete(self, mem: &mut MemoryManager) -> Result<Value, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.delete(name_hash) {
                Some(value) => Ok(value),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, mut idxpath) => match idxpath.pop() {
                Some(index) => {
                    let mut value = slotref
                        .get_mut(mem)
                        .ok_or(RuntimeErrorType::SlotRefInvalid)?;
                    for idx in idxpath {
                        value = value.mut_index(&idx).map_err(RuntimeErrorType::Value)?;
                    }
                    value.delete_index(index).map_err(RuntimeErrorType::Value)
                }
                None => slotref.delete(mem).ok_or(RuntimeErrorType::SlotRefInvalid),
            },

            Self::Literal(_) => Err(RuntimeErrorType::DeleteLiteral),
        }
    }

    /// Attempt to set the value that this source points to
    pub fn set(&mut self, value: Value, mem: &mut MemoryManager) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => {
                *self = Self::Resolved(mem.write(*name_hash, value), vec![])
            }

            Self::Resolved(slotref, idxpath) => match idxpath.last() {
                Some(last_index) => {
                    let mut base = slotref
                        .get_mut(mem)
                        .ok_or(RuntimeErrorType::SlotRefInvalid)?;
                    for idx in &idxpath[..idxpath.len() - 1] {
                        base = base.mut_index(&idx).map_err(RuntimeErrorType::Value)?;
                    }
                    base.set_index(last_index.clone(), value)
                        .map_err(RuntimeErrorType::Value)?;
                }
                None => slotref.set(mem, value),
            },

            Self::Literal(v) => *v = value,
        }

        Ok(())
    }
}
