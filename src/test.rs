// A fuzing tests for the optimizer.
// NOTE: Some programs do no halt so we stop running them

use std::num::Wrapping;

use itertools::Itertools;
use rand::{thread_rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    interpreter::{Interpreter, RunTimeError},
    parser::{optimize_o0, optimize_o1, optimize_o2, optimize_o3},
};

fn random_bf() -> String {
    let mut rng = rand::thread_rng();
    let mut bf = String::new();
    let mut i = 0;

    while i < rng.gen_range(0..100) {
        let r = rng.gen_range(0..8);
        match r {
            0 => bf.push('+'),
            1 => bf.push('-'),
            2 => bf.push('>'),
            3 => bf.push('<'),
            4 => bf.push('.'),
            5 => bf.push(','),
            6 => bf.push('['),
            7 => bf.push(']'),
            _ => panic!("Invalid random number"),
        };
        i += 1;
    }

    // check that the brackets are balanced
    let mut stack = vec![];
    for c in bf.chars() {
        if c == '[' {
            stack.push(());
        } else if c == ']' {
            if let None = stack.pop() {
                return random_bf();
            }
        }
    }

    if stack.is_empty() {
        bf
    } else {
        random_bf()
    }
}

#[test]
fn many() {
    loop {
        let bf = random_bf();
        println!("{}", &bf);
        specific(&bf);
    }
}

#[test]
fn one() {
    let bf = ">++.+[+]+.><[].<";
    println!("{}", &bf);
    specific(&bf);
}

fn specific(bf: &str) {
    let o0 = optimize_o0(&bf);
    let o1 = optimize_o1(&bf);
    let o2 = optimize_o2(&bf);
    let o3 = optimize_o3(&bf);

    // Check that all parses have the same Optimizer error
    if o0.is_err() {
        assert_eq!(o0, o1);
        assert_eq!(o0, o2);
        assert_eq!(o0, o3);
    }

    // If there was an error we stop here
    if o0.is_err() {
        return;
    }

    let o0 = o0.unwrap();
    let o1 = o1.unwrap();
    let o2 = o2.unwrap();
    let o3 = o3.unwrap();

    // Run all programs
    let max_iterations = 1000000;
    let mut i0 = Interpreter::from(o0.clone(), max_iterations);
    let mut i1 = Interpreter::from(o1.clone(), max_iterations);
    let mut i2 = Interpreter::from(o2.clone(), max_iterations);
    let mut i3 = Interpreter::from(o3.clone(), max_iterations);

    println!("O0 {:?}", o0);
    println!("O1 {:?}", o1);
    println!("O2 {:?}", o2);
    println!("O3 {:?}", o3);

    // Generate an infinite stream of random inputs
    let seed = thread_rng().gen::<u64>();

    let inputs0 = {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..).map(move |_| rng.gen::<Wrapping<u8>>())
    };

    let inputs1 = {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..).map(move |_| rng.gen::<Wrapping<u8>>())
    };

    let inputs2 = {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..).map(move |_| rng.gen::<Wrapping<u8>>())
    };

    let inputs3 = {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        (0..).map(move |_| rng.gen::<Wrapping<u8>>())
    };

    let (e0, r0) = i0.run_iter(inputs0);
    let (e1, r1) = i1.run_iter(inputs1);
    let (e2, r2) = i2.run_iter(inputs2);
    let (e3, r3) = i3.run_iter(inputs3);

    if let Some(_) = e0 {
        // Ensure all programs finished with the same error state
        assert_eq!(e0, e1);
        assert_eq!(e0, e2);
        assert_eq!(e0, e3);
    } else {
        // Ensure all programs finished with the same output
        assert_eq!(r0, r1);
        assert_eq!(r0, r2);
        assert_eq!(r0, r3);
    }
}
