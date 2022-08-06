// Parses brainfuck code into an itermediate representation following optimizations strategies presented in http://calmerthanyouare.org/2015/01/07/optimizing-brainfuck.html

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Add { x: i32, offset: i32 },
    Move { over: i32 },
    Print { times: usize, offset: i32 },
    Read { offset: i32 },
    Exact { x: i32, offset: i32 },
    Loop { over: i32, instructions: Vec<IR> },
    Mul { x: i32, y: i32, offset: i32 }, // m[p+x] = m[p] * y
}

impl From<char> for IR {
    fn from(c: char) -> Self {
        match c {
            '+' => IR::Add { x: 1, offset: 0 },
            '-' => IR::Add { x: -1, offset: 0 },
            '>' => IR::Move { over: 1 },
            '<' => IR::Move { over: -1 },
            '.' => IR::Print {
                times: 1,
                offset: 0,
            },
            ',' => IR::Read { offset: 0 },
            _ => panic!("Unrecognized character: {}", c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizerError {
    UnbalancedBrackets,
}

// Removes any Add { x: 0, offset: _ } or Move { over: 0 } instructions.
fn remove_zero_moves_and_adds(v: Vec<IR>) -> Vec<IR> {
    v.into_iter()
        .filter(|x| match x {
            IR::Add { x, offset: _ } => *x != 0,
            IR::Move { over } => *over != 0,
            _ => true,
        })
        // recursively remove zero moves and fixes on loops
        .map(|x| match x {
            IR::Loop { over, instructions } => IR::Loop {
                over,
                instructions: remove_zero_moves_and_adds(instructions),
            },
            _ => x,
        })
        .collect()
}

// Parses brainfuck code into an IR with _no_ optimizations.
pub(crate) fn optimize_o0(bf: &str) -> Result<Vec<IR>, OptimizerError> {
    let mut instructions_stack: Vec<Vec<IR>> = vec![vec![]];

    for c in bf.chars() {
        if c == '[' {
            instructions_stack.push(vec![]);
        } else if c == ']' {
            let loop_instructions = instructions_stack
                .pop()
                .ok_or(OptimizerError::UnbalancedBrackets)?;

            instructions_stack
                .last_mut()
                .ok_or(OptimizerError::UnbalancedBrackets)?
                .push(IR::Loop {
                    over: 0,
                    instructions: loop_instructions,
                });
        } else {
            instructions_stack
                .last_mut()
                .ok_or(OptimizerError::UnbalancedBrackets)?
                .push(c.into());
        }
    }

    if let Some(mut last_instructions) = instructions_stack.pop() {
        last_instructions = remove_zero_moves_and_adds(last_instructions);
        Ok(last_instructions)
    } else {
        Err(OptimizerError::UnbalancedBrackets)
    }
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
pub(crate) fn optimize_o1(bf: &str) -> Result<Vec<IR>, OptimizerError> {
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
                    (
                        IR::Loop {
                            over: 0,
                            instructions: _,
                        },
                        IR::Loop {
                            over: 0,
                            instructions: _,
                        },
                    ) => {}
                    (
                        IR::Exact { x: 0, offset: 0 },
                        IR::Loop {
                            over: 0,
                            instructions: _,
                        },
                    ) => {}
                    // optimizes [-] and [+] into Clear or just recursively optimizes the loop
                    (
                        _,
                        IR::Loop {
                            over: 0,
                            instructions,
                        },
                    ) => {
                        if instructions.len() == 1
                            && (instructions[0] == IR::Add { x: 1, offset: 0 }
                                || instructions[0] == IR::Add { x: -1, offset: 0 })
                        {
                            result.push(IR::Exact { x: 0, offset: 0 });
                        } else {
                            result.push(IR::Loop {
                                over: 0,
                                instructions: o1_optimize_vec(instructions, false),
                            });
                        }
                    }
                    (_, i) => {
                        result.push(i.clone());
                    }
                },
            }
        }

        // remove the initial Clear instruction
        if program_start && !result.is_empty() && result[0] == (IR::Exact { x: 0, offset: 0 }) {
            return result.into_iter().skip(1).collect();
        }

        // Fold adjacent instructions into a single instruction.
        result
    }

    // Start with O0 code
    let instructions = optimize_o0(bf)?;

    // Fold adjacent instructions into a single instruction.
    Ok(remove_zero_moves_and_adds(o1_optimize_vec(
        &instructions,
        true,
    )))
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
pub(crate) fn optimize_o2(bf: &str) -> Result<Vec<IR>, OptimizerError> {
    // Helper function that takes as input a vec<IR>
    fn o2_optimize_vec(v: &Vec<IR>) -> Vec<IR> {
        let mut result: Vec<IR> = vec![];
        // Tracks how the behavior of a cell changes over time.
        let mut behaviors: HashMap<i32, Behavior> = HashMap::new();
        let mut offset = 0;

        for i in v {
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
                IR::Loop {
                    over: 0,
                    instructions,
                } => {
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

                    // recursively optimize the loop
                    result.push(IR::Loop {
                        over: offset,
                        instructions: o2_optimize_vec(instructions),
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

        // Technically a "correct" program we only need to run this within a loop.
        // However, for my use case I don't like side effects and want my program to end at 0.
        if offset != 0 {
            result.push(IR::Move { over: offset })
        }

        result
    }

    // Start with O1 optimize
    let instructions = optimize_o1(bf)?;

    // Optimize the program
    Ok(remove_zero_moves_and_adds(o2_optimize_vec(&instructions)))
}

// Merges move instructions into the offsets of future instructions until we hit a loop
fn merge_moves_into_offset(instructions: Vec<IR>) -> Vec<IR> {
    let mut result: Vec<IR> = vec![];
    let mut new_offset = 0;

    for i in instructions {
        match i {
            IR::Move { over } => {
                new_offset += over;
            }
            IR::Add { x, offset } => {
                result.push(IR::Add {
                    x,
                    offset: offset + new_offset,
                });
            }
            IR::Print { times, offset } => {
                result.push(IR::Print {
                    times,
                    offset: offset + new_offset,
                });
            }
            IR::Read { offset } => {
                result.push(IR::Read {
                    offset: offset + new_offset,
                });
            }
            IR::Exact { x, offset } => {
                result.push(IR::Exact {
                    x,
                    offset: offset + new_offset,
                });
            }
            IR::Mul { x, y, offset } => {
                result.push(IR::Mul {
                    x,
                    y,
                    offset: offset + new_offset,
                });
            }
            IR::Loop { over, instructions } => {
                result.push(IR::Loop {
                    over: over + new_offset,
                    instructions: merge_moves_into_offset(instructions),
                });
            }
        }
    }

    if new_offset != 0 {
        result.push(IR::Move { over: new_offset });
    }

    result
}

// O3 optimizations adds:
// - If a loop has the follow structure:
//   - Loop only has Add and Exact instructions
//   - At offset 0 there is an Add { x: -1, offset: 0 } instruction
//   - TODO: Support Add { x: 1, offset: 0 }
// Then the loop is removed and each Add { x, offset } instruction is replaced with a Mul { x: offset, y: x, offset: loop_offset } instruction.
// The Exact instructions are kept as they are.
// And an Exact { x: 0, offset: 0 } instruction is added at the end.
pub(crate) fn optimize_o3(bf: &str) -> Result<Vec<IR>, OptimizerError> {
    fn o3_optimize_vec(instruction: IR) -> Vec<IR> {
        if let IR::Loop { over, instructions } = instruction {
            // Verify that the loop is only Add and Exact instructions
            let only_add_and_exact = instructions.iter().all(|i| {
                matches!(
                    i,
                    IR::Add { x: _, offset: _ } | IR::Exact { x: _, offset: _ }
                )
            });

            // Verify that there is the Add { x: -1, offset: 0 } instruction
            let is_sub_one = instructions
                .iter()
                .any(|i| matches!(i, IR::Add { x: -1, offset: 0 }));

            if only_add_and_exact && is_sub_one {
                instructions
                    .into_iter()
                    .filter(|i| !matches!(i, IR::Add { x: -1, offset: 0 }))
                    .map(|i| match i {
                        IR::Add { x, offset } => IR::Mul {
                            x: offset,
                            y: x,
                            offset: over,
                        },
                        _ => i,
                    })
                    .chain(std::iter::once(IR::Exact { x: 0, offset: over }))
                    .chain(std::iter::once(IR::Move { over }))
                    .collect()
            } else {
                let mut result = vec![];

                instructions
                    .into_iter()
                    .for_each(|i| result.extend(o3_optimize_vec(i)));

                vec![IR::Loop {
                    over,
                    instructions: result,
                }]
            }
        } else {
            vec![instruction]
        }
    }

    // Start with O2 optimize
    let instructions = optimize_o2(bf)?;

    let mut result = vec![];

    // Optimize the program
    instructions
        .into_iter()
        .for_each(|i| result.extend(o3_optimize_vec(i)));

    Ok(merge_moves_into_offset(result))
}
