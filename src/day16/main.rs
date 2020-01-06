#![warn(clippy::all)]

use clap::{App, Arg};
use text_io::try_scan;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

type Value = i64;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum OpCode {
    AddR,
    AddI,
    MulR,
    MulI,
    BanR,
    BanI,
    BorR,
    BorI,
    SetR,
    SetI,
    GtIR,
    GtRI,
    GtRR,
    EqIR,
    EqRI,
    EqRR,
}

impl OpCode {
    pub fn variants() -> impl IntoIterator<Item = Self> {
        vec![
            OpCode::AddR,
            OpCode::AddI,
            OpCode::MulR,
            OpCode::MulI,
            OpCode::BanR,
            OpCode::BanI,
            OpCode::BorR,
            OpCode::BorI,
            OpCode::SetR,
            OpCode::SetI,
            OpCode::GtIR,
            OpCode::GtRI,
            OpCode::GtRR,
            OpCode::EqIR,
            OpCode::EqRI,
            OpCode::EqRR,
        ]
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Instruction(OpCode, usize, usize, usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct PartialInstruction(usize, usize, usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct UnknownInstruction(usize, usize, usize, usize);

impl UnknownInstruction {
    pub fn partial(self) -> PartialInstruction {
        let UnknownInstruction(_, a, b, c) = self;
        PartialInstruction(a, b, c)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Register {
    values: Vec<Value>,
}

impl Register {
    pub fn apply(&mut self, instr: Instruction) -> Value {
        let Instruction(op, a, b, c) = instr;

        fn int_bool(b: bool) -> Value {
            if b {
                1
            } else {
                0
            }
        }

        let out_value = match op {
            OpCode::AddR => self.values[a] + self.values[b],
            OpCode::AddI => self.values[a] + b as Value,
            OpCode::MulR => self.values[a] * self.values[b],
            OpCode::MulI => self.values[a] * b as Value,
            OpCode::BanR => self.values[a] & self.values[b],
            OpCode::BanI => self.values[a] & b as Value,
            OpCode::BorR => self.values[a] | self.values[b],
            OpCode::BorI => self.values[a] | b as Value,
            OpCode::SetR => self.values[a],
            OpCode::SetI => a as Value,
            OpCode::GtIR => int_bool(a as i64 > self.values[b]),
            OpCode::GtRI => int_bool(self.values[a] > b as Value),
            OpCode::GtRR => int_bool(self.values[a] > self.values[b]),
            OpCode::EqIR => int_bool(a as i64 == self.values[b]),
            OpCode::EqRI => int_bool(self.values[a] == b as Value),
            OpCode::EqRR => int_bool(self.values[a] == self.values[b]),
        };

        self.values[c] = out_value;
        out_value
    }
}

pub fn matching_registers(
    input: &Register,
    output: &Register,
    instr: PartialInstruction,
) -> Vec<OpCode> {
    let mut matching = Vec::new();
    let PartialInstruction(a, b, c) = instr;

    for oc in OpCode::variants() {
        let mut rs = input.clone();
        rs.apply(Instruction(oc, a, b, c));
        if &rs == output {
            matching.push(oc);
        }
    }

    matching
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
struct Triplet(Register, UnknownInstruction, Register);

impl Triplet {
    pub fn matching_codes(&self) -> Vec<OpCode> {
        let Triplet(input, instr, output) = self;
        matching_registers(input, output, instr.partial())
    }
}

#[allow(clippy::cognitive_complexity)]
fn parse_triplet(lines: &mut VecDeque<String>) -> Result<Triplet, failure::Error> {
    let l0 = match lines.front() {
        None => {
            return Err(failure::format_err!("No more left"));
        }
        Some(l) => l,
    };

    let (a, b, c, d): (Value, Value, Value, Value);
    try_scan!(l0.bytes() => "Before: [{}, {}, {}, {}]", a,b,c,d);
    let r1 = Register {
        values: vec![a, b, c, d],
    };

    let _ = lines.pop_front();
    let l1 = lines.pop_front().unwrap();
    let (op, a1, b1, c1): (usize, usize, usize, usize);
    try_scan!(l1.bytes() => "{} {} {} {}", op,a1,b1,c1);
    let instr = UnknownInstruction(op, a1, b1, c1);
    let l2 = lines.pop_front().unwrap();
    let (a2, b2, c2, d2): (Value, Value, Value, Value);
    try_scan!(l2.bytes() => "After:  [{}, {}, {}, {}]", a2,b2,c2,d2);
    let r2 = Register {
        values: vec![a2, b2, c2, d2],
    };
    let _ = lines.pop_front().unwrap();

    Ok(Triplet(r1, instr, r2))
}

struct CodeMap(HashMap<usize, OpCode>);

impl CodeMap {
    pub fn resolve(&self, instr: UnknownInstruction) -> Instruction {
        let UnknownInstruction(code, a, b, c) = instr;
        let op = self.0[&code];
        Instruction(op, a, b, c)
    }
}

fn resolve<T: IntoIterator<Item = Triplet>>(triplets: T) -> CodeMap {
    let mut partially_resolved: HashMap<usize, HashSet<OpCode>> = HashMap::new();

    for t in triplets {
        let Triplet(_, UnknownInstruction(code, _, _, _), _) = t;
        let codes: HashSet<OpCode> = t.matching_codes().into_iter().collect();
        match partially_resolved.entry(code) {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert(codes);
            }
            std::collections::hash_map::Entry::Occupied(mut e) => {
                let code_set = e.get_mut();
                // if code == 5 {
                //     let Triplet(r1, instr, r2) = t;
                //     println!("{:?} - {:?} - {:?}", r1, instr, r2);
                //     println!("Merging {}: {:?} and {:?}", code, code_set, codes);
                // }
                code_set.retain(|v| codes.contains(v));
            }
        }
    }

    let mut resolved: HashMap<usize, OpCode> = HashMap::new();
    let mut set: HashSet<OpCode> = HashSet::new();
    let mut sets = 1;
    while sets > 0 {
        sets = 0;
        let mut seen_counts: HashMap<OpCode, HashSet<usize>> = HashMap::new();
        for (&c, ops) in partially_resolved.iter() {
            if resolved.contains_key(&c) {
                continue;
            }
            for &op in ops.iter() {
                let v = seen_counts.entry(op).or_default();
                v.insert(c);
            }

            let mut ops = ops.clone();
            ops.retain(|c| !set.contains(c));

            if ops.len() != 1 {
                // println!("Found multiple possibilities for {}: {:?}", c, ops);
                continue;
            }

            let op = ops.into_iter().next().unwrap();
            resolved.insert(c, op);
            set.insert(op);
            sets += 1;
        }

        for (op, codes) in seen_counts {
            if codes.len() == 1 && !set.contains(&op) {
                let code = codes.into_iter().next().unwrap();
                resolved.insert(code, op);
                set.insert(op);
                sets += 1;
            }
        }
    }

    CodeMap(resolved)
}

fn parse_instructions(lines: &mut VecDeque<String>) -> Result<UnknownInstruction, failure::Error> {
    while lines.front().map(|s| s.trim()) == Some("") {
        lines.pop_front();
    }

    let l = lines
        .pop_front()
        .ok_or_else(|| failure::format_err!("No more left"))?;

    let (op, a, b, c): (usize, usize, usize, usize);
    try_scan!(l.bytes() => "{} {} {} {}", op,a,b,c);
    Ok(UnknownInstruction(op, a, b, c))
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 16")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day16.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let some_lines: std::io::Result<VecDeque<String>> = buf_reader.lines().collect();
    let mut lines: VecDeque<String> = some_lines?;

    let mut triplets: Vec<Triplet> = Vec::new();
    while let Ok(t) = parse_triplet(&mut lines) {
        triplets.push(t);
    }

    let mut instructions: Vec<UnknownInstruction> = Vec::new();
    while let Ok(i) = parse_instructions(&mut lines) {
        instructions.push(i);
    }

    let count = triplets.len();
    let three_or_more = triplets
        .iter()
        .filter(|&t| t.matching_codes().len() >= 3)
        .count();
    println!("three-or-more: {} / {}", three_or_more, count);
    let code_map = resolve(triplets);
    println!(
        "Resolved {} codes, and {} instructions",
        code_map.0.len(),
        instructions.len()
    );

    let mut r = Register {
        values: vec![0, 0, 0, 0],
    };
    for unknown in instructions {
        let instr = code_map.resolve(unknown);
        r.apply(instr);
    }

    println!("Registers: {:?}", r.values);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instructions() {
        let input = Register {
            values: vec![3, 2, 1, 1],
        };
        let output = Register {
            values: vec![3, 2, 2, 1],
        };

        let ops = matching_registers(&input, &output, PartialInstruction(2, 1, 2));

        assert_eq!(ops, vec![OpCode::AddI, OpCode::MulR, OpCode::SetI]);
    }

    #[test]
    fn test_eq() {
        let mut reg = Register {
            values: vec![1, 0, 1, 3],
        };
        reg.apply(Instruction(OpCode::EqRR, 2, 3, 3));

        assert_eq!(reg.values, [1, 0, 1, 0]);
    }
}
