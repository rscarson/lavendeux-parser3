use super::Node;
use crate::{
    compiler::{Compiler, CompilerError},
    vm::OpCode,
};

/// System call dispatcher
/// Pushes all arguments to the stack and calls the given opcode
pub fn __syscalld<'source>(
    compiler: &mut Compiler<'source>,
    op: OpCode,
    args: Vec<Node<'source>>,
) -> Result<(), CompilerError> {
    for node in args.into_iter().rev() {
        node.compile(compiler)?;
    }
    compiler.push(op);

    Ok(())
}
