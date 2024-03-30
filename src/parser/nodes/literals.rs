use super::*;
use crate::literals;
use crate::types::Currency;
use crate::{error::Error, lexer::Rule, IntoOwned};

node_silent!(LiteralRegexNode(tokens) {
    tokens.start_transaction();
    let token = terminal!(LiteralRegex, tokens)?;
    tokens.apply_transaction();

    let value = token.slice().to_string();
    Some(LiteralStringNode { value, token }.into_node())
});

define_node!(LiteralStringNode(value: String) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralString, tokens)?;
        tokens.apply_transaction();

        match literals::string(token.slice()) {
            Ok(value) => Some(Self { token, value }.into_node()),
            Err(e) => error_node!(Error::InvalidLiteral(token.into_owned(), e)),
        }
    }

    into_node(this) {
        Node::LiteralString(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            value: this.value,
            token: this.token.into_owned()
        }
    }
});

node_silent!(LiteralRadixNode(tokens) {
    tokens.start_transaction();
    let token = terminal!(LiteralRadix, tokens)?;
    tokens.apply_transaction();

    let radix = token.slice().chars().nth(1).unwrap();
    let base = &token.slice()[2..];

    match literals::radix(base, radix) {
        Ok(value) => Some(LiteralIntNode { value, token }.into_node()),
        Err(e) => error_node!(Error::InvalidLiteral(token.into_owned(), e)),
    }
});

define_node!(LiteralIntNode(value: i128) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralInt, tokens)?;
        tokens.apply_transaction();

        match literals::int(token.slice()) {
            Ok(value) => Some(Self { token, value }.into_node()),
            Err(e) => error_node!(Error::InvalidLiteral(token.into_owned(), e)),
        }
    }

    into_node(this) {
        Node::LiteralInt(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            token: this.token.into_owned(),
            value: this.value
        }
    }
});

define_node!(LiteralFloatNode(value: f64) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralFloat, tokens)?;
        tokens.apply_transaction();

        match literals::float(token.slice()) {
            Ok(value) => Some(Self { token, value }.into_node()),
            Err(e) => error_node!(Error::InvalidLiteral(token.into_owned(), e)),
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
        Some(Node::LiteralBool(Box::new(Self { token, value })))
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

define_node!(LiteralCurrencyNode(
    value: Currency
) {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralPrefixedCurrency|LiteralSuffixedCurrency, tokens)?;

        tokens.apply_transaction();
        let value = match token.rule() {
            Rule::LiteralPrefixedCurrency => Currency::new_prefixed(token.slice()),
            _ => Currency::new_suffixed(token.slice())
        };

        Some(Self { token, value }.into_node())
    }

    into_node(this) {
        Node::LiteralCurrency(Box::new(this))
    }

    into_owned(this) {
        Self::Owned {
            value: this.value,
            token: this.token.into_owned()
        }
    }
});

define_node!(LiteralIdentNode() {
    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralIdent, tokens)?;

        tokens.apply_transaction();
        Some(Self { token }.into_node())
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
        Rule::LiteralConstPi => Some(LiteralFloatNode { token, value: std::f64::consts::PI }.into_node()),
        Rule::LiteralConstE => Some(LiteralFloatNode { token, value: std::f64::consts::E }.into_node()),
        Rule::LiteralConstTau => Some(LiteralFloatNode { token, value: std::f64::consts::TAU }.into_node()),
        Rule::LiteralConstNil|Rule::LiteralConstFalse => Some(LiteralBoolNode { token, value: false }.into_node()),
        Rule::LiteralConstTrue => Some(LiteralBoolNode { token, value: true }.into_node()),

        _ => unreachable!("Invalid constant rule: {:?}", token.rule())
    };

    tokens.apply_transaction();
    node
});
