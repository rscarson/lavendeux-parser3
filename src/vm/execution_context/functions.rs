use crate::value::ValueType;
use crate::vm::OpCode;
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

impl FunctionExt for ExecutionContext<'_> {
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
            let name = self.decode_with_iterator::<String>()?;
            let mut args = vec![];

            let n_args = self.read_u16()? as usize;
            if n_args != function.expects.len() {
                return Err(self.emit_err(RuntimeErrorType::InvalidOpcode(OpCode::FSIG as u8)));
            }
            for i in 0..n_args {
                let name = self.decode_with_iterator::<String>()?;
                let default = (&function.expects[i].default).as_ref();
                let ty = function.expects[i].ty;
                args.push((name, ty, default));
            }

            let args = args
                .into_iter()
                .map(|(name, ty, default)| {
                    let type_name = match ty {
                        ValueType::All => "".to_string(),
                        _ => format!(": {ty:?}"),
                    };
                    let default = match default {
                        Some(default) => format!(" = {default:?}"),
                        None => "".to_string(),
                    };

                    format!("{name}{type_name}{default}")
                })
                .collect::<Vec<_>>()
                .join(", ");

            let returns = match function.returns {
                ValueType::All => "".to_string(),
                _ => format!(" -> {:?}", function.returns),
            };

            function.docs.signature = format!("{name}({args}){returns}");
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
        let mut expected_args = function.expects.iter().peekable();
        let arguments = self.read_u64()? as usize;
        if arguments > expected_args.len() {
            return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                function.docs.signature.clone(),
            )));
        }
        let mut args = Vec::with_capacity(arguments);
        for _ in 0..arguments {
            let value = self.pop()?;
            loop {
                match expected_args.next() {
                    None => {
                        return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                            function.docs.signature.clone(),
                        )))
                    }
                    Some(arg) => {
                        // Check type
                        if value.is_type(arg.ty) {
                            // If type matches, add to args
                            args.push((arg.name_hash, value));
                            break;
                        } else if let Ok(value) = value.clone().cast(arg.ty) {
                            // If type doesn't match, try to cast
                            args.push((arg.name_hash, value));
                            break;
                        } else if let Some(default) = &arg.default {
                            // If type doesn't match, check for default value
                            args.push((arg.name_hash, default.clone()));
                        } else {
                            // If no default value, return error
                            return Err(self.emit_err(RuntimeErrorType::IncorrectFunctionArgs(
                                function.docs.signature.clone(),
                            )));
                        }
                    }
                }
            }
        }

        // Allocate new stack frame
        self.mem.scope_in();
        self.mem.scope_lock();
        for (name, value) in args {
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
