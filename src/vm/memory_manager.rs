//! Memory manager for storing variables and their values
//! Also provides scoping, referencing and functions

use core::panic;

use super::{load_stdlib, value_source::ValueSource};
use crate::value::{Function, Value};

mod slot;
pub use slot::Slot;

mod slot_ref;
pub use slot_ref::SlotRef;

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
        for frame in self.frame_ptr.iter() {
            self.stack.truncate(*frame);
        }

        self.locks.clear();
        self.frame_ptr.clear();
    }

    /// Lock the stack at the current depth, hiding values from higher scopes
    pub fn scope_lock(&mut self) {
        self.locks.push(self.stack.len());
    }

    /// Create a new stack frame starting at the current depth
    pub fn scope_in(&mut self) {
        self.frame_ptr.push(self.stack.len());
    }

    /// Scope out until we remove the next lock, or the global scope
    pub fn scope_out_to_lock(&mut self) {
        let n_locks = self.locks.len();
        if n_locks > 0 {
            while self.locks.len() == n_locks {
                self.scope_out();
            }
        }
    }

    /// End the current stack frame, removing all values added since the frame was created
    /// If the frame was locked, the lock will be removed
    pub fn scope_out(&mut self) {
        let frame_ptr = match self.frame_ptr.pop() {
            Some(ptr) => ptr,
            None => panic!("Attempted to scope out of global scope"),
        };

        // Pop the top-most blank stack entry in the dying frame
        // And remove the top-most frame
        let mut return_value = None;
        while self.stack.len() > frame_ptr {
            let next = self.stack.pop().unwrap(); // Safe to unwrap, as we know the stack is not empty
                                                  // Since len() > frame_ptr, even at 0 the stack has 1 element here

            if next.version() == 0 && return_value.is_none() {
                return_value = Some(next);
            }
        }

        // Remove a lock if the stack is now shorter than the lock
        if self.stack.len() <= self.last_valid_scope() {
            self.locks.pop();
        }

        // Pop the return value back onto the stack
        if let Some(value) = return_value {
            self.stack.push(value);
        }
    }

    /// Collects all working stack entries from the stack
    /// These are entries with a version of 0, and will be removed from the stack
    pub fn all_stack_blanks(&mut self) -> Vec<ValueSource> {
        let mut out = vec![];
        for i in 0..self.stack.len() {
            if self.stack[i].version() == 0 {
                let slot = self.stack.remove(i);
                if let Slot::Occupied { value, .. } = slot {
                    out.push(value);
                }
            }
        }
        out
    }

    /// Peek at the top-most blank stack entry
    pub fn peek_blank(&self) -> Option<&ValueSource> {
        for i in (0..self.stack.len()).rev() {
            if self.stack[i].version() == 0 {
                return self.stack[i].as_value();
            }
        }

        None
    }

    /// Pop the top-most blank stack entry
    pub fn pop_blank(&mut self) -> Option<ValueSource> {
        for i in (0..self.stack.len()).rev() {
            if self.stack[i].version() == 0 {
                let value = self.stack.remove(i).take();
                return value;
            }
        }

        None
    }

    /// Push a blank stack entry
    pub fn push_blank(&mut self, value: ValueSource) {
        self.stack.push(Slot::new_blank(value));
    }

    /// Get a reference to all the functions in the memory manager
    /// Only the global scope is checked for functions
    pub fn all_functions(&self) -> Vec<&Function> {
        self.globals
            .iter()
            .filter_map(|slot| match slot {
                Slot::Occupied {
                    value: ValueSource::Literal(Value::Function(func)),
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
    pub fn write_global(&mut self, name_hash: u64, value: ValueSource, write_locked: bool) {
        for slot in self.globals.iter_mut().rev() {
            if slot.check_name(name_hash) {
                slot.put(value);
                return;
            }
        }

        self.globals
            .push(Slot::new_occupied(name_hash, value, write_locked));
    }

    /// Write a value to the stack
    /// Returns a reference to the value
    pub fn write(&mut self, name_hash: u64, value: ValueSource) -> SlotRef {
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

        let slot = Slot::new_occupied(name_hash, value, false);
        let version = slot.version();
        self.stack.push(slot);
        SlotRef::Stack {
            i: self.stack.len() - 1,
            name_hash,
            version,
        }
    }

    /// Get a reference to a value in the memory manager
    pub fn read(&self, name_hash: u64) -> Option<&ValueSource> {
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
    pub fn read_mut(&mut self, name_hash: u64) -> Option<&mut ValueSource> {
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
    pub fn delete(&mut self, name_hash: u64) -> Option<ValueSource> {
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
    pub fn get_ref(&self, name_hash: u64) -> Option<SlotRef> {
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

    fn is_locked(&self, i: usize) -> bool {
        self.locks.iter().any(|lock| *lock == i)
    }

    fn is_frame_boundary(&self, i: usize) -> bool {
        self.frame_ptr.iter().any(|ptr| *ptr == i)
    }
}

impl std::fmt::Display for MemoryManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[Globals]")?;
        for (i, slot) in self.globals.iter().enumerate() {
            if slot.write_locked() {
                continue;
            }

            writeln!(f, "{i:08X}   {slot}")?;
        }

        writeln!(f, "\n[Stack]")?;
        for (i, slot) in self.stack.iter().enumerate() {
            if self.is_frame_boundary(i) {
                writeln!(f, "{i:08X} --- FRAME BOUNDARY ---")?;
                if self.is_locked(i) {
                    writeln!(f, "{i:08X} --- SCOPE LOCKED ---")?;
                }
            }
            writeln!(f, "{i:08X}   {slot}")?;
        }

        if self.is_frame_boundary(self.stack.len()) {
            writeln!(f, "{:08X} --- FRAME BOUNDARY ---", self.stack.len())?;
            if self.is_locked(self.stack.len()) {
                writeln!(f, "{:08X} --- SCOPE LOCKED ---", self.stack.len())?;
            }
        }

        Ok(())
    }
}
