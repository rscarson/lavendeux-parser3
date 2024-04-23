//! Literal parsing functions.
//! Creates primitive values from strings
use fpdec::Decimal;

/// Error occurred while parsing a literal.
#[derive(thiserror::Error, Debug, Clone)]
pub enum LiteralError {
    /// Invalid float literal.
    #[error("Invalid float literal")]
    InvalidFloatLiteral,

    /// Invalid base for radix literal.
    #[error("Invalid base `0{0}` for radix literal")]
    InvalidRadixLiteral(char),

    /// Invalid integer literal.
    #[error("Invalid integer literal")]
    InvalidIntLiteral,

    /// Invalid currency literal.
    #[error("Invalid currency literal")]
    InvalidCurrencyLiteral,

    /// Invalid escape sequence in a string.
    #[error("Invalid escape sequence `\\{0}`")]
    InvalidEscapeSequence(char),
}

/// Parse a string as an integer in the given base
pub fn radix(slice: &str, radix: char) -> Result<i128, LiteralError> {
    let radix = match radix {
        'b' => 2,
        'o' => 8,
        'd' => 10,
        'x' => 16,
        _ => return Err(LiteralError::InvalidRadixLiteral(radix)),
    };

    i128::from_str_radix(&slice.replace('_', ""), radix)
        .map_err(|_| LiteralError::InvalidIntLiteral)
}

/// Parse a string as an integer
pub fn int(slice: &str) -> Result<i128, LiteralError> {
    slice
        .replace('_', "")
        .parse()
        .map_err(|_| LiteralError::InvalidIntLiteral)
}

/// Parse a string as a decimal
pub fn decimal(slice: &str) -> Result<Decimal, LiteralError> {
    slice
        .replace('_', "")
        .parse()
        .map_err(|_| LiteralError::InvalidFloatLiteral)
}

/// Parse a string as a currency
pub fn string(slice: &str) -> Result<String, LiteralError> {
    let mut slice = slice[1..slice.len() - 1].chars();
    let mut output = String::new();

    loop {
        let c = match slice.next() {
            Some(c) => c,
            None => break,
        };

        match c {
            '\\' => match slice.next() {
                Some(c) => match c {
                    '\'' => output.push('\''),
                    '"' => output.push('"'),
                    '\\' => output.push('\\'),
                    'n' => output.push('\n'),
                    'r' => output.push('\r'),
                    't' => output.push('\t'),
                    'u' => {
                        let mut code = String::new();
                        for _ in 0..4 {
                            match slice.next() {
                                Some(c) => code.push(c),
                                None => return Err(LiteralError::InvalidEscapeSequence('\\')),
                            }
                        }
                        let code = u32::from_str_radix(&code, 16)
                            .map_err(|_| LiteralError::InvalidEscapeSequence('\\'))?;
                        output.push(
                            std::char::from_u32(code)
                                .ok_or(LiteralError::InvalidEscapeSequence('\\'))?,
                        );
                    }
                    _ => return Err(LiteralError::InvalidEscapeSequence(c)),
                },
                None => return Err(LiteralError::InvalidEscapeSequence('\\')),
            },

            _ => output.push(c),
        }
    }

    Ok(output)
}
