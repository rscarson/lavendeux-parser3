//! The parser module contains the parser for the language.
//! It takes a stack of tokens and converts it into an AST.
//! `build_ast` is the main entry point for the parser.

#[macro_use]
mod macros;

mod function_compiler;
mod special_functions;

mod error;
pub use error::ParserError;

mod pratt;

mod traits;
pub use traits::ParserNode;

mod nodes;
pub use nodes::*;

/// Parses a stack of tokens into an AST.
pub fn build_ast<'source>(
    mut tokens: crate::lexer::Stack<'source>,
) -> Result<Node<'source>, crate::parser::ParserError> {
    match core::ScriptNode::parse(&mut tokens) {
        Some(ast) => Ok(ast),
        None => Err(tokens.emit_err()),
    }
}
