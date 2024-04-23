use super::*;
use crate::{lexer::Rule, traits::IntoOwned, vm::OpCode};

/// Arithmetic infix expression
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}
impl ArithmeticOp {
    /// Convert a rule to an arithmetic operator
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::Add => ArithmeticOp::Add,
            Rule::Sub => ArithmeticOp::Sub,
            Rule::Mul => ArithmeticOp::Mul,
            Rule::Div => ArithmeticOp::Div,
            Rule::Mod => ArithmeticOp::Mod,
            Rule::Pow => ArithmeticOp::Pow,
            _ => return None,
        })
    }
}

pratt_node!(ArithmeticInfixExprNode(lhs: Node<'source>, op: ArithmeticOp, rhs: Node<'source>) {
    "Infix arithmetic expression"
    "`EXPR (+, -, *, /, %, **) EXPR`"

    build(token, lhs, op, rhs) {
        token.set_rule(Rule::ArithmethicInfixExpr);
        let op = ArithmeticOp::from_rule(op.token().rule())?;
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.lhs.compile(compiler)?;
        this.rhs.compile(compiler)?;
        compiler.push(match this.op {
            ArithmeticOp::Add => OpCode::ADD,
            ArithmeticOp::Sub => OpCode::SUB,
            ArithmeticOp::Mul => OpCode::MUL,
            ArithmeticOp::Div => OpCode::DIV,
            ArithmeticOp::Mod => OpCode::REM,
            ArithmeticOp::Pow => OpCode::POW,
        });

        Ok(())
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

pratt_node!(ArithmeticPrefixExprNode(rhs: Node<'source>) {
    "Prefix arithmetic expression"
    "`(-) EXPR`"

    build(token, rhs, _op) {
        token.set_rule(Rule::ArithmeticPrefixExpr);
        Some(Self { rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.rhs.compile(compiler)?;
        compiler.push(OpCode::NEG);
        Ok(())
    }

    into_node(this) {
        Node::ArithmeticPrefixExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            rhs: this.rhs.into_owned(),
            token: this.token.into_owned(),
        }
    }
});
