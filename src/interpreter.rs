use std::num::Wrapping;

use crate::parser::IR;

type Cell = Wrapping<u8>;

// Implements an interpreter that makes use of the optimizations presented in http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html
// The interpreter is constructed with the BF program it is supposed to execute. Test cases are provided as an iterator of (input: Vec, output: Vec) tuples.
pub struct Interpreter {
    program: Vec<IR>,
    pub memory: Vec<Cell>,
    pub pointer: i32,
}

impl Interpreter {
    pub fn from(program: Vec<IR>) -> Self {
        Self {
            program,
            memory: vec![Wrapping(0); 65536],
            pointer: 0,
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

    pub fn reset(&mut self) {
        self.memory = vec![Wrapping(0); 65536];
        self.pointer = 0;
    }

    pub fn run_vec<'a, I>(
        &mut self,
        instructions: Vec<IR>,
        inputs: &mut I,
        debug: bool,
    ) -> Vec<Wrapping<u8>>
    where
        I: Iterator<Item = &'a Wrapping<u8>>,
    {
        let mut output = Vec::new();
        for instruction in instructions {
            // print the instruction that is about to run
            if debug {
                println!("{:?}", instruction);
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
                    self.memory[(self.pointer + offset) as usize] = *inputs.next().unwrap();
                }
                IR::Exact { x, offset } => {
                    self.memory[(self.pointer + offset) as usize] = Wrapping(x as u8);
                }
                IR::Loop { over, instructions } => {
                    // preform a move
                    self.pointer += over;

                    // then begin the loop
                    while self.memory[self.pointer as usize] != Wrapping(0) {
                        output.extend(self.run_vec(instructions.clone(), inputs, debug));
                    }
                }
                IR::Mul { x, y, offset } => {
                    let add = self.memory[(self.pointer + offset) as usize].0 as i32 * y;
                    self.memory[(self.pointer + offset + x) as usize] += Wrapping(add as u8);
                }
            };

            if debug {
                // print the current state of the memory
                println!(
                    "{}",
                    self.memory
                        .iter()
                        .map(|x| x.0)
                        .fold(String::new(), |acc, x| {
                            // each number is 3 characters wide
                            acc + &format!("{:03} ", x)
                        })
                );
                // print an "^" at the current pointer
                print!(
                    "{}",
                    std::iter::repeat("    ")
                        .take(self.pointer as usize)
                        .collect::<String>()
                );
                println!("^");
            }
        }
        output
    }

    pub fn run(&mut self, inputs: &Vec<Wrapping<u8>>, debug: bool) -> Vec<Wrapping<u8>> {
        self.run_vec(self.program.clone(), &mut inputs.iter(), debug)
    }
}
