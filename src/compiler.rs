//! # Compiler
//! Contains the compiler and its components.
//! This module is responsible for compiling the AST into bytecode.
//! As well as providing a debug profile for error messages.

mod compiler;
pub use compiler::*;

mod debug_profile;
pub use debug_profile::DebugProfile;

mod error;
pub use error::CompilerError;

mod function_docs;
pub use function_docs::FunctionDocs;

pub mod asm_transcoder;
