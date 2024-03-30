use super::*;
use crate::{
    error::Error,
    lexer::{Rule, TokenSpan},
    IntoOwned,
};

#[derive(Debug, Clone)]
pub enum AssignmentTarget<'source> {
    Indexed {
        name_span: TokenSpan,
        path: Vec<Option<Node<'source>>>,
    },

    Plural {
        names: Vec<TokenSpan>,
    },

    Decorator {
        name_span: TokenSpan,
    },
}

impl IntoOwned for AssignmentTarget<'_> {
    type Owned = AssignmentTarget<'static>;

    fn into_owned(self) -> Self::Owned {
        match self {
            AssignmentTarget::Indexed { name_span, path } => AssignmentTarget::Indexed {
                name_span: name_span,
                path: path
                    .into_iter()
                    .map(|node| node.map(Node::into_owned))
                    .collect(),
            },

            AssignmentTarget::Plural { names } => AssignmentTarget::Plural { names: names },

            AssignmentTarget::Decorator { name_span } => AssignmentTarget::Decorator {
                name_span: name_span,
            },
        }
    }
}

impl<'source> AssignmentTarget<'source> {
    pub fn from_node(node: Node<'source>) -> Option<Self> {
        match node {
            Node::Array(expr) => {
                let mut names = vec![];
                for node in expr.elements {
                    if let Node::LiteralIdent(ident) = node {
                        names.push(ident.token.span());
                    } else {
                        return None;
                    }
                }
                Some(Self::Plural { names })
            }

            Node::IndexingExpr(expr) => {
                if let Node::LiteralIdent(ident) = expr.base {
                    Some(Self::Indexed {
                        name_span: ident.token.span(),
                        path: expr.path,
                    })
                } else {
                    None
                }
            }

            Node::LiteralIdent(ident) => Some(Self::Plural {
                names: vec![ident.token.span()],
            }),
            _ => None,
        }
    }
}

pratt_node!(AssignExprNode(target: AssignmentTarget<'source>, value: Node<'source>) {
    build(token, lhs, _op, rhs) {
        token.set_rule(Rule::AssignExpr);
        let lhs_token = lhs.token().clone();
        match AssignmentTarget::from_node(lhs) {
            Some(target) => Some(Self { target, value: rhs, token }.into_node()),
            None => {
                error_node!(Error::AssignmentToConstant(lhs_token.clone().into_owned()))
            }
        }
    }

    into_node(this) {
        Node::AssignExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            value: this.value.into_owned(),
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(AssignArithmeticExprNode(assignment: AssignExprNode<'source>, op: Rule) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::AssignArithmeticExpr);
        let op_rule = op.token().rule();
        let assignment = AssignExprNode::parse(token.clone(), lhs, op, rhs)?;
        let assignment = match assignment {
            Node::AssignExpr(assignment) => *assignment,
            _ => unreachable!(),
        };

        Some(Self { assignment, op: op_rule, token }.into_node())
    }

    into_node(this) {
        Node::AssignArithmeticExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            assignment: this.assignment.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(AssignBitwiseExprNode(assignment: AssignExprNode<'source>, op: Rule) {
    build(token, lhs, op, rhs) {
        token.set_rule(Rule::AssignBitwiseExpr);
        let op_rule = op.token().rule();
        let assignment = AssignExprNode::parse(token.clone(), lhs, op, rhs)?;
        let assignment = match assignment {
            Node::AssignExpr(assignment) => *assignment,
            _ => unreachable!(),
        };

        Some(Self { assignment, op: op_rule, token }.into_node())
    }

    into_node(this) {
        Node::AssignBitwiseExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            assignment: this.assignment.into_owned(),
            op: this.op,
            token: this.token.into_owned(),
        }
    }
});

pratt_node!(DeleteExprNode(target: AssignmentTarget<'source>) {
    build(token, term, op) {
        token.set_rule(Rule::DeleteExpr);

        let target = if op.token().slice().ends_with('@') {
            if let Node::LiteralIdent(ident) = term {
                AssignmentTarget::Decorator {
                    name_span: ident.token.span()
                }
            } else{
                return error_node!(Error::NotADecorator(term.token().clone().into_owned()));
            }
        } else {
            let term_token = term.token().clone();
            match AssignmentTarget::from_node(term) {
                Some(target) => target,
                None => return error_node!(Error::AssignmentToConstant(term_token.into_owned())),
            }
        };

        Some(Self { target, token }.into_node())
    }

    into_node(this) {
        Node::DeleteExpr(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            target: this.target.into_owned(),
            token: this.token.into_owned(),
        }
    }
});
