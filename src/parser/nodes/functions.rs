use super::*;
use crate::{
    lexer::{Rule, TokenSpan},
    IntoOwned,
};

// At? ~ Identifer ~ EOL* ~ LParen ~ EOL* ~ (Identifier ~ EOL* ~ Comma ~ EOL*)* ~ Identifier? ~ EOL* ~ RParen ~ EOL* ~ Eq? ~ Block
define_node!(FnAssignNode(
    name_span: TokenSpan,
    args: Vec<TokenSpan>,
    body: Node<'source>) {

    build(tokens) {
        tokens.start_transaction();

        let decorator = terminal!(Decorator?, tokens);
        let name = terminal!(LiteralIdent, tokens)?;

        let token = match decorator {
            Some(decorator) => {
                let mut token = decorator.child(Rule::FnAssignExpr, decorator.span());
                token.include_span(name.span());
                token
            },
            None => {
                name.child(Rule::FnAssignExpr, name.span())
            }
        };

        skip_eol!(tokens);
        terminal!(LParen, tokens)?;
        skip_eol!(tokens);

        // (Identifier ~ EOL* ~ Comma ~ EOL*)*
        let mut args = vec![];
        loop {
            tokens.start_transaction();

            match terminal!(LiteralIdent?, tokens) {
                Some(arg) => {
                    args.push(arg.span());
                    skip_eol!(tokens);
                    if terminal!(Comma, tokens).is_none() {
                        break;
                    }
                    skip_eol!(tokens);
                },
                None => {
                    tokens.revert_transaction();
                    break;
                }
            }

            tokens.apply_transaction();
        }

        // Identifier? ~ EOL* ~ RParen ~ EOL* ~ Eq? ~ Block
        if let Some(arg) = terminal!(LiteralIdent?, tokens) {
            args.push(arg.span());
        }
        skip_eol!(tokens);
        terminal!(RParen, tokens)?;
        skip_eol!(tokens);
        terminal!(Eq?, tokens);
        skip_eol!(tokens);
        let body = non_terminal!(BlockNode, tokens)?;

        tokens.apply_transaction();
        Some(Self { name_span: name.span(), args, body, token }.into_node())
    }

    into_node(this) {
        Node::FnAssign(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            name_span: this.name_span,
            args: this.args,
            body: this.body.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

// return BLOCK?
define_node!(ReturnNode(value: Option<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Return, tokens)?;
        let value = non_terminal!(BlockNode?, tokens);
        if let Some(value) = &value {
            token.include_span(value.token().span());
        }

        tokens.apply_transaction();
        Some(Self { value, token: token.child(Rule::ReturnExpr, token.span()) }.into_node())
    }

    into_node(this) {
        Node::Return(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            value: this.value.map(|v| v.into_owned()),
            token: this.token.into_owned()
        }
    }
});

pratt_node!(FnCallNode(name_span: TokenSpan, args: Vec<Node<'source>>) {
    build(token, lhs, op) {
        token.set_rule(Rule::FnCallExpr);

        if let Node::PostfixFnCallOperator(op) = op {
            match op.name_span {
                Some(name_span) => {
                    // Method call - LHS is the first argument
                    let mut args = vec![lhs];
                    args.extend(op.args.into_iter());
                    Some(Self {
                        name_span, token, args
                    }.into_node())
                },
                None => {
                    // Normal call - LHS is the function name
                    let name_span = lhs.token().span();
                    Some(Self {
                        name_span, token,
                        args: op.args.into_iter().collect(),
                    }.into_node())
                }
            }
        } else {
            unreachable!("Invalid operator: {:?}", op)
        }
    }

    into_node(this) {
        Node::FnCall(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            name_span: this.name_span,
            args: this.args.into_iter().map(|a| a.into_owned()).collect(),
            token: this.token.into_owned(),
        }
    }
});
