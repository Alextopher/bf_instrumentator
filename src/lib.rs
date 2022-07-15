use std::{fmt::Display, num::Wrapping};

mod interpreter;
mod parser;

#[derive(Debug)]
pub enum BfTestError {
    TestFailed {
        input: Vec<Wrapping<u8>>,
        expected: Vec<Wrapping<u8>>,
        actual: Vec<Wrapping<u8>>,
        tape: Vec<Wrapping<u8>>,
    },
    NonZeroTape {
        input: Vec<Wrapping<u8>>,
        tape: Vec<Wrapping<u8>>,
    },
    NonZeroPointer {
        input: Vec<Wrapping<u8>>,
        tape: Vec<Wrapping<u8>>,
        pointer: i32,
    },
}

impl Display for BfTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BfTestError::TestFailed {
                input,
                expected,
                actual,
                tape,
            } => {
                writeln!(f, "Test failed:")?;
                writeln!(f, "Input: {:?}", input)?;
                writeln!(f, "Expected: {:?}", expected)?;
                writeln!(f, "Actual: {:?}", actual)?;
                writeln!(f, "Tape: {:?}", tape)?;
                Ok(())
            }
            BfTestError::NonZeroTape { input, tape } => {
                writeln!(f, "Non-zero tape:")?;
                writeln!(f, "Input: {:?}", input)?;
                writeln!(f, "Tape: {:?}", tape)?;
                Ok(())
            }
            BfTestError::NonZeroPointer {
                input,
                tape,
                pointer,
            } => {
                writeln!(f, "Non-zero pointer:")?;
                writeln!(f, "Input: {:?}", input)?;
                writeln!(f, "Tape: {:?}", tape)?;
                writeln!(f, "Pointer: {}", pointer)?;
                Ok(())
            }
        }
    }
}

fn run<'a, I, O, F>(bf: &str, inputs: I, outputs: O, optimizer: F) -> Vec<BfTestError>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
    F: FnOnce(&str) -> Vec<parser::IR>,
{
    let instructions = optimizer(bf);

    let mut interpreter = crate::interpreter::Interpreter::from(instructions);

    let mut errors = Vec::new();
    let zipped = inputs.into_iter().zip(outputs);
    for (input, output) in zipped {
        let actual = interpreter.run(&input, false);

        if interpreter.pointer != 0 {
            errors.push(BfTestError::NonZeroPointer {
                input: input.clone(),
                tape: interpreter.return_shrinked_memory(),
                pointer: interpreter.pointer,
            });
        }

        if interpreter.memory.iter().any(|x| x != &Wrapping(0)) {
            errors.push(BfTestError::NonZeroTape {
                input: input.clone(),
                tape: interpreter.return_shrinked_memory(),
            });
        }

        if actual != output {
            errors.push(BfTestError::TestFailed {
                input: input.clone(),
                expected: output,
                actual,
                tape: interpreter.return_shrinked_memory(),
            });
        }

        interpreter.reset();
    }

    errors
}

pub fn run_bf_o3<'a, I, O>(bf: &str, inputs: I, outputs: O) -> Vec<BfTestError>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(bf, inputs, outputs, crate::parser::optimize_o3)
}

pub fn run_bf_o2<'a, I, O>(bf: &str, inputs: I, outputs: O) -> Vec<BfTestError>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(bf, inputs, outputs, crate::parser::optimize_o2)
}

pub fn run_bf_o1<'a, I, O>(bf: &str, inputs: I, outputs: O) -> Vec<BfTestError>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(bf, inputs, outputs, crate::parser::optimize_o1)
}

pub fn run_bf_o0<'a, I, O>(bf: &str, inputs: I, outputs: O) -> Vec<BfTestError>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(bf, inputs, outputs, crate::parser::optimize_o0)
}

#[cfg(test)]
mod test;
