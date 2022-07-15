use std::num::Wrapping;

mod interpreter;
mod parser;

#[derive(Debug, PartialEq)]
pub struct TestFailure {
    typ: TestFailureType,
    input: Vec<Wrapping<u8>>,
    expected_output: Vec<Wrapping<u8>>,
}

#[derive(Debug, PartialEq)]
pub enum TestFailureType {
    RunTimeError { err: interpreter::RunTimeError },
    NonZeroPointer { pointer: i32 },
    NonZeroMemory { memory: Vec<Wrapping<u8>> },
    IncorrectOutput { output: Vec<Wrapping<u8>> },
    OptimizerError(parser::OptimizerError),
}

fn run<'a, I, O, F>(
    bf: &str,
    inputs: I,
    outputs: O,
    optimizer: F,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
    F: FnOnce(&str) -> Result<Vec<parser::IR>, parser::OptimizerError>,
{
    match optimizer(bf) {
        Ok(instructions) => {
            let mut interpreter =
                crate::interpreter::Interpreter::from(instructions, max_iterations);

            let mut errors = Vec::new();
            let zipped = inputs.into_iter().zip(outputs);
            for (input, expected_output) in zipped {
                let (err, actual) = interpreter.run(&input);

                let pointer = interpreter.get_pointer();
                let memory = interpreter.return_shrinked_memory();

                if let Some(err) = err {
                    errors.push(TestFailure {
                        typ: TestFailureType::RunTimeError { err },
                        input: input.clone(),
                        expected_output: expected_output.clone(),
                    })
                }

                // Note: Each valid error is returned, they are not mutual exclusive.
                // For example, if the program halts when max_iterations is exceeded we may return MaxIterationsExceeded and NonZeroPointer.
                if pointer != 0 {
                    errors.push(TestFailure {
                        typ: TestFailureType::NonZeroPointer { pointer },
                        input: input.clone(),
                        expected_output: expected_output.clone(),
                    });
                }

                if memory.iter().any(|x| x != &Wrapping(0)) {
                    errors.push(TestFailure {
                        typ: TestFailureType::NonZeroMemory { memory },
                        input: input.clone(),
                        expected_output: expected_output.clone(),
                    });
                }

                if actual != expected_output {
                    errors.push(TestFailure {
                        typ: TestFailureType::IncorrectOutput { output: actual },
                        input,
                        expected_output,
                    });
                }

                interpreter.reset();
            }

            errors
        }
        Err(_) => todo!(),
    }
}

pub fn run_bf_o3<'a, I, O>(
    bf: &str,
    inputs: I,
    outputs: O,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(
        bf,
        inputs,
        outputs,
        crate::parser::optimize_o3,
        max_iterations,
    )
}

pub fn run_bf_o2<'a, I, O>(
    bf: &str,
    inputs: I,
    outputs: O,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(
        bf,
        inputs,
        outputs,
        crate::parser::optimize_o2,
        max_iterations,
    )
}

pub fn run_bf_o1<'a, I, O>(
    bf: &str,
    inputs: I,
    outputs: O,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(
        bf,
        inputs,
        outputs,
        crate::parser::optimize_o1,
        max_iterations,
    )
}

pub fn run_bf_o0<'a, I, O>(
    bf: &str,
    inputs: I,
    outputs: O,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    run(
        bf,
        inputs,
        outputs,
        crate::parser::optimize_o0,
        max_iterations,
    )
}

#[cfg(test)]
mod test;
