#[derive(thiserror::Error, Debug, Clone)]
pub enum LiteralError {
    #[error("Invalid float literal")]
    InvalidFloatLiteral,

    #[error("Invalid base `0{0}` for radix literal")]
    InvalidRadixLiteral(char),

    #[error("Invalid integer literal")]
    InvalidIntLiteral,

    #[error("Invalid escape sequence `\\{0}`")]
    InvalidEscapeSequence(char),
}

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

pub fn int(slice: &str) -> Result<i128, LiteralError> {
    slice
        .replace('_', "")
        .parse()
        .map_err(|_| LiteralError::InvalidIntLiteral)
}

pub fn float(slice: &str) -> Result<f64, LiteralError> {
    slice
        .replace('_', "")
        .parse()
        .map_err(|_| LiteralError::InvalidFloatLiteral)
}

pub fn string(slice: &str) -> Result<String, LiteralError> {
    let slice = slice[1..slice.len() - 1].chars();
    let mut output = String::new();
    let mut escape = false;
    for c in slice {
        if escape {
            match c {
                '\'' => output.push('\''),
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                _ => return Err(LiteralError::InvalidEscapeSequence(c)),
            }
            escape = false;
        } else {
            match c {
                '\\' => escape = true,
                _ => output.push(c),
            }
        }
    }

    if escape {
        Err(LiteralError::InvalidEscapeSequence('\\'))
    } else {
        Ok(output)
    }
}
