use super::*;
use crate::{lexer::Rule, traits::IntoOwned, vm::OpCode};

/// Bitwise infix expression
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum BitwiseOp {
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
}
impl BitwiseOp {
    /// Convert a rule to a bitwise operator
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::BitwiseAnd => BitwiseOp::And,
            Rule::BitwiseOr => BitwiseOp::Or,
            Rule::Xor => BitwiseOp::Xor,
            Rule::SL => BitwiseOp::ShiftLeft,
            Rule::SR => BitwiseOp::ShiftRight,
            _ => return None,
        })
    }
}

pratt_node!(BitwiseInfixExprNode(lhs: Node<'source>, op: BitwiseOp, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::BitwiseInfixExpr);
        let op = BitwiseOp::from_rule(op.token().rule())?;
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.lhs.compile(compiler)?;
        this.rhs.compile(compiler)?;
        compiler.push(match this.op {
            BitwiseOp::And => OpCode::AND,
            BitwiseOp::Or => OpCode::OR,
            BitwiseOp::Xor => OpCode::XOR,
            BitwiseOp::ShiftLeft => OpCode::SHL,
            BitwiseOp::ShiftRight => OpCode::SHR,
        });

        Ok(())
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

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.rhs.compile(compiler)?;
        compiler.push(OpCode::NOT);

        Ok(())
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
