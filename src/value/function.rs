use super::{Value, ValueType};
use crate::{
    compiler::{DebugProfile, FunctionDocs},
    traits::{ByteDecodeError, SerializeToBytes},
    vm::memory_manager::{MemoryManager, Slot},
};

/// An argument to a function.
#[derive(Debug, Clone)]
pub struct FunctionArgument {
    /// The hash of the name of the argument
    pub name_hash: u64,

    /// The expected type of the argument
    pub ty: ValueType,

    /// The default value of the argument
    pub default: Option<Value>,
}

/// A function value
#[derive(Debug, Clone)]
pub struct Function {
    /// The name of the function
    pub name_hash: u64,

    /// The return type of the function
    pub returns: ValueType,

    /// The arguments the function expects
    pub expects: Vec<FunctionArgument>,

    /// The debug profile of the compiled function
    pub debug: Option<DebugProfile<'static>>,

    /// The documentation of the function
    pub docs: FunctionDocs,

    /// The bytecode of the function
    pub body: Vec<u8>,
}

impl PartialEq for Function {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl Eq for Function {}

/// Represents a set of compiled functions.
/// This is used to store functions in a memory manager.
/// The stdlib works this way
#[derive(Debug, Clone)]
pub struct StdFunctionSet {
    functions: Vec<Function>,
}
impl StdFunctionSet {
    /// Populate a memory manager with the functions in this set.
    pub fn into_mem(self, mem: &mut MemoryManager) {
        for function in self.functions {
            mem.write_global(function.name_hash, Value::Function(function), true);
        }
    }

    /// Create a new function set from the functions in a memory manager.
    pub fn from_mem(mem: &MemoryManager) -> Self {
        let mut functions = vec![];
        for slot in mem.all_globals() {
            match slot {
                Slot::Occupied { value, .. } => {
                    if let Value::Function(function) = value {
                        functions.push(function.clone());
                    }
                }
                _ => {}
            }
        }

        Self { functions }
    }
}

impl SerializeToBytes for FunctionArgument {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.name_hash.serialize_into_bytes());
        bytes.push(self.ty as u8);
        bytes.extend(self.default.serialize_into_bytes());

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let name_hash = u64::deserialize_from_bytes(bytes)?;
        let ty = u8::deserialize_from_bytes(bytes)?;
        let ty = ValueType::from_u8(ty).ok_or_else(|| {
            ByteDecodeError::MalformedData(
                "Function".to_string(),
                "Invalid argument type".to_string(),
            )
        })?;
        let default = Option::<Value>::deserialize_from_bytes(bytes)?;

        Ok(Self {
            name_hash,
            ty,
            default,
        })
    }
}

impl SerializeToBytes for Function {
    fn serialize_into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.extend(self.name_hash.serialize_into_bytes());
        bytes.push(self.returns as u8);
        bytes.extend(self.expects.serialize_into_bytes());
        bytes.extend(self.debug.serialize_into_bytes());
        bytes.extend(self.docs.serialize_into_bytes());
        bytes.extend(self.body.serialize_into_bytes());

        bytes
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        let name_hash = u64::deserialize_from_bytes(bytes)?;
        let returns = u8::deserialize_from_bytes(bytes)?;
        let returns = ValueType::from_u8(returns).ok_or_else(|| {
            ByteDecodeError::MalformedData(
                "Function".to_string(),
                "Invalid return type".to_string(),
            )
        })?;
        let expects = Vec::<FunctionArgument>::deserialize_from_bytes(bytes)?;
        let debug = Option::<DebugProfile<'static>>::deserialize_from_bytes(bytes)?;
        let docs = FunctionDocs::deserialize_from_bytes(bytes)?;
        let body = Vec::<u8>::deserialize_from_bytes(bytes)?;

        Ok(Self {
            name_hash,
            returns,
            expects,
            debug,
            docs,
            body,
        })
    }
}

impl SerializeToBytes for StdFunctionSet {
    fn serialize_into_bytes(self) -> Vec<u8> {
        self.functions.serialize_into_bytes()
    }

    fn deserialize_from_bytes(
        bytes: &mut impl Iterator<Item = u8>,
    ) -> Result<Self, crate::traits::ByteDecodeError> {
        Ok(Self {
            functions: Vec::<Function>::deserialize_from_bytes(bytes)?,
        })
    }
}
