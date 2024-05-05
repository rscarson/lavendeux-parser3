use crate::traits::SafeVecAlloc;
use crate::value::ValueType;
use crate::vm::memory_manager::MemoryManager;
use crate::vm::value_source::ValueSource;
use crate::{
    value::{Function, Value},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{IOExt, StackExt};

pub trait FunctionExt {
    fn alloc_fn(&mut self) -> Result<(), RuntimeError>;
    fn make_fn(&mut self) -> Result<(), RuntimeError>;
    fn push_default_fn_arg(&mut self) -> Result<(), RuntimeError>;
    fn push_fn_signature(&mut self) -> Result<(), RuntimeError>;

    fn call_fn(&mut self) -> Result<(), RuntimeError>;
    fn ret_fn(&mut self) -> Result<(), RuntimeError>;

    fn pop_function(&mut self) -> Result<Function, RuntimeError>;
}

impl FunctionExt for super::VirtualMachine {
    #[inline(always)]
    fn pop_function(&mut self) -> Result<Function, RuntimeError> {
        let function = self.pop_value()?;
        match function {
            Value::Function(function) => Ok(function),
            _ => Err(self.emit_err(RuntimeErrorType::BadType(
                self.last_opcode,
                ValueType::Function,
            ))),
        }
    }

    #[inline(always)]
    fn alloc_fn(&mut self) -> Result<(), RuntimeError> {
        let function = self.pop_function()?;
        let name_hash = function.name_hash;
        self.mem.write_global(
            name_hash,
            ValueSource::Literal(Value::Function(function)),
            false,
        );
        Ok(())
    }

    #[inline(always)]
    fn make_fn(&mut self) -> Result<(), RuntimeError> {
        let _version = self.next_byte()?;
        let function = self.decode_with_iterator::<Function>()?;
        self.push_value(Value::Function(function));
        Ok(())
    }

    #[inline(always)]
    fn push_default_fn_arg(&mut self) -> Result<(), RuntimeError> {
        let default = self.pop_value()?;
        let mut function = self.pop_function()?;

        let i = self.read_u16()? as usize;
        if let Some(arg) = function.expects.get_mut(i) {
            arg.default = Some(default);
        }
        self.push_value(Value::Function(function));
        Ok(())
    }

    #[inline(always)]
    fn push_fn_signature(&mut self) -> Result<(), RuntimeError> {
        let mut function = self.pop_function()?;
        let name = &function.docs.name;
        let args = function
            .docs
            .args
            .iter()
            .zip(&function.expects)
            .map(|(name, arg)| {
                let type_name = match arg.ty {
                    ValueType::All => "".to_string(),
                    _ => format!(": {}", arg.ty),
                };

                let default = match arg.default.as_ref() {
                    Some(default) => format!(" = {default:?}"),
                    None => "".to_string(),
                };

                format!("{name}{type_name}{default}")
            })
            .collect::<Vec<_>>()
            .join(", ");

        let returns = match function.returns {
            ValueType::All => "".to_string(),
            _ => format!(" -> {}", function.returns),
        };

        function.docs.signature = format!("{name}({args}){returns}");
        self.push_value(Value::Function(function));
        Ok(())
    }

    #[inline(always)]
    fn call_fn(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let n_args = self.read_u64()? as usize;

        let function = ValueSource::unresolved(name_hash);
        let function = function
            .into_value(&self.mem)
            .map_err(|e| self.emit_err(e))?;

        let function = match function {
            Value::Function(f) => f,
            _ => return Err(self.emit_err(RuntimeErrorType::UndefinedFunction)),
        }
        .clone();

        // Resolve argument values
        let mut provided = vec![];
        for _ in 0..n_args {
            let mut next = self.pop()?;
            if let ValueSource::Reference(r) = next {
                next = ValueSource::Reference(
                    r.into_resolved(&self.mem).map_err(|e| self.emit_err(e))?,
                );
            }
            provided.push(next);
        }
        provided.reverse();
        let arguments =
            resolve_arguments(&function, provided, &self.mem).map_err(|e| self.emit_err(e))?;

        // Allocate new stack frame
        self.mem.scope_in();
        self.mem.scope_lock();
        for (name, value) in arguments {
            self.mem.write(name, value);
        }

        // Create a new context level for the function to run in
        self.push_context(function.body, function.debug, function.returns);

        Ok(())
    }

    fn ret_fn(&mut self) -> Result<(), RuntimeError> {
        let returns = self.pop_value()?;
        let returns = returns
            .cast(self.context().return_type())
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;
        self.push_value(returns);
        self.mem.scope_out_to_lock();
        let returns = self.pop_value()?;

        self.pop_context();
        self.push_value(returns);
        Ok(())
    }
}

/// Resolve the arguments provided to a function into a list of argument values and the hashes they map to.
fn resolve_arguments(
    function: &Function,
    provided: Vec<ValueSource>,
    mem: &MemoryManager,
) -> Result<Vec<(u64, ValueSource)>, RuntimeErrorType> {
    let n_expected = function.expects.len();
    let n_args = provided.len();
    if n_args > n_expected {
        return Err(RuntimeErrorType::IncorrectFunctionArgCount(
            function.docs.signature.clone(),
        ));
    }

    let mut arguments = Vec::safe_alloc(n_args)?;
    let mut provided = provided.into_iter();
    let mut expected = function.expects.iter();
    let mut i = 1;
    loop {
        let next_expected = match expected.next() {
            Some(arg) => arg,
            None => break,
        };

        let next = provided.next();
        let mut next_value = match next {
            Some(value) => {
                if value.is_a(mem, next_expected.ty)? {
                    value
                } else {
                    return Err(RuntimeErrorType::IncorrectFunctionArg {
                        signature: function.docs.signature.clone(),
                        expected: next_expected.ty,
                        provided: value.type_of(mem)?,
                        index: i,
                    });
                }
            }

            _ if next_expected.default.is_some() => {
                ValueSource::Literal(next_expected.default.clone().unwrap())
            }

            None => {
                return Err(RuntimeErrorType::IncorrectFunctionArgCount(
                    function.docs.signature.clone(),
                ));
            }
        };

        // Squash any references if it is not a reference argument
        if !next_expected.by_ref {
            next_value = ValueSource::Literal(next_value.into_value(mem)?);
        }

        arguments.push((next_expected.name_hash, next_value));

        i += 1;
    }

    if arguments.len() != n_expected {
        // Not enough arguments
        return Err(RuntimeErrorType::IncorrectFunctionArgCount(
            function.docs.signature.clone(),
        ));
    }

    Ok(arguments)
}
