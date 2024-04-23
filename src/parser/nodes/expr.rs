use super::*;
use crate::{lexer::Rule, traits::IntoOwned};

// prefix_op? ~ EOL* ~ TERM ~ postfix_operation* ~ ( EOL* ~ infix_op ~ prefix_op? ~ EOL* ~ TERM ~ postfix_operation*)*
node_silent!(ExpressionNode {
    build(tokens) {
        tokens.start_transaction();

        let mut expr = vec![];

        // prefix_op? ~ EOL* ~ TERM ~ postfix_operation*
        if let Some(n) = non_terminal!(PrefixOperatorNode?, tokens) {
            expr.push(n);
        }
                expr.push(non_terminal!(TermNode, tokens)?);

        expr.extend(non_terminal!(PostfixOperatorNode*, tokens));

        // ( EOL* ~ infix_op ~ prefix_op? ~ EOL* ~ TERM ~ postfix_operation*)*
        loop {
            tokens.start_transaction();
            let mut group = vec![];

                        match non_terminal!(InfixOperatorNode, tokens) {
                Some(t) => group.push(t),
                None => break
            }
            if let Some(n) = non_terminal!(PrefixOperatorNode?, tokens) {
                group.push(n);
            }

                        match non_terminal!(TermNode, tokens) {
                Some(t) => group.push(t),
                None => break
            }

            group.extend(non_terminal!(PostfixOperatorNode*, tokens));

            tokens.apply_transaction();
            expr.extend(group.drain(0..))
        }

        if expr.len() == 1 {
            // Just a single term
            tokens.apply_transaction();
            return Some(expr.pop().unwrap());
        }

        tokens.apply_transaction();
        expr.reverse(); // pratt expects the expression to be in reverse order
        crate::parser::pratt::fold_expression(&mut expr, 0)
    }
});

// "(" ~ EXPR ~ ")" | Array | Object | SKIP_KEYWORD | BREAK_EXPRESSION | RETURN_EXPRESSION | FOR_LOOP_EXPRESSION | SWITCH_EXPRESSION | IF_EXPRESSION | Literal
node_silent!(TermNode {
    build(tokens) {
        tokens.start_transaction();


        if let Some(_) = terminal!(LParen?, tokens) {
            let expr = non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?;
            terminal!(RParen, tokens, skip_eol!(tokens))?;

            tokens.apply_transaction();
            Some(expr)
        } else {
            let t = non_terminal!(
                LiteralStringNode|LiteralRegexNode
                | LiteralIdentNode
                | LiteralFloatNode|LiteralBoolNode|LiteralConstNode
                | LiteralIntNode
                | ArrayNode|ObjectNode
                | ContinueNode|BreakNode|ReturnNode
                | ForNode|SwitchNode|IfNode
            , tokens)?;

            tokens.apply_transaction();
            Some(t)
        }
    }
});

define_node!(InfixOperatorNode(inner: Option<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(
            Contains|Matches|StartsWith|EndsWith
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
            token = token.child(Rule::TernaryOperator, token.span());
            inner = Some(non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?);
            token.include_span(terminal!(Colon, tokens, skip_eol!(tokens))?.span());
        }

        tokens.apply_transaction();
        Some(Self { inner, token }.into_node())
    }

    compile(_this, _compiler) {
        unreachable!("Intermediate node")
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

// NEG | INC | DEC | BIT_NOT | BOOL_NOT | DEL
define_node!(PrefixOperatorNode() {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Sub|BitwiseNot|LogicalNot|Delete, tokens)?;
        if token.rule() == Rule::Sub {
            token.set_rule(Rule::PrefixNeg)
        }

        tokens.apply_transaction();
        Some(Node::PrefixOperator(Box::new(Self { token })))
    }

    compile(_this, _compiler) {
        unreachable!("Intermediate node")
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
node_silent!(PostfixOperatorNode {
    build(tokens) {
        tokens.start_transaction();

        let operator = non_terminal!(
            PostfixIndexingOperatorNode | PostfixFnCallOperatorNode,
        tokens)?;

        tokens.apply_transaction();
        Some(operator)
    }
});
