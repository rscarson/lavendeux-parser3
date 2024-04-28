use crate::value::ValueType;
use crate::{
    value::{Function, Value},
    vm::error::{RuntimeError, RuntimeErrorType},
};

use super::{ExecutionContext, IOExt, RefExt, StackExt};

pub trait FunctionExt {
    fn alloc_fn(&mut self) -> Result<(), RuntimeError>;
    fn make_fn(&mut self) -> Result<(), RuntimeError>;
    fn push_default_fn_arg(&mut self) -> Result<(), RuntimeError>;
    fn push_fn_signature(&mut self) -> Result<(), RuntimeError>;

    fn call_fn(&mut self) -> Result<(), RuntimeError>;
    fn ret_fn(&mut self) -> Result<(), RuntimeError>;
}

impl FunctionExt for ExecutionContext {
    fn alloc_fn(&mut self) -> Result<(), RuntimeError> {
        let function = self.pop()?;
        if let Value::Function(function) = function {
            let name_hash = function.name_hash;
            self.mem
                .write_global(name_hash, Value::Function(function), false);
        }
        Ok(())
    }

    fn make_fn(&mut self) -> Result<(), RuntimeError> {
        let _version = self.next_byte()?;
        let function = self.decode_with_iterator::<Function>()?;
        self.push(Value::Function(function));
        Ok(())
    }

    fn push_default_fn_arg(&mut self) -> Result<(), RuntimeError> {
        let default = self.pop()?;
        let function = self.pop()?;
        if let Value::Function(mut function) = function {
            let i = self.read_u16()? as usize;
            if let Some(arg) = function.expects.get_mut(i) {
                arg.default = Some(default);
            }
            self.push(Value::Function(function));
        }
        Ok(())
    }

    fn push_fn_signature(&mut self) -> Result<(), RuntimeError> {
        let function = self.pop()?;
        if let Value::Function(mut function) = function {
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
            self.push(Value::Function(function));
        }
        Ok(())
    }

    fn call_fn(&mut self) -> Result<(), RuntimeError> {
        let name_hash = self.read_u64()?;
        let function = self
            .mem
            .read(name_hash)
            .ok_or(self.emit_err(RuntimeErrorType::UndefinedFunction))?;

        let function = match function {
            Value::Function(f) => f,
            _ => return Err(self.emit_err(RuntimeErrorType::UndefinedFunction)),
        }
        .clone();

        // Resolve argument values
        let n_expected = function.expects.len();
        let n_args = self.read_u64()? as usize;
        if n_args > n_expected {
            return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                function.docs.signature.clone(),
            )));
        }

        let mut provided = vec![];
        for _ in 0..n_args {
            provided.push(self.pop()?);
        }

        let mut arguments = Vec::with_capacity(n_args);
        let mut provided = provided.into_iter();
        let mut expected = function.expects.into_iter();
        loop {
            let next_expected = match expected.next() {
                Some(arg) => arg,
                None => break,
            };

            let next = provided.next();
            match next {
                Some(value)
                    if value
                        .is_a(next_expected.ty, Some(&mut self.mem))
                        .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))? =>
                {
                    arguments.push((next_expected.name_hash, value));
                }

                _ if next_expected.default.is_some() => {
                    arguments.push((next_expected.name_hash, next_expected.default.unwrap()));
                }

                _ => {
                    return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                        function.docs.signature.clone(),
                    )));
                }
            }
        }

        if arguments.len() != n_expected {
            // Not enough arguments
            return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                function.docs.signature.clone(),
            )));
        }

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
        let returns = self.pop()?;
        let returns = self.resolve_reference(returns)?;
        let returns = returns
            .cast(self.return_type())
            .map_err(|e| self.emit_err(RuntimeErrorType::Value(e)))?;

        self.mem.scope_out();
        self.pop_context();
        self.push(returns);
        Ok(())
    }
}
