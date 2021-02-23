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
    WrapAddStack = OpInfo { args: 0, suffix: 1 }.pack(), // stack--+
    DecStack = OpInfo { args: 0, suffix: 2 }.pack(), // stack-+
    SysOut = OpInfo { args: 0, suffix: 3 }.pack(),  // stack-
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

// effectively a bump allocator for opcodes and their arguments
#[derive(Default)]
pub struct ByteCodeBuf {
    // invariant: sequence of packed (OpCode [usize]) bytes
    bytes: Vec<u8>,
}

pub struct ByteCode<'a> {
    // invariant: contents are entire slice of ByteCodeBuf. We inherit its invariants
    // invariant: adjacent to each
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
        let mut op_arg_buf: OpArgBuf = Default::default();
        while offset < self.bytes.len() {
            let op = unsafe {
                // safe! rely on ByteCode's invariant
                self.read_op_code_at(offset)
            };
            // println!("got arg {:?}", op);
            offset += 1;
            write!(f, "{:?}=0b{:X} [", op, op as u8)?;
            let num_words = op.word_args();
            unsafe {
                // safe! relies on my invariant
                self.read_words_into(offset, &mut op_arg_buf[0..num_words]);
            }
            offset += num_words * WORD_SIZE;
            for arg in op_arg_buf.iter().take(num_words) {
                write!(f, "{:?}=0b{:X},", arg, arg)?;
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
    pub fn bytes_len(&self) -> usize {
        self.bytes.len()
    }

    // safe IFF caller provides offset to valid opcode
    pub(crate) unsafe fn read_op_code_at(&self, offset: usize) -> OpCode {
        std::mem::transmute(*self.bytes.get_unchecked(offset))
    }

    // safe IFF this won't read out of bounds
    pub(crate) unsafe fn read_words_into(&self, offset_start: usize, args_dest: &mut [usize]) {
        // sanity check (already covered by invariant)
        assert!(offset_start + args_dest.len() * WORD_SIZE <= self.bytes.len());

        let src = self.bytes.as_ptr().add(offset_start);
        let dest = args_dest.as_mut_ptr() as *mut u8;
        std::ptr::copy_nonoverlapping(src, dest, args_dest.len() * WORD_SIZE);
    }
}
impl ByteCodeBufBuilder {
    fn push_word_bytes(bytes: &mut Vec<u8>, word: usize) {
        bytes.extend(word.to_ne_bytes().iter().copied())
    }
    pub fn push_with_args(&mut self, op_code: OpCode) {
        self.bcb.bytes.push(op_code as u8);
        for &word in self.args[0..op_code.word_args()].iter() {
            Self::push_word_bytes(&mut self.bcb.bytes, word)
        }
    }
    pub fn finish(self) -> ByteCodeBuf {
        println!("DONE {:?}", &self.bcb.bytes);
        // TODO check that arg ptrs fall within bounds & land on an op code
        self.bcb
    }
}
