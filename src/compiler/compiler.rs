use super::debug_profile::DebugProfile;
use crate::{lexer::Token, traits::SerializeToBytes, value::ValueType, vm::OpCode};
use std::ops::Range;

/// Options for the compiler
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Whether to include debug information in the bytecode
    pub debug: bool,

    /// Whether to allow syscalld calls
    pub allow_syscalld: bool,
}
impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            debug: true,
            allow_syscalld: false,
        }
    }
}

/// Compiles source code into bytecode
/// You don't need to use this directly, use the `compile` function on a `Node` instead
#[derive(Debug, Clone)]
pub struct Compiler {
    bytecode: Vec<u8>,
    loop_stack: Vec<(usize, Vec<Range<usize>>)>, // (start, break targets)
    debug: DebugProfile,
    options: CompilerOptions,
}

impl Compiler {
    /// Create a new compiler
    pub fn new(input: &str, options: CompilerOptions) -> Self {
        Self {
            bytecode: Vec::new(),
            loop_stack: Vec::new(),
            debug: DebugProfile::new(input),
            options,
        }
    }

    /// Get the compiler options
    pub fn options(&self) -> &CompilerOptions {
        &self.options
    }

    /// Push a token to the debug profile
    /// This is used to map bytecode instructions to source code
    /// during error reporting
    pub fn push_token(&mut self, token: Token<'_>) {
        if self.options.debug {
            self.debug.insert(self.bytecode.len(), token);
        }
    }

    /// Push an instruction to the bytecode
    /// Returns the index of the instruction
    pub fn push(&mut self, op: OpCode) -> usize {
        self.bytecode.push(op as u8);
        self.bytecode.len() - 1
    }

    /// Push a type to the bytecode
    /// Used as part of some instructions
    /// Returns the index of the value
    pub fn push_type(&mut self, typecode: ValueType) -> usize {
        self.bytecode.push(typecode as u8);
        self.bytecode.len() - 1
    }

    /// Push a value to the bytecode
    /// Returns the index of the value
    pub fn push_u8(&mut self, value: u8) -> usize {
        self.bytecode.push(value);
        self.bytecode.len() - 1
    }

    /// Push a value to the bytecode
    /// Returns the index of the value
    pub fn push_i32(&mut self, value: i32) -> Range<usize> {
        self.bytecode.extend(&value.serialize_into_bytes());
        self.bytecode.len() - 4..self.bytecode.len()
    }

    /// Push a value to the bytecode
    /// Returns the index of the value
    pub fn push_u64(&mut self, value: u64) -> Range<usize> {
        self.bytecode.extend(&value.serialize_into_bytes());
        self.bytecode.len() - 8..self.bytecode.len()
    }

    /// Push the 64bit hash of a string to the bytecode
    /// Used for memory access
    /// Returns the index of the value
    pub fn push_strhash(&mut self, input: &str) -> Range<usize> {
        let hash = input.hash_str();
        self.push_u64(hash)
    }

    /// Push a block of bytes to the bytecode
    /// Returns the index of the value
    pub fn extend(&mut self, other: Vec<u8>) -> Range<usize> {
        let pos = self.bytecode.len();
        self.bytecode.extend(other);
        pos..self.bytecode.len()
    }

    /// Returns the length of the bytecode
    pub fn len(&self) -> usize {
        self.bytecode.len()
    }

    /// Replace a range of the bytecode with a new value
    pub fn replace(&mut self, range: std::ops::Range<usize>, value: Vec<u8>) {
        self.bytecode.splice(range, value);
    }

    /// Get a reference to the bytecode
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Consume the compiler and return the bytecode
    pub fn into_bytecode(self) -> Vec<u8> {
        self.bytecode
    }

    /// Decompose the compiler into its components
    pub fn decompose(self) -> (DebugProfile, Vec<u8>) {
        (self.debug, self.bytecode)
    }
}

/// Extensions for the compiler to handle loops properly
pub trait LoopCompilationExt {
    /// Start a new loop
    fn start_loop(&mut self);

    /// End the current loop
    fn end_loop(&mut self);

    /// Push a break instruction
    fn push_break(&mut self);

    /// Push a continue instruction
    fn push_continue(&mut self);
}

impl LoopCompilationExt for Compiler {
    fn start_loop(&mut self) {
        self.loop_stack.push((self.bytecode.len(), Vec::new()));
    }

    fn end_loop(&mut self) {
        let (_, breaks) = self.loop_stack.pop().unwrap();
        for target in breaks {
            let pos = self.bytecode.len();
            self.replace(target, pos.serialize_into_bytes());
        }
    }

    fn push_break(&mut self) {
        self.push(OpCode::JMP);
        let target = self.push_u64(0);

        let (_, breaks) = self.loop_stack.last_mut().unwrap();
        breaks.push(target);
    }

    fn push_continue(&mut self) {
        self.push(OpCode::JMP);
        let start = self.loop_stack.last().unwrap().0;
        self.push_u64(start as u64);
    }
}

/// Hash a string to a u64
pub trait HashString {
    /// Hash a string to a u64
    fn hash_str(&self) -> u64;
}

impl HashString for str {
    fn hash_str(&self) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(self.as_bytes());
        hasher.finish()
    }
}
