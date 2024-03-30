use super::*;
use crate::{lexer::Rule, IntoOwned};

pratt_node!(BitwiseInfixExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::BitwiseInfixExpr);
        let op = op.token().rule();
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    into_node(this) {
        Node::BitwiseInfixExpr(Box::new(this))
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

pratt_node!(BitwiseNotNode(rhs: Node<'source>) {
    build(token, rhs, _op) {
        token.set_rule(Rule::BitwiseNot);
        Some(Self { rhs, token }.into_node())
    }

    into_node(this) {
        Node::BitwiseNot(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            rhs: this.rhs.into_owned(),
            token: this.token.into_owned(),
        }
    }
});
