/// Main trait for the AST's nodes
/// Covers parsing, execution, reconstitution, and eventually JS transpilation
pub trait ParserNode<'source>
where
    Self: crate::IntoOwned,
{
    fn into_node(self) -> crate::parser::Node<'source>;
    fn parse(tokens: &mut crate::lexer::Stack<'source>) -> Option<crate::parser::Node<'source>>;
}
