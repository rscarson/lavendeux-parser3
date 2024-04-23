use super::*;
use crate::literals::{self};
use crate::parser::{ParserError, ParserNode};
use crate::traits::SerializeToBytes;
use crate::value::{Number, NumberSymbol, Primitive};
use crate::vm::OpCode;
use crate::{lexer::Rule, traits::IntoOwned};

node_silent!(LiteralRegexNode {
    "Regular expression literal - literally just a string."
    "`/regex/flags`"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralRegex, tokens)?;
        tokens.apply_transaction();

        let value = token.slice().to_string();
        Some(LiteralStringNode { value: Primitive::String(value), token }.into_node())
    }
});

define_node!(LiteralStringNode(value: Primitive) {
    "String literal - a sequence of characters enclosed in double or single quotes."
    "Supports escape sequences for quotes, backslashes, and \\n \\r \\t, plus unicode escapes like \\u{1234}"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralString, tokens)?;
        tokens.apply_transaction();

        match literals::string(token.slice()) {
            Ok(value) => Some(Self { token, value: Primitive::String(value) }.into_node()),
            Err(e) => error_node!(ParserError::InvalidLiteral(token.into_owned(), e)),
        }
    }

    // PUSH STRING <len> <bytes>
    compile(this, compiler) {
        compiler.push_token(this.token);

        compiler.push(OpCode::PUSH);
        compiler.extend(this.value.serialize_into_bytes());

        Ok(())
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

define_node!(LiteralIntNode(value: Primitive) {
    "integer literal."
    "Can contain underscores for readability, e.g. 1_000_000"
    "Can be in binary, octal, decimal, or hexadecimal format, e.g. `0b1010`, `0o755`, `123`, `0xdeadbeef`"
    "Can be suffixed with `u` or `i` for unsigned or signed integers, e.g. `123u8`, `0xdeadbeefi64`"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(
            LiteralInt|LiteralRadix,
            tokens)?;
        tokens.apply_transaction();

        let value = match token.rule() {
            Rule::LiteralInt => match literals::int(token.slice()) {
                Ok(value) => value,
                Err(e) => return error_node!(ParserError::InvalidLiteral(token.into_owned(), e)),
            },

            Rule::LiteralRadix => {
                let base = token.slice().chars().nth(1).unwrap();
                let radix = &token.slice()[2..];
                match literals::radix(radix, base) {
                    Ok(value) => value,
                    Err(e) => return error_node!(ParserError::InvalidLiteral(token.into_owned(), e)),
                }
            },

            _ => unreachable!("Invalid integer rule: {:?}", token.rule())
        };

        Some(Self { token, value: Primitive::Integer(value) }.into_node())
    }

    // PUSH INT <bytes>
    compile(this, compiler) {
        compiler.push_token(this.token);

        compiler.push(OpCode::PUSH);
        compiler.extend(this.value.serialize_into_bytes());

        Ok(())
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

define_node!(LiteralFloatNode(value: Primitive) {
    "Decimal fixed-point literal."
    "Can contain underscores for readability, e.g. `1_000_000.0`."
    "Can be in scientific notation, e.g. `1e6` or `1.0e6`."
    "Integer part can be omitted, e.g. `.5`"
    "Can be prefixed or suffixed with a currency symbol or code, e.g. `$1.00`, `1.00€`."

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralFloat|LiteralPrefixedCurrency|LiteralSuffixedCurrency, tokens)?;
        tokens.apply_transaction();

        let (value, symbol) = match token.rule() {
            Rule::LiteralFloat => (token.slice(), None),
            Rule::LiteralPrefixedCurrency => (&token.slice()[1..], token.slice().chars().next().map(|c| NumberSymbol::Prefix(c.to_string()))),
            Rule::LiteralSuffixedCurrency => (&token.slice()[..token.slice().len()-1], token.slice().chars().last().map(|c| NumberSymbol::Suffix(c.to_string()))),
            _ => unreachable!("Invalid float rule: {:?}", token.rule())
        };
        let value = match literals::decimal(value) {
            Ok(value) => value,
            Err(e) => return error_node!(ParserError::InvalidLiteral(token.into_owned(), e)),
        };
        let precision = symbol.as_ref().map(|_| value.n_frac_digits() as i8);

        let value = Number::new(value, symbol, precision);
        Some(Self { token, value: Primitive::Decimal(value) }.into_node())
    }

    // PUSH DEC <len> <bytes>
    compile(this, compiler) {
        compiler.push_token(this.token);

        compiler.push(OpCode::PUSH);
        compiler.extend(this.value.serialize_into_bytes());

        Ok(())
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

define_node!(LiteralBoolNode(value: Primitive) {
    "Boolean literal - either `true` or `false`"

    build(tokens) {
        tokens.start_transaction();
        let token = terminal!(LiteralConstFalse|LiteralConstTrue|LiteralConstNil, tokens)?;
        let value = match token.rule() {
            Rule::LiteralConstTrue => true,
            _ => false,
        };
        tokens.apply_transaction();
        Some(Node::LiteralBool(Box::new(Self { token, value: Primitive::Boolean(value) })))
    }

    // PUSH BOOL <bytes>
    compile(this, compiler) {
        compiler.push_token(this.token);

        compiler.push(OpCode::PUSH);
        compiler.extend(this.value.serialize_into_bytes());

        Ok(())
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

    compile(this, compiler) {
        let name = this.token.slice().to_string();
        compiler.push_token(this.token);
        compiler.push(OpCode::REF);
        compiler.push_strhash(&name);
        Ok(())
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
            Rule::LiteralConstPi => Some(LiteralFloatNode { token, value: Primitive::Decimal(Number::pi()) }.into_node()),
            Rule::LiteralConstE => Some(LiteralFloatNode { token, value: Primitive::Decimal(Number::e()) }.into_node()),
            Rule::LiteralConstTau => Some(LiteralFloatNode { token, value: Primitive::Decimal(Number::tau()) }.into_node()),
            Rule::LiteralConstNil|Rule::LiteralConstFalse => Some(LiteralBoolNode { token, value: Primitive::Boolean(false.into()) }.into_node()),
            Rule::LiteralConstTrue => Some(LiteralBoolNode { token, value: Primitive::Boolean(true.into()) }.into_node()),

            _ => unreachable!("Invalid constant rule: {:?}", token.rule())
        };

        tokens.apply_transaction();
        node
    }
});
