use super::*;
use crate::{error::Error, tokenizer::Rule, IntoOwned};

define_node!(LiteralStringNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralString, tokens)?;
        tokens.apply_transaction();
        Ok(Node::LiteralString(Box::new(Self { token })))
    }

    into_node(this) {
        Node::LiteralString(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

define_node!(LiteralFloatNode(value: f64) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralFloat, tokens)?;
        match token.slice().replace('_', "").parse() {
            Ok(value) => {
                tokens.apply_transaction();
                Ok(Self { token, value }.into_node())
            },
            Err(_) => {
                tokens.revert_transaction();
                Err(Error::InvalidFloatLiteral(token.into_owned()))
            },
        }
    }

    into_node(this) {
        Node::LiteralFloat(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned(),
            value: this.value
        }
    }
});

define_node!(LiteralBoolNode(value: bool) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralConstFalse|LiteralConstTrue|LiteralConstNil, tokens)?;
        let value = match token.rule() {
            Rule::LiteralConstTrue => true,
            _ => false,
        };
        tokens.apply_transaction();
        Ok(Node::LiteralBool(Box::new(Self { token, value })))
    }

    into_node(this) {
        Node::LiteralBool(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned(),
            value: this.value
        }
    }
});

define_node!(LiteralCurrencyNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralCurrency, tokens)?;
        tokens.apply_transaction();
        Ok(Self { token }.into_node())
    }

    into_node(this) {
        Node::LiteralCurrency(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

define_node!(LiteralDecimalNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralDecimal, tokens)?;
        tokens.apply_transaction();
        Ok(Self { token }.into_node())
    }

    into_node(this) {
        Node::LiteralDecimal(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});

macro_rules! define_intliteral_node {
    ($name:ident : $size:ty : $token:ident) => {
        define_node!($name(value: $size) {
            build(tokens) {
                tokens.start_transaction();
                let token = terminal!($token, tokens)?;
                match token.slice().replace('_', "").parse::<$size>() {
                    Ok(value) => {
                        tokens.apply_transaction();
                        Ok(Self { token, value }.into_node())
                    },
                    Err(_) => {
                        tokens.revert_transaction();
                        Err(Error::InvalidIntLiteral(token.into_owned()))
                    },
                }
            }

            into_node(this) {
                Node::$token(Box::new(this))
            }

            into_owned(this) {
                Self::Owned {
                    token: this.token.into_owned(),
                    value: this.value
                }
            }
        });
    };
}
define_intliteral_node!(LiteralI64Node: i64: LiteralI64);
define_intliteral_node!(LiteralI32Node: i32: LiteralI32);
define_intliteral_node!(LiteralI16Node: i16: LiteralI16);
define_intliteral_node!(LiteralI8Node: i8: LiteralI8);
define_intliteral_node!(LiteralU64Node: u64: LiteralU64);
define_intliteral_node!(LiteralU32Node: u32: LiteralU32);
define_intliteral_node!(LiteralU16Node: u16: LiteralU16);
define_intliteral_node!(LiteralU8Node: u8: LiteralU8);

define_node!(LiteralIdentNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralIdent, tokens)?;

        tokens.apply_transaction();
        Ok(Self { token }.into_node())
    }

    into_node(this) {
        Node::LiteralIdent(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned()
        }
    }
});
impl LiteralIdentNode<'_> {
    pub fn name(&self) -> &str {
        self.token.slice()
    }
}

node_silent!(LiteralConstNode(tokens) {
    tokens.start_transaction();
    let token = terminal!(LiteralConstPi|LiteralConstE|LiteralConstTau|LiteralConstNil|LiteralConstTrue|LiteralConstFalse, tokens)?;
    let node = match token.rule() {
        Rule::LiteralConstPi => Ok(LiteralFloatNode { token, value: std::f64::consts::PI }.into_node()),
        Rule::LiteralConstE => Ok(LiteralFloatNode { token, value: std::f64::consts::E }.into_node()),
        Rule::LiteralConstTau => Ok(LiteralFloatNode { token, value: std::f64::consts::TAU }.into_node()),
        Rule::LiteralConstNil|Rule::LiteralConstFalse => Ok(LiteralBoolNode { token, value: false }.into_node()),
        Rule::LiteralConstTrue => Ok(LiteralBoolNode { token, value: true }.into_node()),

        _ => unreachable!("Invalid constant rule: {:?}", token.rule())
    };

    tokens.apply_transaction();
    node
});
