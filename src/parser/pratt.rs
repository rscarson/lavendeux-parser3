use crate::{
    lexer::Rule,
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
    PRATT_PRIORITY[rule as usize]
}

fn generate_pratttable() -> Vec<Option<(u8, u8)>> {
    struct Precedence(u8);
    impl Precedence {
        fn lhs(&mut self) -> (u8, u8) {
            (self.0, self.0 + 1)
        }

        fn rhs(&mut self) -> (u8, u8) {
            (self.0, self.0 + 1)
        }

        fn prefix(&mut self) -> (u8, u8) {
            (self.0 + 1, 0)
        }

        fn postfix(&mut self) -> (u8, u8) {
            (0, self.0 + 1)
        }

        fn advance(&mut self) {
            self.0 += 1;
        }
    }

    macro_rules! bind {
        ($table:ident, $precedence:ident :: Right => $($rule:ident)|+) => {
            $( $table[Rule::$rule as usize] = Some($precedence.rhs()); )+
            $precedence.advance();
        };
        ($table:ident, $precedence:ident :: Left => $($rule:ident)|+) => {
            $( $table[Rule::$rule as usize] = Some($precedence.lhs()); )+
            $precedence.advance();
        };
        ($table:ident, $precedence:ident :: Prefix => $($rule:ident)|+) => {
            $( $table[Rule::$rule as usize] = Some($precedence.prefix()); )+
            $precedence.advance();
        };
        ($table:ident, $precedence:ident :: Postfix => $($rule:ident)|+) => {
            $( $table[Rule::$rule as usize] = Some($precedence.postfix()); )+
            $precedence.advance();
        };
    }

    let mut precedence = Precedence(1);
    let mut table = Vec::with_capacity(Rule::Error as usize);
    table.resize(Rule::Error as usize, None);

    bind!(table, precedence::Right => Assign|AssignAdd|AssignSub|AssignPow|AssignMod|AssignMul|AssignDiv|AssignAnd|AssignOr|AssignXor|AssignSL|AssignSR);

    bind!(table, precedence::Prefix => Delete);
    bind!(table, precedence::Right => Range);
    bind!(table, precedence::Right => TernaryOperator);
    bind!(table, precedence::Postfix => DecoratorOperator);

    bind!(table, precedence::Left => LogicalOr);
    bind!(table, precedence::Left => LogicalAnd);

    bind!(table, precedence::Left => Matches|Contains|StartsWith|EndsWith);

    bind!(table, precedence::Left => BitwiseOr);
    bind!(table, precedence::Left => Xor);
    bind!(table, precedence::Left => BitwiseAnd);

    bind!(table, precedence::Left => Eq|Ne|SEq|SNe);
    bind!(table, precedence::Left => Lt|Gt|Le|Ge);

    bind!(table, precedence::Left => SL|SR);

    bind!(table, precedence::Left => Add|Sub);
    bind!(table, precedence::Left => Mul|Div|Mod);
    bind!(table, precedence::Right => Pow);

    bind!(table, precedence::Prefix => PrefixNeg|BitwiseNot|LogicalNot);
    bind!(table, precedence::Postfix => FnCallOperator|IndexingOperator);

    bind!(table, precedence::Right => As);

    table
}

lazy_static::lazy_static! {
    pub static ref PRATT_PRIORITY: Vec<Option<(u8, u8)>> = generate_pratttable();
}
