#[test]
fn parser() {
    let bf = ">,>,<[>[>+>+<<-]>>[<<+>>-]<<<-]>[-]>[-<<+>>]<<.[-]<";
    println!("{bf}");
    let instructions = crate::parser::optimize_o2(bf);
    println!("{:?}", instructions);

    let mut interpreter = crate::interpreter::Interpreter::from(instructions);
    let output = interpreter.run(vec![200, 100]);
    println!("{:?}", output);
}
