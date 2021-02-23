const WORD_SIZE: usize = std::mem::size_of::<usize>();

struct OpInfo {
    args: u8,
    suffix: u8,
}
impl OpInfo {
    const fn pack(self) -> u8 {
        (self.args << 4) | self.suffix
    }
    const fn unpack(data: u8) -> Self {
        Self { args: data >> 4, suffix: data & 0b1111 }
    }
}
// TODO

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum OpCode {
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
type OpArgs = [usize; 1];

#[derive(Debug)]
struct Vm<'a> {
    stack: Vec<usize>,
    bp: usize,
    parser: VmParser<'a>,
}

#[derive(Debug)]
struct VmParser<'a> {
    bytecode: ByteCode<'a>,  // immutable. from ByteCodeBuf
    next_code_offset: usize, // depends on bc
    res_buf: VmParserResult, // contents reflect most recent parse
}
#[derive(Debug, Default)]
struct VmParserResult {
    code: Option<OpCode>,
    args: OpArgs,
}

// effectively a bump allocator for (OpCode [usize]) structures
#[derive(Default)]
struct ByteCodeBuf {
    bytes: Vec<u8>, // packed data
}
#[derive(Debug, Default)]
struct ByteCodeBuilder {
    pub args: OpArgs,
    bytes: Vec<u8>,
}
struct ByteCode<'a> {
    bytes: &'a [u8],
}
/////////////////////////////////////////////////////
impl ByteCodeBuf {
    fn as_bytecode(&self) -> ByteCode {
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
impl ByteCodeBuilder {
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
        self.bytes.push(unsafe { std::mem::transmute(op_code) });
        for &word in self.args[0..op_code.word_args()].iter() {
            Self::push_word_bytes(&mut self.bytes, word)
        }
    }
    pub fn finish(self) -> ByteCodeBuf {
        let Self { bytes, .. } = self;
        ByteCodeBuf { bytes }
    }
}
impl<'a> VmParser<'a> {
    pub fn new(bytecode: ByteCode<'a>) -> Self {
        Self { res_buf: Default::default(), bytecode, next_code_offset: 0 }
    }
    pub fn parse_next(&mut self) {
        if let Some(op_code) = self.bytecode.read_op_code_at(self.next_code_offset) {
            match op_code.word_args() {
                0 => {
                    // success
                    self.next_code_offset += 1;
                }
                1 => {
                    if let [Some(arg0)] = [self.bytecode.read_word_at(self.next_code_offset + 1)] {
                        // success
                        self.res_buf.args[0] = arg0;
                        self.next_code_offset += 1 + WORD_SIZE;
                    } else {
                        // failure
                        self.res_buf.code = None;
                        return;
                    }
                }
                _ => unreachable!(),
            }
            self.res_buf.code = Some(op_code);
        }
    }
}
impl<'a> Vm<'a> {
    pub fn new_run(bytecode: ByteCode<'a>) {
        let mut vm = Self { stack: Default::default(), bp: 0, parser: VmParser::new(bytecode) };
        loop {
            vm.parser.parse_next();
            if !vm.take_do_parsed() {
                break;
            }
        }
    }
    pub fn take_do_parsed(&mut self) -> bool {
        if let Some(op_code) = self.parser.res_buf.code.take() {
            use OpCode as Oc;
            println!("VM handling {:?} with args {:?}", op_code, &self.parser.res_buf.args);
            let args = &self.parser.res_buf.args;
            match op_code {
                Oc::PushConst => self.stack.push(args[0]),
                Oc::TosDown => drop(self.stack.pop().unwrap()),
                Oc::Load => self.stack.push(*self.stack.get(args[0]).unwrap()),
                Oc::Store => *self.stack.get_mut(args[0]).unwrap() = self.stack.pop().unwrap(),
                Oc::JmpTo => self.parser.next_code_offset = args[0],
                Oc::DecStack => *self.stack.last_mut().unwrap() -= 1,
                Oc::IfNzJmp => {
                    if self.stack.pop().unwrap() != 0 {
                        self.parser.next_code_offset = args[0]
                    }
                }
                Oc::WrapAddStack => {
                    let [a, b] = [self.stack.pop().unwrap(), self.stack.pop().unwrap()];
                    self.stack.push(a + b)
                }
            }
            true
        } else {
            false
        }
    }
}

fn main() {
    let mut bcb = ByteCodeBuilder::default();
    bcb.args[0] = 0;
    bcb.push_with_args(OpCode::PushConst);
    let bc = bcb.finish();
    println!("{:?}", &bc);

    Vm::new_run(bc.as_bytecode());
}
