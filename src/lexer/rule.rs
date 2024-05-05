use logos::{Logos, Skip};
use serde::{Deserialize, Serialize};

use crate::traits::SerializeToBytes;

/// Main lexer rule set for the language
#[allow(missing_docs)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Logos, Serialize, Deserialize,
)]
#[logos(skip r"[ \t\r\f]+")]
#[logos(subpattern currency_symbol = r"[$¢£¤¥֏؋߾߿৲৳৻૱௹฿៛₠₡₢₣₤₥₦₧₨₩₪₫€₭₮₯₰₱₲₳₴₵₶₷₸₹₺₻₼₽₾₿꠸﷼﹩＄￠￡￥￦]")]
#[logos(extras = usize)]
#[repr(u16)]
#[derive(strum_macros::Display)]
pub enum Rule {
    #[strum(to_string = "end of input")]
    EOI,

    #[token("\\\n", callback = |lex| {
        lex.extras += 1;
        Skip
    })]
    #[token("\\\r\n", callback = |lex| {
        lex.extras += 1;
        Skip
    })]
    SkippedEOL,

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
    #[regex("\n", |l| {l.extras += 1})]
    #[regex(";")]
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
    #[regex(r"//#[^\n]*")]
    DocBlockComment,
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
    #[strum(to_string = "starts_with")]
    #[regex("starts_with|startswith")]
    StartsWith,
    #[strum(to_string = "ends_with")]
    #[regex("ends_with|endswith")]
    EndsWith,

    #[strum(to_string = "ref")]
    #[token("ref")]
    Reference,

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

    #[regex(r"0[a-zA-Z][a-zA-Z0-9]+")]
    LiteralRadix,

    #[regex(r"[0-9](\d|_)*", priority = 2)]
    LiteralInt,

    #[regex(
        r"(([1-9](?:\d|_)*(\.(?:\d|_)+)?)|(\.(?:\d|_)+))((?&currency_symbol))",
        priority = 3
    )]
    LiteralSuffixedCurrency,

    #[regex(
        r"(?&currency_symbol)(([1-9](?:\d|_)*(\.(?:\d|_)+)?)|(\.(?:\d|_)+))",
        priority = 3
    )]
    LiteralPrefixedCurrency,

    #[regex(
        r"(?:(([0-9](?:\d|_)*(\.(?:\d|_)+))|(\.(?:\d|_)+)))(?:[eE][+-]?\d+)?",
        priority = 3
    )]
    LiteralFloat,

    #[regex(r#"r"([^"\\]+|\\.)*"[a-zA-Z]*"#, priority = 1)] // Regex literal "
    #[regex(r#"r'([^'\\]+|\\.)*'[a-zA-Z]*"#, priority = 1)] // Regex literal '
    LiteralRegex,

    #[regex(r#""([^"\\]+|\\.)*""#)] // " string literal "
    #[regex(r#"'([^'\\]+|\\.)*'"#)] // " string literal '
    LiteralString,

    Error,
}

impl SerializeToBytes for Rule {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(self as u16).serialize_into_bytes());
        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let rule = u16::deserialize_from_bytes(bytes)?;
        if rule > Rule::Error as u16 {
            return Err(crate::traits::ByteDecodeError::MalformedData(
                "Rule".to_string(),
                format!("No rule with ID #{}", rule),
            ));
        }

        let rule = unsafe { std::mem::transmute::<u16, Rule>(rule) };
        Ok(rule)
    }
}
