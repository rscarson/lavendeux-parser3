use super::*;
use crate::{lexer::Rule, traits::IntoOwned, vm::OpCode};

// LBrack ~ RBrack | LBrack ~ ~ EOL* ~ EXPRESSION ~ (EOL* ~ Comma ~ EOL* ~ EXPRESSION)* ~ EOL* ~ RBrack
define_node!(ArrayNode(elements: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(LBrack, tokens)?;
        token = token.child(Rule::Array, token.span());

        match terminal!(RBrack?, tokens, skip_eol!(tokens)) {
            Some(t) => {
                token.include_span(t.span());

                tokens.apply_transaction();
                Some(Self { elements: vec![], token }.into_node())
            }
            None => {
                let mut elements = vec![non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?];
                loop {
                    tokens.start_transaction();

                    if terminal!(Comma, tokens, skip_eol!(tokens)).is_none() {
                        break;
                    }

                    match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                        Some(e) => elements.push(e),
                        None => break,
                    }

                    tokens.apply_transaction();
                }

                token.include_span(terminal!(RBrack, tokens, skip_eol!(tokens))?.span());

                tokens.apply_transaction();
                Some(Self { elements, token }.into_node())
            }
        }
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        let len = this.elements.len();
        this.elements.into_iter().rev().map(|e| e.compile(compiler)).collect::<Result<Vec<_>, _>>()?;

        compiler.push(OpCode::MKAR);
        compiler.push_u64(len as u64);

        Ok(())
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

// LBrace ~ RBrace | LBrace ~ ~ EOL* ~ EXPRESSION ~ COLON ~ EXPRESSION ~ (EOL* ~ Comma ~ EOL* ~ EXPRESSION ~ COLON ~ EXPRESSION)* ~ EOL* ~ RBrace
define_node!(ObjectNode(elements: Vec<(Node<'source>, Node<'source>)>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(LBrace, tokens)?;
        token = token.child(Rule::Object, token.span());

        match terminal!(RBrace?, tokens, skip_eol!(tokens)) {
            Some(t) => {
                token.include_span(t.span());

                tokens.apply_transaction();
                Some(Self { elements: vec![], token }.into_node())
            }
            None => {
                let mut elements = vec![];

                let key = non_terminal!(ExpressionNode, tokens)?;
                terminal!(Colon, tokens)?;
                let value = non_terminal!(ExpressionNode, tokens)?;

                elements.push((key, value));
                loop {
                    tokens.start_transaction();

                    if terminal!(Comma, tokens, skip_eol!(tokens)).is_none() {
                        break;
                    }

                    let key = non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?;
                    terminal!(Colon, tokens, skip_eol!(tokens))?;
                    let value = non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?;

                    elements.push((key, value));
                    tokens.apply_transaction();
                }

                token.include_span(terminal!(RBrace, tokens, skip_eol!(tokens))?.span());
                tokens.apply_transaction();
                Some(Self { elements, token }.into_node())
            }
        }
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        let len = this.elements.len();
        this.elements.into_iter().map(|(k, v)| {
            v.compile(compiler)?;
            k.compile(compiler)
        }).collect::<Result<Vec<_>, _>>()?;

        compiler.push(OpCode::MKOB);
        compiler.push_u64(len as u64);

        Ok(())
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

/*
// LBrack ~ EOL* ~ (Array | (Expression ~ EOL* ~ Comma ~ EOL*)) ~ Expression? ~ RBrack
define_node!(ArrayNode(elements: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(LBrack, tokens)?;
                token = token.child(Rule::Array, token.span());

        let mut can_have_comma = false;
        let mut could_have_comma_but_dont = false;
        let mut subtoken = vec![];
        let mut stack = vec![vec![]];
        loop {

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
                token = token.child(Rule::Object, token.span());

        let mut can_have_comma = false;
        let mut could_have_comma_but_dont = false;
        let mut subtoken = vec![];
        let mut stack = vec![vec![]];
        loop {

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
                                terminal!(Comma, tokens)?;
            }

            let next_is_key = stack.last().unwrap().len() % 2 == 0;

            stack.last_mut().unwrap().push(non_terminal!(ExpressionNode, tokens)?);

            if next_is_key {
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
 */

pratt_node!(RangeExprNode(start: Node<'source>, end: Node<'source>) {
    build(token, start, _op, end) {
        token.set_rule(Rule::RangeExpr);
        Some(Self { start, end, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        this.start.compile(compiler)?;
        this.end.compile(compiler)?;
        compiler.push(OpCode::MKRG);

        Ok(())
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

pratt_node!(IndexingExprNode(base: Node<'source>, path: Vec<Node<'source>>) {
    build(token, base, op) {
        token.set_rule(Rule::IndexingExpr);
        let path = if let Node::PostfixIndexingOperator(op) = op { op } else {
            unreachable!("Invalid operator: {:?}", op)
        }.path;
        Some(Self { base, path, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        this.base.compile(compiler)?;
        for idx in this.path {
            idx.compile(compiler)?;
            compiler.push(OpCode::IDEX);
        }

        Ok(())
    }

    into_node(this) {
        Node::IndexingExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            base: this.base.into_owned(),
            path: this.path.into_iter().map(|e| e.into_owned()).collect(),
            token: this.token.into_owned(),
        }
    }
});

// (symbol_opensquare ~ EOL* ~ EXPR ~ EOL* ~ symbol_closesquare)+
define_node!(PostfixIndexingOperatorNode(path: Vec<Node<'source>>) {
    build(tokens) {
        tokens.start_transaction();
        let mut path = vec![];

        let mut start = 0..0;
        let mut end = 0..0;

        let token = tokens.peek().cloned();

        loop {
            tokens.start_transaction();

            match terminal!(LBrack, tokens, skip_eol!(tokens)) {
                Some(b) => end = b.span(),
                None => {
                    break;
                }
            }

            let expr = match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                Some(e) => e,
                None => {
                    break;
                }

            };

            match terminal!(RBrack, tokens, skip_eol!(tokens)) {
                Some(b) => start = b.span(),
                None => {
                    break;
                }
            }

            path.push(expr);
            tokens.apply_transaction();
        }

        // [ ... ]+
        if path.len() == 0 {
            tokens.revert_transaction();
            return None;
        }

        let token = match token {
            Some(token) => {
                let mut token = token.child(Rule::IndexingOperator, start);
                token.include_span(end);
                token
            },
            None => {
                tokens.revert_transaction();
                return None;
            }
        };

        tokens.apply_transaction();
        Some(Self { path, token }.into_node())
    }

    compile(_this, _compiler) {
        unreachable!("Intermediate node")
    }

    into_node(this) {
        Node::PostfixIndexingOperator(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            path: this.path.into_iter().map(|e| e.into_owned()).collect(),
            token: this.token.into_owned()
        }
    }
});
