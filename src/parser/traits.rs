/// Main trait for the AST's nodes
/// Covers parsing, execution, reconstitution, and eventually JS transpilation
pub trait ParserNode<'source>
where
    Self: crate::traits::IntoOwned,
{
    /// Convert into a Node
    fn into_node(self) -> crate::parser::Node<'source>;

    /// Parse a node from the token stream
    fn parse(tokens: &mut crate::lexer::Stack<'source>) -> Option<crate::parser::Node<'source>>;

    /// Compile the node into bytecode
    fn compile(
        self,
        compiler: &mut crate::compiler::Compiler<'source>,
    ) -> Result<(), crate::compiler::CompilerError>;
}
