// Parses brainfuck code into an itermediate representation following optimizations presented in http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Add { x: i32, offset: i32 },
    Move { over: i32 },
    Print { times: usize, offset: i32 },
    Read { offset: i32 },
    Exact { x: i32, offset: i32 },
    Loop { instructions: Vec<IR> },
    Mul { x: i32, y: i32, offset: i32 }, // m[p+x] = m[p] * y
}

// Parses brainfuck code into an IR with _no_ optimizations.
pub(crate) fn optimize_o0(bf: &str) -> Vec<IR> {
    let mut instructions_stack: Vec<Vec<IR>> = vec![vec![]];

    for c in bf.chars() {
        match c {
            '+' => instructions_stack
                .last_mut()
                .unwrap()
                .push(IR::Add { x: 1, offset: 0 }),
            '-' => instructions_stack
                .last_mut()
                .unwrap()
                .push(IR::Add { x: -1, offset: 0 }),
            '>' => instructions_stack
                .last_mut()
                .unwrap()
                .push(IR::Move { over: 1 }),
            '<' => instructions_stack
                .last_mut()
                .unwrap()
                .push(IR::Move { over: -1 }),
            '.' => instructions_stack.last_mut().unwrap().push(IR::Print {
                times: 1,
                offset: 0,
            }),
            ',' => instructions_stack
                .last_mut()
                .unwrap()
                .push(IR::Read { offset: 0 }),
            '[' => {
                instructions_stack.push(vec![]);
            }
            ']' => {
                let loop_instructions = instructions_stack.pop().unwrap();
                instructions_stack.last_mut().unwrap().push(IR::Loop {
                    instructions: loop_instructions,
                });
            }
            _ => {}
        }
    }

    if instructions_stack.len() != 1 {
        panic!("Unbalanced brackets");
    }

    instructions_stack.pop().unwrap()
}

// Parses brainfuck code into an IR with some optimizations.
// ALL OFFSETS ARE STILL 0
// Optimizations:
// - Join adjacent Add, Print, and Move instructions into a single instruction.
//      Note for move: It is reasonable to treat moving off the tape as undefined behavior. Therefor, I am comfortable with allowing this program `<<<>>>>+`
//      to compile down to `Move { 1 } Add { 1 }`
// - Join adjacent Print instructions into a single instruction.
// - Add before a Read destroys the Add
// - Clear before a Read destroys the Clear
// - Optimizes [-] and [+] into Clear
// - Adjacent loops are deleted. `[.-][.]` becomes `[.-]` because the second loop will never be executed.
pub(crate) fn optimize_o1(bf: &str) -> Vec<IR> {
    // Helper function that takes as input a vec<IR>
    fn o1_optimize_vec(v: &Vec<IR>, program_start: bool) -> Vec<IR> {
        let mut result: Vec<IR> = if program_start {
            // Adds an implicit clear on program start
            vec![IR::Exact { x: 0, offset: 0 }]
        } else {
            vec![]
        };

        for i in v {
            match result.last_mut() {
                None => {
                    result.push(i.clone());
                }
                Some(last) => match (last, i) {
                    // Joins adjacent Add and Move instructions into a single instruction.
                    (IR::Add { x: a, offset: 0 }, IR::Add { x: b, offset: 0 }) => *a += b,
                    (IR::Move { over: a }, IR::Move { over: b }) => *a += b,
                    // Add followed by Read destroys the Add
                    (IR::Add { x: _, offset: 0 }, IR::Read { offset: 0 }) => {
                        result.pop();
                        result.push(i.clone());
                    }
                    // Clear followed by Read destroys the Clear
                    (IR::Exact { x: 0, offset: 0 }, IR::Read { offset: 0 }) => {
                        result.pop();
                        result.push(i.clone());
                    }
                    (
                        IR::Print {
                            times: a,
                            offset: _,
                        },
                        IR::Print {
                            times: b,
                            offset: _,
                        },
                    ) => {
                        *a += b;
                    }
                    // loops immediately following a loop are ignored
                    (IR::Loop { instructions: _ }, IR::Loop { instructions: _ }) => {}
                    (IR::Exact { x: 0, offset: 0 }, IR::Loop { instructions: _ }) => {}
                    // optimizes [-] and [+] into Clear or just recursively optimizes the loop
                    (_, IR::Loop { instructions: i }) => {
                        if i.len() == 1
                            && (i[0] == IR::Add { x: 1, offset: 0 }
                                || i[0] == IR::Add { x: -1, offset: 0 })
                        {
                            result.push(IR::Exact { x: 0, offset: 0 });
                        } else {
                            result.push(IR::Loop {
                                instructions: o1_optimize_vec(&i, false),
                            });
                        }
                    }
                    (_, i) => {
                        result.push(i.clone());
                    }
                },
            }
        }

        if program_start {
            // remove the initial Clear instruction
            return result.into_iter().skip(1).collect();
        }

        // Fold adjacent instructions into a single instruction.
        result
    }

    // Start with O0 code
    let instructions = optimize_o0(bf);

    // Fold adjacent instructions into a single instruction.
    o1_optimize_vec(&instructions, true)
}

// This type is used to merge nonadjacent Clear and Add instructions that update the same memory cell.
enum Behavior {
    Add(i32),
    // Sets the memory cell to be an exact value.
    Exact(i32),
}

// In addition to the optimizations in O1 this function also optimizes the following:
// - Adds offset to Add instructions when the offset is known
//   for example at program start `>++++>+++++[loop]` becomes Add { x: 4, offset: 1 } Add { x: 5, offset: 2 } Move { over: 2} ...
//   similarily if we are within a loop that only consists of Add and Move instructions and all the Move instructions add to 0
//   then we can remove the moves by adding offsets to the Add instructions.
// - Non-adjacent Adds that change the same cell are merged
pub(crate) fn optimize_o2(bf: &str) -> Vec<IR> {
    // Helper function that takes as input a vec<IR>
    fn o2_optimize_vec(v: &Vec<IR>, inLoop: bool) -> Vec<IR> {
        let mut result: Vec<IR> = vec![];
        // Tracks how the behavior of a cell changes over time.
        let mut behaviors: HashMap<i32, Behavior> = HashMap::new();
        let mut offset = 0;

        let mut iter = v.iter();

        while let Some(i) = iter.next() {
            match i {
                IR::Move { over } => {
                    offset += *over;
                }
                IR::Add { x, offset: 0 } => {
                    let behavior = behaviors.get(&offset);
                    let result = match behavior {
                        Some(Behavior::Add(y)) => Behavior::Add(*y + *x),
                        Some(Behavior::Exact(y)) => Behavior::Exact(*y + *x),
                        None => Behavior::Add(*x),
                    };
                    behaviors.insert(offset, result);
                }
                IR::Exact { x: 0, offset: 0 } => {
                    behaviors.insert(offset, Behavior::Exact(0));
                }
                IR::Read { offset: 0 } => {
                    // Drop the history and return the read instruction.
                    behaviors.remove(&offset);
                    result.push(IR::Read { offset });
                }
                IR::Print { times, offset: 0 } => {
                    // When we see a Print instruction we need to
                    // 1. Apply the behavior
                    // 2. Drop the history
                    // 3. Print
                    let behavior = behaviors.get(&offset);
                    match behavior {
                        Some(Behavior::Add(x)) => result.push(IR::Add { x: *x, offset }),
                        Some(Behavior::Exact(x)) => result.push(IR::Exact { x: *x, offset }),
                        _ => {}
                    }
                    behaviors.remove(&offset);
                    result.push(IR::Print {
                        times: *times,
                        offset,
                    });
                }
                IR::Loop { instructions } => {
                    // When we see a Loop instruction we need to
                    // 1. Consider if the behavior at this offset is Exact(0), if so we can remove the loop and consider as normal
                    // 2. Apply all of the behaviors that have been tracked so far
                    // 3. Drop the history
                    // 4. Move { offset }
                    // 5. Recursively optimize the loop
                    let behavior = behaviors.get(&offset);

                    if let Some(Behavior::Exact(0)) = behavior {
                        // continue as normal
                        continue;
                    }

                    // apply the behaviors
                    for (o, b) in behaviors.iter() {
                        result.push(match b {
                            Behavior::Add(x) => IR::Add { x: *x, offset: *o },
                            Behavior::Exact(x) => IR::Exact { x: *x, offset: *o },
                        });
                    }

                    // drop the history
                    behaviors.clear();

                    // move the offset
                    result.push(IR::Move { over: offset });

                    // recursively optimize the loop
                    result.push(IR::Loop {
                        instructions: o2_optimize_vec(&instructions, true),
                    });

                    // reset the offset counter and continue as normal
                    offset = 0;
                }
                _ => {
                    panic!("Unexpected instruction in program {i:?}");
                }
            }
        }

        // At the end of the list we need to apply the behaviors
        for (o, b) in behaviors.iter() {
            result.push(match b {
                Behavior::Add(x) => IR::Add { x: *x, offset: *o },
                Behavior::Exact(x) => IR::Exact { x: *x, offset: *o },
            });
        }

        // If we are in a loop we need to add a Move instruction
        if inLoop && offset != 0 {
            result.push(IR::Move { over: offset });
        }

        result
    }

    // Start with O1 optimize
    let instructions = optimize_o1(bf);

    // Optimize the program
    o2_optimize_vec(&instructions, false)
}
