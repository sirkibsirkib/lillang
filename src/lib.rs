pub mod bytecode;
mod tests;
pub mod vm;

const WORD_SIZE: usize = std::mem::size_of::<usize>();
type OpArgs = [usize; 1];

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

// effectively a bump allocator for (OpCode [usize]) structures
#[derive(Default)]
pub struct ByteCodeBuf {
    bytes: Vec<u8>, // packed data
}
#[derive(Debug, Default)]
pub struct ByteCodeBuilder {
    pub args: OpArgs,
    bytes: Vec<u8>,
}
pub struct ByteCode<'a> {
    bytes: &'a [u8],
}

////////////////////////////
