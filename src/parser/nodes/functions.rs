use std::str::FromStr;

use super::*;
use crate::{
    compiler::{CompilerError, FunctionDocs},
    lexer::{Rule, Token, TokenSpan},
    parser::{
        function_compiler::{FunctionArgumentCompiler, FunctionArgumentDefault, FunctionCompiler},
        special_functions, ParserError,
    },
    traits::IntoOwned,
    value::{Primitive, Value, ValueType},
    vm::OpCode,
};

define_node!(FnAssignNode(
    name_span: TokenSpan,
    returns: Option<TokenSpan>,
    args: Vec<(TokenSpan, Option<TokenSpan>, Option<Node<'source>>)>, // (name, type, default)
    body: Node<'source>,
    docs: Vec<Token<'source>>,
) {

    "Function assignment - Registers a function with the given name."
    "The function can be called later using the name."
    "Can be preceded by a docblock"
    "Args can have optional types and default values, e.g. `foo(a: int, b: int = 0) {}`"
    "`
        DocBlockComment* ~ At? ~ Identifer ~ EOL* ~ LParen ~ EOL* ~
            (Identifier ~ (EOL* ~ COLON ~ IDENTIFIER)? ~ (Assign ~ EXPR)? ~ EOL* ~ Comma ~ EOL*)* ~ (Identifier ~ (EOL* ~ COLON ~ IDENTIFIER)? ~ (Assign ~ EXPR)?)? ~ EOL* ~ 
        RParen ~ (EOL* ~ COLON ~ IDENTIFIER)? ~ EOL* ~ Assign ~ Block
    `"

    build(tokens) {
        tokens.start_transaction();


        // DocBlockComment*
        let mut docs = vec![];
        while let Some(doc) = terminal!(DocBlockComment?, tokens) {
            docs.push(doc);
            skip_eol!(tokens);
        }

        // At? ~ Identifer
        let decorator = terminal!(Decorator?, tokens);
        let has_decorator = decorator.is_some();
        let name = terminal!(LiteralIdent, tokens)?;

        // Build the token
        let mut token = name.child(Rule::FnAssignExpr, name.span());
        if let Some(t) = decorator {
            token.include_span(t.span());
        }

        // EOL* ~ LParen ~ EOL*
        terminal!(LParen, tokens, skip_eol!(tokens))?;

        // (Identifier ~ (EOL* ~ COLON ~ IDENTIFIER)? ~ (Assign ~ EXPR)? ~ EOL* ~ Comma ~ EOL*)*
        let mut args = vec![];
        loop {
            tokens.start_transaction();

            match terminal!(LiteralIdent, tokens) {
                Some(arg) => {
                    // (EOL* ~ COLON ~ IDENTIFIER)?
                    let argtype = {
                        tokens.start_transaction();
                        match terminal!(Colon, tokens, skip_eol!(tokens)) {
                            Some(_) => {
                                match terminal!(LiteralIdent, tokens, skip_eol!(tokens)) {
                                    Some(t) => {
                                        let ty = t.span();
                                        tokens.apply_transaction();
                                        Some(ty)
                                    },
                                    None => None
                                }
                            },
                            None => {
                                None
                            }
                        }
                    };

                    // (Assign ~ EXPR)?
                    let default = {
                        tokens.start_transaction();
                        match terminal!(Assign, tokens, skip_eol!(tokens)) {
                            Some(_) => {
                                match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                                    Some(expr) => {
                                        tokens.apply_transaction();
                                        Some(expr)
                                    },
                                    None => None
                                }
                            },
                            None => {
                                None
                            }
                        }
                    };

                    args.push((arg.span(), argtype, default));
                    if terminal!(Comma?, tokens, skip_eol!(tokens)).is_none() {
                        tokens.apply_transaction();
                        break;
                    }
                },
                None => {
                    break;
                }
            }

            tokens.apply_transaction();
        }

        // Identifier? ~ EOL*
        if let Some(arg) = terminal!(LiteralIdent?, tokens, skip_eol!(tokens)) {
            // (EOL* ~ COLON ~ IDENTIFIER)?
            let argtype = {
                tokens.start_transaction();
                match terminal!(Colon, tokens, skip_eol!(tokens)) {
                    Some(_) => {
                        match terminal!(LiteralIdent, tokens, skip_eol!(tokens)) {
                            Some(t) => {
                                let ty = t.span();
                                tokens.apply_transaction();
                                Some(ty)
                            },
                            None => None
                        }
                    },
                    None => {
                        None
                    }
                }
            };

            // (Assign ~ EXPR)?
            let default = {
                tokens.start_transaction();
                match terminal!(Assign, tokens, skip_eol!(tokens)) {
                    Some(_) => {
                        match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                            Some(expr) => {
                                tokens.apply_transaction();
                                Some(expr)
                            },

                            None => None
                        }
                    },
                    None => {
                        None
                    }
                }
            };

            args.push((arg.span(), argtype, default));
        }

        // RParen
        terminal!(RParen, tokens, skip_eol!(tokens))?;

        // (EOL* ~ COLON ~ IDENTIFIER)?
        let returns = {
            tokens.start_transaction();
            match terminal!(Colon, tokens, skip_eol!(tokens)) {
                Some(_) => {
                    match terminal!(LiteralIdent, tokens, skip_eol!(tokens)) {
                        Some(t) => {
                            let ty = t.span();
                            tokens.apply_transaction();
                            Some(ty)
                        },
                        None => None
                    }
                },
                None => {
                    None
                }
            }
        };

        //  EOL* ~ Assign ~ Block
        terminal!(Assign, tokens, skip_eol!(tokens))?;
        let body = non_terminal!(BlockNode, tokens, skip_eol!(tokens))?;

        if has_decorator && args.len() != 1 {
            return error_node!(ParserError::DecoratorSignature(token.into_owned()));
        }

        if args.len() > 255 {
            return error_node!(ParserError::TooManyArguments(token.into_owned()));
        }

        tokens.apply_transaction();
        Some(Self { name_span: name.span(), returns, args, body, docs, token }.into_node())
    }

    compile(this, compiler) {
        let name = this.token.input();
        let name = name[this.name_span.start..this.name_span.end].to_string();

        let arguments = this.args.into_iter().map(|(name, ty, default)| {
            let name = this.token.input()[name.start..name.end].to_string();
            let ty = ty.map(|ty| {
                let ty = &this.token.input()[ty.start..ty.end];
                ValueType::from_str(ty).unwrap_or(ValueType::All)
            }).unwrap_or(ValueType::All);

            let default = match default {
                Some(Node::LiteralString(s)) => FunctionArgumentDefault::Static(Value::Primitive(s.value)),
                Some(Node::LiteralInt(i)) => FunctionArgumentDefault::Static(Value::Primitive(i.value)),
                Some(Node::LiteralFloat(f)) => FunctionArgumentDefault::Static(Value::Primitive(f.value)),
                Some(Node::LiteralBool(b)) => FunctionArgumentDefault::Static(Value::Primitive(b.value)),
                Some(node) => FunctionArgumentDefault::Stack(node),
                None => FunctionArgumentDefault::None
            };

            (name, ty, default)
        }).collect::<Vec<_>>();

        let returns = this.returns.map(|returns| {
            let returns = &this.token.input()[returns.start..returns.end];
            ValueType::from_str(returns).unwrap_or(ValueType::All)
        }).unwrap_or(ValueType::All);

        compiler.push_token(this.token);

        let arg_names = arguments.iter().map(|(name, _, _)| name.as_str()).collect::<Vec<_>>();
        let doc_strings = this.docs.iter().map(|t| t.slice()[3..].trim()).collect::<Vec<_>>();
        let doc = FunctionDocs::parse_docblock(&name, arg_names.as_slice(), &doc_strings);

        let fcomp = FunctionCompiler {
            name,
            body: this.body,
            ty: returns,
            dbg: None,
            doc,
            args: arguments.into_iter().map(|(name, ty, default)| {
                FunctionArgumentCompiler {
                    name,
                    ty,
                    default
                }
            }).collect()
        };

        fcomp.compile(compiler)?;

        Ok(())
    }

    into_node(this) {
        Node::FnAssign(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            name_span: this.name_span,
            returns: this.returns,
            args: this.args.into_iter().map(|(name, ty, default)| {
                (name, ty, default.map(|d| d.into_owned()))
            }).collect(),
            body: this.body.into_owned(),
            docs: this.docs.into_iter().map(|t| t.into_owned()).collect(),
            token: this.token.into_owned(),
        }
    }
});

// return BLOCK?
define_node!(ReturnNode(value: Node<'source>) {
    "Return statement - returns from the current function."
    "Omitting the value is an error"
    "`return BLOCK?`"

    build(tokens) {
        tokens.start_transaction();

        let mut token = terminal!(Return, tokens)?;
        let value = non_terminal!(BlockNode?, tokens);
        if let Some(value) = &value {
            token.include_span(value.token().span());
        }

        tokens.apply_transaction();
        let token = token.child(Rule::ReturnExpr, token.span());
        match value {
            Some(value) => {
                Some(Self { value, token }.into_node())
            },
            None => error_node!(ParserError::MustReturnAValue(token.into_owned()))
        }
    }

    compile(this, compiler) {
        compiler.push_token(this.token);
        this.value.compile(compiler)?;
        compiler.push(OpCode::RET);
        Ok(())
    }

    into_node(this) {
        Node::Return(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            value: this.value.into_owned(),
            token: this.token.into_owned()
        }
    }
});

pratt_node!(FnCallNode(name_span: TokenSpan, args: Vec<Node<'source>>) {
    "Function call expression."
    "Can be a normal function call (e.g. `foo(1, 2, 3)`)"
    "Or a method call (e.g. `foo.bar(1, 2, 3)`)"

    build(token, lhs, op) {
        token.set_rule(Rule::FnCallExpr);

        // This is a bit of a hack, but it works
        let op = if let Node::PostfixFnCallOperator(op) = op { op } else { unreachable!() };

        // The operator either looks like `(args)` or `.name(args)`
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
    }

    compile(this, compiler) {
        let name = this.token.input();
        let name = name[this.name_span.start..this.name_span.end].to_string();

        let _token = this.token.clone();
        compiler.push_token(this.token);

        match name.as_str() {
            //
            // System call dispatch function
            // Calls arbitrary CPU ops and is used for the stdlib
            "__syscalld" if compiler.options().allow_syscalld => {
                let opcode = this.args.get(0).map(|a| a.token().slice()).unwrap_or("");
                let opcode = OpCode::from_str(opcode).map_err(|_| CompilerError::InvalidSyscallOpcode(_token.into_owned(), opcode.to_string()))?;
                special_functions::__syscalld(compiler, opcode, this.args[1..].to_vec())?;
            }

            "dissasemble" => {
                if this.args.len() != 1 {
                    return Err(CompilerError::InvalidArgumentCount(_token.into_owned(), name, 1, this.args.len()));
                }
                let expr = this.args.get(0).unwrap().clone();
                special_functions::__dissasemble(compiler, expr)?;
            }

            "include" => {
                if this.args.len() != 1 {
                    return Err(CompilerError::InvalidArgumentCount(_token.into_owned(), name, 1, this.args.len()));
                }
                let filename = match this.args.get(0).unwrap().clone() {
                    Node::LiteralString(s) => match s.value {
                        Primitive::String(s) => s,
                        _ => unreachable!()
                    },
                    _ => return Err(CompilerError::InvalidInclude(_token.into_owned()))
                };

                special_functions::__include(compiler, _token, filename)?;
            }

            //
            // Normal function call
            _ => {
                let n_args = this.args.len();
                for arg in this.args {
                    arg.compile(compiler)?;
                }

                compiler.push(OpCode::CALL);
                compiler.push_strhash(&name);
                compiler.push_u64(n_args as u64);
            }
        }

        Ok(())
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

            match terminal!(Dot, tokens, skip_eol!(tokens)) {
                Some(dot) => {
                    token = Some(dot);
                },
                _ => {
                    break 'object_mode;
                }
            }

            match terminal!(LiteralIdent, tokens, skip_eol!(tokens)) {
                Some(name) => {
                    name_span = Some(name.span());
                },
                _ => {
                    break 'object_mode;
                }
            }

            tokens.apply_transaction();
        }

        // "(" ~ EOL*
        match terminal!(LParen, tokens, skip_eol!(tokens)) {
            Some(t) => {
                let mut t = t.child(Rule::FnCallOperator, t.span());
                if let Some(token) = token {
                    t.include_span(token.span());
                }
                token = Some(t);
            },
            None => return None
        }

        let mut token = token.unwrap();

        // (EXPR ~ EOL* ~ symbol_comma ~ EOL*)*
        loop {
            tokens.start_transaction();

            let expr = match non_terminal!(ExpressionNode, tokens, skip_eol!(tokens)) {
                Some(expr) => expr,
                None => {
                    break;
                }
            };

            if terminal!(Comma, tokens, skip_eol!(tokens)).is_none() {
                break;
            }

            args.push(expr);
            tokens.apply_transaction();
        }

        //  EXPR? ~ EOL* ~ ")"
        if let Some(arg) = non_terminal!(ExpressionNode?, tokens, skip_eol!(tokens)) {
            args.push(arg);
        }

        token.include_span(terminal!(RParen, tokens, skip_eol!(tokens))?.span());

        tokens.apply_transaction();
        Some(Self { name_span, args, token }.into_node())
    }

    compile(_this, _compiler) {
        unreachable!("Intermediate node")
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
