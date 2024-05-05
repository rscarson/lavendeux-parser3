use super::{core::ScriptNode, Node, ParserNode};
use crate::{
    compiler::{asm_transcoder::ASMTranscoder, Compiler, CompilerError},
    lexer::{Lexer, Stack, Token},
    traits::{IntoOwned, SerializeToBytes},
    value::Primitive,
    vm::OpCode,
};

/// System call dispatcher
/// Pushes all arguments to the stack and calls the given opcode
pub fn __syscalld<'source>(
    compiler: &mut Compiler,
    op: OpCode,
    args: Vec<Node<'source>>,
) -> Result<(), CompilerError> {
    for node in args.into_iter().rev() {
        node.compile(compiler)?;
    }
    compiler.push(op);

    Ok(())
}

pub fn __dissasemble<'source>(
    compiler: &mut Compiler,
    expr: Node<'source>,
) -> Result<(), CompilerError> {
    let input = expr.token().input().to_string();
    let mut _compiler = Compiler::new(&input, compiler.options().clone());
    expr.compile(&mut _compiler)?;

    let (debug, bytecode) = _compiler.decompose();
    let debug = if compiler.options().debug {
        Some(debug)
    } else {
        None
    };

    let transcoder = ASMTranscoder::new(&bytecode, debug);
    let out = transcoder.disassemble_as_string();

    compiler.push(OpCode::PUSH);
    let value = Primitive::String(out);
    compiler.extend(value.serialize_into_bytes());

    Ok(())
}

pub fn __include<'source>(
    compiler: &mut Compiler,
    token: Token<'source>,
    filename: String,
) -> Result<(), CompilerError> {
    let token = token.into_owned();
    // So we are effectively going to create a new locked scope here
    // We need to lex and parse the source, with the filename in the tokens
    // Then we compile that tree into the current compiler
    // And finally we need to pop the scope to remove the side-effects

    // Lex the file
    let source = std::fs::read_to_string(&filename)
        .map_err(|_| CompilerError::FileNotFound(token.clone(), filename.clone()))?;
    let lexer = Lexer::with_filename(&source, Some(filename));
    let mut stack = Stack::new(lexer.all_tokens().map_err(|e| {
        CompilerError::IncludeError(token.clone(), Box::new(crate::Error::Lexer(e)))
    })?);

    // Parse the file
    let ast = ScriptNode::parse(&mut stack)
        .ok_or_else(|| stack.emit_err())
        .map_err(|e| {
            CompilerError::IncludeError(token.clone(), Box::new(crate::Error::Parser(e)))
        })?;

    // Create the locked scope for the included file
    compiler.push(OpCode::SCI);
    compiler.push(OpCode::SCL);

    // Compile the file
    ast.compile(compiler)?;

    // Pop the locked scope
    compiler.push(OpCode::SCO);

    Ok(())
}
