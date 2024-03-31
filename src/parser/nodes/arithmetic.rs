use super::*;
use crate::{lexer::Rule, IntoOwned};

pratt_node!(ArithmeticInfixExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    "Infix arithmetic expression"
    "`EXPR (+, -, *, /, %, **) EXPR`"

    build(token, lhs, op, rhs) {
        token.set_rule(Rule::ArithmethicInfixExpr);
        let op = op.token().rule();
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    into_node(this) {
        Node::ArithmeticInfixExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            lhs: this.lhs.into_owned(),
            op: this.op,
            rhs: this.rhs.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(ArithmeticPrefixExprNode(rhs: Node<'source>, op: Rule) {
    "Prefix arithmetic expression"
    "`(++, --, -) EXPR`"

    build(token, rhs, op) {
        token.set_rule(Rule::ArithmeticPrefixExpr);
        let op = op.token().rule();
        Some(Self { rhs, op, token }.into_node())
    }

    into_node(this) {
        Node::ArithmeticPrefixExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            rhs: this.rhs.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(ArithmeticPostfixExprNode(lhs: Node<'source>, op: Rule) {
    "Postfix arithmetic expression"
    "`EXPR (++ | --)`"

    build(token, lhs, op) {
        token.set_rule(Rule::ArithmeticPostfixExpr);
        let op = op.token().rule();
        Some(Self { lhs, op, token }.into_node())
    }

    into_node(this) {
        Node::ArithmeticPostfixExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            lhs: this.lhs.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});
