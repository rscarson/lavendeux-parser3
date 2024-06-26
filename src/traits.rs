//! Traits used for various parts of the compiler

/// Trait for catching memory allocation errors
pub trait SafeVecAlloc {
    fn safe_alloc(size: usize) -> Result<Self, std::collections::TryReserveError>
    where
        Self: Sized;
}

impl<T> SafeVecAlloc for Vec<T> {
    fn safe_alloc(size: usize) -> Result<Self, std::collections::TryReserveError>
    where
        Self: Sized,
    {
        let mut vec = Self::new();
        vec.try_reserve(size)?;
        Ok(vec)
    }
}

impl<K, V> SafeVecAlloc for std::collections::HashMap<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + PartialEq,
{
    fn safe_alloc(size: usize) -> Result<Self, std::collections::TryReserveError>
    where
        Self: Sized,
    {
        let mut map = Self::new();
        map.try_reserve(size)?;
        Ok(map)
    }
}

/// Trait for converting a type into an owned version of itself
pub trait IntoOwned {
    /// The owned version of the type
    type Owned;

    /// Convert the type into an owned version
    fn into_owned(self) -> Self::Owned;
}

pub(crate) trait NextN {
    fn next_n(&mut self, n: usize) -> Option<Vec<u8>>;
}

impl<T: Iterator<Item = u8>> NextN for T {
    fn next_n(&mut self, n: usize) -> Option<Vec<u8>> {
        let mut bytes = Vec::with_capacity(n);
        for _ in 0..n {
            match self.next() {
                Some(byte) => bytes.push(byte),
                None => return None,
            }
        }
        Some(bytes)
    }
}

/// Error found when decoding bytes
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ByteDecodeError {
    /// Buffer was too short
    #[error("Unexpected end of data\n= In `{0}`")]
    UnexpectedEnd(String),

    /// No data was available
    #[error("No data available\n= In `{0}`")]
    NoData(String),

    /// Invalid data was found
    #[error("Invalid data: {1}\n= In `{0}`")]
    MalformedData(String, String),

    /// Error during memory allocation
    #[error("{0}")]
    MemoryAllocation(#[from] std::collections::TryReserveError),
}

/// Trait for serializing and deserializing types to bytes
pub trait SerializeToBytes
where
    Self: Sized,
{
    /// Convert the type to bytes
    fn serialize_into_bytes(self) -> Vec<u8>;

    /// Convert bytes to the type
    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError>;
}

impl SerializeToBytes for i128 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(16)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("i128".to_string()))?;
        let mut buf = [0; 16];
        buf.copy_from_slice(&bytes);
        Ok(i128::from_be_bytes(buf))
    }
}

impl SerializeToBytes for u8 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(1)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("u8".to_string()))?;
        let mut buf = [0; 1];
        buf.copy_from_slice(&bytes);
        Ok(u8::from_be_bytes(buf))
    }
}

impl SerializeToBytes for i8 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(1)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("i8".to_string()))?;
        let mut buf = [0; 1];
        buf.copy_from_slice(&bytes);
        Ok(i8::from_be_bytes(buf))
    }
}

impl SerializeToBytes for i32 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(4)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("i32".to_string()))?;
        let mut buf = [0; 4];
        buf.copy_from_slice(&bytes);
        Ok(i32::from_be_bytes(buf))
    }
}

impl SerializeToBytes for u64 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(8)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("u64".to_string()))?;
        let mut buf = [0; 8];
        buf.copy_from_slice(&bytes);
        Ok(u64::from_be_bytes(buf))
    }
}

impl SerializeToBytes for usize {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(8)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("usize".to_string()))?;
        let mut buf = [0; 8];
        buf.copy_from_slice(&bytes);
        Ok(usize::from_be_bytes(buf))
    }
}

impl SerializeToBytes for u16 {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let bytes = bytes
            .next_n(2)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("u16".to_string()))?;
        let mut buf = [0; 2];
        buf.copy_from_slice(&bytes);
        Ok(u16::from_be_bytes(buf))
    }
}

impl SerializeToBytes for String {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.len().serialize_into_bytes());
        bytes.extend_from_slice(self.as_bytes());
        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let len = u64::deserialize_from_bytes(bytes)?;
        let bytes = bytes
            .next_n(len as usize)
            .ok_or_else(|| ByteDecodeError::UnexpectedEnd("String".to_string()))?;
        Ok(String::from_utf8(bytes).map_err(|_| {
            ByteDecodeError::MalformedData("String".to_string(), "Invalid UTF-8".to_string())
        })?)
    }
}

impl<T> SerializeToBytes for Vec<T>
where
    T: SerializeToBytes,
{
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.len().serialize_into_bytes());
        for item in self {
            bytes.extend_from_slice(&item.serialize_into_bytes());
        }
        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        let len = u64::deserialize_from_bytes(bytes)?;
        let mut items = Vec::safe_alloc(len as usize)?;
        for _ in 0..len {
            items.push(T::deserialize_from_bytes(bytes)?);
        }
        Ok(items)
    }
}

impl<T> SerializeToBytes for Option<T>
where
    T: SerializeToBytes,
{
    fn serialize_into_bytes(self) -> Vec<u8> {
        match self {
            Some(value) => {
                let mut bytes = Vec::new();
                bytes.push(1);
                bytes.extend_from_slice(&value.serialize_into_bytes());
                bytes
            }
            None => vec![0],
        }
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, ByteDecodeError> {
        match bytes.next() {
            Some(0) => Ok(None),
            Some(1) => Ok(Some(T::deserialize_from_bytes(bytes)?)),
            _ => Err(ByteDecodeError::MalformedData(
                "Option".to_string(),
                "Invalid header".to_string(),
            )),
        }
    }
}
