#[macro_use]
mod macros;

mod pratt;

mod traits;
pub use traits::ParserNode;

mod nodes;
pub use nodes::*;

pub fn build_ast<'source>(
    tokens: &mut crate::lexer::Stack<'source>,
) -> Result<Node<'source>, crate::error::Error> {
    match core::ScriptNode::parse(tokens) {
        Some(ast) => Ok(ast),
        None => Err(tokens.emit_err()),
    }
}
