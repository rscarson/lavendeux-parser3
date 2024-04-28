//! Transcodes a bytecode buffer into a human-readable assembly-like format
//! This is useful for debugging and testing purposes
//!
//! You can't actually reassemble the assembly back into bytecode yet
//! The compiler's `--asm` flag makes use of this module
use crate::lexer::Token;
use crate::traits::{IntoOwned, SerializeToBytes};
use crate::value::{Function, Primitive, Value, ValueType};
use crate::vm::OpCode;

use super::DebugProfile;

/// An error that can occur during disassembly
/// This error is usually caused by invalid bytecode
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// An opcode that is not recognized by the disassembler
    #[error("{}\n= Bad Opcode (offset {1})", .0.as_ref().map(|t|t.to_string()).unwrap_or("".to_string()))]
    BadOp(Option<Token<'static>>, usize),

    /// A jump instruction that points to an invalid offset
    #[error("Bad jump at offset {0}")]
    BadJump(usize),
}

/// An instruction in the disassembled bytecode
#[derive(Clone, Debug)]
pub enum Instruction {
    /// A basic opcode that does not require any additional data outside the stack
    Simple(OpCode),

    /// An instruction that pushes a value onto the stack
    Push(Value),

    /// An instruction that reads or writes to memory
    Mem(OpCode, u64),

    /// An instruction that jumps to a position
    Jump(OpCode, u64),

    /// An instruction that casts the top value on the stack to a different type
    Cast(ValueType),

    /// An instruction that accepts an N value (used for arrays and objects)
    AcceptsN(OpCode, u64),

    /// An instruction that creates a function object
    MkFn(Function),

    /// An instruction that creates a function argument
    FnArg(u16),

    /// An instruction that adds documentation to a function

    /// An instruction that calls a function
    FnCall(u64, u64),

    //
    // Meta instructions
    //
    /// A comment - generated from the debug profile
    Comment(String),

    /// A label - generated from a jump instruction
    Label(String),

    /// A jump instruction that points to a label
    JumpTo(OpCode, String, u64),

    /// An error that occurred during disassembly
    Error(Error),
}

/// A disassembler for bytecode buffers
/// This struct is used to transcode a bytecode buffer into a human-readable assembly-like format
/// This is a one-way operation; you can't reassemble the assembly back into bytecode
#[derive(Clone, Debug)]
pub struct ASMTranscoder<'src> {
    instructions: Vec<(Instruction, usize)>,
    labels: LabelGun,
    buffer: std::iter::Copied<std::slice::Iter<'src, u8>>,
    debug_profile: Option<DebugProfile>,
    hashref: std::collections::HashMap<u64, String>,

    label_jumps: bool,
}
impl<'src> ASMTranscoder<'src> {
    /// Create a new disassembler with the given bytecode buffer and debug profile (optional)
    /// If a debug profile is provided, the disassembler will add comments to the output
    pub fn new<'buf: 'src>(buffer: &'buf [u8], debug_profile: Option<DebugProfile>) -> Self {
        Self {
            instructions: Vec::new(),
            labels: LabelGun::new(),
            buffer: buffer.iter().copied(),
            debug_profile,
            hashref: std::collections::HashMap::new(),
            label_jumps: true,
        }
    }

    fn populate_instructions(&mut self) {
        self.all_instructions();
        self.add_debuginfo();
        if self.label_jumps {
            self.intersperse_labels();
        }
        self.process_functions();
    }

    /// Set whether to label jump instructions with their target labels
    pub fn label_jumps(&mut self, label_jumps: bool) {
        self.label_jumps = label_jumps;
    }

    /// Get all the instructions in the disassembled bytecode
    /// Returns a vector of instructions
    /// If an error occurs, it will appear in the output
    pub fn disassemble_as_vec(mut self) -> Vec<Instruction> {
        self.populate_instructions();
        self.instructions.iter().map(|(i, _)| i.clone()).collect()
    }

    /// Perform the disassembly operation
    /// Returns a string containing the disassembled bytecode
    /// If an error occurs, it will appear in the output
    pub fn disassemble_as_string(mut self) -> String {
        self.populate_instructions();
        let mut output = String::new();

        let mut offset = 0;
        for (instruction, len) in self.instructions.iter() {
            if !self.label_jumps {
                output.push_str(format!("{:08X}:\n", offset).as_str());
            }
            offset += len;

            match instruction {
                Instruction::Simple(opcode) => output.push_str(&format!("  {opcode:?}\n")),
                Instruction::Push(value) => output.push_str(&format!("  PUSH {value:?}\n")),
                Instruction::Mem(opcode, hash) => {
                    let label = if self.hashref.contains_key(hash) {
                        self.hashref.get(hash).unwrap().clone()
                    } else {
                        let label = format!("VAR_{}", self.labels.next());
                        self.hashref.insert(*hash, label.clone());
                        label
                    };
                    output.push_str(&format!("  {opcode:?} {label}\n"))
                }
                Instruction::Jump(opcode, pos) => {
                    output.push_str(&format!("  {opcode:?} {pos:08X}\n"))
                }
                Instruction::Cast(type_name) => output.push_str(&format!("  CAST {type_name:?}\n")),
                Instruction::AcceptsN(opcode, n) => {
                    output.push_str(&format!("  {opcode:?} {n:08X}\n"))
                }

                Instruction::MkFn(function) => {
                    let label = format!("FN_{}", self.hashref.get(&function.name_hash).unwrap());
                    output.push_str(&format!("  MKFN {label}\n"))
                }

                Instruction::FnArg(idx) => output.push_str(&format!("  FDFT {idx:02X}\n")),

                Instruction::FnCall(name_hash, n) => {
                    output.push_str(&format!("  CALL {name_hash:08X} {n}\n"))
                }

                Instruction::Label(label) => output.push_str(&format!("{label}:\n")),
                Instruction::JumpTo(opcode, label, pos) => {
                    if self.label_jumps {
                        output.push_str(&format!("  {opcode:?} {label}\n"))
                    } else {
                        output.push_str(&format!("  {opcode:?} ({pos:08X})\n"))
                    }
                }

                Instruction::Comment(comment) => output.push_str(&format!(
                    "{}",
                    comment
                        .split('\n')
                        .map(|l| format!("; {l}\n"))
                        .collect::<String>()
                )),

                Instruction::Error(err) => {
                    output.push_str(&format!("\n==== ERROR ====\n {err}\n==== ERROR ====\n"))
                }
            }
        }

        output
    }

    /// Break up the bytecode buffer into individual instructions
    fn all_instructions(&mut self) {
        let mut offset = 0;
        loop {
            match self.next_opcode() {
                Some((instruction, len)) => {
                    self.instructions.push((instruction, len));
                    offset += len;
                }
                None => {
                    let token = self
                        .debug_profile
                        .as_ref()
                        .and_then(|p| p.current_token(offset).map(|t| t.into_owned()));
                    self.instructions
                        .push((Instruction::Error(Error::BadOp(token, offset)), 0));
                    break;
                }
            }

            if self.buffer.len() == 0 {
                break;
            }
        }
    }

    /// Add debug information to the instruction list
    pub fn add_debuginfo(&mut self) {
        if let Some(profile) = &self.debug_profile {
            let mut instructions = Vec::new();
            let mut offset = 0;
            let mut debug_labels = profile.all_slices().peekable();

            for (instruction, len) in self.instructions.drain(..) {
                while let Some((start, slice)) = debug_labels.peek() {
                    if *start != offset {
                        break;
                    }

                    instructions.push((Instruction::Comment(format!("{}", slice)), 0));
                    debug_labels.next();
                }

                instructions.push((instruction, len));

                offset += len;
            }

            self.instructions = instructions;
        }
    }

    /// Replace all Jump(offset) instructions with JumpTo(label) instructions
    /// While inserting Label(label) instructions at the right places in the instruction list
    fn intersperse_labels(&mut self) {
        let mut labels = std::collections::HashMap::new();

        for (instruction, _) in self.instructions.iter_mut() {
            match instruction {
                Instruction::Jump(opcode, pos) => {
                    let position = *pos as usize;
                    let label = if !labels.contains_key(&position) {
                        let label = format!("JUMP_{}", self.labels.next());
                        labels.insert(position, label.clone());
                        label
                    } else {
                        labels.get(&position).unwrap().clone()
                    };
                    *instruction = Instruction::JumpTo(*opcode, label, *pos);
                }
                _ => {}
            }
        }

        for (jmp_pos, label) in labels {
            let mut i = 0;
            let mut offset = 0;
            for (_, len) in &self.instructions {
                if offset >= jmp_pos {
                    break;
                }

                offset += *len;
                i += 1;
            }

            self.instructions.insert(i, (Instruction::Label(label), 0));
        }
    }

    fn process_functions(&mut self) {
        let mut functions = vec![];
        for (instruction, _) in &self.instructions {
            if let Instruction::MkFn(function) = instruction {
                let mut transcoder = ASMTranscoder::new(&function.body, function.debug.clone());
                transcoder.populate_instructions();
                let instructions = transcoder.instructions.clone();
                let label = self.hashref.get(&function.name_hash).unwrap();
                functions.push((Instruction::Comment(format!("fn {label}")), 0));
                functions.extend(instructions.into_iter());
            }
        }

        self.instructions.extend(functions);
    }

    /// Get the next opcode from the buffer
    fn next_opcode(&mut self) -> Option<(Instruction, usize)> {
        let opcode = self.buffer.next()?;
        let opcode = OpCode::from_u8(opcode)?;

        println!("Opcode: {:?}", opcode);
        match &opcode {
            OpCode::PUSH => {
                let len = self.buffer.len();
                let value = Primitive::deserialize_from_bytes(&mut self.buffer).ok()?;
                let value = Value::Primitive(value);
                let len = len - self.buffer.len();
                let instruction = Instruction::Push(value);
                Some((instruction, 1 + len))
            }

            OpCode::JMP | OpCode::JMPT | OpCode::JMPF | OpCode::JMPE | OpCode::JMPNE => {
                let pos = u64::deserialize_from_bytes(&mut self.buffer).ok()?;
                let instruction = Instruction::Jump(opcode, pos);
                Some((instruction, 1 + 4))
            }

            OpCode::REF => {
                let hash = u64::deserialize_from_bytes(&mut self.buffer).ok()?;
                let instruction = Instruction::Mem(opcode, hash);
                Some((instruction, 1 + 8))
            }

            OpCode::CAST => {
                let type_name = ValueType::from_u8(self.buffer.next()?)?;
                let instruction = Instruction::Cast(type_name);
                Some((instruction, 1 + 1))
            }

            OpCode::MKAR | OpCode::MKOB | OpCode::READF => {
                let n = u64::deserialize_from_bytes(&mut self.buffer).ok()?;
                let instruction = Instruction::AcceptsN(opcode, n);
                Some((instruction, 1 + 4))
            }

            OpCode::MKFN => {
                let len = self.buffer.len();
                let _version = self.buffer.next()?;
                let function = Function::deserialize_from_bytes(&mut self.buffer).ok()?;
                let len = len - self.buffer.len();
                self.hashref.insert(function.name_hash, self.labels.next());
                let instruction = Instruction::MkFn(function);
                Some((instruction, 1 + len))
            }

            OpCode::FDFT => {
                let idx = u16::deserialize_from_bytes(&mut self.buffer).ok()?;
                let instruction = Instruction::FnArg(idx);
                Some((instruction, 1 + 2))
            }

            OpCode::CALL => {
                let name_hash = u64::deserialize_from_bytes(&mut self.buffer).ok()?;
                let n = u64::deserialize_from_bytes(&mut self.buffer).ok()?;
                let instruction = Instruction::FnCall(name_hash, n);
                Some((instruction, 1 + 8 + 8))
            }

            OpCode::POP
            | OpCode::DUP
            | OpCode::SWP
            | OpCode::RREF
            | OpCode::WREF
            | OpCode::DREF
            | OpCode::SCI
            | OpCode::SCO
            | OpCode::SCL
            | OpCode::NEXT
            | OpCode::LCST
            | OpCode::MKRG
            | OpCode::PSAR
            | OpCode::PSOB
            | OpCode::IDEX
            | OpCode::ADD
            | OpCode::SUB
            | OpCode::MUL
            | OpCode::DIV
            | OpCode::REM
            | OpCode::POW
            | OpCode::NEG
            | OpCode::AND
            | OpCode::OR
            | OpCode::XOR
            | OpCode::NOT
            | OpCode::SHL
            | OpCode::SHR
            | OpCode::EQ
            | OpCode::NE
            | OpCode::SEQ
            | OpCode::SNE
            | OpCode::LT
            | OpCode::LE
            | OpCode::GT
            | OpCode::GE
            | OpCode::LAND
            | OpCode::LOR
            | OpCode::LNOT
            | OpCode::MTCH
            | OpCode::CNTN
            | OpCode::STWT
            | OpCode::EDWT
            | OpCode::WRFN
            | OpCode::FSIG
            | OpCode::RET
            | OpCode::PRNT
            | OpCode::LSTFN
            | OpCode::LEN
            | OpCode::SSPLT
            | OpCode::TAN
            | OpCode::SIN
            | OpCode::COS
            | OpCode::ATAN2
            | OpCode::ATAN
            | OpCode::ASIN
            | OpCode::ACOS
            | OpCode::TANH
            | OpCode::SINH
            | OpCode::COSH
            | OpCode::ROUND
            | OpCode::LOG
            | OpCode::ILOG
            | OpCode::ROOT
            | OpCode::NOP => Some((Instruction::Simple(opcode), 1)),
        }
    }
}

/// A label generator for the disassembler
/// Creates human-readable labels for jump instructions and variables
/// Examples of output:
/// ```
/// salamander_noodle
/// kangaroo
/// arbitrary_cabbage_marmalade
/// ```
#[derive(Clone, Debug)]
struct LabelGun(usize);
impl LabelGun {
    #[rustfmt::skip]
    const DICT: &'static [&'static str] = &[
        "arbitrary", "bananas", "cabbage", "dolphin", "pointbreak",
        "alabaster", "umbrella", "grapefruit", "hedgehog", "jellybean",
        "kangaroo", "lumberjack", "marmalade", "noodle", "octopus",
        "penguin", "quarantine", "rhubarb", "salamander", "tangerine",
    ];

    /// Create a new label generator
    pub fn new() -> Self {
        Self(0)
    }

    /// Get the next label
    pub fn next(&mut self) -> String {
        let label = Self::to_basen(self.0);
        self.0 += 1;

        // Prefix with a random dict entry
        let rand = rand::random::<usize>() % Self::DICT.len();
        Self::DICT[rand].to_owned() + "_" + &label
    }

    /// Convert a number to a human-readable label
    fn to_basen(n: usize) -> String {
        if n < Self::DICT.len() {
            Self::DICT[n].to_owned()
        } else {
            Self::to_basen(n / Self::DICT.len() - 1)
                + "_"
                + &Self::DICT[1 + n % (Self::DICT.len() - 1)..][..1]
                    .join("_")
                    .to_string()
        }
    }
}
