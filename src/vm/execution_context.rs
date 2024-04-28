use super::{
    error::{RuntimeError, RuntimeErrorType},
    memory_manager::MemoryManager,
    opcodes::OpCode,
};
use crate::{
    compiler::DebugProfile,
    value::{CheckedArithmetic, CheckedBitwise, CheckedBoolean, CheckedMatching, Value, ValueType},
};

// syscall helpers
mod math;

mod alu;
mod collections;
mod control;
mod functions;
mod index;
mod io;
mod references;
mod stack;

use alu::ALUExt;
use collections::CollectionExt;
use control::ControlExt;
use functions::FunctionExt;
use index::IndexExt;
use io::IOExt;
use references::RefExt;
use stack::StackExt;

/// The execution context for the Lavendeux VM.
/// This is the actual VM that runs the bytecode.
/// In practice you should access this through the `Lavendeux` struct.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    stack: Vec<Value>,
    mem: MemoryManager,
    last_opcode: OpCode,

    context: Vec<(
        Vec<u8>,              // code
        usize,                // pc
        Option<DebugProfile>, // debug profile
        ValueType,            // return type
    )>,
}

impl ExecutionContext {
    /// Creates a new execution context with the given bytecode and debug profile.
    /// Uses the specified memory manager.
    pub fn with_mem(
        code: Vec<u8>,
        debug_profile: Option<DebugProfile>,
        mem: MemoryManager,
    ) -> Self {
        ExecutionContext {
            stack: Vec::new(),
            mem,
            last_opcode: OpCode::NOP,
            context: vec![(code, 0, debug_profile, ValueType::All)],
        }
    }

    /// Creates a new execution context with the given bytecode and debug profile.
    /// Uses a new memory manager.
    pub fn new(code: Vec<u8>, debug_profile: Option<DebugProfile>) -> Self {
        Self::with_mem(code, debug_profile, MemoryManager::new())
    }

    /// Move the current context's PC to the specified value.
    pub fn set_pc(&mut self, pc: usize) {
        self.context.last_mut().unwrap().1 = pc;
    }

    /// Read the current context's PC
    pub fn pc(&self) -> usize {
        self.context.last().unwrap().1
    }

    /// Read the current context's bytecode
    pub fn code(&self) -> &[u8] {
        &self.context.last().unwrap().0
    }

    /// Read the current context's debug profile
    pub fn debug_profile(&self) -> Option<&DebugProfile> {
        self.context.last().unwrap().2.as_ref()
    }

    /// Read the current context's return type
    /// Panics if the context does not have a return type.
    fn return_type(&self) -> ValueType {
        self.context.last().unwrap().3
    }

    /// Add a new function context to the stack.
    fn push_context(
        &mut self,
        code: Vec<u8>,
        debug_profile: Option<DebugProfile>,
        ret_type: ValueType,
    ) {
        self.context.push((code, 0, debug_profile, ret_type));
    }

    /// Remove the top function context from the stack.
    fn pop_context(&mut self) {
        self.context.pop();
    }

    /// Emit an error at the current position
    fn emit_err(&self, error: RuntimeErrorType) -> RuntimeError {
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

    /// Run the program until it finishes.
    /// Returns the value, or values (1 per line)
    pub fn run(&mut self) -> Result<Value, RuntimeError> {
        while self.pc() < self.code().len() {
            self.next()?;
        }

        // The stack now contains the result of the program.
        let v1: Vec<Value> = self.stack.drain(..).collect();
        let mut v2 = vec![];
        for value in v1.into_iter() {
            let value = self.resolve_reference(value)?;
            v2.push(value)
        }
        match v2.len() {
            1 => Ok(v2.into_iter().next().unwrap()),
            _ => Ok(Value::Array(v2)),
        }
    }

    /// Consumes the execution context and returns the memory manager.
    pub fn destroy(self) -> MemoryManager {
        self.mem
    }

    /// Run the next instruction in the current context.
    pub fn next(&mut self) -> Result<(), RuntimeError> {
        let opcode = self.read_opcode()?;

        #[cfg(feature = "debug_compiler_internal")]
        {
            print!("{:?} ", opcode);
            println!("stack={:?}", self.stack);
        }

        self.last_opcode = opcode;
        match opcode {
            ////////////////////////
            // Stack manipulation //
            ////////////////////////
            OpCode::PUSH => self.op_push()?,
            OpCode::POP => self.op_pop()?,
            OpCode::DUP => self.dup()?,
            OpCode::SWP => self.swap()?,

            //////////////////
            // Control flow //
            //////////////////
            OpCode::JMP => self.jump()?,
            OpCode::JMPT => self.jump_if_true()?,
            OpCode::JMPF => self.jump_if_false()?,
            OpCode::JMPE => self.jump_if_empty()?,
            OpCode::JMPNE => self.jump_if_not_empty()?,

            /////////////////////////
            // Memory manipulation //
            /////////////////////////
            OpCode::REF => self.read_reference()?,
            OpCode::RREF => self.consume_reference()?,
            OpCode::WREF => self.write_reference()?,
            OpCode::DREF => self.delete_reference()?,

            ////////////////////////
            // Scope manipulation //
            ////////////////////////
            OpCode::SCI => self.mem.scope_in(),
            OpCode::SCO => self.mem.scope_out(),
            OpCode::SCL => self.mem.scope_lock(),

            ////////////////////////
            // Value manipulation //
            ////////////////////////
            OpCode::CAST => {
                let type_name = self.read_type()?;
                let value = self.pop()?;
                let value = self.resolve_reference(value)?;
                let value = value
                    .cast(type_name)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                self.push(value);
            }
            OpCode::NEXT => {
                let value = self.pop()?;
                let value = self.resolve_reference(value)?;
                let (first, rest) = match value {
                    Value::Primitive(p) => (Value::Primitive(p), Value::Array(vec![])),
                    Value::Function(f) => (Value::Function(f), Value::Array(vec![])),
                    Value::Reference(_) => unreachable!(),

                    Value::Array(mut array) => {
                        if array.is_empty() {
                            return Err(self.emit_err(RuntimeErrorType::IteratorEmpty));
                        } else {
                            let first = array.remove(0);
                            (first, Value::Array(array))
                        }
                    }
                    Value::Object(obj) => {
                        if obj.is_empty() {
                            return Err(self.emit_err(RuntimeErrorType::IteratorEmpty));
                        } else {
                            let mut iter = obj.into_iter();
                            let (key, _) = iter.next().unwrap();
                            let obj = iter.collect();
                            (Value::Primitive(key), Value::Object(obj))
                        }
                    }
                    Value::Range(range) => {
                        if range.is_empty() {
                            return Err(self.emit_err(RuntimeErrorType::IteratorEmpty));
                        } else {
                            let first = Value::integer(range.start);
                            let rest = Value::Range(range.start + 1..range.end);
                            (first, rest)
                        }
                    }
                };

                self.push(rest);
                self.push(first);
            }
            OpCode::LCST => {
                let right = self.pop()?;
                let left = self.pop()?;

                let lefttype = left
                    .type_of(Some(&mut self.mem))
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                let right = right
                    .cast(lefttype)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                self.push(left);
                self.push(right);
            }

            OpCode::MKAR => self.make_array()?,
            OpCode::MKOB => self.make_object()?,
            OpCode::MKRG => self.make_range()?,

            OpCode::PSAR => self.push_array()?,
            OpCode::PSOB => self.push_object()?,

            OpCode::IDEX => self.index_into()?,

            ////////////////////
            // Arithmetic ops //
            ////////////////////
            OpCode::ADD => self.op_binary(Value::checked_add)?,
            OpCode::SUB => self.op_binary(Value::checked_sub)?,
            OpCode::MUL => self.op_binary(Value::checked_mul)?,
            OpCode::DIV => self.op_binary(Value::checked_div)?,
            OpCode::REM => self.op_binary(Value::checked_rem)?,
            OpCode::POW => self.op_binary(Value::checked_pow)?,
            OpCode::NEG => self.op_unary(Value::checked_neg)?,

            /////////////////
            // Bitwise ops //
            /////////////////
            OpCode::AND => self.op_binary(Value::checked_and)?,
            OpCode::OR => self.op_binary(Value::checked_or)?,
            OpCode::XOR => self.op_binary(Value::checked_xor)?,
            OpCode::SHL => self.op_binary(Value::checked_shl)?,
            OpCode::SHR => self.op_binary(Value::checked_shr)?,
            OpCode::NOT => self.op_unary(Value::checked_not)?,

            ////////////////////
            // Comparison ops //
            ////////////////////
            OpCode::EQ => self.op_binary(Value::checked_eq)?,
            OpCode::NE => self.op_binary(Value::checked_ne)?,
            OpCode::SEQ => self.op_binary(Value::checked_seq)?,
            OpCode::SNE => self.op_binary(Value::checked_sne)?,
            OpCode::LT => self.op_binary(Value::checked_lt)?,
            OpCode::LE => self.op_binary(Value::checked_le)?,
            OpCode::GT => self.op_binary(Value::checked_gt)?,
            OpCode::GE => self.op_binary(Value::checked_ge)?,

            /////////////////
            // Logical ops //
            /////////////////
            OpCode::LAND => self.op_binary(Value::checked_logical_and)?,
            OpCode::LOR => self.op_binary(Value::checked_logical_or)?,
            OpCode::LNOT => self.op_unary(Value::checked_logical_not)?,

            //////////////////
            // Matching ops //
            //////////////////
            OpCode::MTCH => self.op_binary(Value::checked_matches)?,
            OpCode::CNTN => self.op_binary(Value::checked_contains)?,
            OpCode::STWT => self.op_binary(Value::checked_starts_with)?,
            OpCode::EDWT => self.op_binary(Value::checked_ends_with)?,

            //////////////////
            // Function ops //
            //////////////////
            OpCode::WRFN => self.alloc_fn()?,
            OpCode::MKFN => self.make_fn()?,
            OpCode::FDFT => self.push_default_fn_arg()?,
            OpCode::FSIG => self.push_fn_signature()?,

            OpCode::CALL => self.call_fn()?,
            OpCode::RET => self.ret_fn()?,

            //////////////
            // Misc ops //
            //////////////
            OpCode::PRNT => {
                let value = self.pop()?;
                let value = self.resolve_reference(value)?;
                println!("{value:?}");
                self.push(value);
            }

            OpCode::READF => todo!(),

            OpCode::LSTFN => {
                let functions = self.mem.all_functions();
                let values = functions
                    .into_iter()
                    .cloned()
                    .map(Value::Function)
                    .collect();
                self.push(Value::Array(values));
            }

            ////////////////////
            // Collection ops //
            ////////////////////
            OpCode::LEN => {
                let value = self.pop()?;
                let value = self.resolve_reference(value)?;
                self.push(Value::integer(value.len()));
            }

            OpCode::SSPLT => {
                let pat = self.pop()?;
                let value = self.pop()?;

                let pat = self.resolve_reference(pat)?;
                let pat = pat
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                let value = self.resolve_reference(value)?;
                let value = value
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                let result = value
                    .split(&pat)
                    .map(|s| Value::string(s.to_string()))
                    .collect::<Vec<_>>();
                self.push(Value::Array(result));
            }

            ////////////////
            // Trigonomic //
            ////////////////
            OpCode::ATAN2 => self.op_binary(math::atan2)?,
            OpCode::TAN => self.op_unary(math::tan)?,
            OpCode::SIN => self.op_unary(math::sin)?,
            OpCode::COS => self.op_unary(math::cos)?,

            OpCode::ATAN => self.op_unary(math::atan)?,
            OpCode::ASIN => self.op_unary(math::asin)?,
            OpCode::ACOS => self.op_unary(math::acos)?,

            OpCode::TANH => self.op_unary(math::tanh)?,
            OpCode::SINH => self.op_unary(math::sinh)?,
            OpCode::COSH => self.op_unary(math::cosh)?,

            OpCode::ROUND => self.op_binary(math::round)?,

            OpCode::LOG => self.op_binary(math::log)?,
            OpCode::ILOG => self.op_binary(math::ilog)?,

            OpCode::ROOT => self.op_binary(math::root)?,

            OpCode::NOP => {}
        }

        Ok(())
    }
}
