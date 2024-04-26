use crate::{
    traits::{ByteDecodeError, SerializeToBytes},
    value::{Primitive, Value},
};

/// Function documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDocs {
    /// The name of the function.
    pub name: String,

    /// The names of  the function's arguments.
    pub args: Vec<String>,

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
            name: "".to_string(),
            args: Vec::new(),
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
    pub fn parse_docblock(name: &str, args: &[&str], lines: &[&str]) -> Self {
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

        let desc = if desc.is_empty() { None } else { Some(desc) };

        Self {
            name: name.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            category,
            signature: "".to_string(),
            short,
            desc,
            example,
        }
    }

    /// Convert the function documentation into a hashmap.
    pub fn into_hashmap(self) -> std::collections::HashMap<Primitive, Value> {
        let mut docs = std::collections::HashMap::new();

        docs.insert(
            Primitive::String("name".to_string()),
            Value::string(self.name),
        );

        let args = self
            .args
            .into_iter()
            .map(|arg| Value::string(arg))
            .collect::<Vec<_>>();
        docs.insert(Primitive::String("args".to_string()), Value::Array(args));

        docs.insert(
            Primitive::String("category".to_string()),
            Value::string(self.category),
        );

        docs.insert(
            Primitive::String("signature".to_string()),
            Value::string(self.signature),
        );

        if let Some(short) = self.short {
            docs.insert(Primitive::String("short".to_string()), Value::string(short));
        }
        if let Some(desc) = self.desc {
            docs.insert(Primitive::String("desc".to_string()), Value::string(desc));
        }
        if let Some(example) = self.example {
            docs.insert(
                Primitive::String("example".to_string()),
                Value::string(example),
            );
        }

        docs
    }
}

impl SerializeToBytes for FunctionDocs {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.name.serialize_into_bytes());
        bytes.extend_from_slice(&self.args.serialize_into_bytes());
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
        let name = String::deserialize_from_bytes(bytes)?;
        let args = Vec::<String>::deserialize_from_bytes(bytes)?;
        let category = String::deserialize_from_bytes(bytes)?;
        let signature = String::deserialize_from_bytes(bytes)?;
        let short = Option::<String>::deserialize_from_bytes(bytes)?;
        let desc = Option::<String>::deserialize_from_bytes(bytes)?;
        let example = Option::<String>::deserialize_from_bytes(bytes)?;
        Ok(Self {
            name,
            args,
            category,
            signature,
            short,
            desc,
            example,
        })
    }
}
