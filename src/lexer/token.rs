use super::Rule;
use crate::traits::{IntoOwned, SerializeToBytes};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A token span
pub type TokenSpan = std::ops::Range<usize>;

/// A token, with string input removed. Call `unpack` to get the full token
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializedToken {
    /// The line number of the token
    pub line: usize,

    /// The span of the token
    pub span: TokenSpan,

    /// The rule of the token
    pub rule: Rule,
}

impl SerializedToken {
    /// Pack a token into a serializable form
    pub fn pack(token: Token<'_>) -> Self {
        Self {
            line: token.line,
            span: token.span.clone(),
            rule: token.rule.clone(),
        }
    }

    /// Unpack a token from a serializable form
    pub fn unpack<'source>(&self, input: &'source str) -> Token<'source> {
        Token {
            line: self.line,
            span: self.span.clone(),
            rule: self.rule.clone(),
            input: Cow::Borrowed(input),
        }
    }
}

impl SerializeToBytes for SerializedToken {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.line.serialize_into_bytes());
        bytes.extend_from_slice(&self.span.start.serialize_into_bytes());
        bytes.extend_from_slice(&self.span.end.serialize_into_bytes());
        bytes.extend_from_slice(&self.rule.serialize_into_bytes());
        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let line = usize::deserialize_from_bytes(bytes)?;
        let start = usize::deserialize_from_bytes(bytes)?;
        let end = usize::deserialize_from_bytes(bytes)?;
        let rule = Rule::deserialize_from_bytes(bytes)?;

        Ok(Self {
            line,
            span: start..end,
            rule,
        })
    }
}

/// A token, with a reference to the input string
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token<'source> {
    line: usize,
    span: TokenSpan,
    rule: Rule,
    input: Cow<'source, str>,
}
impl<'source> IntoOwned for Token<'source> {
    type Owned = Token<'static>;
    fn into_owned(self) -> Self::Owned {
        Self::Owned {
            line: self.line,
            span: self.span,
            rule: self.rule,
            input: Cow::Owned(self.input.into_owned()),
        }
    }
}
impl<'source> Token<'source> {
    pub(crate) fn new(line: usize, span: TokenSpan, rule: Rule, input: Cow<'source, str>) -> Self {
        Self {
            line,
            span,
            rule,
            input,
        }
    }

    /// Get the input string
    pub fn borrow_input(&self) -> Cow<'source, str> {
        self.input.clone()
    }

    /// Get a new token with a different rule, but the same span and line
    pub fn child(&self, rule: Rule, span: TokenSpan) -> Self {
        Token {
            line: self.line,
            span: span.start..span.end,
            rule,
            input: self.borrow_input(),
        }
    }

    /// Set the rule of the token
    pub fn set_rule(&mut self, rule: Rule) {
        self.rule = rule;
    }

    /// Expand self to include the span of another token
    pub fn include_span(&mut self, span: TokenSpan) {
        self.span.start = self.span.start.min(span.start);
        self.span.end = self.span.end.max(span.end);
    }

    /// Get the line number of the token
    pub fn line(&self) -> usize {
        self.line
    }

    /// Get a slice of the input at the given line
    fn line_slice(&self) -> (&str, usize) {
        let start = self.input[..self.span.start]
            .rfind('\n')
            .map_or(0, |pos| pos + 1);
        let mut end = self.input[self.span.end..]
            .find('\n')
            .map_or(self.input.len(), |pos| pos + self.span.end);

        if end > self.span.end {
            end = self.span.end;
        }

        (&self.input[start..end], self.span.start - start)
    }

    /// Returns a slice of the input surrounding and ending after the token, with a maximum of 50 characters
    /// Will include a line beneath highlighting the token
    pub fn context_slice(&self) -> String {
        const CONTEXT_LEN: usize = 50;
        let (mut line, mut highlight_start) = self.line_slice();
        let mut highlight_len = line.len() - highlight_start;

        if highlight_len > CONTEXT_LEN {
            line = &line[highlight_start..highlight_start + CONTEXT_LEN];
            highlight_len = CONTEXT_LEN;
            highlight_start = 0;
        } else if line.len() > CONTEXT_LEN {
            line = &line[line.len() - CONTEXT_LEN..];
            highlight_start = CONTEXT_LEN - highlight_len;
        }

        let line = line
            .split('\n')
            .map(|l| format!("| {}", l))
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "{}\n| {}{}",
            line,
            " ".repeat(highlight_start),
            "^".repeat(highlight_len)
        )
    }

    /// Get the span of the token
    pub fn span(&self) -> TokenSpan {
        self.span.clone()
    }

    /// Get the rule of the token
    pub fn rule(&self) -> Rule {
        self.rule.clone()
    }

    /// Get the input string
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Get a slice of the input string
    pub fn slice(&self) -> &str {
        &self.input[self.span.clone()]
    }

    /// Get a slice of the input string at a given span
    pub fn slice_at(&self, span: TokenSpan) -> &str {
        &self.input[span]
    }

    /// Check if the token is of a given rule
    pub fn is_a(&self, v: &[Rule]) -> bool {
        v.iter().any(|r| self.rule == *r)
    }

    /// Alter this token to point to a new borrowed input string
    /// Alters the lifespan of the token
    pub fn recombobulate(&mut self, new_input: &'source str) {
        self.input = Cow::Borrowed(new_input);
    }
}

impl std::fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}({}:{:?} `{}`)",
            self.rule,
            self.line(),
            self.span(),
            self.slice().replace("\n", "\\n")
        )
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line {}\n{}", self.line(), self.context_slice())
    }
}
