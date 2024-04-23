//! The lexer module for the language
//! Splits the input into tokens
//! `Lexer` is the main entry point for the lexer.
//! `Token` is the main token type.
//! `Rule` is the set of rules for the lexer.
//! `Stack` is a token queue with rewind used by the parser.
use crate::traits::IntoOwned;
use logos::Logos;
use std::borrow::Cow;

mod error;
pub use error::LexerError;

mod stack;
pub use stack::Stack;

mod rule;
pub use rule::Rule;

mod token;
pub use token::{SerializedToken, Token, TokenSpan};

mod category;
pub use category::Category;

/// A lexer for the language
/// Splits the input into tokens
pub struct Lexer<'source>(logos::Lexer<'source, Rule>);
impl<'source> Lexer<'source> {
    /// Creates a new lexer from the input
    pub fn new(input: &'source str) -> Self {
        Self(Rule::lexer_with_extras(input, 1))
    }

    /// Consumes and returns the next token
    pub fn consume_next(&mut self) -> Token<'source> {
        let token = self.0.next().unwrap_or_else(|| Ok(Rule::EOI));
        let input = self.0.source();
        Token::new(
            self.0.extras,
            self.0.span(),
            token.unwrap_or_else(|_| Rule::Error),
            Cow::Borrowed(input),
        )
    }

    /// Consumes this iterator, returning all tokens in the input
    pub fn all_tokens(mut self) -> Result<Vec<Token<'source>>, LexerError> {
        let mut tokens = vec![];
        loop {
            let next = self.consume_next();
            if next.rule() == Rule::Error {
                return Err(LexerError::UnrecognizedToken(next.into_owned()));
            }

            match next {
                t if t.rule() == Rule::EOI => {
                    tokens.push(t);
                    break;
                }

                t => tokens.push(t),
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_tokens {
        ($input:literal, $tokens:expr) => {{
            let t = Lexer::new($input).all_tokens().unwrap();
            assert_eq!($tokens, t.into_iter().map(|t| t.rule()).collect::<Vec<_>>());
        }};
    }

    #[test]
    fn test_comments() {
        assert_tokens!(
            r#"
            // This is a line comment
            /* This is a 
            block 
            comment 
            
            */
            a
            "#,
            vec![
                Rule::EOL,
                Rule::EOL,
                Rule::EOL,
                Rule::LiteralIdent,
                Rule::EOL,
                Rule::EOI
            ]
        );
    }

    #[test]
    fn test_const_indents_keywords() {
        assert_tokens!("returned", vec![Rule::LiteralIdent, Rule::EOI]);
        assert_tokens!("pies", vec![Rule::LiteralIdent, Rule::EOI]);
        assert_tokens!("applepi", vec![Rule::LiteralIdent, Rule::EOI]);
    }
}
