use crate::{tokenizer::Token, IntoOwned};

/// Attempt to match a single token
/// Examples:
/// ```rust
/// terminal!(ConstLiteral, tokens)?; // Match exactly one token
/// terminal!(ConstLiteral?, tokens)?; // Match 0 or one token
/// terminal!(ConstLiteral*, tokens)?; // Match 0 or more token
/// terminal!(ConstLiteral+, tokens)?; // Match 1 or more token
/// ````
macro_rules! terminal {
    (& $type:ident $(| $($subtype:ident)|+)?, $tokens:expr) => {
        $tokens.try_peek_a(&[$crate::tokenizer::Rule::$type $(, $($crate::tokenizer::Rule::$subtype,)+)?]).cloned()
    };
    ($type:ident $(| $($subtype:ident)|+)? ?, $tokens:expr) => {
        match terminal!(&$type $(| $($subtype)|+)?, $tokens).ok() {
            Some(_) => $tokens.pop(),
            None => None
        }
    };
    ($type:ident $(| $($subtype:ident)|+)? *, $tokens:expr) => {{
        let mut v = Vec::new();
        while let Some(t) = terminal!($type$(| $($subtype)|+)? ?, $tokens) {
            v.push(t)
        }
        v
    }};
    ($type:ident $(| $($subtype:ident)|+)? +, $tokens:expr) => {
        match terminal!($type $(| $($subtype)|+)?, $tokens) {
            Ok(t) => {
                let mut v = vec![t];
                while let Some(t) = terminal!($type $(| $($subtype)|+)? ?, $tokens) {
                    v.push(t)
                }
                Ok(v)
            }
            Err(e) =>  Err(e)
        }
    };
    ($type:ident $(| $($subtype:ident)|+)?, $tokens:expr) => {
        match $tokens.try_pop_a(&[$crate::tokenizer::Rule::$type $(, $($crate::tokenizer::Rule::$subtype,)+)?]) {
            Ok(t) => Ok(t),
            Err(e) => {
                Err(e)
            }
        }
    };
}

/// Skip all EOL tokens
macro_rules! skip_eol {
    ($tokens:expr) => {
        terminal!(EOL*, $tokens)
    };
}

macro_rules! build_nt {
    ($type:ident, $tokens:expr) => {{
        match $tokens.len() == 0 {
            true => Err($crate::error::Error::UnexpectedEndOfInput),
            _ => {
                #[cfg(feature = "debug_compiler_internal")]
                println!(
                    "{}Parsing {}: next={:?}",
                    "  ".repeat($tokens.depth()),
                    stringify!($type),
                    $tokens.peek()
                );
                $type::parse($tokens)
            }
        }
    }};
}

/// Attempt to match a NT
/// Examples:
/// ```rust
/// non_terminal!(Expression, tokens)?; // Match exactly one expression
/// non_terminal!(Expression?, tokens)?; // Match 0 or one expression
/// non_terminal!(Expression*, tokens)?; // Match 0 or more expression
/// non_terminal!(Expression+, tokens)?; // Match 1 or more expression
/// ````
macro_rules! non_terminal {
    (! $type:ident, $tokens:expr) => {{
        match non_terminal!($type?, $tokens) {
            None => Ok(()),
            Some(nt) => Err(Error::Syntax {
                expected: vec![],
                unexpected: vec![nt.token()]
            })
        }
    }};
    ($type:ident ?, $tokens:expr) => {
        build_nt!($type, $tokens).ok()
    };
    ($type:ident *, $tokens:expr) => {{
        let mut v = Vec::new();
        while let Some(t) = non_terminal!($type?, $tokens) {
            v.push(t)
        }
        v
    }};
    ($type:ident +, $tokens:expr) => {
        match non_terminal!($type, $tokens) {
            Ok(t) => {
                let mut v = vec![t];
                while let Some(t) = non_terminal!($type?, $tokens) {
                    v.push(t)
                }
                Ok(v)
            }
            Err(e) =>  Err(e)
        }
    };
    ($type:ident  $(| $($subtype:ident)|+)?, $tokens:expr) => {
        match build_nt!($type, $tokens) {
            Ok(t) => Ok(t),
            Err(e) => {
                #[allow(unused_mut)]
                let mut result = Err(e);
                $(
                    'block: {
                        if !matches!(result, Err(crate::error::Error::Syntax { .. })) {
                            // If it's not a syntax error, break
                            break 'block;
                        }

                        $(
                            match build_nt!($subtype, $tokens) {
                                Ok(t) => {
                                    result = Ok(t);
                                    break 'block;
                                },
                                Err(e) => {
                                    result = Err(e);
                                    if !matches!(result, Err(crate::error::Error::Syntax { .. })) {
                                        // If it's not a syntax error, break
                                        break 'block;
                                    }
                                }
                            }
                        )+
                    }
                )?

                if result.is_err() {
                    $tokens.revert_transaction();
                }

                result
            }
        }
    };
}

/// Main trait for the AST's nodes
/// Covers parsing, execution, reconstitution, and eventually JS transpilation
pub trait ParserNode<'source>
where
    Self: IntoOwned,
{
    fn into_node(self) -> crate::parser::Node<'source>;
    fn parse(
        tokens: &mut crate::Stack<'source>,
    ) -> Result<crate::parser::Node<'source>, crate::error::Error>;
}

macro_rules! define_node {
    (
        $name:ident ( $($an:ident : $at:ty),* $(,)?) {
            build($bstack_arg:ident) $bblock:block
            into_node($nselfarg:ident) $nblock:block
            into_owned($oselfarg:ident) $oblock:block
        }
    ) => {
        #[derive(Clone, Debug)]
        pub struct $name<'source> {
            $(pub $an: $at,)*
            pub token: $crate::tokenizer::Token<'source>,
        }
        impl<'source> IntoOwned for $name<'source> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                let $oselfarg = self;
                $oblock
            }
        }
        impl<'source> $crate::parser::ParserNode<'source> for $name<'source> {
            fn into_node(self) -> $crate::parser::Node<'source> {
                let $nselfarg = self;
                $nblock
            }
            fn parse($bstack_arg: &mut $crate::Stack<'source>) -> Result<$crate::parser::Node<'source>, $crate::error::Error> $bblock
        }
    };
}

macro_rules! pratt_node {
    (
        $name:ident ( $($an:ident : $at:ty),* $(,)?) {
            build($bt_arg:ident, $bl_arg:ident, $bo_arg:ident $(, $br_arg:ident)?) $bblock:block
            into_node($nselfarg:ident) $nblock:block
            into_owned($oselfarg:ident) $oblock:block
        }
    ) => {
        #[derive(Clone, Debug)]
        pub struct $name<'source> {
            $(pub $an: $at,)*
            pub token: $crate::tokenizer::Token<'source>,
        }
        impl<'source> IntoOwned for $name<'source> {
            type Owned = $name<'static>;
            fn into_owned(self) -> Self::Owned {
                let $oselfarg = self;
                $oblock
            }
        }
        impl<'source> $name<'source> {
            pub fn into_node(self) -> $crate::parser::Node<'source> {
                let $nselfarg = self;
                $nblock
            }
            pub fn parse(mut $bt_arg: Token<'source>, $bl_arg: Node<'source>, $bo_arg: Node<'source>$(, $br_arg: Node<'source>)?) -> Result<$crate::parser::Node<'source>, $crate::error::Error> $bblock
        }
    };
}

macro_rules! pratt_node_silent {
    (
        $name:ident {
            build($bt_arg:ident, $bl_arg:ident, $bo_arg:ident $(, $br_arg:ident)?) $bblock:block
        }
    ) => {
        #[derive(Clone, Debug)]
        pub struct $name { }
        impl $name {
            pub fn parse<'source>(mut $bt_arg: Token<'source>, $bl_arg: Node<'source>, $bo_arg: Node<'source>$(, $br_arg: Node<'source>)?) -> Result<$crate::parser::Node<'source>, $crate::error::Error> $bblock
        }
    };
}

macro_rules! node_silent {
    (
        $name:ident($bstack_arg:ident) $bblock:block
    ) => {
        define_node!(
            $name() {
                build($bstack_arg) $bblock
                into_node(_this) {
                    unimplemented!("Node {} cannot be built directly", stringify!($name));
                }
                into_owned(_this) {
                    unimplemented!("Node {} cannot be built directly", stringify!($name));
                }
            }
        );
    };
}

macro_rules! define_parser {
    ($($name:ident : $src:ident),+ $(,)?) => {
        #[derive(Clone)]
        pub enum Node<'source> {
            $(
                $name(Box<$src<'source>>),
            )+
        }

        impl<'source> Node<'source> {
            pub fn token(&self) -> &Token<'source> {
                match self {
                    $(
                        Self::$name(n) => &n.token,
                    )+
                }
            }
        }

        impl<'source> IntoOwned for Node<'source> {
            type Owned = Node<'static>;
            fn into_owned(self) -> Self::Owned {
                match self {
                    $(
                        Self::$name(n) => Self::Owned::$name(Box::new(n.into_owned())),
                    )+
                }
            }
        }

        impl std::fmt::Debug for Node<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$name(n) => write!(f, "{:?}", n),
                    )+
                }
            }
        }
    };
}

pub mod core;
use core::*;

pub mod expr;
use expr::*;

pub mod functions;
use functions::*;

pub mod arithmetic;
use arithmetic::*;

pub mod literals;
use literals::*;

pub mod iterators;
use iterators::*;

pub mod collections;
use collections::*;

pub mod conditionals;
use conditionals::*;

pub mod bitwise;
use bitwise::*;

pub mod boolean;
use boolean::*;

pub mod assignment;
use assignment::*;

define_parser!(
    // Core lang nodes
    Script: ScriptNode,
    Line: LineNode,
    Block: BlockNode,
    CastExpr: CastExprNode,
    DecoratorExpr: DecoratorExprNode,

    // Assignment nodes
    AssignExpr: AssignExprNode,
    AssignArithmeticExpr: AssignArithmeticExprNode,
    AssignBitwiseExpr: AssignBitwiseExprNode,
    DeleteExpr: DeleteExprNode,

    // Expression generics and pratt nodes
    Expression: ExpressionNode,
    PrefixOperator: PrefixOperatorNode,
    InfixOperator: InfixOperatorNode,
    PostfixIndexingOperator: PostfixIndexingOperatorNode,
    PostfixFnCallOperator: PostfixFnCallOperatorNode,
    PostfixIncDecOperator: PostfixIncDecOperatorNode,

    // Arithmetic nodes
    ArithmeticInfixExpr: ArithmeticInfixExprNode,
    ArithmeticPrefixExpr: ArithmeticPrefixExprNode,
    ArithmeticPostfixExpr: ArithmeticPostfixExprNode,

    // Bit and bool nodes
    BitwiseNot: BitwiseNotNode,
    LogicalNot: LogicalNotNode,
    BitwiseInfixExpr: BitwiseInfixExprNode,
    ComparisonExpr: ComparisonExprNode,
    LogicalExpr: LogicalExprNode,
    MatchExpr: MatchExprNode,

    // Function related nodes
    FnCall: FnCallNode,
    FnAssign: FnAssignNode,
    Return: ReturnNode,

    // Iterator related nodes
    Continue: ContinueNode,
    Break: BreakNode,
    Switch: SwitchNode,
    For: ForNode,

    // Collection related nodes
    Array: ArrayNode,
    Object: ObjectNode,
    RangeExpr: RangeExprNode,
    IndexingExpr: IndexingExprNode,

    // Conditional related nodes
    If: IfNode,

    // Literals and constants
    LiteralString: LiteralStringNode,
    LiteralFloat: LiteralFloatNode,
    LiteralBool: LiteralBoolNode,
    LiteralCurrency: LiteralCurrencyNode,
    LiteralDecimal: LiteralDecimalNode,
    LiteralI64: LiteralI64Node,
    LiteralI32: LiteralI32Node,
    LiteralI16: LiteralI16Node,
    LiteralI8: LiteralI8Node,
    LiteralU64: LiteralU64Node,
    LiteralU32: LiteralU32Node,
    LiteralU16: LiteralU16Node,
    LiteralU8: LiteralU8Node,
    LiteralIdent: LiteralIdentNode,
);
