#![warn(clippy::all)]

use clap::{App, Arg};

use text_io::try_scan;

pub type Val = i64;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec(Val, Val, Val, Val);

impl std::ops::Sub for Vec {
    type Output = Self;

    fn sub(self: Self, rhs: Self) -> Self {
        Vec(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl Vec {
    pub fn manhattan(self) -> Val {
        self.0.abs() + self.1.abs() + self.2.abs() + self.3.abs()
    }

    pub fn parse_line(line: &str) -> Result<Self, failure::Error> {
        let (x, y, z, t): (i64, i64, i64, i64);
        try_scan!(line.bytes() => "({},{},{},{})", x,y,z,t);
        Ok(Vec(x, y, z, t))
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 25")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day25.txt");

    eprintln!("Using input {}", input_path);

    Ok(())
}
