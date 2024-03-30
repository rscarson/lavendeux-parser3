use super::*;
use crate::{
    error::Error,
    tokenizer::{Rule, TokenSpan},
    IntoOwned,
};

// continue
define_node!(ContinueNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(Continue, tokens)?;

        tokens.apply_transaction();
        Some(Self { token: token.child(Rule::ContinueExpr, token.span()) }.into_node())
    }

    into_node(this) {
        Node::Continue(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

// break BLOCK?
define_node!(BreakNode(value: Option<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Break, tokens)?;
        let value = non_terminal!(BlockNode?, tokens);
        if let Some(value) = &value {
            token.include_span(value.token().span());
        }

        tokens.apply_transaction();
        Some(Self { value, token: token.child(Rule::BreakExpr, token.span()) }.into_node())
    }

    into_node(this) {
        Node::Break(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            value: this.value.map(|v| v.into_owned()),
            token: this.token.into_owned()
        }
    }
});

// for (ident in )? EXPR do? BLOCK (where EXPR)?
define_node!(ForNode(
    name_span: Option<TokenSpan>,
    expr: Node<'source>,
    block: Node<'source>,
    condition: Option<Node<'source>>
) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(For, tokens)?;
        token = token.child(Rule::ForExpr, token.span());

        // (ident in )?
        tokens.start_transaction();
        let name_span = match non_terminal!(LiteralIdentNode, tokens) {
            Some(ident) => {
                let name_span = ident.token().span();
                if terminal!(In, tokens).is_some() {
                    tokens.apply_transaction();
                    Some(name_span)
                } else {
                    None
                }
            },
            None => None
        };

        // EXPR do? BLOCK
        let expr = non_terminal!(ExpressionNode, tokens)?;
        terminal!(Do?, tokens);
        let block = non_terminal!(BlockNode, tokens)?;
        token.include_span(block.token().span());

        // (where EXPR)?
        tokens.start_transaction();
        let condition = match terminal!(Where, tokens) {
            Some(_) => {
                match non_terminal!(ExpressionNode, tokens) {
                    Some(expr) => {
                        tokens.apply_transaction();
                        token.include_span(expr.token().span());
                        Some(expr)
                    },
                    None => None
                }
            },
            None => None,
        };

        tokens.apply_transaction();
        Some(Self { name_span, expr, block, condition, token }.into_node())
    }

    into_node(this) {
        Node::For(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            name_span: this.name_span,
            expr: this.expr.into_owned(),
            block: this.block.into_owned(),
            condition: this.condition.map(|c| c.into_owned()),
            token: this.token.into_owned()
        }
    }
});

// switch EXPR { ((CmpOp)? EXPR => BLOCK) (, (CmpOp)? EXPR => BLOCK)* }
define_node!(SwitchNode(
    expr: Node<'source>,
    cases: Vec<(Option<Rule>, Node<'source>, Node<'source>)>,
    default: Option<Node<'source>>
) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Switch, tokens)?;
        let expr = non_terminal!(ExpressionNode, tokens)?;

        terminal!(LBrace, tokens)?;

        let cmp = terminal!(SEq|SNe|Eq|Ne|Le|Lt|Ge|Gt ?, tokens).map(|t| t.rule());
        let value = non_terminal!(ExpressionNode, tokens)?;
        terminal!(FatArrow, tokens)?;
        let block = non_terminal!(BlockNode, tokens)?;

        let (mut cases, mut default): (Vec<_>, Option<Node<'source>>) = match value {
            Node::LiteralIdent(i) if i.name() == "_" => {
                (vec![], Some(block))
            },
            _ => (vec![(cmp, value, block)], None),
        };

        loop {
            tokens.start_transaction();

            if terminal!(Comma, tokens).is_none() {
                break;
            }

            let cmp = terminal!(SEq|SNe|Eq|Ne|Le|Lt|Ge|Gt ?, tokens).map(|t| t.rule());
            let value = match non_terminal!(ExpressionNode, tokens) {
                Some(v) => v,
                None => break,
            };
            if terminal!(FatArrow, tokens).is_none() {
                break;
            }
            let block = match non_terminal!(BlockNode, tokens) {
                Some(b) => b,
                None => break,
            };

            if default.is_some() {
                tokens.revert_transaction();
                return error_node!(Error::UnreachableSwitchCase(value.token().clone().into_owned()));
            } else {
                tokens.apply_transaction();
                match value {
                    Node::LiteralIdent(i) if i.name() == "_" => {
                        default = Some(block);
                    },
                    _ => {
                        cases.push((cmp, value, block));
                    }
                };
            }
        }

        token = token.child(Rule::SwitchExpr, token.span().start .. terminal!(RBrace, tokens)?.span().end);
        tokens.apply_transaction();
        Some(Self { expr, cases, default, token }.into_node())
    }

    into_node(this) {
        Node::Switch(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            expr: this.expr.into_owned(),
            cases: this.cases.into_iter().map(|(r, c, b)| (r, c.into_owned(), b.into_owned())).collect(),
            default: this.default.map(|d| d.into_owned()),
            token: this.token.into_owned()
        }
    }
});
