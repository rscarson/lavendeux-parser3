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
