use crate::traits::{ByteDecodeError, SerializeToBytes};

/// Function documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDocs {
    /// The category of the function.
    /// Defaults to "Misc"
    pub category: String,

    /// The function's signature.
    pub signature: String,

    /// A short description of the function.
    pub short: Option<String>,

    /// A longer description of the function.
    pub desc: Option<String>,

    /// An example of how to use the function.
    pub example: Option<String>,
}

impl Default for FunctionDocs {
    fn default() -> Self {
        Self {
            category: "Misc".to_string(),
            signature: "".to_string(),
            short: None,
            desc: None,
            example: None,
        }
    }
}

impl FunctionDocs {
    /// Parse a docblock using the following format:
    /// ```text
    /// //# category: Category Name
    /// //# Single line description
    /// //# Multi-line description
    /// //# Can have multiple lines
    /// //# ```lav
    /// //# Example code
    /// //# ``` (optional closing tag)
    /// ```
    pub fn parse_docblock(lines: &[&str]) -> Self {
        // Check if the first line is a category line
        let mut iter = lines.iter().peekable();

        // Category at the top of the docblock
        let category = match iter.peek() {
            Some(&line) if line.starts_with("category:") => {
                iter.next();
                let category = line.trim_start_matches("category:").trim();
                category.to_string()
            }
            _ => "Misc".to_string(),
        };

        // The next line is the short description
        let short = iter.next().map(|line| line.to_string());

        // All lines until the code block are the description
        let mut desc = String::new();
        while let Some(line) = iter.next() {
            if line.starts_with("```") {
                break;
            }
            desc.push_str(line);
            desc.push('\n');
        }

        // The code block is the example
        let mut example = None;
        match iter.next() {
            Some(line) if line.starts_with("```") => {
                let mut example_code = String::new();
                for line in iter {
                    if line.starts_with("```") {
                        break;
                    }
                    example_code.push_str(line);
                    example_code.push('\n');
                }
                example = Some(example_code);
            }
            _ => {}
        }

        Self {
            category,
            signature: "".to_string(),
            short,
            desc: Some(desc),
            example,
        }
    }
}

impl SerializeToBytes for FunctionDocs {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.category.serialize_into_bytes());
        bytes.extend_from_slice(&self.signature.serialize_into_bytes());
        bytes.extend_from_slice(&self.short.serialize_into_bytes());
        bytes.extend_from_slice(&self.desc.serialize_into_bytes());
        bytes.extend_from_slice(&self.example.serialize_into_bytes());
        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let category = String::deserialize_from_bytes(bytes)?;
        let signature = String::deserialize_from_bytes(bytes)?;
        let short = Option::<String>::deserialize_from_bytes(bytes)?;
        let desc = Option::<String>::deserialize_from_bytes(bytes)?;
        let example = Option::<String>::deserialize_from_bytes(bytes)?;
        Ok(Self {
            category,
            signature,
            short,
            desc,
            example,
        })
    }
}
