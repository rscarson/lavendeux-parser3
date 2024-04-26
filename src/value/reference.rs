use super::{Value, ValueError};
use crate::vm::memory_manager::{MemoryManager, SlotRef};

/// A reference to a value in memory
/// Can have a path of indexes to traverse to get to the value
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Reference {
    /// A reference that has not been resolved yet
    /// Or does not exist in memory
    Unresolved(u64),

    /// A reference that has been resolved
    /// Contains the SlotRef to the value
    /// And a path of indexes to traverse to get to the value
    Resolved(SlotRef, Vec<Value>),
}

impl Reference {
    /// Resolve unresolved references to their SlotRef
    pub fn resolve(&mut self, mem: &mut MemoryManager) -> Result<(), ValueError> {
        match self {
            Reference::Unresolved(name_hash) => match mem.get_ref(*name_hash) {
                Some(value) => {
                    *self = Reference::Resolved(value, vec![]);
                }
                None => {
                    return Err(ValueError::HashNotFound);
                }
            },
            Reference::Resolved(_, _) => {}
        }
        Ok(())
    }

    /// Add an index to the path of indexes to traverse to get to the value
    /// If the reference is not resolved, this will fail
    pub fn add_index(&mut self, index: Value) -> Result<(), ValueError> {
        match self {
            Reference::Unresolved(_) => {
                return Err(ValueError::ReferenceNotResolved("add_index".to_string()))
            }
            Reference::Resolved(_, idx_path) => idx_path.push(index),
        }
        Ok(())
    }

    /// Get the hash of the reference
    pub fn hash(&self) -> u64 {
        match self {
            Reference::Unresolved(hash) => *hash,
            Reference::Resolved(slotref, _) => slotref.name_hash(),
        }
    }

    /// Get the value that this reference points to
    /// If the reference is not resolved, this will fail
    pub fn value<'mem>(&self, mem: &'mem mut MemoryManager) -> Result<&'mem Value, ValueError> {
        match self {
            Reference::Resolved(slotref, idxpath) => {
                let mut value = slotref.get(mem).ok_or(ValueError::SlotRefInvalid)?;
                for idx in idxpath {
                    value = value.get_index(idx.clone())?;
                }
                Ok(value)
            }

            Reference::Unresolved(_) => Err(ValueError::ReferenceNotResolved("value".to_string())),
        }
    }

    /// Get the value that this reference points to
    /// If the reference is not resolved, this will fail
    pub fn value_mut<'mem>(
        &mut self,
        mem: &'mem mut MemoryManager,
    ) -> Result<&'mem mut Value, ValueError> {
        match self {
            Reference::Resolved(slotref, idxpath) => {
                let mut value = slotref.get_mut(mem).ok_or(ValueError::SlotRefInvalid)?;
                for idx in idxpath {
                    value = value.mut_index(idx)?;
                }
                Ok(value)
            }

            Reference::Unresolved(_) => {
                Err(ValueError::ReferenceNotResolved("value_mut".to_string()))
            }
        }
    }

    /// Write a value to the reference
    pub fn write(&mut self, mem: &mut MemoryManager, value: Value) -> Result<(), ValueError> {
        self.resolve(mem).ok(); // Try to resolve the reference, but ignore the result
        match self {
            Reference::Resolved(slotref, idx_path) => {
                match idx_path.len() {
                    0 => slotref.set(mem, value),
                    _ => {
                        let target = idx_path.last().unwrap();
                        let path = &idx_path[..idx_path.len() - 1];
                        let mut base = slotref.get_mut(mem).ok_or(ValueError::SlotRefInvalid)?;
                        for idx in path {
                            base = base.mut_index(&idx)?;
                        }
                        base.set_index(target.clone(), value)?;
                    }
                }
                Ok(())
            }

            Reference::Unresolved(name_hash) => {
                let slot = mem.write(*name_hash, value);
                *self = Reference::Resolved(slot, vec![]);
                Ok(())
            }
        }
    }

    /// Delete the value that this reference points to
    /// If the reference is not resolved, this will fail
    pub fn delete(self, mem: &mut MemoryManager) -> Result<Value, ValueError> {
        match self {
            Reference::Resolved(slotref, mut idx_path) => match idx_path.is_empty() {
                true => slotref.delete(mem).ok_or(ValueError::SlotRefInvalid),
                false => {
                    let target = idx_path.pop().unwrap();
                    let mut base = slotref.get_mut(mem).ok_or(ValueError::SlotRefInvalid)?;
                    for mut idx in idx_path {
                        base = base.mut_index(&mut idx)?;
                    }
                    base.delete_index(target)
                }
            },

            Reference::Unresolved(_) => Err(ValueError::ReferenceNotResolved("delete".to_string())),
        }
    }
}
