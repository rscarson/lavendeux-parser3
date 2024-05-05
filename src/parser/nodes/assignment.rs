use super::*;
use crate::{lexer::Rule, traits::IntoOwned, vm::OpCode};

/*
TODO
lhs for assignments need a new node type
a non-resolving reference

We then adjust normal reference types to resolve IN compilation
Then the compiler never needs to deal with em? Maybe a value cache eventually */

pratt_node!(AssignExprNode(target: Node<'source>, value: Node<'source>) {
    build(token, lhs, _op, rhs) {
        token.set_rule(Rule::AssignExpr);
        Some(Self { target: lhs, value: rhs, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.value.compile(compiler)?;
        this.target.compile(compiler)?;

        compiler.push(OpCode::WREF);

        Ok(())
    }

    into_node(this) {
        Node::AssignExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            value: this.value.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(AssignArithmeticExprNode(target: Node<'source>, value: Node<'source>, op: ArithmeticOp) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::AssignArithmeticExpr);
        let op = match op.token().rule() {
            Rule::AssignAdd => ArithmeticOp::Add,
            Rule::AssignSub => ArithmeticOp::Sub,
            Rule::AssignMul => ArithmeticOp::Mul,
            Rule::AssignDiv => ArithmeticOp::Div,
            Rule::AssignMod => ArithmeticOp::Mod,
            Rule::AssignPow => ArithmeticOp::Pow,
            _ => return None,
        };

        Some(Self { target: lhs, value: rhs, op, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        this.target.compile(compiler)?;
        compiler.push(OpCode::DUP); // Duplicate target reference, one for assignment and one for arithmetic operation

        this.value.compile(compiler)?;
        compiler.push(OpCode::LCST); // Cast value to target type

        compiler.push(match this.op {
            ArithmeticOp::Add => OpCode::ADD,
            ArithmeticOp::Sub => OpCode::SUB,
            ArithmeticOp::Mul => OpCode::MUL,
            ArithmeticOp::Div => OpCode::DIV,
            ArithmeticOp::Mod => OpCode::REM,
            ArithmeticOp::Pow => OpCode::POW,
        });

        compiler.push(OpCode::SWP); // Swap target reference to top of stack
        compiler.push(OpCode::WREF);
        Ok(())
    }

    into_node(this) {
        Node::AssignArithmeticExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            value: this.value.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(AssignBitwiseExprNode(target: Node<'source>, value: Node<'source>, op: BitwiseOp) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::AssignBitwiseExpr);
        let op = match op.token().rule() {
            Rule::AssignAnd => BitwiseOp::And,
            Rule::AssignOr => BitwiseOp::Or,
            Rule::AssignXor => BitwiseOp::Xor,
            Rule::AssignSL => BitwiseOp::ShiftLeft,
            Rule::AssignSR => BitwiseOp::ShiftRight,
            _ => return None,
        };
        Some(Self { target: lhs, value: rhs, op, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        compiler.push(OpCode::DUP); // Duplicate target reference, one for assignment and one for arithmetic operation

        this.value.compile(compiler)?;
        compiler.push(OpCode::LCST); // Cast value to target type

        compiler.push(match this.op {
            BitwiseOp::And => OpCode::AND,
            BitwiseOp::Or => OpCode::OR,
            BitwiseOp::Xor => OpCode::XOR,
            BitwiseOp::ShiftLeft => OpCode::SHL,
            BitwiseOp::ShiftRight => OpCode::SHR,
        });

        this.target.compile(compiler)?;
        compiler.push(OpCode::WREF);
        Ok(())
    }

    into_node(this) {
        Node::AssignBitwiseExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            value: this.value.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(DeleteExprNode(target: Node<'source>) {
    build(token, term, _op) {
        token.set_rule(Rule::DeleteExpr);

        Some(Self { target: term, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        this.target.compile(compiler)?;
        compiler.push(OpCode::DREF);

        Ok(())
    }

    into_node(this) {
        Node::DeleteExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            token: this.token.into_owned(),
        }
    }
});
