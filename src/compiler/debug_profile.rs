use crate::{
    lexer::{SerializedToken, Token},
    traits::SerializeToBytes,
};

/// Maps ranges in bytecode to source code locations.
#[derive(Debug, Clone)]
pub struct DebugProfile {
    sources: Vec<(String, String)>,
    map: Vec<(usize, SerializedToken)>,
}

impl DebugProfile {
    /// Create a new DebugProfile from the given source code.
    pub fn new(input: &str) -> Self {
        Self {
            sources: vec![(String::new(), input.to_string())],
            map: Vec::new(),
        }
    }

    /// Get the source code for the given filename
    fn get_source(&self, filename: Option<&str>) -> Option<&str> {
        match filename {
            None => Some(&self.sources[0].1),
            Some(filename) => {
                for (name, source) in &self.sources {
                    if name == filename {
                        return Some(source);
                    }
                }

                None
            }
        }
    }

    /// Get the token at the given index.
    fn get(&self, index: usize) -> Option<(usize, Token<'_>)> {
        let (start, packed) = self.map.get(index)?;
        let source = self.get_source(packed.filename.as_deref())?;
        let token = SerializedToken::unpack(packed, source);
        Some((*start, token))
    }

    /// Insert a token into the profile.
    pub fn insert(&mut self, start_pos: usize, token: Token<'_>) {
        if self.get_source(token.filename().as_deref()).is_none() {
            self.sources.push((
                token.filename().unwrap_or_default().to_string(),
                token.input().to_string(),
            ));
        }
        self.map.push((start_pos, SerializedToken::pack(token)))
    }

    /// Offset all token source starts backwards by the given amount.
    pub fn offset(&mut self, filename: Option<String>, offset: usize) {
        for (name, source) in &mut self.sources {
            if Some(name.as_str()) == filename.as_deref() {
                *source = source[offset..].to_string();
            }
        }
        for (_, token) in &mut self.map {
            if token.filename != filename {
                continue;
            }
            token.span.start -= offset;
            token.span.end -= offset;
        }
    }

    /// Get the token at the given index.
    pub fn current_token(&self, index: usize) -> Option<Token<'_>> {
        // Search the map, returning the last token that starts before the index.
        let i = self.map.partition_point(|(start, _)| *start <= index);
        let i = match i {
            0 => 0,
            i => i - 1,
        };

        let (_, token) = self.get(i)?;
        Some(token)
    }

    /// Get the source code for all tokens, mapped to their start position.
    pub fn all_slices(&self) -> impl Iterator<Item = (usize, String)> + '_ {
        (0..self.map.len())
            .map(|i| self.get(i).unwrap())
            .map(|(s, t)| (s, t.slice().to_string()))
    }
}

impl SerializeToBytes for DebugProfile {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Serialize source map
        bytes.extend_from_slice(&self.sources.len().serialize_into_bytes());
        for (name, source) in self.sources {
            bytes.extend_from_slice(&name.serialize_into_bytes());
            bytes.extend_from_slice(&source.serialize_into_bytes());
        }

        // Serialize token map
        bytes.extend_from_slice(&self.map.len().serialize_into_bytes());
        for (start, token) in self.map {
            bytes.extend_from_slice(&start.serialize_into_bytes());
            bytes.extend_from_slice(&token.serialize_into_bytes());
        }

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let nsources = usize::deserialize_from_bytes(bytes)?;
        let mut sources = Vec::with_capacity(nsources);
        for _ in 0..nsources {
            let name = String::deserialize_from_bytes(bytes)?;
            let source = String::deserialize_from_bytes(bytes)?;
            sources.push((name, source));
        }

        let ntokens = usize::deserialize_from_bytes(bytes)?;
        let mut map = Vec::with_capacity(ntokens);
        for _ in 0..ntokens {
            let start = usize::deserialize_from_bytes(bytes)?;
            let token = SerializedToken::deserialize_from_bytes(bytes)?;
            map.push((start, token));
        }

        Ok(Self { sources, map })
    }
}
