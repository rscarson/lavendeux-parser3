use super::*;
use crate::{error::Error, lexer::Rule, parser::ParserNode, IntoOwned};

// If ~ Expression (Then? ~ Block) (Else ~ Block)?
define_node!(IfNode(
    condition: Node<'source>,
    then_block: Node<'source>,
    else_block: Node<'source>
) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(If, tokens)?;
        token = token.child(Rule::IfExpr, token.span());

        skip_eol!(tokens);
        let condition = non_terminal!(ExpressionNode, tokens)?;

        skip_eol!(tokens);
        terminal!(Then?, tokens);
        skip_eol!(tokens);

        let then_block = non_terminal!(BlockNode, tokens)?;
        token.include_span(then_block.token().span());

        skip_eol!(tokens);
        let else_block = match terminal!(Else?, tokens) {
            Some(_) => {
                skip_eol!(tokens);
                let block = non_terminal!(BlockNode, tokens)?;
                token.include_span(block.token().span());
                block
            }
            None => return error_node!(Error::MissingElse(token.into_owned()))
        };


        tokens.apply_transaction();
        Some(Self { condition, then_block, else_block, token }.into_node())
    }

    into_node(this) {
        Node::If(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            condition: this.condition.into_owned(),
            then_block: this.then_block.into_owned(),
            else_block: this.else_block.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

pratt_node_silent!(TernaryExprNode {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::TernaryExpr);
        Some(IfNode {
            condition: lhs,
            then_block: if let Node::InfixOperator(op) = op { op.inner.unwrap() } else {
                unreachable!("Invalid operator: {:?}", op)
            },
            else_block: rhs,
            token
        }.into_node())
    }
});
