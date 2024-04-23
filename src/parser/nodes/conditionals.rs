use super::*;
use crate::{
    lexer::Rule,
    parser::{ParserError, ParserNode},
    traits::IntoOwned,
    vm::OpCode,
};

// If ~ Expression (Then? ~ Block) (Else ~ Block)?
define_node!(IfNode(
    condition: Node<'source>,
    then_block: Node<'source>,
    else_block: Node<'source>
) {
    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(If, tokens, skip_eol!(tokens))?;
        token = token.child(Rule::IfExpr, token.span());
        let condition = non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?;
        terminal!(Then?, tokens, skip_eol!(tokens));

        let then_block = non_terminal!(BlockNode, tokens, skip_eol!(tokens))?;
        token.include_span(then_block.token().span());

        let else_block = match terminal!(Else?, tokens, skip_eol!(tokens)) {
            Some(_) => {
                let block = non_terminal!(BlockNode, tokens, skip_eol!(tokens))?;
                token.include_span(block.token().span());
                block
            }
            None => return error_node!(ParserError::MissingElse(token.into_owned()))
        };


        tokens.apply_transaction();
        Some(Self { condition, then_block, else_block, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        // IF <CONDITION>
        this.condition.compile(compiler)?;
        compiler.push(OpCode::JMPF);
        let jmp1 = compiler.push_i32(0); // Placeholder for a forward jump

        // THEN <BLOCK>
        this.then_block.compile(compiler)?;
        compiler.push(OpCode::JMP);
        let jmp2 = compiler.push_i32(0); // Placeholder for a forward jump

        // Fill in the first else-jump
        let jmp1_value = (compiler.len() - jmp1.end) as i32;
        compiler.replace(jmp1, jmp1_value.to_be_bytes().to_vec());

        // ELSE <BLOCK>
        this.else_block.compile(compiler)?;

        // Fill in the second jump
        let jmp2_value = (compiler.len() - jmp2.end) as i32;
        compiler.replace(jmp2, jmp2_value.to_be_bytes().to_vec());

        Ok(())
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

define_node!(SwitchNode(
    expr: Node<'source>,
    cases: Vec<(ComparisonOp, Node<'source>, Node<'source>)>,
    default: Node<'source>
) {
    "Switch statement - branches based on the value of an expression."
    "Can include multiple cases and an optional default case."
    "Must include a default case."
    "`switch EXPR { ((CmpOp)? EXPR => BLOCK) (, (CmpOp)? EXPR => BLOCK)* }`"

    build(tokens) {
        tokens.start_transaction();

        // switch EXPR {
        let mut token = terminal!(Switch, tokens)?;
        let expr = non_terminal!(ExpressionNode, tokens)?;
        terminal!(LBrace, tokens)?;

        // First case - make sure we have at least one
        // ((CmpOp)? EXPR => BLOCK)
        let cmp = terminal!(SEq|SNe|Eq|Ne|Le|Lt|Ge|Gt ?, tokens, skip_eol!(tokens)).map(|t| ComparisonOp::from_rule(t.rule()))?;
        let value = non_terminal!(ExpressionNode, tokens, skip_eol!(tokens))?;
        terminal!(FatArrow, tokens, skip_eol!(tokens))?;
        let block = non_terminal!(BlockNode, tokens, skip_eol!(tokens))?;

        let mut raw_cases = vec![(cmp, value, block)];

        // Remaining cases
        // (, (CmpOp)? EXPR => BLOCK)*
        loop {
            tokens.start_transaction();

            if terminal!(Comma, tokens, skip_eol!(tokens)).is_none() {
                break;
            }

            // (CmpOp)? EXPR
            let cmp = terminal!(SEq|SNe|Eq|Ne|Le|Lt|Ge|Gt ?, tokens, skip_eol!(tokens)).map(|t| ComparisonOp::from_rule(t.rule()))?;
            let value = match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                Some(v) => v,
                None => break,
            };

            // => BLOCK
            if terminal!(FatArrow, tokens, skip_eol!(tokens)).is_none() {
                break;
            }

            let block = match non_terminal!(BlockNode, tokens, skip_eol!(tokens)) {
                Some(b) => b,
                None => break,
            };

            raw_cases.push((cmp, value, block));

            // Check for default case
            tokens.apply_transaction();
        }

        token = token.child(Rule::SwitchExpr, token.span().start .. terminal!(RBrace, tokens, skip_eol!(tokens))?.span().end);
        tokens.apply_transaction();

        let mut iter = raw_cases.into_iter().rev();

        let default = match iter.next() {
            Some((None, Node::LiteralIdent(ident), block)) if ident.token.slice() == "_" => {
                block
            }
            _ => return error_node!(ParserError::MissingDefaultCase(token.into_owned()))
        };

        let mut cases = vec![];
        for (cmp, value, block) in iter.rev() {
            let cmp = match cmp {
                Some(cmp) => cmp,
                None => ComparisonOp::Eq
            };

            let x = match Some(value) {
                Some(Node::LiteralIdent(ident)) if ident.token.slice() == "_" => {
                    return error_node!(ParserError::UnreachableSwitchCase(ident.token.into_owned()))
                }
                Some(value) => (cmp, value, block),
                _ => unreachable!()
            };

            cases.push(x);
        }

        Some(Self { expr, cases, default, token }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);

        // This will effectively take the form of a series of if-else statements
        // Repeated checks and jumps for each case

        this.expr.compile(compiler)?;

        // All cases will be [dup calc-condition, jmpf, calc-block, jmp]
        // We need to remember to pop the value off the stack we end

        let mut end_jmps = vec![];
        for (cmp, value, block) in this.cases {
            // Duplicate the value and ready comparison
            compiler.push(OpCode::DUP);
            value.compile(compiler)?;

            // Perform the comparison
            compiler.push(match cmp {
                ComparisonOp::Eq => OpCode::EQ,
                ComparisonOp::Neq => OpCode::NE,
                ComparisonOp::Gt => OpCode::GT,
                ComparisonOp::Lt => OpCode::LT,
                ComparisonOp::Gte => OpCode::GE,
                ComparisonOp::Lte => OpCode::LE,
                ComparisonOp::SEq => OpCode::SEQ,
                ComparisonOp::SNeq => OpCode::SNE,
            });

            // Jump if false to next
            compiler.push(OpCode::JMPF);
            let skip_jmp = compiler.push_i32(0);

            // The block
            block.compile(compiler)?;

            // Jump to end
            compiler.push(OpCode::JMP);
            end_jmps.push(compiler.push_i32(0));

            // Fill in the skip jump
            let skip_jmp_value = (compiler.len() - skip_jmp.end) as i32;
            compiler.replace(skip_jmp, skip_jmp_value.to_be_bytes().to_vec());
        }

        // Pop the extra value off the stack
        compiler.push(OpCode::POP);

        // Default case
        this.default.compile(compiler)?;

        // Fill in the end jumps
        for jmp in end_jmps {
            let jmp_value = (compiler.len() - jmp.end) as i32;
            compiler.replace(jmp, jmp_value.to_be_bytes().to_vec());
        }

        Ok(())
    }

    into_node(this) {
        Node::Switch(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            expr: this.expr.into_owned(),
            cases: this.cases.into_iter().map(|(r, c, b)| (r, c.into_owned(), b.into_owned())).collect(),
            default: this.default.into_owned(),
            token: this.token.into_owned()
        }
    }
});
