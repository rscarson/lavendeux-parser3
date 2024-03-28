use super::*;
use crate::{tokenizer::Rule, IntoOwned};

pratt_node!(LogicalExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::LogicalExpr);
        let op = op.token().rule();
        Ok(Self { lhs, op, rhs, token }.into_node())
    }

    into_node(this) {
        Node::LogicalExpr(Box::new(this))
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

pratt_node!(ComparisonExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::ComparisonExpr);
        let op = op.token().rule();
        Ok(Self { lhs, op, rhs, token }.into_node())
    }

    into_node(this) {
        Node::ComparisonExpr(Box::new(this))
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

pratt_node!(MatchExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::MatchExpr);
        let op = op.token().rule();
        Ok(Self { lhs, op, rhs, token }.into_node())
    }

    into_node(this) {
        Node::MatchExpr(Box::new(this))
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

pratt_node!(LogicalNotNode(rhs: Node<'source>) {
    build(token, rhs, _op) {
        token.set_rule(Rule::LogicalNot);
        Ok(Self { rhs, token }.into_node())
    }

    into_node(this) {
        Node::LogicalNot(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            rhs: this.rhs.into_owned(),
            token: this.token.into_owned(),
        }
    }
});
