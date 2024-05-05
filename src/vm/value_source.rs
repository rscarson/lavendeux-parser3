//! Value sources are used to represent the source of a value
//! This can be a literal value, or a reference to a value in memory
use crate::value::{IndexingExt, Value, ValueError, ValueIndexResult, ValueType};

use super::{
    error::RuntimeErrorType,
    memory_manager::{MemoryManager, SlotRef},
};

/// A source for a value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueSource {
    /// A literal value
    Literal(Value),

    /// A reference to a value
    Reference(ValueReference),
}

impl ValueSource {
    /// Create a new ref value source
    pub fn unresolved(name_hash: u64) -> Self {
        Self::Reference(ValueReference::Unresolved(name_hash))
    }

    /// Create a new ref value source
    pub fn resolved(slotref: SlotRef, idxpath: Vec<Value>) -> Self {
        Self::Reference(ValueReference::Resolved(slotref, idxpath))
    }

    /// Consumes the source and returns the value
    /// If the source is a reference, the value will be cloned
    pub fn into_value(self, mem: &MemoryManager) -> Result<Value, RuntimeErrorType> {
        match self {
            Self::Literal(value) => Ok(value),
            Self::Reference(reference) => Ok(reference.into_value(mem)?),
        }
    }

    pub fn value<'mem>(
        &'mem self,
        mem: &'mem MemoryManager,
    ) -> Result<ValueIndexResult<'_>, RuntimeErrorType> {
        match self {
            Self::Literal(value) => Ok(ValueIndexResult::Immutable(&value)),
            Self::Reference(reference) => reference.value(mem),
        }
    }

    /// Attempt to get a mutable reference to the value
    pub fn value_mut<'mem>(
        &'mem mut self,
        mem: &'mem mut MemoryManager,
    ) -> Result<ValueIndexResult<'_>, RuntimeErrorType> {
        match self {
            Self::Literal(value) => Ok(ValueIndexResult::Mutable(value)),
            Self::Reference(reference) => reference.value_mut(mem),
        }
    }

    /// Similar to `set`, but will fail if the source is a literal
    pub fn ref_set(
        &mut self,
        value: Value,
        mem: &mut MemoryManager,
    ) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Literal(_) => Err(RuntimeErrorType::SetLiteral),
            Self::Reference(reference) => reference.set(value, mem),
        }
    }

    /// Overwrite the value
    pub fn set(&mut self, value: Value, mem: &mut MemoryManager) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Literal(_) => *self = Self::Literal(value),
            Self::Reference(reference) => reference.set(value, mem)?,
        }

        Ok(())
    }

    /// Attempt to delete the value
    /// This will fail if the source is a literal
    pub fn delete(self, mem: &mut MemoryManager) -> Result<Value, RuntimeErrorType> {
        match self {
            Self::Literal(_) => Err(RuntimeErrorType::DeleteLiteral),
            Self::Reference(reference) => reference.delete(mem),
        }
    }

    pub fn is_a(&self, mem: &MemoryManager, ty: ValueType) -> Result<bool, RuntimeErrorType> {
        Ok(self.value(mem)?.value().is_a(ty))
    }

    pub fn type_of(&self, mem: &MemoryManager) -> Result<ValueType, RuntimeErrorType> {
        Ok(self.value(mem)?.value().type_of())
    }
}

/// A referential source for a value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueReference {
    /// A reference that has not been resolved yet
    /// Or does not exist in memory
    Unresolved(u64),

    /// A reference that has been resolved
    /// Contains the SlotRef to the value
    /// And a path of indexes to traverse to get to the value
    Resolved(SlotRef, Vec<Value>),
}

impl ValueReference {
    pub fn value<'mem>(
        &'mem self,
        mem: &'mem MemoryManager,
    ) -> Result<ValueIndexResult<'_>, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.read(*name_hash) {
                Some(value) => value.value(mem),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, idxpath) => {
                let mut value = slotref
                    .get(mem)
                    .ok_or_else(|| RuntimeErrorType::SlotRefInvalid)?
                    .value(mem)?;

                let mut iter = idxpath.iter().peekable();
                loop {
                    let next = match iter.next() {
                        Some(next) => next,
                        None => break,
                    };

                    if let ValueIndexResult::Owned(v) = value {
                        if iter.peek().is_none() {
                            return Ok(ValueIndexResult::Owned(v));
                        } else {
                            return Err(RuntimeErrorType::Value(ValueError::CannotIndexInto(
                                v.type_of(),
                            )));
                        }
                    }

                    value = next
                        .ref_index(next.clone())
                        .map_err(RuntimeErrorType::Value)?;
                }

                Ok(value)
            }
        }
    }

    pub fn value_mut<'mem>(
        &'mem self,
        mem: &'mem mut MemoryManager,
    ) -> Result<ValueIndexResult<'_>, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.get_ref(*name_hash) {
                Some(slotref) => Self::resolve(&slotref, mem),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, idxpath) => {
                let mut value = Self::resolve(&slotref.clone(), mem)?;

                let mut iter = idxpath.iter().peekable();
                loop {
                    let next = match iter.next() {
                        Some(next) => next,
                        None => break,
                    };

                    if let ValueIndexResult::Owned(v) = value {
                        if iter.peek().is_none() {
                            return Ok(ValueIndexResult::Owned(v));
                        } else {
                            return Err(RuntimeErrorType::Value(ValueError::CannotIndexInto(
                                v.type_of(),
                            )));
                        }
                    }

                    value = value
                        .ref_index(next.clone())
                        .map_err(RuntimeErrorType::Value)?;
                }

                Ok(value)
            }
        }
    }

    /// Attempt to get the value that this source points to
    /// This will clone the value chain
    pub fn into_value(self, mem: &MemoryManager) -> Result<Value, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.read(name_hash) {
                Some(value) => Ok(value.clone().into_value(mem)?),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, idxpath) => {
                let mut value = slotref
                    .get(mem)
                    .ok_or_else(|| RuntimeErrorType::SlotRefInvalid)?
                    .clone()
                    .into_value(mem)?;
                for idx in idxpath {
                    value = value.into_index(idx).map_err(RuntimeErrorType::Value)?;
                }
                Ok(value)
            }
        }
    }

    /// Attempt to delete the value that this source points to
    /// This will fail if the source is a literal
    pub fn delete(self, mem: &mut MemoryManager) -> Result<Value, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.delete(name_hash) {
                Some(value) => Ok(value.into_value(mem)?),
                None => Err(RuntimeErrorType::HashNotFound),
            },

            Self::Resolved(slotref, mut idxpath) => match idxpath.pop() {
                Some(index) => {
                    let mut base = Self::resolve(&slotref, mem)?;

                    let mut iter = idxpath.iter().peekable();
                    loop {
                        let next = match iter.next() {
                            Some(next) => next,
                            None => break,
                        };

                        if let ValueIndexResult::Owned(v) = base {
                            if iter.peek().is_none() {
                                return Ok(v);
                            } else {
                                return Err(RuntimeErrorType::Value(ValueError::CannotIndexInto(
                                    v.type_of(),
                                )));
                            }
                        }

                        base = base
                            .ref_index(next.clone())
                            .map_err(RuntimeErrorType::Value)?;
                    }
                    base.delete_index(index).map_err(RuntimeErrorType::Value)
                }
                None => slotref
                    .delete(mem)
                    .ok_or_else(|| RuntimeErrorType::SlotRefInvalid)?
                    .into_value(mem),
            },
        }
    }

    /// Attempt to overwrite the value that this source points to
    pub fn set(&mut self, value: Value, mem: &mut MemoryManager) -> Result<(), RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => {
                if let Some(ValueSource::Reference(r)) = mem.read(*name_hash) {
                    r.clone().set(value, mem)?;
                } else {
                    *self =
                        Self::Resolved(mem.write(*name_hash, ValueSource::Literal(value)), vec![])
                }
            }

            Self::Resolved(slotref, idxpath) => match idxpath.last() {
                Some(last_index) => {
                    let mut base = Self::resolve(&slotref, mem)?;
                    for idx in &idxpath[..idxpath.len() - 1] {
                        base = base
                            .mut_index(idx.clone())
                            .map_err(RuntimeErrorType::Value)?;
                    }
                    base.set_index(last_index.clone(), value)
                        .map_err(RuntimeErrorType::Value)?;
                }
                None => match slotref.get_mut(mem) {
                    Some(slot) => match slot {
                        ValueSource::Literal(v) => *v = value,
                        ValueSource::Reference(r) => r.clone().set(value, mem)?,
                    },
                    None => return Err(RuntimeErrorType::SlotRefInvalid),
                },
            },
        }

        Ok(())
    }

    /// Attempt to resolve the reference
    pub fn into_resolved(self, mem: &MemoryManager) -> Result<Self, RuntimeErrorType> {
        match self {
            Self::Unresolved(name_hash) => match mem.get_ref(name_hash) {
                Some(slot) => Ok(Self::Resolved(slot, vec![])),
                None => Err(RuntimeErrorType::HashNotFound),
            },
            _ => Ok(self),
        }
    }

    /// A very bad idea to do a very bad thing
    fn resolve<'mem>(
        slot: &SlotRef,
        mem: &'mem mut MemoryManager,
    ) -> Result<ValueIndexResult<'mem>, RuntimeErrorType> {
        let _mem: *mut MemoryManager = mem;
        let _mem = unsafe { &mut *_mem };
        let target = slot
            .get_mut(_mem)
            .ok_or_else(|| RuntimeErrorType::SlotRefInvalid)?;
        if let ValueSource::Literal(v) = target {
            Ok(ValueIndexResult::Mutable(v))
        } else {
            let target = target.clone();
            match target {
                ValueSource::Reference(r) => r.value_mut(mem),
                _ => unreachable!(),
            }
        }
    }
}
