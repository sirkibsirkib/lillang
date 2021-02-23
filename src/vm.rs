use super::*;

#[derive(Debug)]
pub struct Vm<'a> {
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

///////////////////////////////////////////

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
