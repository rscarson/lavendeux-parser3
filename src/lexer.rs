use logos::Logos;
use std::borrow::Cow;

mod stack;
pub use stack::Stack;

mod rule;
pub use rule::Rule;

mod token;
pub use token::{Token, TokenSpan};

mod category;
pub use category::Category;

pub fn lex<'source>(input: &'source str) -> Stack<'source> {
    Stack::new(Lexer::new(input).all_tokens())
}

pub struct Lexer<'source>(logos::Lexer<'source, Rule>);
impl<'source> Lexer<'source> {
    pub fn new(input: &'source str) -> Self {
        Self(Rule::lexer_with_extras(input, 1))
    }

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

    /// Consumes this iterator, returning all tokens
    pub fn all_tokens(mut self) -> Vec<Token<'source>> {
        let mut tokens = vec![];
        loop {
            let next = self.consume_next();
            match next {
                t if t.rule() == Rule::EOI => {
                    tokens.push(t);
                    break;
                }

                t => tokens.push(t),
            }
        }

        tokens
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_tokens {
        ($input:literal, $tokens:expr) => {{
            let t = Lexer::new($input).all_tokens();
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
