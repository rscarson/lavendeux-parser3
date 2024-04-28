//! Memory manager for storing variables and their values
//! Also provides scoping, referencing and functions

use super::load_stdlib;
use crate::value::{Function, Value};

/// A memory manager for storing variables and their values
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryManager {
    globals: Vec<Slot>,
    stack: Vec<Slot>,
    locks: Vec<usize>,
    frame_ptr: Vec<usize>,
}
impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            globals: Vec::new(),
            stack: Vec::new(),
            locks: Vec::new(),
            frame_ptr: Vec::new(),
        }
    }

    /// Create a new memory manager with the same global scope as this one
    pub fn create_child(&self) -> Self {
        Self {
            globals: self.globals.clone(),
            stack: Vec::new(),
            locks: Vec::new(),
            frame_ptr: Vec::new(),
        }
    }

    /// Merge the child memory manager into this one, keeping the global scope
    /// of the child
    pub fn eat_child(&mut self, child: Self) {
        self.globals = child.globals;
    }

    /// Load the standard library into this memory manager
    pub fn load_stdlib(&mut self) {
        load_stdlib::load_stdlib(self);
    }

    /// Reset the memory manager, clearing all variables but keeping the global scope
    pub fn reset(&mut self) {
        self.locks.clear();
        self.frame_ptr.clear();
        self.stack.clear();
    }

    /// Lock the stack at the current depth, hiding values from higher scopes
    pub fn scope_lock(&mut self) {
        self.locks
            .push(self.stack.len().checked_sub(1).unwrap_or(0));
    }

    /// Create a new stack frame starting at the current depth
    pub fn scope_in(&mut self) {
        self.frame_ptr.push(self.stack.len());
    }

    /// End the current stack frame, removing all values added since the frame was created
    /// If the frame was locked, the lock will be removed
    pub fn scope_out(&mut self) {
        match self.frame_ptr.pop() {
            Some(ptr) => self.stack.truncate(ptr),
            None => panic!("Attempted to scope out of global scope"),
        }

        while self.stack.len() < self.last_valid_scope() {
            self.locks.pop();
        }
    }

    /// Get a reference to all the functions in the memory manager
    /// Only the global scope is checked for functions
    pub fn all_functions(&self) -> Vec<&Function> {
        self.globals
            .iter()
            .filter_map(|slot| match slot {
                Slot::Occupied {
                    value: Value::Function(func),
                    ..
                } => Some(func),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    /// Get a reference to all the global variables
    pub fn all_globals(&self) -> &[Slot] {
        &self.globals
    }

    /// Write a value to the global scope
    pub fn write_global(&mut self, name_hash: u64, value: Value, readonly: bool) {
        for slot in self.globals.iter_mut().rev() {
            if slot.check_name(name_hash) {
                slot.put(value);
                return;
            }
        }

        self.globals.push(Slot::Occupied {
            write_locked: readonly,
            name_hash,
            version: 0,
            value,
        });
    }

    /// Write a value to the stack
    /// Returns a reference to the value
    pub fn write(&mut self, name_hash: u64, value: Value) -> SlotRef {
        let ref_offset = self.last_valid_scope();
        for (i, slot) in self.valid_stack_slice_mut().iter_mut().rev().enumerate() {
            if slot.check_name(name_hash) {
                slot.put(value);
                return SlotRef::Stack {
                    i: ref_offset + i,
                    name_hash,
                    version: slot.version(),
                };
            }
        }

        self.stack.push(Slot::Occupied {
            write_locked: false,
            name_hash,
            version: 0,
            value,
        });

        SlotRef::Stack {
            i: self.stack.len() - 1,
            name_hash,
            version: 0,
        }
    }

    /// Get a reference to a value in the memory manager
    pub fn read(&self, name_hash: u64) -> Option<&Value> {
        // Check main stack
        for slot in self.valid_stack_slice().iter().rev() {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.as_value() {
                return Some(value);
            }
        }

        // Check global stack
        for slot in self.globals.iter().rev() {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.as_value() {
                return Some(value);
            }
        }

        None
    }

    /// Get a mutable reference to a value in the memory manager
    pub fn read_mut(&mut self, name_hash: u64) -> Option<&mut Value> {
        let _self: *mut Self = self;

        // Check main stack
        for slot in unsafe { &mut *_self }
            .valid_stack_slice_mut()
            .iter_mut()
            .rev()
        {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.as_value_mut() {
                return Some(value);
            }
        }

        // Check global stack
        for slot in self.globals.iter_mut().rev() {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.as_value_mut() {
                return Some(value);
            }
        }

        None
    }

    /// Delete a value from the memory manager, returning the value
    pub fn delete(&mut self, name_hash: u64) -> Option<Value> {
        // Check main stack
        for slot in self.valid_stack_slice_mut().iter_mut() {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.take() {
                return Some(value);
            }
        }

        // Check global stack
        for slot in self.globals.iter_mut() {
            if !slot.check_name(name_hash) {
                continue;
            } else if let Some(value) = slot.take() {
                return Some(value);
            }
        }

        None
    }

    /// Get a reference to a value in the memory manager
    pub fn get_ref(&mut self, name_hash: u64) -> Option<SlotRef> {
        // Check main stack
        for (i, slot) in self.valid_stack_slice().iter().enumerate().rev() {
            match slot {
                Slot::Occupied {
                    name_hash: slot_hash,
                    version,
                    ..
                } if name_hash == *slot_hash => {
                    return Some(SlotRef::Stack {
                        i: self.last_valid_scope() + i,
                        name_hash,
                        version: *version,
                    })
                }
                _ => {}
            }
        }

        // Check global stack
        for (i, slot) in self.globals.iter().enumerate().rev() {
            match slot {
                Slot::Occupied {
                    name_hash: slot_hash,
                    version,
                    ..
                } if name_hash == *slot_hash => {
                    return Some(SlotRef::Global {
                        i,
                        name_hash,
                        version: *version,
                    })
                }
                _ => {}
            }
        }

        None
    }

    fn valid_stack_slice(&self) -> &[Slot] {
        &self.stack[self.last_valid_scope()..]
    }

    fn valid_stack_slice_mut(&mut self) -> &mut [Slot] {
        let ptr = self.last_valid_scope();
        &mut self.stack[ptr..]
    }

    fn last_valid_scope(&self) -> usize {
        for lock in self.locks.iter().rev() {
            if *lock <= self.stack.len() {
                return *lock;
            }
        }
        0
    }
}

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
    pub fn get<'m>(&self, memory: &'m MemoryManager) -> Option<&'m Value> {
        match self.get_ref(memory) {
            Some(slot) => slot.as_value(),
            None => None,
        }
    }

    /// Get a mutable reference to the value this SlotRef points to
    pub fn get_mut<'m>(&self, memory: &'m mut MemoryManager) -> Option<&'m mut Value> {
        match self.get_mutref(memory) {
            Some(slot) => slot.as_value_mut(),
            None => None,
        }
    }

    /// Set the value this SlotRef points to
    pub fn set(&self, memory: &mut MemoryManager, value: Value) {
        if let Some(slot) = self.get_mutref(memory) {
            slot.put(value);
        }
    }

    /// Delete the value this SlotRef points to, returning the value
    pub fn delete(&self, memory: &mut MemoryManager) -> Option<Value> {
        match self.get_mutref(memory) {
            Some(slot) => slot.take(),
            None => None,
        }
    }
}

/// A slot in the memory manager
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    /// An empty slot
    Vacant {
        /// The version number of the slot
        version: u32,
    },

    /// A slot containing a value
    Occupied {
        /// The name hash of the value in the slot
        name_hash: u64,

        /// Whether the slot is write-locked
        write_locked: bool,

        /// The version number of the slot
        version: u32,

        /// The value in the slot
        value: Value,
    },
}
impl Slot {
    /// Get the version number of the slot
    pub fn version(&self) -> u32 {
        match self {
            Slot::Occupied { version, .. } => *version,
            Slot::Vacant { version } => *version,
        }
    }

    /// Check the version number of the slot
    /// If the slot is vacant, the reference being checked is invalid
    /// so this will return false
    pub fn check_version(&self, version: u32) -> bool {
        match self {
            Slot::Occupied {
                version: slot_version,
                ..
            } => *slot_version == version,
            Slot::Vacant { .. } => false,
        }
    }

    /// Check if the name hash of the slot matches the given name hash
    /// If the slot is vacant, the reference being checked is invalid
    /// so this will return false
    pub fn check_name(&self, name_hash: u64) -> bool {
        match self {
            Slot::Occupied {
                name_hash: slot_hash,
                ..
            } => *slot_hash == name_hash,
            Slot::Vacant { .. } => false,
        }
    }

    /// Convert the Slot into a Value, consuming the Slot
    /// This is a delete operation, and will increment the version number
    pub fn take(&mut self) -> Option<Value> {
        match self {
            Slot::Occupied {
                version,
                write_locked,
                ..
            } => {
                if *write_locked {
                    return None;
                }
                let mut s = Slot::Vacant {
                    version: version.wrapping_add(1),
                };
                std::mem::swap(self, &mut s);
                if let Slot::Occupied { value, .. } = s {
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Replace the value in the slot, if the slot is occupied
    pub fn put(&mut self, value: Value) {
        match self {
            Slot::Occupied {
                value: slot_value,
                write_locked,
                ..
            } => {
                if !*write_locked {
                    *slot_value = value;
                }
            }

            _ => {}
        }
    }

    /// Get a reference to the value in the slot
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            Slot::Occupied { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Get a mutable reference to the value in the slot
    pub fn as_value_mut(&mut self) -> Option<&mut Value> {
        match self {
            Slot::Occupied { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Lock the slot, preventing writes
    pub fn lock(&mut self) {
        match self {
            Slot::Occupied { write_locked, .. } => *write_locked = true,
            _ => {}
        }
    }
}
