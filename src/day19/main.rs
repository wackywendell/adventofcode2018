#![warn(clippy::all)]

use aoc::device::{Instruction, OpCode, Register, Value};

use clap::{App, Arg};
use text_io::try_scan;

use std::collections::VecDeque;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub struct Device {
    pub register: Register,
    pub bound: usize,
    pub pointer: usize,
    pub instructions: Vec<Instruction>,
}

impl Device {
    pub fn new(registers: usize, bound: usize, instructions: Vec<Instruction>) -> Self {
        let values = std::iter::repeat(0 as Value).take(registers).collect();
        Device {
            register: Register { values },
            bound,
            pointer: 0,
            instructions,
        }
    }

    pub fn apply(&mut self) -> bool {
        let instruction = match self.instructions.get(self.pointer) {
            None => return false,
            Some(&v) => v,
        };
        self.register.values[self.bound] = self.pointer as Value;
        self.register.apply(instruction);
        self.pointer = self.register.values[self.bound] as usize;
        self.pointer += 1;

        true
    }
}

pub fn parse_instructions<I, S>(lines: I) -> Result<(usize, Vec<Instruction>), failure::Error>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut pointer = None;
    let mut instructions: Vec<Instruction> = Vec::new();

    for l in lines {
        let l = l.as_ref().trim();
        if l.is_empty() {
            continue;
        }

        if pointer.is_none() {
            let pointer_value;
            try_scan!(l.bytes() => "#ip {}", pointer_value);
            pointer = Some(pointer_value);
            continue;
        }

        let (op_str, a, b, c): (String, usize, usize, usize);
        try_scan!(l.bytes() => "{} {} {} {}", op_str, a, b, c);
        let maybe_op = OpCode::from_string(&op_str);
        let op = match maybe_op {
            None => return Err(failure::format_err!("Unrecognized op {}", op_str)),
            Some(op) => op,
        };

        instructions.push(Instruction(op, a, b, c));
    }

    Ok((pointer.unwrap_or(0), instructions))
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 19")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day19.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let some_lines: std::io::Result<VecDeque<String>> = buf_reader.lines().collect();
    let lines: VecDeque<String> = some_lines?;
    let (pointer, instructions) = parse_instructions(lines)?;
    println!(
        "Found pointer {}, instructions {}",
        pointer,
        instructions.len()
    );
    let mut d = Device::new(6, pointer, instructions.clone());

    let mut steps = 0;
    while d.apply() {
        steps += 1;
        // println!("{} Pointer {}, {:?}", steps, d.pointer, d.register.values);
    }

    println!("Finished after {} steps: {:?}", steps, d.register.values);

    let n = 10_551_311;
    println!("Factors of {}: {:?}", n, primes::factors(n));

    let mut d2 = Device::new(6, pointer, instructions.clone());

    // d2.register.values[0] = 1;

    // d2.register.values = vec![0, 10_551_311, 1, 0, 7, 10_551_310];
    // d2.pointer = 8;

    // d2.register.values = vec![1, 10_551_311, 2, 0, 7, 10_551_309];
    // d2.pointer = 8;

    // d2.register.values = vec![1, 10_551_311, 431, 0, 7, 24480];
    // d2.pointer = 8;

    // d2.register.values = vec![432, 10_551_311, 24481, 0, 7, 430];
    // d2.pointer = 8;

    // d2.register.values = vec![24913, 10_551_311, 10_551_310, 0, 7, 10_551_310];
    // d2.pointer = 8;

    // It sums the factors - in this case, 1 + 431 + 24481 + 10551311
    d2.register.values = vec![10_576_224, 10_551_311, 10_551_311, 0, 7, 10_551_310];
    d2.pointer = 8;

    let mut steps = 0;

    while d2.apply() {
        steps += 1;
        println!(
            "{} Pointer {} ({:?}), {:?}",
            steps,
            d2.pointer,
            instructions.get(d2.pointer),
            d2.register.values
        );

        if steps > 100 {
            break;
        }
    }

    println!("Finished after {} steps: {:?}", steps, d2.register.values);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r#"
        #ip 0
        seti 5 0 1
        seti 6 0 2
        addi 0 1 0
        addr 1 2 3
        setr 1 0 0
        seti 8 0 4
        seti 9 0 5
    "#;

    fn get_test_device() -> Result<Device, failure::Error> {
        let lines: Vec<&str> = EXAMPLE_INPUT.split('\n').collect();
        let (pointer, instructions) = parse_instructions(lines)?;
        Ok(Device::new(6, pointer, instructions))
    }

    #[test]
    fn test_apply() {
        let mut dev = get_test_device().unwrap();

        assert_eq!(dev.register.values, vec![0, 0, 0, 0, 0, 0]);
        let applied = dev.apply();
        assert!(applied);
        assert_eq!(dev.register.values, vec![0, 5, 0, 0, 0, 0]);
        assert_eq!(dev.pointer, 1);
        let applied = dev.apply();
        assert!(applied);
        assert_eq!(dev.register.values, vec![1, 5, 6, 0, 0, 0]);
        assert_eq!(dev.pointer, 2);
        let applied = dev.apply();
        assert!(applied);
        assert_eq!(dev.register.values, vec![3, 5, 6, 0, 0, 0]);
        assert_eq!(dev.pointer, 4);
        let applied = dev.apply();
        assert!(applied);
        assert_eq!(dev.register.values, vec![5, 5, 6, 0, 0, 0]);
        assert_eq!(dev.pointer, 6);
        let applied = dev.apply();
        assert!(applied);
        assert_eq!(dev.register.values, vec![6, 5, 6, 0, 0, 9]);
        assert_eq!(dev.pointer, 7);
    }
}
