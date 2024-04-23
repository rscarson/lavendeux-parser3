use super::*;
use crate::{lexer::Rule, traits::IntoOwned, vm::OpCode};

/// Logical operators
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum LogicalOp {
    And,
    Or,
}

pratt_node!(LogicalExprNode(lhs: Node<'source>, op: LogicalOp, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::LogicalExpr);
        let op = match op.token().rule() {
            Rule::LogicalAnd => LogicalOp::And,
            Rule::LogicalOr => LogicalOp::Or,
            _ => unreachable!("Invalid logical operator"),
        };
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.lhs.compile(compiler)?;
        this.rhs.compile(compiler)?;
        compiler.push(match this.op {
            LogicalOp::And => OpCode::LAND,
            LogicalOp::Or => OpCode::LOR,
        });

        Ok(())
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

/// Comparison operators
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
    SEq,
    SNeq,
}
impl ComparisonOp {
    /// Convert a rule to a comparison operator
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::Eq => ComparisonOp::Eq,
            Rule::Ne => ComparisonOp::Neq,
            Rule::Gt => ComparisonOp::Gt,
            Rule::Lt => ComparisonOp::Lt,
            Rule::Ge => ComparisonOp::Gte,
            Rule::Le => ComparisonOp::Lte,
            Rule::SEq => ComparisonOp::SEq,
            Rule::SNe => ComparisonOp::SNeq,
            _ => return None,
        })
    }
}

pratt_node!(ComparisonExprNode(lhs: Node<'source>, op: ComparisonOp, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::ComparisonExpr);
        let op = ComparisonOp::from_rule(op.token().rule())?;
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.lhs.compile(compiler)?;
        this.rhs.compile(compiler)?;
        compiler.push(match this.op {
            ComparisonOp::Eq => OpCode::EQ,
            ComparisonOp::Neq => OpCode::NE,
            ComparisonOp::Gt => OpCode::GT,
            ComparisonOp::Lt => OpCode::LT,
            ComparisonOp::Gte => OpCode::GE,
            ComparisonOp::Lte => OpCode::LE,
            ComparisonOp::SEq => OpCode::SEQ,
            ComparisonOp::SNeq => OpCode::SNE,
        });

        Ok(())
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

/// Matching operators
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
pub enum MatchingOp {
    Matches,
    StartsWith,
    EndsWith,
    Contains,
}
impl MatchingOp {
    /// Convert a rule to a matching operator
    pub fn from_rule(rule: Rule) -> Option<Self> {
        Some(match rule {
            Rule::Matches => Self::Matches,
            Rule::StartsWith => Self::StartsWith,
            Rule::EndsWith => Self::EndsWith,
            Rule::Contains => Self::Contains,
            _ => return None,
        })
    }
}

pratt_node!(MatchExprNode(lhs: Node<'source>, op: Rule, rhs: Node<'source>) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::MatchExpr);
        let op = op.token().rule();
        Some(Self { lhs, op, rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.lhs.compile(compiler)?;
        this.rhs.compile(compiler)?;
        compiler.push(match this.op {
            Rule::Matches => OpCode::MTCH,
            Rule::Contains => OpCode::CNTN,
            Rule::StartsWith => OpCode::STWT,
            Rule::EndsWith => OpCode::EDWT,
            _ => unreachable!("Invalid matching operator"),
        });

        Ok(())
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
        Some(Self { rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.rhs.compile(compiler)?;
        compiler.push(OpCode::LNOT);
        Ok(())
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
