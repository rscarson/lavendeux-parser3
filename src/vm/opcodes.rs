use strum::EnumString;

/// The set of opcodes that the VM can execute
/// if the `--allow-syscalld` compiler flag is set, the compiler will allow the use of the `__syscalld` function
/// which can be used to call opcodes directly from the source code
/// See the stdlib source code for examples of how to use this function
/// (Please do not use this function)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
#[rustfmt::skip]
pub enum OpCode {
    ////////////////////////
    // Stack manipulation //
    ////////////////////////
    
    /// Push a value onto the stack
    /// `PUSH <TypeCode> <Value>`
    PUSH,

    /// Remove and discard the top value from the stack
    /// Consumes 1 stack value
    /// `POP``
    POP,

    /// Duplicate the top value on the stack
    /// Pushes 1 value onto the stack
    /// `DUP`
    DUP,

    /// Swap the top two values on the stack
    /// Consumes 2 stack values
    /// `SWP`
    SWP,

    ///////////////////
    /// Control flow //
    ///////////////////
    
    /// Jump forward or backward by `n` bytes
    /// `JMP <i32>`
    JMP,

    /// Jump forward or backward by `n` bytes if the top value on the stack is true
    /// Consumes 1 stack value
    /// `JMPR <i32>`
    JMPT,

    /// Jump forward or backward by `n` bytes if the top value on the stack is false
    /// Consumes 1 stack value
    /// `JMPR <i32>`
    JMPF,

    /// Jump forward or backward by `n` bytes if the top value on the stack is empty
    /// Consumes 1 stack value
    /// `JMPE <i32>`
    JMPE,

    /////////////////////////
    // Memory manipulation //
    /////////////////////////
     
    /// Write a value to memory
    /// Consumes 1 stack value
    /// Pushes 1 value onto the stack
    /// `WRITE <Name Hash>`
    WRIT,

    /// Read a value from memory
    /// Pushes 1 value onto the stack
    /// `READ <Name Hash>`
    READ,

    /// Delete a value from memory
    /// `DEL <Name Hash>`
    DEL,

    /// Read a value from memory by reference
    /// Pushes 1 value onto the stack
    /// `REF <Name Hash>`
    REF,

    /// Write a value to a reference
    /// If the last part of the reference is not valid, it is created
    /// Consumes 2 stack values (reference, value)
    /// Pushes 1 value onto the stack
    /// `WREF`
    WREF,

    /// Delete a value from a reference
    /// Consumes  stack value (reference)
    /// Pushes 1 value onto the stack
    /// `WREF`
    DREF,

    ////////////////////////
    // Scope manipulation //
    ////////////////////////

    /// Enter a new scope
    /// `SCI`
    SCI,

    /// Exit the current scope
    /// `SCO`
    SCO,

    /// Lock the current scope
    /// `SCL`
    SCL,

    ////////////////////////
    // Value manipulation //
    ////////////////////////
    
    /// Convert the top value on the stack to a type
    /// Consumes 1 stack value; [Input Value]
    /// Pushes 1 value onto the stack; [Output Value]
    /// `CAST <TypeCode>`
    CAST,

    /// Remove the first element from a collection
    /// Consumes 1 stack value; [Collection]
    /// Pushes 2 values onto the stack; [Rest; First]
    NEXT,

    /// Attempt to convert the top value on the stack to the type of the second value
    /// Consumes 2 stack values [Left, Right]
    /// Pushes 2 values onto the stack [Left, CastedRight]
    /// `LCST`
    LCST,

    /// Build a new array from the top `n` values on the stack
    /// Consumes `n` stack values; [Value1, Value2, ..., ValueN]
    /// Pushes 1 value onto the stack; [Array]
    /// `MKAR <n>`
    MKAR,

    /// Build a new object from the top `2n` values on the stack
    /// Consumes `2n` stack values; [Key1, Value1, Key2, Value2, ..., KeyN, ValueN]
    /// Pushes 1 value onto the stack; [Object]
    /// `MKOB <n>`
    MKOB,

    /// Attempt to create a range from the top two values on the stack
    /// Consumes 2 stack values; [Start, End]
    /// Pushes 1 value onto the stack; [Range]
    /// `MKRG`
    MKRG,

    /// Push a value onto an array
    /// Consumes 2 stack values; [Array, Value]
    /// Pushes 1 value onto the stack
    /// `PSAR`
    PSAR,

    /// Push a value onto an object
    /// Consumes 3 stack values; [Object, Key, Value]
    /// Pushes 1 value onto the stack
    /// `PSOB`
    PSOB,

    /// Index into the top value on the stack
    /// If the base is a reference, the index is added to it
    /// Consumes 2 stack values (base, index)
    /// Pushes 1 value onto the stack
    /// `IDEX`
    IDEX,

    ////////////////////
    // Arithmetic ops //
    ////////////////////

    /// Add the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `ADD`
    ADD,

    /// Subtract the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `SUB`
    SUB,

    /// Multiply the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `MUL`
    MUL,

    /// Divide the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `DIV`
    DIV,

    /// Get the remainder of the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `REM`
    REM,

    /// Raise the top value on the stack to the power of the second value
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `POW`
    POW,

    /// Negate the top value on the stack
    /// Consumes 1 stack value
    /// Pushes 1 value onto the stack
    /// `NEG`
    NEG,


    /////////////////
    // Bitwise ops //
    /////////////////

    /// Bitwise AND the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `AND`
    AND,

    /// Bitwise OR the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `OR`
    OR,

    /// Bitwise XOR the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `XOR`
    XOR,

    /// Bitwise NOT the top value on the stack
    /// Consumes 1 stack value
    /// Pushes 1 value onto the stack
    /// `NOT`
    NOT,

    /// Bitwise shift left the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `SHL`
    SHL,

    /// Bitwise shift right the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `SHR`
    SHR,

    ////////////////////
    // Comparison ops //
    ////////////////////
    
    /// Compare the top two values on the stack for equality
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `EQ`
    EQ,

    /// Compare the top two values on the stack for inequality
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `NE`
    NE,

    /// Compare the top two values on the stack for strict equality
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `SEQ`
    SEQ,

    /// Compare the top two values on the stack for strict inequality
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `SNE`
    SNE,

    /// Compare the top two values on the stack for less than
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `LT`
    LT,

    /// Compare the top two values on the stack for less than or equal
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `LE`
    LE,

    /// Compare the top two values on the stack for greater than
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `GT`
    GT,

    /// Compare the top two values on the stack for greater than or equal
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `GE`
    GE,

    /////////////////
    // Logical ops //
    /////////////////
    
    /// Logical AND the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `LAND`
    LAND,

    /// Logical OR the top two values on the stack
    /// Consumes 2 stack values
    /// Pushes 1 value onto the stack
    /// `LOR`
    LOR,

    /// Logical NOT the top value on the stack
    /// Consumes 1 stack value
    /// Pushes 1 value onto the stack
    /// `LNOT`
    LNOT,

    //////////////////
    // Matching ops //
    //////////////////
    
    /// Match the top two values on the stack
    /// Consumes 2 stack values (value, pattern)
    /// Pushes 1 value onto the stack
    /// `MTCH`
    MTCH,

    /// Match the top two values on the stack (contains)
    /// Consumes 2 stack values (value, pattern)
    /// Pushes 1 value onto the stack
    /// `CNTN`
    CNTN,

    /// Match the top two values on the stack (starts with)
    /// Consumes 2 stack values (value, pattern)
    /// Pushes 1 value onto the stack
    /// `STWT`
    STWT,

    /// Match the top two values on the stack (ends with)
    /// Consumes 2 stack values (value, pattern)
    /// Pushes 1 value onto the stack
    /// `EDWT`
    EDWT,

    //////////////////
    // Function ops //
    //////////////////
    
    /// Write a new function to memory
    /// Consumes 1 stack value; [Function]
    /// `WRFN`
    WRFN,
    
    /// Create a new function
    /// Pushes 1 value onto the stack; [Function]
    /// `MKFN <string: name> <u8 returns> <u64 len> [ body ]`
    MKFN,

    /// Add a default value to a function argument
    /// Consumes 2 stack values; [Function, Default]
    /// Pushes 1 value onto the stack; [Function]
    /// `FDFT <u16: idx>`
    FDFT,

    /// Set the signature of a function
    /// Consumes 1 stack value; [Function]
    /// Pushes 1 value onto the stack; [Function]
    /// `FSIG <string: name> <u16 nargs> [ arg_names ]`
    FSIG,

    /// Call a function
    /// Consumes `n` stack values; [Arg1, Arg2, ..., ArgN]
    /// Pushes 1 value onto the stack; [Return]
    /// `CALL <u64: name_hash> <u64: N>`
    CALL,

    /// Return from a function
    /// Consumes 1 stack value; [Return]
    /// `RET`
    RET,

    //////////////
    // Misc ops //
    //////////////
    
    /// Print the top value on the stack
    /// Consumes 1 stack value
    /// `PRNT`
    PRNT,
    
    /// Calculate the tangent of the top value on the stack
    /// Expects a value in radians
    TAN,

    /// Calculate the sine of the top value on the stack
    /// Expects a value in radians
    SIN,

    /// Calculate the cosine of the top value on the stack
    /// Expects a value in radians
    COS,

    /// Calculate the arctangent2 of the top value on the stack
    /// Consumes 2 stack values; [Y, X]
    ATAN2,

    /// Calculate the arctangent of the top value on the stack
    ATAN,

    /// Calculate the arcsine of the top value on the stack
    ASIN,

    /// Calculate the arccosine of the top value on the stack
    ACOS,

    /// Calculate the hyperbolic tangent of the top value on the stack
    TANH,

    /// Calculate the hyperbolic sine of the top value on the stack
    SINH,

    /// Calculate the hyperbolic cosine of the top value on the stack
    COSH,

    /// Round a value to a precision
    /// Consumes 2 stack values; [Value, Precision]
    ROUND,

    /// Logarithm
    /// Consumes 2 stack values; [Value, Base]
    LOG,

    /// Inverse logarithm
    /// Consumes 2 stack values; [Value, Base]
    ILOG,

    /// Root
    /// Consumes 2 stack values; [Value, Root]
    ROOT,

    /// No operation
    NOP,
}

impl OpCode {
    /// Convert a u8 to an OpCode
    pub fn from_u8(value: u8) -> Option<Self> {
        if value <= OpCode::NOP as u8 {
            Some(unsafe { std::mem::transmute(value) })
        } else {
            None
        }
    }
}
