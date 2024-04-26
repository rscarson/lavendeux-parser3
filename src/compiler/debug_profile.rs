use std::borrow::Cow;

use crate::{
    lexer::{SerializedToken, Token},
    traits::{IntoOwned, SerializeToBytes},
};

/// Maps ranges in bytecode to source code locations.
#[derive(Debug, Clone)]
pub struct DebugProfile<'source> {
    input: Cow<'source, str>,
    source: Vec<(String, String)>,
    map: Vec<(usize, SerializedToken)>,
}

impl<'source> DebugProfile<'source> {
    /// Create a new DebugProfile from the given source code.
    pub fn new(input: &'source str) -> Self {
        Self {
            input: Cow::Borrowed(input),
            map: Vec::new(),
        }
    }

    fn get(&'source self, index: usize) -> Option<Token<'source>> {
        self.map
            .get(index)
            .map(move |(_, token)| SerializedToken::unpack(&token, &self.input))
    }

    /// Insert a token into the profile.
    pub fn insert(&mut self, start_pos: usize, token: Token<'source>) {
        self.map.push((start_pos, SerializedToken::pack(token)))
    }

    /// Offset all token source starts backwards by the given amount.
    pub fn offset(&mut self, offset: usize) {
        for (_, token) in &mut self.map {
            token.span.start -= offset;
            token.span.end -= offset;
        }
    }

    /// Get the token at the given index.
    pub fn current_token(&'source self, index: usize) -> Option<Token<'source>> {
        // Search the map, returning the last token that starts before the index.
        let i = self.map.partition_point(|(start, _)| *start <= index);
        let i = match i {
            0 => 0,
            i => i - 1,
        };

        self.get(i)
    }

    /// Get the source code for all tokens, mapped to their start position.
    pub fn all_slices(&self) -> impl Iterator<Item = (usize, &str)> {
        self.map
            .iter()
            .map(move |(start, token)| (*start, &self.input[token.span.clone()]))
    }
}

impl IntoOwned for DebugProfile<'_> {
    type Owned = DebugProfile<'static>;

    fn into_owned(self) -> Self::Owned {
        Self::Owned {
            input: Cow::Owned(self.input.into_owned()),
            map: self.map,
        }
    }
}

impl SerializeToBytes for DebugProfile<'_> {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let input = self.input.into_owned();
        let tokens = self.map;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&input.serialize_into_bytes());

        bytes.extend_from_slice(&tokens.len().serialize_into_bytes());
        for (start, token) in tokens {
            bytes.extend_from_slice(&start.serialize_into_bytes());
            bytes.extend_from_slice(&token.serialize_into_bytes());
        }

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let input = String::deserialize_from_bytes(bytes)?;
        let tokens = usize::deserialize_from_bytes(bytes)?;

        let mut inst = Self {
            input: Cow::Owned(input),
            map: Vec::with_capacity(tokens),
        };

        for _ in 0..tokens {
            let start = usize::deserialize_from_bytes(bytes)?;
            let token = SerializedToken::deserialize_from_bytes(bytes)?;
            inst.map.push((start, token));
        }

        Ok(inst)
    }
}
