use crate::{
    compiler::{Compiler, CompilerError, DebugProfile, FunctionDocs, HashString},
    traits::{IntoOwned, SerializeToBytes},
    value::{Function, FunctionArgument, Value, ValueType},
    vm::OpCode,
};

use super::Node;

pub enum FunctionArgumentDefault<'source> {
    None,
    Static(Value),
    Stack(Node<'source>),
}

pub struct FunctionArgumentCompiler<'source> {
    pub name: String,
    pub default: FunctionArgumentDefault<'source>,
    pub ty: ValueType,
}

pub struct FunctionCompiler<'source> {
    pub name: String,
    pub args: Vec<FunctionArgumentCompiler<'source>>,
    pub body: Node<'source>,
    pub ty: ValueType,
    pub dbg: Option<DebugProfile<'source>>,
    pub doc: FunctionDocs,
}

impl<'source> FunctionCompiler<'source> {
    // Creating a function takes a few steps:
    // 1. use MKFN to create a function object on the stack
    // 2. Use FDFT to update defaults from the stack, one `FDFT {u16 idx}` per default
    // 3. Use FSIG to set the signature of the function: `FSIG {name} {arg names}`
    // 4. Use WRFN to write the function to memory

    pub fn compile(self, compiler: &mut Compiler<'source>) -> Result<(), CompilerError> {
        let name_hash = self.name.hash_str();

        let mut arg_names = vec![];
        let mut arg_defaults = vec![];
        let mut args = vec![];

        for (i, arg) in self.args.into_iter().enumerate() {
            match arg.default {
                FunctionArgumentDefault::None => args.push(FunctionArgument {
                    name_hash: arg.name.hash_str(),
                    ty: arg.ty,
                    default: None,
                }),
                FunctionArgumentDefault::Static(value) => args.push(FunctionArgument {
                    name_hash: arg.name.hash_str(),
                    ty: arg.ty,
                    default: Some(value),
                }),
                FunctionArgumentDefault::Stack(node) => {
                    args.push(FunctionArgument {
                        name_hash: arg.name.hash_str(),
                        ty: arg.ty,
                        default: None,
                    });
                    arg_defaults.push((i as u16, node));
                }
            }
            arg_names.push(arg.name);
        }

        let function_slice = self.body.token().slice().to_string();
        let offset = self.body.token().span().start;

        let mut fcompiler = Compiler::new(&function_slice, compiler.options().clone());
        self.body.compile(&mut fcompiler)?;
        fcompiler.push(OpCode::RET);
        let (mut debug, body) = fcompiler.decompose();

        debug.offset(offset);
        let debug = match compiler.options().debug {
            true => Some(debug.into_owned()),
            false => None,
        };

        let function = Function {
            name_hash,
            returns: self.ty,
            expects: args,
            debug,
            docs: self.doc,
            body,
        };
        let function = function.serialize_into_bytes();

        compiler.push(OpCode::MKFN);
        compiler.push_u8(0u8); // Version code
        compiler.extend(function);

        // Set function stack defaults
        for (i, node) in arg_defaults {
            node.compile(compiler)?;
            compiler.push(OpCode::FDFT);
            compiler.extend(i.serialize_into_bytes());
        }

        // Update the function signature
        compiler.push(OpCode::FSIG);

        compiler.extend(self.name.len().serialize_into_bytes());
        compiler.extend(self.name.as_bytes().to_vec());

        compiler.extend((arg_names.len() as u16).serialize_into_bytes());
        for arg in arg_names {
            compiler.extend(arg.len().serialize_into_bytes());
            compiler.extend(arg.as_bytes().to_vec());
        }

        // Finally, write the function to memory
        compiler.push(OpCode::WRFN);

        Ok(())
    }
}
