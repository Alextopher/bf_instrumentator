use std::num::Wrapping;

use crate::parser::IR;

type Cell = Wrapping<u8>;

// Implements an interpreter that makes use of the optimizations presented in http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html
// The interpreter is constructed with the BF program it is supposed to execute. Test cases are provided as an iterator of (input: Vec, output: Vec) tuples.
pub struct Interpreter {
    program: Vec<IR>,
    memory: Vec<Cell>,
    pointer: i32,
}

impl Interpreter {
    pub fn from(program: Vec<IR>) -> Self {
        Self {
            program,
            memory: vec![Wrapping(0); 10],
            pointer: 0,
        }
    }

    pub fn run_vec<I>(&mut self, instructions: Vec<IR>, inputs: &mut I) -> Vec<u8>
    where
        I: Iterator<Item = Wrapping<u8>>,
    {
        let mut output = Vec::new();
        for instruction in instructions {
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
                        std::iter::repeat(self.memory[(self.pointer + offset) as usize].0)
                            .take(times),
                    );
                }
                IR::Read { offset } => {
                    self.memory[(self.pointer + offset) as usize] = inputs.next().unwrap();
                }
                IR::Exact { x, offset } => {
                    self.memory[(self.pointer + offset) as usize] = Wrapping(x as u8);
                }
                IR::Loop { instructions } => {
                    while self.memory[self.pointer as usize] != Wrapping(0) {
                        output.extend(self.run_vec(instructions.clone(), inputs));
                    }
                }
                IR::Mul { x, y, offset } => {
                    let add = self.memory[(self.pointer + offset) as usize].0 as i32 * y;
                    self.memory[(self.pointer + x + offset) as usize] += Wrapping(add as u8);
                }
            };
        }
        output
    }

    pub fn run<I>(&mut self, inputs: I) -> Vec<u8>
    where
        I: IntoIterator<Item = u8>,
    {
        self.run_vec(self.program.clone(), &mut inputs.into_iter().map(Wrapping))
    }
}
