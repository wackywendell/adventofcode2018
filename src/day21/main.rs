#![warn(clippy::all)]

use aoc::device::{parse_instructions, Device};

use clap::{App, Arg};

use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn device_emulator(x: &mut [i64]) -> Vec<i64> {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();

    x[1] = x[3] | 65536;
    x[3] = 14_906_355;

    'a: loop {
        x[4] = x[1] & 255;
        x[3] = (((x[3] + (x[1] & 255)) & 16_777_215) * 65899) & 16_777_215;

        if 256 > x[1] {
            // println!("outer loop: {}, {}", x[1], x[3]);
            ordered.push(x[3]);
            if !seen.insert((x[1], x[3])) {
                // println!("Already seen x[3]!");
                // println!("Values seen: {:?}", ordered);
                return ordered;
            }

            if x[3] == x[0] {
                return ordered;
            }
            x[1] = x[3] | 65536;
            x[3] = 14_906_355;
            continue;
        } else {
            x[4] = 0;
        }
        loop {
            x[2] = if (x[4] + 1) * 256 > x[1] { 1 } else { 0 };
            if x[2] != 0 {
                x[1] = x[4];
                continue 'a;
            }
            x[4] += 1;
        }
    }
}

fn find_last(values: Vec<i64>) -> i64 {
    println!("------------------------------");
    if values.is_empty() {
        return 0;
    }
    let mut seen = HashSet::new();
    let mut last = values[0];

    for (i, &v) in values.iter().enumerate() {
        if !seen.insert(v) {
            // already seen
            continue;
        }

        last = v;
        if i > 100 {
            continue;
        }

        println!("{}: {}", i, v);
    }

    println!("------------------------------");
    println!("Last: {}", last);
    println!("------------------------------");

    last
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 21")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day21.txt");

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
    d.register.values[0] = 0;
    // Register 3 will have this at some point
    // d.register.values[0] = 3_173_684;
    // d.register.values[0] = 1_566_402;
    d.register.values[0] = 13_202_558;

    let mut emulated = d.register.values.clone();
    let values = device_emulator(emulated.as_mut_slice());
    find_last(values);

    let max_steps = 1_000_000;
    // let last_n = 100;

    let mut steps = 0;
    let mut pointer = d.pointer;
    let mut instruction = instructions.get(d.pointer);
    let mut values = d.register.values.clone();

    while d.apply() {
        // if steps < last_n
        //     || steps + last_n >= max_steps
        //     || pointer >= 26
        //     || d.pointer >= 26
        //     || pointer < 18
        //     || d.pointer < 18
        if pointer == 28 {
            println!(
                "{} Pointer {} -> {} ({:?}), {:?} -> {:?}",
                steps, pointer, d.pointer, instruction, values, d.register.values,
            );
        }
        pointer = d.pointer;
        instruction = instructions.get(d.pointer);
        values = d.register.values.clone();
        steps += 1;

        if steps >= max_steps {
            println!("Breaking after {} steps: {:?}", steps, d.register.values);
            break;
        }
    }

    println!("Finished after {} steps: {:?}", steps, d.register.values);

    Ok(())
}
