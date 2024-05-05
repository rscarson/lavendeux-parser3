use super::{
    error::{RuntimeError, RuntimeErrorType},
    execution_context::ExecutionContext,
    memory_manager::MemoryManager,
    opcodes::OpCode,
};
use crate::{
    compiler::DebugProfile,
    value::{
        CheckedArithmetic, CheckedBitwise, CheckedBoolean, CheckedMatching, Primitive, Value,
        ValueType,
    },
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
pub struct VirtualMachine {
    mem: MemoryManager,
    last_opcode: OpCode,

    context: Vec<ExecutionContext>,
}

impl VirtualMachine {
    /// Creates a new execution context with the given bytecode and debug profile.
    /// Uses the specified memory manager.
    pub fn with_mem(mem: MemoryManager) -> Self {
        Self {
            mem,
            last_opcode: OpCode::NOP,
            context: vec![],
        }
    }

    /// Creates a new execution context with the given bytecode and debug profile.
    /// Uses a new memory manager.
    pub fn new() -> Self {
        Self::with_mem(MemoryManager::new())
    }

    /// Clears the stack and resets the VM to its initial state.
    pub fn reset(&mut self) {
        self.last_opcode = OpCode::NOP;
        self.context.clear();
        self.mem.reset();
    }

    /// Get a reference to the current execution context
    pub fn context(&self) -> &ExecutionContext {
        self.context.last().unwrap()
    }

    /// Get a mutable reference to the current execution context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        self.context.last_mut().unwrap()
    }

    /// Add a new function context to the stack.
    fn push_context(
        &mut self,
        code: Vec<u8>,
        debug_profile: Option<DebugProfile>,
        ret_type: ValueType,
    ) {
        self.context
            .push(ExecutionContext::new(code, debug_profile, ret_type));
    }

    /// Remove the top function context from the stack.
    fn pop_context(&mut self) {
        self.context.pop();
    }

    /// Emit an error at the current position
    fn emit_err(&self, error: RuntimeErrorType) -> RuntimeError {
        let mut e = self.context().emit_err(error);

        let mut cur = &mut e;
        for context in self.context[..self.context.len() - 1].iter().rev() {
            cur.parent = Some(Box::new(context.emit_err(RuntimeErrorType::Function)));
            cur = cur.parent.as_mut().unwrap();
        }
        e
    }

    /// Run the program until it finishes.
    /// Returns the value, or values (1 per line)
    pub fn run(
        &mut self,
        bytecode: Vec<u8>,
        profile: Option<DebugProfile>,
    ) -> Result<Value, RuntimeError> {
        self.reset();
        self.push_context(bytecode, profile, ValueType::All);
        while self.context().pc() < self.context().code().len() {
            self.next()?;
        }

        // Collect work stack values from memory
        let stack = self.mem.all_stack_blanks();
        let stack = stack
            .into_iter()
            .map(|blank| blank.into_value(&self.mem))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| self.emit_err(e))?;
        match stack.len() {
            1 => Ok(stack.into_iter().next().unwrap()),
            _ => Ok(Value::Array(stack)),
        }
    }

    /// Consumes the execution context and returns the memory manager.
    pub fn destroy(self) -> MemoryManager {
        self.mem
    }

    /// Run the next instruction in the current context.
    pub fn next(&mut self) -> Result<(), RuntimeError> {
        let opcode = self.read_opcode()?;

        #[cfg(feature = "debug_compiler_internal_vm")]
        {
            print!("{:?}@{:02X} ", opcode, self.context().pc());
            println!("{}", self.mem);
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
            OpCode::JMPT => self.op_jump_if_true()?,
            OpCode::JMPF => self.op_jump_if_false()?,
            OpCode::JMPE => self.op_jump_if_empty()?,
            OpCode::JMPNE => self.op_jump_if_not_empty()?,

            /////////////////////////
            // Memory manipulation //
            /////////////////////////
            OpCode::REF => self.read_reference()?,
            OpCode::VREF => self.verify_reference()?,
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
            OpCode::TYPE => {
                let value = self.pop_value()?;
                let value = Value::string(value.type_of().to_string());
                self.push_value(value);
            }
            OpCode::CAST => {
                let type_name = self.read_type()?;
                let value = self.pop_value()?;
                let value = value
                    .cast(type_name)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
                self.push_value(value);
            }
            OpCode::NEXT => {
                let value = self.pop_value()?;
                let (first, rest) = match value {
                    Value::Primitive(Primitive::String(s)) => {
                        if s.is_empty() {
                            return Err(self.emit_err(RuntimeErrorType::IteratorEmpty));
                        } else {
                            let first = Value::string(s.chars().next().unwrap().to_string());
                            let rest = Value::string(s.chars().skip(1).collect());
                            (first, rest)
                        }
                    }
                    Value::Primitive(p) => (Value::Primitive(p), Value::Array(vec![])),
                    Value::Function(f) => (Value::Function(f), Value::Array(vec![])),

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

                self.push_value(rest);
                self.push_value(first);
            }
            OpCode::LCST => {
                let right = self.pop_value()?;
                let left = self.pop_value()?;

                let lefttype = left.type_of();
                let right = right
                    .cast(lefttype)
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                self.push_value(left);
                self.push_value(right);
            }

            OpCode::MKAR => self.op_make_array()?,
            OpCode::MKOB => self.op_make_object()?,
            OpCode::MKRG => self.op_make_range()?,

            OpCode::PSAR => self.op_push_array()?,
            OpCode::PSOB => self.op_push_object()?,

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

            ///////////////
            // Debug ops //
            ///////////////
            OpCode::PRNTM => {
                let s = self.mem.to_string();
                let value = Value::string(s);
                self.push_value(value);
            }

            OpCode::PRNT => {
                let value = self.pop_value()?;
                println!("{value:?}");
                self.push_value(value);
            }

            OpCode::THRW => {
                let msg = self.pop_value()?.to_string();
                return Err(self.emit_err(RuntimeErrorType::Custom(msg)));
            }

            //////////////
            // Misc ops //
            //////////////
            OpCode::SORT => {
                let mut value = self.pop_value()?;
                value.sort();
                self.push_value(value);
            }

            OpCode::READF => todo!(),

            OpCode::LSTFN => {
                let functions = self.mem.all_functions();
                let values = functions
                    .into_iter()
                    .cloned()
                    .map(Value::Function)
                    .collect();
                self.push_value(Value::Array(values));
            }

            ////////////////////
            // Collection ops //
            ////////////////////
            OpCode::LEN => {
                let value = self.pop_value()?;
                self.push_value(Value::integer(value.len()));
            }

            OpCode::SSPLT => {
                let pat = self.pop_value()?;
                let value = self.pop_value()?;

                let pat = pat
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                let value = value
                    .cast_string()
                    .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

                let result = value
                    .split(&pat)
                    .map(|s| Value::string(s.to_string()))
                    .collect::<Vec<_>>();
                self.push_value(Value::Array(result));
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
