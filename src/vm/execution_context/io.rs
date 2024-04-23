use crate::{
    traits::SerializeToBytes,
    value::{Primitive, Value, ValueType},
    vm::{
        error::{RuntimeError, RuntimeErrorType},
        opcodes::OpCode,
    },
};

pub trait IOExt {
    fn next_byte(&mut self) -> Result<u8, RuntimeError>;
    fn next_bytes(&mut self, len: usize) -> Result<&[u8], RuntimeError>;

    fn read_opcode(&mut self) -> Result<OpCode, RuntimeError>;
    fn read_type(&mut self) -> Result<ValueType, RuntimeError>;
    fn read_value(&mut self) -> Result<Value, RuntimeError>;
    fn read_u64(&mut self) -> Result<u64, RuntimeError>;
    fn read_u16(&mut self) -> Result<u16, RuntimeError>;
    fn read_i32(&mut self) -> Result<i32, RuntimeError>;

    fn decode_with_iterator<T>(&mut self) -> Result<T, RuntimeError>
    where
        T: SerializeToBytes;
}

impl IOExt for super::ExecutionContext<'_> {
    fn next_byte(&mut self) -> Result<u8, RuntimeError> {
        self.set_pc(self.pc() + 1);
        match self.code().get(self.pc() - 1) {
            Some(byte) => Ok(*byte),
            None => Err(self.emit_err(RuntimeErrorType::UnexpectedEnd(self.last_opcode))),
        }
    }

    fn next_bytes(&mut self, len: usize) -> Result<&[u8], RuntimeError> {
        let start = self.pc();
        self.set_pc(self.pc() + len);

        match self.code().get(start..self.pc()) {
            Some(bytes) => Ok(bytes),
            None => Err(self.emit_err(RuntimeErrorType::UnexpectedEnd(self.last_opcode))),
        }
    }

    #[inline(always)]
    fn read_opcode(&mut self) -> Result<OpCode, RuntimeError> {
        self.next_byte().map(|c| {
            OpCode::from_u8(c).ok_or_else(|| self.emit_err(RuntimeErrorType::InvalidOpcode(c)))
        })?
    }

    #[inline(always)]
    fn read_type(&mut self) -> Result<ValueType, RuntimeError> {
        self.next_byte().map(|c| {
            ValueType::from_u8(c).ok_or_else(|| self.emit_err(RuntimeErrorType::InvalidType(c)))
        })?
    }

    #[inline(always)]
    fn read_value(&mut self) -> Result<Value, RuntimeError> {
        let value = self.decode_with_iterator::<Primitive>()?;
        let value = Value::Primitive(value);
        Ok(value)
    }

    #[inline(always)]
    fn read_u64(&mut self) -> Result<u64, RuntimeError> {
        self.decode_with_iterator::<u64>()
    }

    #[inline(always)]
    fn read_u16(&mut self) -> Result<u16, RuntimeError> {
        self.decode_with_iterator::<u16>()
    }

    #[inline(always)]
    fn read_i32(&mut self) -> Result<i32, RuntimeError> {
        self.decode_with_iterator::<i32>()
    }

    #[inline(always)]
    fn decode_with_iterator<T>(&mut self) -> Result<T, RuntimeError>
    where
        T: SerializeToBytes,
    {
        let pc = self.pc();
        let mut iter = self.code().iter().copied().skip(pc);
        let len = iter.len();
        let result = T::deserialize_from_bytes(&mut iter)
            .map_err(|e| self.emit_err(RuntimeErrorType::Decode(self.last_opcode, e)))?;
        let len = len - iter.len();

        if len > 0 {
            self.set_pc(pc + len);
        }

        Ok(result)
    }
}
