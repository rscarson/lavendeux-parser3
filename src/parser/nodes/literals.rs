use super::*;
use crate::literals;
use crate::types::Currency;
use crate::{error::Error, lexer::Rule, IntoOwned};

node_silent!(LiteralRegexNode {
    "Regular expression literal - literally just a string."
    "`/regex/flags`"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralRegex, tokens)?;
        tokens.apply_transaction();

        let value = token.slice().to_string();
        Some(LiteralStringNode { value, token }.into_node())
    }
});

define_node!(LiteralStringNode(value: String) {
    "String literal - a sequence of characters enclosed in double or single quotes."
    "Supports escape sequences for quotes, backslashes, and \\n \\r \\t, plus unicode escapes like \\u{1234}"

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

node_silent!(LiteralRadixNode {
    "Radix literal - an integer literal with a base prefix."
    "Supports binary (0b), octal (0o), and hexadecimal (0x) bases."
    "Can contain underscores for readability, e.g. 0b_1010_1010"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralRadix, tokens)?;
        tokens.apply_transaction();

        let radix = token.slice().chars().nth(1).unwrap();
        let base = &token.slice()[2..];

        match literals::radix(base, radix) {
            Ok(value) => Some(LiteralIntNode { value, token }.into_node()),
            Err(e) => error_node!(Error::InvalidLiteral(token.into_owned(), e)),
        }
    }
});

define_node!(LiteralIntNode(value: i128) {
    "128bit integer literal."
    "Can contain underscores for readability, e.g. 1_000_000"

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
    "64bit floating point literal."
    "Can contain underscores for readability, e.g. `1_000_000.0`."
    "Can be in scientific notation, e.g. `1e6` or `1.0e6`."
    "Integer part can be omitted, e.g. `.5`"

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
    "Boolean literal - either `true` or `false`"

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

define_node!(LiteralCurrencyNode(value: Currency) {
    "Currency literal - a number with a currency symbol or code."
    "Can be prefixed or suffixed with the currency symbol or code."
    "Suffix can also be one of [`dDfF`] for fixed-point literals"

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
    "Identifier literal."
    "Can contain letters, numbers, and underscores, but not start with a number."
    "Usually identifies a variable or function name"

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
    /// Get the name of the identifier
    pub fn name(&self) -> &str {
        self.token.slice()
    }
}

node_silent!(LiteralConstNode {
    "Constant literal - a predefined constant value."
    "Can be `nil`, `true`, `false`, `pi`, `e`, or `tau`"

    build(tokens) {
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
    }
});
