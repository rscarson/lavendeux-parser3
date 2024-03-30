use super::Rule;
use crate::IntoOwned;
use std::borrow::Cow;

pub type TokenSpan = std::ops::Range<usize>;

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

    pub fn child(&self, rule: Rule, span: TokenSpan) -> Self {
        Token {
            line: self.line,
            span: span.start..span.end,
            rule,
            input: self.input.clone(),
        }
    }

    pub fn set_rule(&mut self, rule: Rule) {
        self.rule = rule;
    }

    /// Expand self to include the span of another token
    pub fn include_span(&mut self, span: TokenSpan) {
        self.span.start = self.span.start.min(span.start);
        self.span.end = self.span.end.max(span.end);
    }

    pub fn line(&self) -> usize {
        self.line
    }

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

        format!(
            "| {}\n| {}{}",
            line,
            " ".repeat(highlight_start),
            "^".repeat(highlight_len)
        )
    }

    pub fn span(&self) -> TokenSpan {
        self.span.clone()
    }

    pub fn rule(&self) -> Rule {
        self.rule.clone()
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn slice(&self) -> &str {
        &self.input[self.span.clone()]
    }

    pub fn is_a(&self, v: &[Rule]) -> bool {
        v.iter().any(|r| self.rule == *r)
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
