use super::*;
use crate::{lexer::Rule, IntoOwned};

// LBrack ~ EOL* ~ (Array | (Expression ~ EOL* ~ Comma ~ EOL*)) ~ Expression? ~ RBrack
define_node!(ArrayNode(elements: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(LBrack, tokens)?;
        skip_eol!(tokens);
        token = token.child(Rule::Array, token.span());

        let mut can_have_comma = false;
        let mut could_have_comma_but_dont = false;
        let mut subtoken = vec![];
        let mut stack = vec![vec![]];
        loop {
            skip_eol!(tokens);

            if can_have_comma {
                could_have_comma_but_dont = terminal!(Comma?, tokens).is_none();
                can_have_comma = false;
                continue;
            }

            if !could_have_comma_but_dont {
                if let Some(t) = terminal!(LBrack?, tokens) {
                    can_have_comma = false;
                    subtoken.push(t.child(Rule::Array, t.span()));
                    stack.push(vec![]);
                    continue;
                }
            }

            if let Some(t) = terminal!(RBrack?, tokens) {
                match stack.len() {
                    1 => {
                        token.include_span(t.span());
                        break;
                    }
                    _ => {
                        if let Some(subtoken) = subtoken.last_mut() {
                            subtoken.include_span(t.span());
                        }
                        let array = Self { elements: stack.pop().unwrap(), token: subtoken.pop().unwrap() }.into_node();
                        stack.last_mut().unwrap().push(array);
                        continue;
                    }
                }
            }

            // Force a comma at this point
            if could_have_comma_but_dont {
                skip_eol!(tokens);
                terminal!(Comma, tokens)?;
                could_have_comma_but_dont = false;
            }

            stack.last_mut().unwrap().push(non_terminal!(ExpressionNode, tokens)?);
            can_have_comma = true;
        }

        tokens.apply_transaction();

        let elements = stack.pop().unwrap();
        Some(Self { elements, token }.into_node())
    }

    into_node(this) {
        Node::Array(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            elements: this.elements.into_iter().map(|e| e.into_owned()).collect(),
            token: this.token.into_owned()
        }
    }
});

fn pairs_from_vec<T>(mut vec: Vec<T>) -> Vec<(T, T)> {
    let mut pairs = vec![];
    while !vec.is_empty() {
        pairs.push((vec.remove(0), vec.remove(0)));
    }
    pairs
}

// LBrace ~ EOL* ~ ((Expression ~ Colon ~ EOL* ~ Object) | (Expression ~ Colon ~ EOL* ~ Expression ~ EOL* ~ Comma ~ EOL*)) ~ Expression? ~ RBrace
define_node!(ObjectNode(elements: Vec<(Node<'source>, Node<'source>)>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(LBrace, tokens)?;
        skip_eol!(tokens);
        token = token.child(Rule::Object, token.span());

        let mut can_have_comma = false;
        let mut could_have_comma_but_dont = false;
        let mut subtoken = vec![];
        let mut stack = vec![vec![]];
        loop {
            skip_eol!(tokens);

            if can_have_comma {
                could_have_comma_but_dont = terminal!(Comma?, tokens).is_none();
                can_have_comma = false;
                continue;
            }

            if !could_have_comma_but_dont {
                if let Some(t) = terminal!(LBrace?, tokens) {
                    can_have_comma = false;
                    subtoken.push(t.child(Rule::Object, t.span()));
                    stack.push(vec![]);
                    continue;
                }
            }

            if let Some(t) = terminal!(RBrace?, tokens) {
                match stack.len() {
                    1 => {
                        token.include_span(t.span());
                        break;
                    }
                    _ => {
                        if let Some(subtoken) = subtoken.last_mut() {
                            subtoken.include_span(t.span());
                        } else {
                            panic!("subtoken is None at : \n{}", t)
                        }
                        let elements = pairs_from_vec(stack.pop().unwrap());
                        let obj = Self { elements, token: subtoken.pop().unwrap() }.into_node();
                        stack.last_mut().unwrap().push(obj);
                        continue;
                    }
                }
            }

            // Force a comma at this point
            if could_have_comma_but_dont {
                could_have_comma_but_dont = false;
                skip_eol!(tokens);
                terminal!(Comma, tokens)?;
            }

            let next_is_key = stack.last().unwrap().len() % 2 == 0;

            stack.last_mut().unwrap().push(non_terminal!(ExpressionNode, tokens)?);

            if next_is_key {
                skip_eol!(tokens);
                terminal!(Colon, tokens)?;
                can_have_comma = false;
            } else {
                can_have_comma = true;
            }
        }

        tokens.apply_transaction();

        let elements = pairs_from_vec(stack.pop().unwrap());
        Some(Self { elements, token }.into_node())
    }

    into_node(this) {
        Node::Object(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            elements: this.elements.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect(),
            token: this.token.into_owned()
        }
    }
});

pratt_node!(RangeExprNode(start: Node<'source>, end: Node<'source>) {
    build(token, start, _op, end) {
        token.set_rule(Rule::RangeExpr);
        Some(Self { start, end, token }.into_node())
    }

    into_node(this) {
        Node::RangeExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            start: this.start.into_owned(),
            end: this.end.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(IndexingExprNode(base: Node<'source>, path: Vec<Option<Node<'source>>>) {
    build(token, base, op) {
        token.set_rule(Rule::IndexingExpr);
        let path = if let Node::PostfixIndexingOperator(op) = op { op } else {
            unreachable!("Invalid operator: {:?}", op)
        }.path;
        Some(Self { base, path, token }.into_node())
    }

    into_node(this) {
        Node::IndexingExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            base: this.base.into_owned(),
            path: this.path.into_iter().map(|p| p.map(|p| p.into_owned())).collect(),
            token: this.token.into_owned(),
        }
    }
});
