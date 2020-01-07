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
