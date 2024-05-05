use crate::vm::value_source::ValueSource;

use super::{MemoryManager, Slot};

/// A reference to a value in the memory manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotRef {
    /// A reference to a value on the stack
    Stack {
        /// The index of the value in the stack
        i: usize,

        /// The expected name hash of the value
        name_hash: u64,

        /// The expected version number of the slot
        version: u32,
    },

    /// A reference to a value in the global scope
    Global {
        /// The index of the value in the stack
        i: usize,

        /// The expected name hash of the value
        name_hash: u64,

        /// The expected version number of the slot
        version: u32,
    },
}
impl SlotRef {
    fn get_ref<'m>(&self, memory: &'m MemoryManager) -> Option<&'m Slot> {
        match self {
            SlotRef::Stack {
                i,
                name_hash,
                version,
                ..
            } => match memory.stack.get(*i) {
                Some(slot) if slot.check_version(*version) && slot.check_name(*name_hash) => {
                    Some(slot)
                }
                _ => None,
            },

            SlotRef::Global {
                i,
                name_hash,
                version,
                ..
            } => match memory.globals.get(*i) {
                Some(slot) if slot.check_version(*version) && slot.check_name(*name_hash) => {
                    Some(slot)
                }
                _ => None,
            },
        }
    }

    fn get_mutref<'m>(&self, memory: &'m mut MemoryManager) -> Option<&'m mut Slot> {
        match self {
            SlotRef::Stack {
                i,
                name_hash,
                version,
                ..
            } => match memory.stack.get_mut(*i) {
                Some(slot) if slot.check_version(*version) && slot.check_name(*name_hash) => {
                    Some(slot)
                }
                _ => None,
            },

            SlotRef::Global {
                i,
                name_hash,
                version,
                ..
            } => match memory.globals.get_mut(*i) {
                Some(slot) if slot.check_version(*version) && slot.check_name(*name_hash) => {
                    Some(slot)
                }
                _ => None,
            },
        }
    }

    /// Get the name hash of the value this SlotRef points to
    pub fn name_hash(&self) -> u64 {
        match self {
            SlotRef::Stack { name_hash, .. } => *name_hash,
            SlotRef::Global { name_hash, .. } => *name_hash,
        }
    }

    /// Get the version number of the value this SlotRef points to
    pub fn version(&self) -> u32 {
        match self {
            SlotRef::Stack { version, .. } => *version,
            SlotRef::Global { version, .. } => *version,
        }
    }

    /// Get a reference to the value this SlotRef points to
    pub fn get<'m>(&self, memory: &'m MemoryManager) -> Option<&'m ValueSource> {
        match self.get_ref(memory) {
            Some(slot) => slot.as_value(),
            None => None,
        }
    }

    /// Get a mutable reference to the value this SlotRef points to
    pub fn get_mut<'m>(&self, memory: &'m mut MemoryManager) -> Option<&'m mut ValueSource> {
        match self.get_mutref(memory) {
            Some(slot) => slot.as_value_mut(),
            None => None,
        }
    }

    /// Set the value this SlotRef points to
    pub fn set(&self, memory: &mut MemoryManager, value: ValueSource) {
        if let Some(slot) = self.get_mutref(memory) {
            slot.put(value);
        }
    }

    /// Delete the value this SlotRef points to, returning the value
    pub fn delete(&self, memory: &mut MemoryManager) -> Option<ValueSource> {
        match self.get_mutref(memory) {
            Some(slot) => slot.take(),
            None => None,
        }
    }
}
