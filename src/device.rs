use text_io::try_scan;

pub type Value = i64;

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
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "addr" => Some(OpCode::AddR),
            "addi" => Some(OpCode::AddI),
            "mulr" => Some(OpCode::MulR),
            "muli" => Some(OpCode::MulI),
            "banr" => Some(OpCode::BanR),
            "bani" => Some(OpCode::BanI),
            "borr" => Some(OpCode::BorR),
            "bori" => Some(OpCode::BorI),
            "setr" => Some(OpCode::SetR),
            "seti" => Some(OpCode::SetI),
            "gtir" => Some(OpCode::GtIR),
            "gtri" => Some(OpCode::GtRI),
            "gtrr" => Some(OpCode::GtRR),
            "eqir" => Some(OpCode::EqIR),
            "eqri" => Some(OpCode::EqRI),
            "eqrr" => Some(OpCode::EqRR),
            _ => None,
        }
    }

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
pub struct Instruction(pub OpCode, pub usize, pub usize, pub usize);

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Register {
    pub values: Vec<Value>,
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
