use std::num::Wrapping;

use either::Either;
use interpreter::{Interpreter, RunTimeError};

mod interpreter;
mod parser;

#[derive(Debug, PartialEq, Eq)]
pub struct TestFailure {
    typ: TestFailureType,
    input: Vec<Wrapping<u8>>,
    expected_output: Vec<Wrapping<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestFailureType {
    RunTimeError { err: interpreter::RunTimeError },
    NonZeroPointer { pointer: i32 },
    NonZeroMemory { memory: Vec<Wrapping<u8>> },
    IncorrectOutput { output: Vec<Wrapping<u8>> },
    OptimizerError(parser::OptimizerError),
}

pub enum OptimizationLevel {
    O0,
    O1,
    O2,
    O3,
}

pub fn test<I, O>(
    bf: &str,
    inputs: I,
    outputs: O,
    optimization_level: OptimizationLevel,
    max_iterations: usize,
) -> Vec<TestFailure>
where
    I: IntoIterator<Item = Vec<Wrapping<u8>>>,
    O: IntoIterator<Item = Vec<Wrapping<u8>>>,
{
    let optimizer = match optimization_level {
        OptimizationLevel::O0 => parser::optimize_o0,
        OptimizationLevel::O1 => parser::optimize_o1,
        OptimizationLevel::O2 => parser::optimize_o2,
        OptimizationLevel::O3 => parser::optimize_o3,
    };

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

pub fn run(
    bf: &str,
    input: &[Wrapping<u8>],
    optimization_level: OptimizationLevel,
    max_iterations: usize,
) -> Result<Vec<Wrapping<u8>>, Either<RunTimeError, parser::OptimizerError>> {
    let optimizer = match optimization_level {
        OptimizationLevel::O0 => parser::optimize_o0,
        OptimizationLevel::O1 => parser::optimize_o1,
        OptimizationLevel::O2 => parser::optimize_o2,
        OptimizationLevel::O3 => parser::optimize_o3,
    };

    match optimizer(bf) {
        Ok(instructions) => {
            let mut interpreter = Interpreter::from(instructions, max_iterations);
            let (err, output) = interpreter.run(input);

            if let Some(err) = err {
                Err(Either::Left(err))
            } else {
                Ok(output)
            }
        }
        Err(e) => Err(Either::Right(e)),
    }
}

#[cfg(test)]
mod test;
