#![deny(elided_lifetimes_in_paths)]
#![allow(unused_assignments)]

pub trait IntoOwned {
    type Owned;
    fn into_owned(self) -> Self::Owned;
}

pub mod error;
pub mod lexer;
pub mod literals;
pub mod parser;
pub mod types;

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
        let tokens = Lexer::new(&input).all_tokens();
        let mut tokens = Stack::new(tokens);
        let _ast =
            parser::core::ScriptNode::parse(&mut tokens).expect("Could not parse zarbandata.lav");
    }

    #[test]
    fn test_load_zarban() {
        let input = std::fs::read_to_string("example_scripts/zarbans_grotto.lav").unwrap();
        let tokens = Lexer::new(&input).all_tokens();
        let mut tokens = Stack::new(tokens);
        let _ast = parser::core::ScriptNode::parse(&mut tokens)
            .expect("Could not parse zarbans_grotto.lav");
    }
}
