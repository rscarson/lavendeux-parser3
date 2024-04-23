use super::*;
use crate::{
    compiler::LoopCompilationExt,
    lexer::{Rule, TokenSpan},
    traits::IntoOwned,
    vm::OpCode,
};

define_node!(ContinueNode() {
    "Continue statement - jumps to the next iteration of the current loop."
    "`continue`"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(Continue, tokens)?;

        tokens.apply_transaction();
        Some(Self { token: token.child(Rule::ContinueExpr, token.span()) }.into_node())
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        compiler.push_continue();
        Ok(())
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
    "Break statement - jumps out of the current loop."
    "Can optionally include a value to return for the current iteration."
    "`break BLOCK?`"

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

    compile(this, compiler) {
        compiler.push_token(this.token);
        if let Some(value) = this.value {
            value.compile(compiler)?;
        }

        compiler.push_break();
        Ok(())
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

define_node!(ForNode(
    name_span: Option<TokenSpan>,
    expr: Node<'source>,
    block: Node<'source>,
    condition: Option<Node<'source>>
) {
    "For loop - iterates over a range or collection."
    "Optional iteration variable name and filter condition."
    "`for (ident in )? EXPR do? BLOCK (where BLOCK)?`"

    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(For, tokens)?;
        token = token.child(Rule::ForExpr, token.span());

        // Optional iteration variable name
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

        // Iterable and the execution block
        // EXPR do? BLOCK
        let expr = non_terminal!(ExpressionNode, tokens)?;
        terminal!(Do?, tokens);
        let block = non_terminal!(BlockNode, tokens)?;
        token.include_span(block.token().span());

        // Optional filter condition
        // (where EXPR)?
        let condition = match terminal!(Where?, tokens) {
            Some(_) => {
                match non_terminal!(BlockNode, tokens) {
                    Some(expr) => {
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

    compile(this, compiler) {
        let name = this.token.borrow_input();
        let name = this.name_span.map(|s| name[s].to_string());

        compiler.push_token(this.token);

        // Compile the iteration variable
        this.expr.compile(compiler)?;

        compiler.push(OpCode::SCI);
        compiler.start_loop();
        let len_pos = compiler.push_u64(0);

        // The iterable variable
        compiler.push(OpCode::NEXT);
        if let Some(name) = name {
            compiler.push(OpCode::REF);
            compiler.push_strhash(&name);
            compiler.push(OpCode::WREF);
        }
        compiler.push(OpCode::POP);

        // Conditional jump to the end of the loop
        if let Some(condition) = this.condition {
            condition.compile(compiler)?;
            compiler.push(OpCode::JMPT);
            compiler.push_u64(1);
            compiler.push_break();
        }

        // Compile the loop block
        this.block.compile(compiler)?;
        compiler.push_continue();

        let len = compiler.len();
        compiler.replace(len_pos, len.to_be_bytes().to_vec());
        compiler.end_loop();
        compiler.push(OpCode::SCO);

        Ok(())
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
