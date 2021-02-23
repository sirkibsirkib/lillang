use crate::{vm::Vm, ByteCodeBuilder, OpCode as Oc};

#[test]
fn just_push() {
    let mut bcb = ByteCodeBuilder::default();
    bcb.args[0] = 0;
    bcb.push_with_args(Oc::PushConst);
    let bc = bcb.finish();
    println!("{:?}", &bc);

    Vm::new_run(bc.as_bytecode());
}
