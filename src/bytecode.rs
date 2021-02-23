use crate::{OpArgBuf, WORD_SIZE};

struct OpInfo {
    args: u8,
    suffix: u8,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum OpCode {
    // 0 arg ops
    TosDown = OpInfo { args: 0, suffix: 0 }.pack(), // stack-
    WrapAddStack = OpInfo { args: 0, suffix: 1 }.pack(), // stack--
    DecStack = OpInfo { args: 0, suffix: 2 }.pack(), // stack
    // 1 arg ops
    PushConst = OpInfo { args: 1, suffix: 0 }.pack(), // [value] stack+
    Load = OpInfo { args: 1, suffix: 1 }.pack(),      // [data at] stack+
    Store = OpInfo { args: 1, suffix: 2 }.pack(),     // [data to] stack=
    JmpTo = OpInfo { args: 1, suffix: 3 }.pack(),     // [new offset] stack
    IfNzJmp = OpInfo { args: 1, suffix: 4 }.pack(),   // [new offset] stack
}

#[derive(Debug, Default)]
pub struct ByteCodeBufBuilder {
    pub args: OpArgBuf,
    bcb: ByteCodeBuf, // being constructed
}

// effectively a bump allocator for (OpCode [usize]) structures
#[derive(Default)]
pub struct ByteCodeBuf {
    bytes: Vec<u8>, // packed data
}
pub struct ByteCode<'a> {
    bytes: &'a [u8],
}

////////////////////////////////////////////////////////////

impl OpInfo {
    pub const fn pack(self) -> u8 {
        (self.args << 4) | self.suffix
    }
    pub const fn unpack(data: u8) -> Self {
        Self { args: data >> 4, suffix: data & 0b1111 }
    }
}
impl ByteCodeBuf {
    pub fn as_bytecode(&self) -> ByteCode {
        ByteCode { bytes: &self.bytes }
    }
}
impl std::fmt::Debug for ByteCodeBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_bytecode().fmt(f)
    }
}
impl std::fmt::Debug for ByteCode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut offset = 0;
        write!(f, "ByteCode [")?;
        while offset < self.bytes.len() {
            let op = self.read_op_code_at(offset).unwrap();
            offset += 1;
            write!(f, "{:?}=0b{:X} [", op, op as u8)?;
            for _ in 0..op.word_args() {
                let word = self.read_word_at(offset).unwrap();
                write!(f, "{:?}=0b{:X},", word, word)?;
                offset += WORD_SIZE;
            }
            write!(f, "]")?;
        }
        write!(f, "]")
    }
}
impl OpCode {
    pub fn word_args(self) -> usize {
        OpInfo::unpack(self as u8).args as usize
    }
}
impl ByteCode<'_> {
    pub fn read_op_code_at(&self, offset: usize) -> Option<OpCode> {
        if let Some(&byte) = self.bytes.get(offset) {
            Some(unsafe { std::mem::transmute(byte) })
        } else {
            None
        }
    }
    pub fn read_word_at(&self, offset: usize) -> Option<usize> {
        if offset < self.bytes.len() + WORD_SIZE {
            Some(unsafe {
                (self.bytes.get_unchecked(offset) as *const u8 as *const usize).read_unaligned()
            })
        } else {
            None
        }
    }
}
impl ByteCodeBufBuilder {
    fn push_word_bytes(bytes: &mut Vec<u8>, word: usize) {
        // 1. write WORD_SIZE bytes of nonsense into the vec (making space)
        let dummy_iter = std::iter::repeat(0u8).take(WORD_SIZE);
        bytes.extend(dummy_iter);
        // 2. overview nonsense bytes with `word` bytes
        unsafe {
            let len = bytes.len() - WORD_SIZE;
            let ptr = bytes.get_unchecked_mut(len) as *mut u8 as *mut usize;
            ptr.write_unaligned(word);
        }
    }
    pub fn push_with_args(&mut self, op_code: OpCode) {
        self.bcb.bytes.push(unsafe { std::mem::transmute(op_code) });
        for &word in self.args[0..op_code.word_args()].iter() {
            Self::push_word_bytes(&mut self.bcb.bytes, word)
        }
    }
    pub fn finish(self) -> ByteCodeBuf {
        self.bcb
    }
}
