#![warn(clippy::all)]

use clap::{App, Arg};
use text_io::try_scan;

use core::ops::RangeInclusive;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

enum Direction {
    Vertical,
    Horizontal,
}

struct Wall {
    direction: Direction,
    loc: i64,
    range: RangeInclusive<i64>,
}

impl Wall {
    fn parse_line(line: &str) -> Result<Wall, failure::Error> {
        let (dir, loc, dir2, start, end): (String, i64, String, i64, i64);
        try_scan!(line.bytes() => "{}={}, {}={}..{}", dir, loc, dir2, start, end);

        let direction = if dir == "x" {
            Direction::Horizontal
        } else {
            Direction::Vertical
        };

        Ok(Wall {
            direction,
            loc,
            range: start..=end,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Walls {
    verticals: HashMap<i64, Vec<RangeInclusive<i64>>>,
    horizontals: HashMap<i64, Vec<RangeInclusive<i64>>>,
}

impl Walls {
    fn parse_lines(lines: &mut VecDeque<String>) -> Result<Walls, failure::Error> {
        let some_walls: Result<Vec<Wall>, failure::Error> =
            lines.drain(..).map(|l| Wall::parse_line(&l)).collect();
        let wall_vec: Vec<Wall> = some_walls?;

        let mut walls: Walls = Default::default();
        for wall in wall_vec {
            let e = match wall.direction {
                Direction::Horizontal => walls.horizontals.entry(wall.loc),
                Direction::Vertical => walls.verticals.entry(wall.loc),
            };
            let v = e.or_default();
            v.push(wall.range);
            v.sort_unstable_by_key(|r| (*r.start(), *r.end()));
        }

        Ok(walls)
    }

    fn find_bottom(&self, x: i64, y: i64) -> Option<Wall> {}
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 17")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day17.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let some_lines: std::io::Result<VecDeque<String>> = buf_reader.lines().collect();
    let mut lines: VecDeque<String> = some_lines?;
    let walls = Walls::parse_lines(&mut lines)?;

    Ok(())
}
