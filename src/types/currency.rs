#[derive(Debug, Clone)]
pub struct Currency {
    pub symbol: char,
    pub is_prefixed: bool,
    pub value: fpdec::Decimal,
}

impl Currency {
    pub fn new_prefixed(input: &str) -> Self {
        let symbol = input.chars().nth(0).unwrap();
        let value = input[1..].parse().unwrap();
        Self {
            symbol,
            is_prefixed: true,
            value,
        }
    }

    pub fn new_suffixed(input: &str) -> Self {
        let symbol = input.chars().last().unwrap();
        let value = input[..input.len() - 1].parse().unwrap();
        Self {
            symbol,
            is_prefixed: false,
            value,
        }
    }
}
