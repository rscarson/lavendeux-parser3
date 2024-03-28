use crate::IntoOwned;

use logos::{Logos, Skip};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Logos)]
#[logos(skip r"[ \t\r\f]+|(\\\n)+")]
#[logos(subpattern currency_symbol = r"[$¢£¤¥֏؋߾߿৲৳৻૱௹฿៛₠₡₢₣₤₥₦₧₨₩₪₫€₭₮₯₰₱₲₳₴₵₶₷₸₹₺₻₼₽₾₿꠸﷼﹩＄￠￡￥￦]")]
#[logos(subpattern integer_literal = r"(?:([0-9](\d|_)*)|(?:0[a-zA-Z][a-fA-F0-9]+))")]
#[logos(extras = usize)]
#[repr(u16)]
#[derive(strum_macros::Display)]
pub enum Rule {
    #[strum(to_string = "end of input")]
    EOI,

    Script,
    Block,

    #[strum(to_string = "[expression] as [type]")]
    CastExpr,

    #[strum(to_string = "[expression] @[decorator]")]
    DecoratorExpr,

    RangeExpr,
    IndexingExpr,

    DeleteExpr,
    AssignExpr,
    AssignArithmeticExpr,
    AssignBitwiseExpr,

    PrefixNeg,
    PrefixInc,
    PostfixInc,
    PrefixDec,
    PostfixDec,
    IndexingOperator,
    DecoratorOperator,
    FnCallOperator,
    TernaryOperator,

    ArithmethicInfixExpr,
    ArithmeticPrefixExpr,
    ArithmeticPostfixExpr,

    LogicalExpr,
    ComparisonExpr,
    MatchExpr,
    BitwiseInfixExpr,

    Array,
    Object,

    IfExpr,
    TernaryExpr,
    SwitchExpr,
    ForExpr,
    ReturnExpr,
    BreakExpr,
    ContinueExpr,

    FnCallExpr,
    FnAssignExpr,

    //
    // Symbols
    //
    #[strum(to_string = "end of line")]
    #[regex("\n|;", |l| {l.extras += 1})]
    EOL,

    #[strum(to_string = "(")]
    #[token("(")]
    LParen,
    #[strum(to_string = ")")]
    #[token(")")]
    RParen,

    #[strum(to_string = "{")]
    #[token("{")]
    LBrace,
    #[strum(to_string = "}")]
    #[token("}")]
    RBrace,

    #[strum(to_string = "[")]
    #[token("[")]
    LBrack,
    #[strum(to_string = "]")]
    #[token("]")]
    RBrack,

    #[strum(to_string = ",")]
    #[token(",")]
    Comma,
    #[strum(to_string = ".")]
    #[token(".")]
    Dot,

    #[strum(to_string = ":")]
    #[token(":")]
    Colon,
    #[strum(to_string = "?")]
    #[token("?")]
    Question,

    #[strum(to_string = "=>")]
    #[token("=>")]
    FatArrow,
    #[strum(to_string = "..")]
    #[token("..")]
    Range,

    #[strum(to_string = "@")]
    #[token("@")]
    Decorator,

    // Assignment operators
    #[strum(to_string = "=")]
    #[token("=")]
    Assign,

    #[strum(to_string = "+=")]
    #[token("+=")]
    AssignAdd,
    #[strum(to_string = "-=")]
    #[token("-=")]
    AssignSub,
    #[strum(to_string = "**=")]
    #[token("**=")]
    AssignPow,
    #[strum(to_string = "*=")]
    #[token("*=")]
    AssignMul,
    #[strum(to_string = "/=")]
    #[token("/=")]
    AssignDiv,
    #[strum(to_string = "%=")]
    #[token("%=")]
    AssignMod,

    #[strum(to_string = "|=")]
    #[token("|=")]
    AssignOr,
    #[strum(to_string = "&=")]
    #[token("&=")]
    AssignAnd,
    #[strum(to_string = "^=")]
    #[token("^=")]
    AssignXor,
    #[strum(to_string = "<<=")]
    #[token("<<=")]
    AssignSL,
    #[strum(to_string = ">>=")]
    #[token(">>=")]
    AssignSR,

    // Comments
    #[regex(r"//[^\n]*", |_| Skip)]
    LineComment,
    #[regex(r"/\*([^*]|\*[^/])*\*/", |lex| {
        // Count the number of newlines in the block comment
        let mut count = 0;
        for c in lex.slice().chars() {
            if c == '\n' {
                count += 1;
            }
        }
        lex.extras += count;
        Skip
    })]
    BlockComment,

    // Arithmetic operators
    #[strum(to_string = "++")]
    #[token("++")]
    Inc,
    #[strum(to_string = "--")]
    #[token("--")]
    Dec,
    #[strum(to_string = "+")]
    #[token("+")]
    Add,
    #[strum(to_string = "-")]
    #[token("-")]
    Sub,
    #[strum(to_string = "**")]
    #[token("**")]
    Pow,
    #[strum(to_string = "*")]
    #[token("*")]
    Mul,
    #[strum(to_string = "/")]
    #[token("/")]
    Div,
    #[strum(to_string = "%")]
    #[token("%")]
    Mod,

    // Bitwise operators
    #[strum(to_string = "~")]
    #[token("~")]
    BitwiseNot,
    #[strum(to_string = "|")]
    #[token("|")]
    BitwiseOr,
    #[strum(to_string = "&")]
    #[token("&")]
    BitwiseAnd,
    #[strum(to_string = "^")]
    #[token("^")]
    Xor,
    #[strum(to_string = "<<")]
    #[token("<<")]
    SL,
    #[strum(to_string = ">>")]
    #[token(">>")]
    SR,

    // Boolean operators
    #[strum(to_string = "||")]
    #[token("||")]
    LogicalOr,
    #[strum(to_string = "&&")]
    #[token("&&")]
    LogicalAnd,
    #[strum(to_string = "!")]
    #[token("!")]
    LogicalNot,

    // Comparison operators
    #[strum(to_string = "===")]
    #[token("===")]
    SEq,
    #[strum(to_string = "!==")]
    #[token("!==")]
    SNe,
    #[strum(to_string = "==")]
    #[token("==")]
    Eq,
    #[strum(to_string = "!=")]
    #[token("!=")]
    Ne,
    #[strum(to_string = "<=")]
    #[token("<=")]
    Le,
    #[strum(to_string = ">=")]
    #[token(">=")]
    Ge,
    #[strum(to_string = "<")]
    #[token("<")]
    Lt,
    #[strum(to_string = ">")]
    #[token(">")]
    Gt,

    //
    // Keywords
    //
    #[strum(to_string = "if")]
    #[token("if")]
    If,
    #[strum(to_string = "then")]
    #[token("then")]
    Then,
    #[strum(to_string = "else")]
    #[token("else")]
    Else,

    #[strum(to_string = "for")]
    #[token("for")]
    For,
    #[strum(to_string = "in")]
    #[token("in")]
    In,
    #[strum(to_string = "do")]
    #[token("do")]
    Do,
    #[strum(to_string = "where")]
    #[token("where")]
    Where,

    #[strum(to_string = "switch")]
    #[token("switch")]
    Switch,

    #[strum(to_string = "return")]
    #[token("return")]
    Return,
    #[strum(to_string = "continue")]
    #[token("continue")]
    Continue,
    #[strum(to_string = "break")]
    #[token("break")]
    Break,
    #[strum(to_string = "del")]
    #[regex("delete|del|unset")]
    Delete,

    #[strum(to_string = "as")]
    #[token("as")]
    As,

    #[strum(to_string = "contains")]
    #[token("contains")]
    Contains,
    #[strum(to_string = "matches")]
    #[token("matches")]
    Matches,
    #[strum(to_string = "is")]
    #[token("is")]
    Is,
    #[strum(to_string = "starts_with")]
    #[regex("starts_with|startswith")]
    StartsWith,
    #[strum(to_string = "ends_with")]
    #[regex("ends_with|endswith")]
    EndsWith,

    //
    // Value Literals
    //
    #[token("pi", priority = 2)]
    LiteralConstPi,
    #[token("e", priority = 2)]
    LiteralConstE,
    #[token("tau", priority = 2)]
    LiteralConstTau,
    #[token("nil", priority = 2)]
    LiteralConstNil,
    #[token("true", priority = 2)]
    LiteralConstTrue,
    #[token("false", priority = 2)]
    LiteralConstFalse,

    #[regex(r"[a-zA-Z_][0-9A-Za-z_]*", priority = 1)]
    LiteralIdent,

    #[regex(r"(?&integer_literal)u8")]
    LiteralU8,

    #[regex(r"(?&integer_literal)u16")]
    LiteralU16,

    #[regex(r"(?&integer_literal)u32")]
    LiteralU32,

    #[regex(r"(?&integer_literal)u64")]
    LiteralU64,

    #[regex(r"(?&integer_literal)i8")]
    LiteralI8,

    #[regex(r"(?&integer_literal)i16")]
    LiteralI16,

    #[regex(r"(?&integer_literal)i32")]
    LiteralI32,

    #[regex(r"(?&integer_literal)(i64)?")]
    LiteralI64,

    #[regex(r"(?:(?:[1-9](?:\d|_)*\.(?:\d|_)+)|(?:\.(?:\d|_)+))[dDfF]")]
    LiteralDecimal,

    #[regex(r"(?:(?:(?:[1-9](?:\d|_)*\.(?:\d|_)+)|(?:\.(?:\d|_)+))(?&currency_symbol))")]
    #[regex(r"(?:(?&currency_symbol)(?:(?:[1-9](?:\d|_)*\.(?:\d|_)+)|(?:\.(?:\d|_)+)))")]
    LiteralCurrency,

    #[regex(r"(?:(?:[1-9](?:\d|_)*\.(?:\d|_)+)|(?:\.(?:\d|_)+))(?:[eE][+-]?\d+)?")]
    LiteralFloat,

    #[regex(r#"(?:/(?:\\.|[^\\/])+/[a-zA-Z]*)"#)] // Regex literal
    #[regex(r#"(?:"(?:(?:[^"\\])|(?:\\.))*")"#)] // " string literal "
    #[regex(r#"(?:'(?:(?:[^'\\])|(?:\\.))*')"#)] // ' string literal '
    LiteralString,

    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Operator(Vec<Rule>),
    Symbol(Vec<Rule>),
    Keyword(Vec<Rule>),
    Identifier,
    Literal,
    EOL,
    EOI,
}
impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Operator(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Operator({inner})")
            }
            Category::Symbol(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("{r}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Symbol(`{inner}`)")
            }
            Category::Keyword(r) => {
                let inner = r
                    .iter()
                    .map(|r| format!("`{r}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "Keyword({inner})")
            }
            Category::Identifier => write!(f, "identifier"),
            Category::Literal => write!(f, "literal value"),
            Category::EOL => write!(f, "linebreak"),
            Category::EOI => write!(f, "end of input"),
        }
    }
}
impl Category {
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::EOI => Category::EOI,
            Rule::EOL => Category::EOL,

            Rule::LParen
            | Rule::RParen
            | Rule::LBrace
            | Rule::RBrace
            | Rule::LBrack
            | Rule::RBrack
            | Rule::Comma
            | Rule::Colon
            | Rule::Range
            | Rule::Dot
            | Rule::Question
            | Rule::Decorator => Category::Symbol(vec![rule]),

            Rule::Assign
            | Rule::AssignAdd
            | Rule::AssignSub
            | Rule::AssignPow
            | Rule::AssignMul
            | Rule::AssignDiv
            | Rule::AssignMod
            | Rule::AssignOr
            | Rule::AssignAnd
            | Rule::AssignXor
            | Rule::AssignSL
            | Rule::AssignSR
            | Rule::Inc
            | Rule::Dec
            | Rule::Add
            | Rule::Sub
            | Rule::Pow
            | Rule::Mul
            | Rule::Div
            | Rule::Mod
            | Rule::BitwiseNot
            | Rule::BitwiseOr
            | Rule::BitwiseAnd
            | Rule::Xor
            | Rule::SL
            | Rule::SR
            | Rule::LogicalOr
            | Rule::LogicalAnd
            | Rule::LogicalNot
            | Rule::SEq
            | Rule::SNe
            | Rule::Eq
            | Rule::Ne
            | Rule::Le
            | Rule::Ge
            | Rule::Lt
            | Rule::Gt
            | Rule::FatArrow => Category::Operator(vec![rule]),

            Rule::If
            | Rule::Then
            | Rule::Else
            | Rule::For
            | Rule::In
            | Rule::Do
            | Rule::Where
            | Rule::Switch
            | Rule::Return
            | Rule::Continue
            | Rule::Break
            | Rule::Delete
            | Rule::As
            | Rule::Contains
            | Rule::Matches
            | Rule::Is
            | Rule::StartsWith
            | Rule::EndsWith => Category::Keyword(vec![rule]),

            Rule::LiteralIdent => Category::Identifier,

            Rule::LiteralConstPi
            | Rule::LiteralConstE
            | Rule::LiteralConstTau
            | Rule::LiteralConstNil
            | Rule::LiteralConstTrue
            | Rule::LiteralConstFalse
            | Rule::LiteralU8
            | Rule::LiteralU16
            | Rule::LiteralU32
            | Rule::LiteralU64
            | Rule::LiteralI8
            | Rule::LiteralI16
            | Rule::LiteralI32
            | Rule::LiteralI64
            | Rule::LiteralDecimal
            | Rule::LiteralCurrency
            | Rule::LiteralFloat
            | Rule::LiteralString => Category::Literal,

            _ => return None,
        })
    }

    pub fn from_ruleset(rules: &[Rule]) -> Vec<Self> {
        let categories = rules
            .iter()
            .filter_map(|r| Category::from_rule(*r))
            .collect::<Vec<_>>();

        let mut symbols = std::collections::HashSet::new();
        let mut keywords = std::collections::HashSet::new();
        let mut operators = std::collections::HashSet::new();
        let mut set = std::collections::HashSet::new();

        for c in categories.into_iter() {
            match c {
                Category::Symbol(r) => symbols.extend(r),
                Category::Keyword(r) => keywords.extend(r),
                Category::Operator(r) => operators.extend(r),
                _ => {
                    set.insert(c);
                }
            }
        }

        let mut categories = vec![];
        if !symbols.is_empty() {
            categories.push(Category::Symbol(symbols.into_iter().collect()));
        }
        if !keywords.is_empty() {
            categories.push(Category::Keyword(keywords.into_iter().collect()));
        }
        if !operators.is_empty() {
            categories.push(Category::Operator(operators.into_iter().collect()));
        }
        categories.extend(set.into_iter());
        categories
    }

    pub fn many_to_string(this: &Vec<Self>) -> String {
        format!(
            "{}",
            this.iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub fn format_rules(this: &Vec<Rule>) -> String {
        let categories = Category::from_ruleset(this);
        Category::many_to_string(&categories)
    }
}

pub struct Tokenizer<'source>(logos::Lexer<'source, Rule>);
impl<'source> Tokenizer<'source> {
    pub fn new(input: &'source str) -> Self {
        Self(Rule::lexer_with_extras(input, 1))
    }

    pub fn consume_next(&mut self) -> Token<'source> {
        let token = self.0.next().unwrap_or_else(|| Ok(Rule::EOI));
        let input = self.0.source();
        Token {
            line: self.0.extras,
            span: self.0.span(),
            rule: token.unwrap_or_else(|_| Rule::Error),
            input: Cow::Borrowed(input),
        }
    }

    /// Consumes this iterator, returning all tokens
    pub fn all_tokens(mut self) -> Vec<Token<'source>> {
        let mut tokens = vec![];
        loop {
            tokens.push(self.consume_next());
            if tokens.last().unwrap().rule == Rule::EOI {
                break;
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
            let t = Tokenizer::new($input).all_tokens();
            println!("{:?}", t);
            assert_eq!($tokens, t.into_iter().map(|t| t.rule).collect::<Vec<_>>());
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
