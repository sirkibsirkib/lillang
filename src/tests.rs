use crate::{
    bytecode::{ByteCodeBuf, ByteCodeBufBuilder, OpCode as Oc},
    vm::Vm,
};

fn build_with(mut f: impl FnMut(&mut ByteCodeBufBuilder)) -> ByteCodeBuf {
    let mut bcbb = ByteCodeBufBuilder::default();
    f(&mut bcbb);
    bcbb.finish()
}

#[test]
fn build_just_push() {
    let bcb = build_with(|bcbb| {
        // push(0)
        bcbb.args[0] = 0;
        bcbb.push_with_args(Oc::PushConst);
    });
    println!("{:?}", &bcb);
}

#[test]
fn run_just_push() {
    let bcb = build_with(|bcbb| {
        // push(0)
        bcbb.args[0] = 0;
        bcbb.push_with_args(Oc::PushConst);
    });
    Vm::new_run(bcb.as_bytecode());
}

#[test]
fn run_hello() {
    let bcb = build_with(|bcbb| {
        let string = b"hello\n";
        for &byte in string.iter().rev() {
            bcbb.args[0] = byte as usize;
            bcbb.push_with_args(Oc::PushConst);
        }
        for _ in string {
            bcbb.push_with_args(Oc::SysOut);
        }
    });
    Vm::new_run(bcb.as_bytecode());
}
