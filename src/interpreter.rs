use std::num::Wrapping;

use crate::parser::IR;

type Cell = Wrapping<u8>;

#[derive(Debug, PartialEq)]
pub enum RunTimeError {
    OutOfBounds,
    OutOfInputs,
    MaxIterationsExceeded,
}

// Implements an interpreter that makes use of the optimizations presented in http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html
// The interpreter is constructed with the BF program it is supposed to execute. Test cases are provided as an iterator of (input: Vec, output: Vec) tuples.
pub struct Interpreter {
    program: Vec<IR>,
    memory: Vec<Cell>,
    pointer: i32,
    iterations: usize,
    max_iterations: usize,
}

impl Interpreter {
    pub fn from(program: Vec<IR>, max_iterations: usize) -> Self {
        Self {
            program,
            memory: vec![Wrapping(0); 65536],
            pointer: 0,
            iterations: 0,
            max_iterations,
        }
    }

    pub fn return_shrinked_memory(&self) -> Vec<Cell> {
        // find the last non-zero cell
        let mut last_non_zero_cell = 0;
        for (i, cell) in self.memory.iter().enumerate() {
            if *cell != Wrapping(0) {
                last_non_zero_cell = i;
            }
        }

        self.memory[0..=last_non_zero_cell].to_vec()
    }

    pub fn get_pointer(&self) -> i32 {
        self.pointer
    }

    pub fn reset(&mut self) {
        self.memory = vec![Wrapping(0); 65536];
        self.pointer = 0;
        self.iterations = 0;
    }

    pub fn run_vec<'a, I>(
        &mut self,
        instructions: Vec<IR>,
        inputs: &mut I,
    ) -> (Option<RunTimeError>, Vec<Wrapping<u8>>)
    where
        I: Iterator<Item = &'a Wrapping<u8>>,
    {
        let mut output = Vec::new();
        for instruction in instructions {
            self.iterations += 1;
            if self.iterations > self.max_iterations {
                return (Some(RunTimeError::MaxIterationsExceeded), output);
            }

            match instruction {
                IR::Add { x, offset } => {
                    if x < 0 {
                        self.memory[(self.pointer + offset) as usize] -= Wrapping(-x as u8);
                    } else if x > 0 {
                        self.memory[(self.pointer + offset) as usize] += Wrapping(x as u8);
                    }
                }
                IR::Move { over } => {
                    self.pointer += over;
                }
                IR::Print { times, offset } => {
                    output.extend(
                        std::iter::repeat(self.memory[(self.pointer + offset) as usize])
                            .take(times),
                    );
                }
                IR::Read { offset } => {
                    if let Some(input) = inputs.next() {
                        self.memory[(self.pointer + offset) as usize] = *input;
                    } else {
                        return (Some(RunTimeError::OutOfInputs), output);
                    }
                }
                IR::Exact { x, offset } => {
                    self.memory[(self.pointer + offset) as usize] = Wrapping(x as u8);
                }
                IR::Loop { over, instructions } => {
                    // preform a move
                    self.pointer += over;

                    // then begin the loop
                    while self.memory[self.pointer as usize] != Wrapping(0) {
                        let (err, outputs) = self.run_vec(instructions.clone(), inputs);
                        output.extend(outputs);

                        if err.is_some() {
                            return (err, output);
                        }
                    }
                }
                IR::Mul { x, y, offset } => {
                    let add = self.memory[(self.pointer + offset) as usize].0 as i32 * y;
                    self.memory[(self.pointer + offset + x) as usize] += Wrapping(add as u8);
                }
            };
        }
        (None, output)
    }

    pub fn run(&mut self, inputs: &Vec<Wrapping<u8>>) -> (Option<RunTimeError>, Vec<Wrapping<u8>>) {
        self.run_vec(self.program.clone(), &mut inputs.iter())
    }
}
