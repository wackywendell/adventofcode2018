#![warn(clippy::all)]

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use clap::{App, Arg};

use text_io::try_scan;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Point(i64, i64, i64);

impl Point {
    pub fn distance(self, other: Self) -> i64 {
        let dx = (self.0 - other.0).abs();
        let dy = (self.1 - other.1).abs();
        let dz = (self.2 - other.2).abs();

        dx + dy + dz
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanobot {
    signal: i64,
    loc: Point,
}

impl Nanobot {
    pub fn in_range(&self, other: &Self) -> bool {
        let dist = self.loc.distance(other.loc);

        dist <= self.signal
    }

    pub fn parse_line(line: &str) -> Result<Self, failure::Error> {
        let (x, y, z, signal): (i64, i64, i64, i64);
        try_scan!(line.bytes() => "pos=<{},{},{}>, r={}", x,y,z,signal);
        Ok(Nanobot {
            loc: Point(x, y, z),
            signal,
        })
    }

    pub fn parse_lines<I, S>(lines: I) -> Result<Vec<Self>, failure::Error>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        lines
            .into_iter()
            .filter_map(|l| {
                let trimmed = l.as_ref().trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Self::parse_line(trimmed))
                }
            })
            .collect()
    }
}

// strongest_range finds the nanobot with the strongest signal, and calculates
// what the
pub fn strongest_range(bots: &[Nanobot]) -> Option<(Nanobot, isize)> {
    if bots.is_empty() {
        return None;
    }

    let strongest = bots.iter().max_by_key(|b| b.signal)?;

    let in_range: isize = bots
        .iter()
        .filter(|b| strongest.in_range(b))
        .map(|_| 1)
        .sum();

    Some((strongest.clone(), in_range))
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 23")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day23.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let bots = Nanobot::parse_lines(buf_reader.lines())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#"
        pos=<0,0,0>, r=4
        pos=<1,0,0>, r=1
        pos=<4,0,0>, r=3
        pos=<0,2,0>, r=1
        pos=<0,5,0>, r=3
        pos=<0,0,3>, r=1
        pos=<1,1,1>, r=1
        pos=<1,1,2>, r=1
        pos=<1,3,1>, r=1
        "#;

    fn get_test_bots(s: &str) -> Result<Vec<Nanobot>, failure::Error> {
        let lines: Vec<&str> = s.split('\n').collect();
        Nanobot::parse_lines(lines)
    }

    #[test]
    fn test_parse() {
        let bots = get_test_bots(TEST_INPUT).unwrap();
        assert_eq!(bots.len(), 9);

        let (strongest, in_range) = strongest_range(&bots).unwrap();

        assert_eq!(
            strongest,
            Nanobot {
                loc: Point(0, 0, 0),
                signal: 4
            }
        );

        assert_eq!(in_range, 7);
    }
}
