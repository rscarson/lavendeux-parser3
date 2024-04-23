use crate::{
    lexer::Rule::{self, *},
    parser::{
        arithmetic::*,
        assignment::*,
        bitwise::*,
        boolean::*,
        collections::{IndexingExprNode, RangeExprNode},
        conditionals::TernaryExprNode,
        core::{CastExprNode, DecoratorExprNode},
        functions::FnCallNode,
        Node,
    },
};

use lazy_static::lazy_static;
use std::collections::HashMap;

macro_rules! bind {
    ($($rule:ident)|+ => Infix::Left, $map:ident :: $cur:ident) => {{
        let v = [$(
            ($rule, ($cur+1, $cur)),
        )+];
        $cur += 2;
        $map.extend(v.into_iter());
    }};
    ($($rule:ident)|+ => Infix::Right, $map:ident :: $cur:ident) => {{
        let v = [$(
            ($rule, ($cur, $cur+1)),
        )+];
        $cur += 2;
        $map.extend(v.into_iter());
    }};
    ($($rule:ident)|+ => Prefix, $map:ident :: $cur:ident) => {{
        let v = [$(
            ($rule, ($cur, 0)),
        )+];
        $cur += 1;
        $map.extend(v.into_iter());
    }};
    ($($rule:ident)|+ => Postfix, $map:ident :: $cur:ident) => {{
        let v = [$(
            ($rule, (0, $cur)),
        )+];
        $cur += 1;
        $map.extend(v.into_iter());
    }};
}

/// Attempts to use the Pratt parsing algorithm to parse an expression of the form
/// prefix_op? ~ EOL* ~ TERM ~ postfix_operation* ~ ( EOL* ~ infix_op ~ prefix_op? ~ EOL* ~ TERM ~ postfix_operation*)*
/// Binding power is given by priority_of(rule), and the tuple is (left, right) binding power
/// Where a higher binding power means a higher precedence
///
/// It will attempt to create an AST from the expression
pub fn fold_expression<'source>(
    expr: &mut Vec<Node<'source>>,
    min_bp: u8,
) -> Option<Node<'source>> {
    // Get the left hand side of the expression
    // It's either a term or a prefix operator
    let mut lhs = match expr.pop() {
        Some(lhs) => lhs,
        None => return None,
    };

    // Check if the left hand side is a prefix operator
    match priority_of(lhs.token().rule()) {
        Some((left_bp, 0)) if left_bp > 0 => {
            // Prefix
            let rhs = fold_expression(expr, left_bp)?;
            lhs = build_pratt_unary(rhs, lhs)?;
        }

        None => { /* Do nothing - term */ }

        // Postfix operator in wrong spot
        _ => unreachable!("Unregistered operator: {:?}", lhs.token().rule()),
    }

    loop {
        // Get the operator
        let op = match expr.last() {
            Some(op) => op,
            None => break,
        };
        let op_rule = op.token().rule();

        // Get the binding power of the operator
        // Check if it's a postfix operator
        let (left_bp, right_bp) = priority_of(op_rule)
            .or_else(|| unreachable!("Unregistered operator: {:?}", op_rule))?;
        match (left_bp, right_bp) {
            (0, right_bp) => {
                // Postfix
                if right_bp < min_bp {
                    break;
                }
                let op = expr.pop().unwrap();
                lhs = build_pratt_unary(lhs, op)?;
                continue;
            }

            // Prefix operator in wrong spot
            (_, 0) => unreachable!("Unregistered operator: {:?}", op_rule),

            _ => { /* Do nothing - infix */ }
        }

        if left_bp < min_bp {
            break;
        }

        // Build the expression
        let op = expr.pop().unwrap();
        let rhs = fold_expression(expr, right_bp)?;
        lhs = build_pratt_binary(lhs, op, rhs)?;
    }

    Some(lhs)
}

/// Convert a set of nodes into a single infix expression node
#[inline(always)]
pub fn build_pratt_binary<'source>(
    lhs: Node<'source>,
    op: Node<'source>,
    rhs: Node<'source>,
) -> Option<Node<'source>> {
    use crate::lexer::Rule::*;
    let token = lhs.token().child(
        op.token().rule(),
        lhs.token().span().start..rhs.token().span().end,
    );
    match op.token().rule() {
        Add | Sub | Mul | Div | Mod | Pow => ArithmeticInfixExprNode::parse(token, lhs, op, rhs),

        SL | SR | BitwiseOr | BitwiseAnd | Xor => BitwiseInfixExprNode::parse(token, lhs, op, rhs),
        Eq | Ne | SEq | SNe | Lt | Gt | Le | Ge => ComparisonExprNode::parse(token, lhs, op, rhs),
        LogicalAnd | LogicalOr => LogicalExprNode::parse(token, lhs, op, rhs),

        Matches | Contains | StartsWith | EndsWith => MatchExprNode::parse(token, lhs, op, rhs),

        Range => RangeExprNode::parse(token, lhs, op, rhs),

        Assign => AssignExprNode::parse(token, lhs, op, rhs),
        AssignAdd | AssignSub | AssignPow | AssignMod | AssignMul | AssignDiv => {
            AssignArithmeticExprNode::parse(token, lhs, op, rhs)
        }
        AssignAnd | AssignOr | AssignXor | AssignSL | AssignSR => {
            AssignBitwiseExprNode::parse(token, lhs, op, rhs)
        }

        TernaryOperator => TernaryExprNode::parse(token, lhs, op, rhs),
        DecoratorOperator => DecoratorExprNode::parse(token, lhs, op, rhs),

        As => CastExprNode::parse(token, lhs, op, rhs),

        _ => unreachable!("Unregistered operator: {:?}", op.token().rule()),
    }
}

/// Convert a set of nodes into a single unary expression node
#[inline(always)]
pub fn build_pratt_unary<'source>(term: Node<'source>, op: Node<'source>) -> Option<Node<'source>> {
    use crate::lexer::Rule::*;
    let mut token = op.token().clone();
    token.include_span(term.token().span());
    match op.token().rule() {
        Delete => DeleteExprNode::parse(token, term, op),

        PrefixNeg => ArithmeticPrefixExprNode::parse(token, term, op),
        LogicalNot => LogicalNotNode::parse(token, term, op),
        BitwiseNot => BitwiseNotNode::parse(token, term, op),

        FnCallOperator => FnCallNode::parse(token, term, op),
        IndexingOperator => IndexingExprNode::parse(token, term, op),

        _ => unreachable!("Unregistered operator: {:?}", op.token().rule()),
    }
}

/// Get the binding power of a rule
/// Higher binding power means higher precedence
/// Returns None if the rule is not in the map
/// The tuple is (left, right) binding power
fn priority_of(rule: Rule) -> Option<(u8, u8)> {
    PRATT_PRIORITY.get(&rule).cloned()
}
lazy_static! {
    pub static ref PRATT_PRIORITY: HashMap<Rule, (u8, u8)> = {
        let mut cur = 1;
        let mut map: Vec<(Rule, (u8, u8))> = vec![];

        bind!(
            Assign
            |AssignAdd|AssignSub|AssignPow|AssignMod|AssignMul|AssignDiv
            |AssignAnd|AssignOr|AssignXor|AssignSL|AssignSR
        => Infix::Right, map::cur);

        bind!(Delete => Prefix, map::cur);
        bind!(Range => Infix::Left, map::cur);
        bind!(TernaryOperator => Infix::Right, map::cur);
        bind!(DecoratorOperator => Postfix, map::cur);

        bind!(LogicalOr => Infix::Left, map::cur);
        bind!(LogicalAnd => Infix::Left, map::cur);

        bind!(Matches|Contains|StartsWith|EndsWith => Infix::Left, map::cur);

        bind!(BitwiseOr => Infix::Left, map::cur);
        bind!(Xor => Infix::Left, map::cur);
        bind!(BitwiseAnd => Infix::Left, map::cur);

        bind!(Eq|Ne|SEq|SNe => Infix::Left, map::cur);
        bind!(Lt|Gt|Le|Ge => Infix::Left, map::cur);

        bind!(SL|SR => Infix::Left, map::cur);

        bind!(Add|Sub => Infix::Left, map::cur);
        bind!(Mul|Div|Mod => Infix::Left, map::cur);
        bind!(Pow => Infix::Right, map::cur);

        bind!(PrefixNeg|BitwiseNot|LogicalNot => Prefix, map::cur);
        bind!(FnCallOperator|IndexingOperator => Postfix, map::cur);

        bind!(As => Infix::Right, map::cur);

        map.into_iter().collect()
    };
}
