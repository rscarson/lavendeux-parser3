#![allow(missing_docs)]

pub mod arithmetic;
pub mod assignment;
pub mod bitwise;
pub mod boolean;
pub mod collections;
pub mod conditionals;
pub mod core;
pub mod expr;
pub mod functions;
pub mod iterators;
pub mod literals;

use arithmetic::*;
use assignment::*;
use bitwise::*;
use boolean::*;
use collections::*;
use conditionals::*;
use core::*;
use expr::*;
use functions::*;
use iterators::*;
use literals::*;

use super::traits::ParserNode;

define_parser!(
    // Core lang nodes
    Script: ScriptNode,
    Block: BlockNode,
    CastExpr: CastExprNode,
    DecoratorExpr: DecoratorExprNode,

    // Assignment nodes
    AssignExpr: AssignExprNode,
    AssignArithmeticExpr: AssignArithmeticExprNode,
    AssignBitwiseExpr: AssignBitwiseExprNode,
    DeleteExpr: DeleteExprNode,

    // Expression generics and pratt nodes
    PrefixOperator: PrefixOperatorNode,
    InfixOperator: InfixOperatorNode,
    PostfixIndexingOperator: PostfixIndexingOperatorNode,
    PostfixFnCallOperator: PostfixFnCallOperatorNode,

    // Arithmetic nodes
    ArithmeticInfixExpr: ArithmeticInfixExprNode,
    ArithmeticPrefixExpr: ArithmeticPrefixExprNode,

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
    LiteralInt: LiteralIntNode,
    LiteralIdent: LiteralIdentNode,
);
