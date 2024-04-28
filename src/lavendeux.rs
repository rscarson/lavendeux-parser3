use crate::{
    compiler::{CompilerOptions, DebugProfile},
    error::Error,
    lexer::Stack,
    parser::{core::ScriptNode, ParserNode},
    value::Value,
    vm::{memory_manager::MemoryManager, ExecutionContext},
};

/// Main structure for interacting with the Lavendeux parser
/// Allows compiling and running Lavendeux source code
pub struct Lavendeux {
    memory: Option<MemoryManager>,
    options: CompilerOptions,
}

impl Lavendeux {
    /// Create a new Lavendeux instance
    /// Allocates a new memory manager for the instance
    pub fn new() -> Self {
        Self {
            memory: Some(MemoryManager::new()),
            options: Default::default(),
        }
    }

    /// Create a new Lavendeux instance with custom compiler options
    /// Allocates a new memory manager for the instance
    pub fn with_options(options: CompilerOptions) -> Self {
        let mut mem = MemoryManager::new();
        mem.load_stdlib();

        Self {
            memory: Some(mem),
            options,
        }
    }

    /// Compile a source string into a debug profile and bytecode.
    /// Returns an error if the source string is invalid.
    ///
    /// Debug profile is used for error message generation.
    /// You can can write it to a file using `into_bytes` and return it using `from_bytes`.
    ///
    /// Example:
    /// ```rust
    /// # use lavendeux::{Lavendeux, Error};
    /// # fn main() -> Result<(), Error> {
    /// let mut lav = Lavendeux::new();
    /// let (profile, bytecode) = lav.compile("!true")?;
    /// assert_eq!(bytecode, vec![0x00, 0x01, 0x31]); // [PUSH true; LNOT]
    /// # Ok(())
    /// # }
    pub fn compile<'source>(
        &mut self,
        source: &'source str,
    ) -> Result<(DebugProfile, Vec<u8>), Error> {
        let lexer = crate::lexer::Lexer::new(source);
        let tokens = lexer.all_tokens()?;

        let mut stack = Stack::new(tokens);
        let ast = ScriptNode::parse(&mut stack).ok_or(stack.emit_err())?;

        let mut compiler = crate::compiler::Compiler::new(source, self.options.clone());
        ast.compile(&mut compiler)?;

        Ok(compiler.decompose())
    }

    /// Run a compiled program with the given bytecode and debug profile.
    /// Returns the result of the program or an error if the program crashes.
    ///
    /// Example:
    /// ```rust
    /// # use lavendeux::{Lavendeux, Error};
    /// # fn main() -> Result<(), Error> {
    /// let mut lav = Lavendeux::new();
    ///
    pub fn execute<'source>(
        &mut self,
        bytecode: Vec<u8>,
        profile: Option<DebugProfile>,
    ) -> Result<Value, Error> {
        let mut context =
            ExecutionContext::with_mem(bytecode, profile, self.memory.take().unwrap());
        let result = context.run();
        self.memory = Some(context.destroy());
        Ok(result?)
    }

    /// Run a source string.
    /// Returns the result of the program as an array of values (1 per line)
    /// or an error if the program crashes.
    ///
    /// Example:
    /// ```rust
    /// # use lavendeux::{Lavendeux, Error, Value};
    /// # fn main() -> Result<(), Error> {
    /// let mut lav = Lavendeux::new();
    /// let result = lav.run("1 + 2")?;
    /// assert_eq!(result, Value::Integer(3));
    /// # Ok(())
    /// # }
    pub fn run<'source>(&mut self, source: &'source str) -> Result<Value, Error> {
        let (profile, bytecode) = self.compile(source)?;
        self.execute(bytecode, Some(profile))
    }
}
