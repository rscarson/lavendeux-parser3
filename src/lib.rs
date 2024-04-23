//!
#![warn(missing_docs)]
#![deny(elided_lifetimes_in_paths)]
#![allow(unused_assignments)]

pub mod traits;

mod lavendeux;
pub use lavendeux::Lavendeux;

mod error;
pub use error::Error;

pub mod compiler;
pub mod lexer;
pub mod literals;
pub mod parser;
pub mod value;
pub mod vm;

#[cfg(test)]
mod test {
    use self::{
        lexer::{Lexer, Stack},
        parser::ParserNode,
    };
    use super::*;

    #[test]
    fn test_load_zarbandata() {
        let input = std::fs::read_to_string("example_scripts/zarban_storydata.lav").unwrap();
        let tokens = Lexer::new(&input).all_tokens().expect("Could not lex");
        let mut tokens = Stack::new(tokens);
        if parser::core::ScriptNode::parse(&mut tokens).is_none() {
            println!("{}", tokens.emit_err());
            panic!("Could not parse zarban_storydata.lav");
        }
    }

    #[test]
    fn test_load_zarban() {
        let input = std::fs::read_to_string("example_scripts/zarbans_grotto.lav").unwrap();
        let tokens = Lexer::new(&input).all_tokens().expect("Could not lex");
        println!("{:#?}", tokens);
        let mut tokens = Stack::new(tokens);
        if parser::core::ScriptNode::parse(&mut tokens).is_none() {
            println!("{}", tokens.emit_err());
            panic!("Could not parse zarbans_grotto.lav");
        }
    }
}
