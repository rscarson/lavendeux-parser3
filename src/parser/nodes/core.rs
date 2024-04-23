use super::*;
use crate::{
    lexer::{Rule, TokenSpan},
    parser::ParserError,
    traits::IntoOwned,
    value::ValueType,
    vm::OpCode,
};

// LINE*
define_node!(ScriptNode(lines: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut lines = vec![];
        loop {
            if tokens.len() <= 1 {
                break;
            }

            lines.push(non_terminal!(LineNode?, tokens)?);
        }

        let token = match (lines.first(), lines.last()) {
            (Some(s), Some(e)) => {
                let mut token = s.token().child(Rule::Script, s.token().span());
                token.include_span(e.token().span());
                token
            }
            _ => {
                terminal!(EOI, tokens)?.child(Rule::Script, 0..0)
            },
        };

        tokens.apply_transaction();
        Some(Self { lines, token }.into_node())
    }

    compile(this, compiler) {
        for line in this.lines {
            line.compile(compiler)?;
        }

        Ok(())
    }

    into_node(this) {
        Node::Script(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            lines: this.lines.into_iter().map(|l| l.into_owned()).collect(),
            token: this.token.into_owned()
        }
    }
});

// (EXPR | FN_ASSIGNMENT) ~ (EOL | EOI)
node_silent!(LineNode {
    build(tokens) {
        tokens.start_transaction();

        terminal!(EOI|EOL*, tokens);
        let expr = non_terminal!(FnAssignNode|ExpressionNode, tokens)?;
        terminal!(EOI|EOL+, tokens)?;

        tokens.apply_transaction();
        Some(expr)
    }
});

// "{" ~ LINE* ~ EXPR? ~ "}" | EXPR
define_node!(BlockNode(lines: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        match terminal!(LBrace?, tokens) {
            Some(token) => {
                let mut lines = non_terminal!(LineNode*, tokens);
                if let Some(expr) = non_terminal!(ExpressionNode?, tokens) {
                    lines.push(expr);
                }

                let etoken = terminal!(RBrace, tokens)?;
                let mut token = token.child(Rule::Block, token.span().start..etoken.span().end);

                tokens.apply_transaction();
                if lines.is_empty() {
                    // Empty block is just an empty object right?
                    token.set_rule(Rule::Object);
                    Some(ObjectNode {
                        elements: vec![],
                        token
                    }.into_node())
                } else {
                    Some(Self { lines, token }.into_node())
                }
            }
            None => {
                let expr = non_terminal!(ExpressionNode, tokens)?;
                tokens.apply_transaction();
                Some(expr)
            }
        }
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        compiler.push(OpCode::SCI); // Block scope

        let len = this.lines.len();
        for (i, line) in this.lines.into_iter().enumerate() {
            line.compile(compiler)?;
            if i < len - 1 {
                compiler.push(OpCode::POP);
            }
        }

        compiler.push(OpCode::SCO); // Block scope
        Ok(())
    }

    into_node(this) {
        Node::Block(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            lines: this.lines.into_iter().map(|l| l.into_owned()).collect(),
            token: this.token.into_owned()
        }
    }
});

pratt_node!(CastExprNode(expr: Node<'source>, type_name: ValueType) {
    build(token, lhs, _op, rhs) {
        token.set_rule(Rule::CastExpr);
        let type_name = match ValueType::from_str(rhs.token().slice()) {
            Some(t) => t,
            None => {
                return error_node!(ParserError::CannotCastToType(rhs.token().clone().into_owned()));
            }
        };
        Some(Self { expr: lhs, type_name, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        this.expr.compile(compiler)?;
        compiler.push(OpCode::CAST);
        compiler.push_type(this.type_name);

        Ok(())
    }

    into_node(this) {
        Node::CastExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            expr: this.expr.into_owned(),
            type_name: this.type_name,
            token: this.token.into_owned()
        }
    }
});

pratt_node!(DecoratorExprNode(expr: Node<'source>, name_span: TokenSpan) {
    build(token, lhs, _op, rhs) {
        token.set_rule(Rule::DecoratorExpr);
        Some(Self { expr: lhs, name_span: rhs.token().span(), token }.into_node())
    }

    compile(_this, _compiler) {
        todo!()
        // Validate function dignature
        // Then normal function call
    }

    into_node(this) {
        Node::DecoratorExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            expr: this.expr.into_owned(),
            name_span: this.name_span,
            token: this.token.into_owned()
        }
    }
});
