use super::*;
use crate::{
    pratt,
    tokenizer::{Rule, TokenSpan},
    IntoOwned,
};

node_silent!(InfixExpressionNode(tokens) {
    ExpressionNode::parse(tokens)
});

// prefix_op? ~ EOL* ~ TERM ~ postfix_operation* ~ ( EOL* ~ infix_op ~ prefix_op? ~ EOL* ~ TERM ~ postfix_operation*)*
node_silent!(ExpressionNode(tokens) {
    tokens.start_transaction();

    let mut expr = vec![];

    // prefix_op? ~ EOL* ~ TERM ~ postfix_operation*
    if let Some(n) = non_terminal!(PrefixOperatorNode?, tokens) {
        expr.push(n);
    }
    skip_eol!(tokens);
    expr.push(non_terminal!(TermNode, tokens)?);

    expr.extend(non_terminal!(PostfixOperatorNode*, tokens));

    // ( EOL* ~ infix_op ~ prefix_op? ~ EOL* ~ TERM ~ postfix_operation*)*
    loop {
        tokens.start_transaction();
        let mut group = vec![];

        skip_eol!(tokens);
        match non_terminal!(InfixOperatorNode, tokens) {
            Ok(t) => group.push(t),
            Err(_) => break
        }
        if let Some(n) = non_terminal!(PrefixOperatorNode?, tokens) {
            group.push(n);
        }

        skip_eol!(tokens);
        match non_terminal!(TermNode, tokens) {
            Ok(t) => group.push(t),
            Err(_) => break
        }

        group.extend(non_terminal!(PostfixOperatorNode*, tokens));

        tokens.apply_transaction();
        expr.extend(group.drain(0..))
    }

    if expr.len() == 1 {
        // Just a single term
        tokens.apply_transaction();
        return Ok(expr.pop().unwrap());
    }

    tokens.apply_transaction();
    expr.reverse(); // pratt expects the expression to be in reverse order
    pratt::fold_expression(&mut expr, u8::MAX)
});

// "(" ~ EXPR ~ ")" | Array | Object | SKIP_KEYWORD | BREAK_EXPRESSION | RETURN_EXPRESSION | FOR_LOOP_EXPRESSION | SWITCH_EXPRESSION | IF_EXPRESSION | Literal
node_silent!(TermNode(tokens) {
    tokens.start_transaction();


    if let Some(_) = terminal!(LParen?, tokens) {
        let expr = non_terminal!(ExpressionNode, tokens)?;
        terminal!(RParen, tokens)?;

        tokens.apply_transaction();
        Ok(expr)
    } else {
        let t = non_terminal!(
            LiteralStringNode|LiteralIdentNode|LiteralCurrencyNode|LiteralDecimalNode|LiteralFloatNode|LiteralBoolNode|LiteralConstNode
            | LiteralU8Node|LiteralU16Node|LiteralU32Node|LiteralU64Node|LiteralI8Node|LiteralI16Node|LiteralI32Node|LiteralI64Node
            | ArrayNode|ObjectNode
            | ContinueNode|BreakNode|ReturnNode
            | ForNode|SwitchNode|IfNode
        , tokens)?;

        tokens.apply_transaction();
        Ok(t)
    }
});

define_node!(InfixOperatorNode(inner: Option<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(
            Contains|Matches|Is|StartsWith|EndsWith
            | Assign
            | AssignAdd|AssignSub|AssignMul|AssignDiv|AssignMod|AssignPow
            | AssignAnd|AssignOr|AssignXor|AssignSL|AssignSR
            | Decorator
            | Add|Sub|Mul|Div|Mod|Pow
            | BitwiseOr|BitwiseAnd|Xor|SL|SR
            | LogicalAnd|LogicalOr
            | SEq|SNe | Eq|Ne | Le|Lt | Ge|Gt
            | As | Range
            | Question
        , tokens)?;

        // ternary part... "?" ~ EXPR
        let mut inner = None;
        if token.rule() == Rule::Question {
            skip_eol!(tokens);
            inner = Some(non_terminal!(ExpressionNode, tokens)?);

            skip_eol!(tokens);
            token.include_span(terminal!(Colon, tokens)?.span());
        }

        tokens.apply_transaction();
        Ok(Self { inner, token }.into_node())
    }

    into_node(this) {
        Node::InfixOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            inner: this.inner.map(|n| n.into_owned()),
            token: this.token.into_owned()
        }
    }
});

// NEG | INC | DEC | BIT_NOT | BOOL_NOT | (DEL ~ symbol_at?)
define_node!(PrefixOperatorNode() {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Sub|Inc|Dec|BitwiseNot|LogicalNot|Delete, tokens)?;
        if token.rule() == Rule::Delete {
            if let Some(decorator) = terminal!(Decorator?, tokens) {
                token.include_span(decorator.span());
            }
        } else if token.rule() == Rule::Sub {
            token.set_rule(Rule::PrefixNeg)
        } else if token.rule() == Rule::Inc {
            token.set_rule(Rule::PrefixInc)
        } else if token.rule() == Rule::Dec {
            token.set_rule(Rule::PrefixDec)
        }

        tokens.apply_transaction();
        Ok(Node::PrefixOperator(Box::new(Self { token })))
    }

    into_node(this) {
        Node::PrefixOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

// PostfixIncDecOperator | PostfixIndexingOperator | PostfixDecoratorOperator | PostfixFnCallOperator
node_silent!(PostfixOperatorNode(tokens) {
    tokens.start_transaction();

    let operator = non_terminal!(
        PostfixIncDecOperatorNode
        | PostfixIndexingOperatorNode
        | PostfixFnCallOperatorNode,
    tokens)?;

    tokens.apply_transaction();
    Ok(operator)
});

define_node!(PostfixIncDecOperatorNode() {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Inc|Dec, tokens)?;
        if token.rule() == Rule::Inc {
            token.set_rule(Rule::PostfixInc)
        } else if token.rule() == Rule::Dec {
            token.set_rule(Rule::PostfixDec)
        }

        tokens.apply_transaction();
        Ok(Node::PostfixIncDecOperator(Box::new(Self { token })))
    }

    into_node(this) {
        Node::PostfixIncDecOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

// ("." ~ EOL* ~ identifier ~ EOL*)? ~ "(" ~ EOL* ~ (EXPR ~ EOL* ~ symbol_comma ~ EOL*)* ~ EXPR? ~ EOL* ~ ")"
define_node!(PostfixFnCallOperatorNode(
    name_span: Option<TokenSpan>,
    args: Vec<Node<'source>>
) {
    build(tokens) {
        tokens.start_transaction();
        let mut args = vec![];
        let mut token = None;

        // ("." ~ EOL* ~ identifier ~ EOL*)?
        let mut name_span = None;
        'object_mode: {
            tokens.start_transaction();

            skip_eol!(tokens);
            match terminal!(Dot, tokens) {
                Ok(dot) => {
                    token = Some(dot);
                },
                _ => {
                    break 'object_mode;
                }
            }

            skip_eol!(tokens);
            match terminal!(LiteralIdent, tokens) {
                Ok(name) => {
                    skip_eol!(tokens);
                    name_span = Some(name.span());
                },
                _ => {
                    break 'object_mode;
                }
            }

            tokens.apply_transaction();
        }

        // "(" ~ EOL*
        match terminal!(LParen, tokens) {
            Ok(t) => {
                let mut t = t.child(Rule::FnCallOperator, t.span());
                if let Some(token) = token {
                    t.include_span(token.span());
                }
                token = Some(t);
            },
            Err(e) => return Err(e)
        }
        skip_eol!(tokens);
        let mut token = token.unwrap();

        // (EXPR ~ EOL* ~ symbol_comma ~ EOL*)*
        loop {
            tokens.start_transaction();

            match non_terminal!(ExpressionNode, tokens) {
                Ok(expr) => args.push(expr),
                Err(_) => {
                    break;
                }
            }

            skip_eol!(tokens);
            if terminal!(Comma, tokens).is_err() {
                args.pop();
                break;
            }

            skip_eol!(tokens);

            tokens.apply_transaction();
        }

        //  EXPR? ~ EOL* ~ ")"
        if let Some(arg) = non_terminal!(ExpressionNode?, tokens) {
            args.push(arg);
        }
        skip_eol!(tokens);
        token.include_span(terminal!(RParen, tokens)?.span());

        tokens.apply_transaction();
        Ok(Self { name_span, args, token }.into_node())
    }

    into_node(this) {
        Node::PostfixFnCallOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            name_span: this.name_span,
            args: this.args.into_iter().map(|a| a.into_owned()).collect(),
            token: this.token.into_owned()
        }
    }
});

// (symbol_opensquare ~ EOL* ~ EXPR? ~ EOL* ~ symbol_closesquare)+
define_node!(PostfixIndexingOperatorNode(path: Vec<Option<Node<'source>>>) {
    build(tokens) {
        tokens.start_transaction();
        let mut path = vec![];

        let mut start = 0..0;
        let mut end = 0..0;

        let mut last_err = None;

        let token = tokens.peek().cloned();

        loop {
            tokens.start_transaction();

            match terminal!(LBrack, tokens) {
                Ok(b) => end = b.span(),
                Err(e) => {
                    last_err = Some(e);
                    break;
                }
            }

            skip_eol!(tokens);
            let expr = non_terminal!(ExpressionNode?, tokens);
            skip_eol!(tokens);

            match terminal!(RBrack, tokens) {
                Ok(b) => start = b.span(),
                Err(e) => {
                    last_err = Some(e);
                    break;
                }
            }

            path.push(expr);
            tokens.apply_transaction();
        }

        // [ ... ]+
        if path.len() == 0 {
            tokens.revert_transaction();
            return Err(last_err.unwrap());
        }

        let token = match token {
            Some(token) => {
                let mut token = token.child(Rule::IndexingOperator, start);
                token.include_span(end);
                token
            },
            None => {
                tokens.revert_transaction();
                return Err(tokens.emit_err());
            }
        };

        tokens.apply_transaction();
        Ok(Self { path, token }.into_node())
    }

    into_node(this) {
        Node::PostfixIndexingOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            path: this.path.into_iter().map(|e| e.map(|n| n.into_owned())).collect(),
            token: this.token.into_owned()
        }
    }
});
