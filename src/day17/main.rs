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
    filled: HashSet<(i64, i64)>,
    top: i64,
    bottom: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Edge {
    Wall,
    FreeFall,
}

impl Walls {
    fn parse_lines(lines: &mut VecDeque<String>) -> Result<Walls, failure::Error> {
        let some_walls: Result<Vec<Wall>, failure::Error> =
            lines.drain(..).map(|l| Wall::parse_line(&l)).collect();
        let wall_vec: Vec<Wall> = some_walls?;

        let mut filled = HashSet::new();
        let (mut top, mut bottom) = (None, None);
        for wall in wall_vec {
            for second in wall.range {
                let (px, py) = match wall.direction {
                    Direction::Horizontal => (wall.loc, second),
                    Direction::Vertical => (second, wall.loc),
                };
                top = Some(match top {
                    None => py,
                    Some(topx) => std::cmp::min(topx, py),
                });
                bottom = Some(match bottom {
                    None => py,
                    Some(bottomx) => std::cmp::max(bottomx, py),
                });
                filled.insert((px, py));
            }
        }

        Ok(Walls {
            filled,
            top: top.unwrap(),
            bottom: bottom.unwrap(),
        })
    }

    fn find_bottom(&self, x: i64, y: i64) -> Option<i64> {
        for cy in y + 1..=self.bottom {
            if self.filled.contains(&(x, cy)) {
                return Some(y - 1);
            }
        }

        None
    }

    fn find_sides(&self, x: i64, y: i64) -> ((Edge, i64), (Edge, i64)) {
        let left: (Edge, i64);
        let right: (Edge, i64);

        let mut cx = x;
        loop {
            if !self.filled.contains(&(cx, y + 1)) {
                left = (Edge::FreeFall, cx);
                break;
            }
            if self.filled.contains(&(cx - 1, y)) {
                left = (Edge::Wall, cx);
                break;
            }
            cx -= 1;
        }

        cx = x;
        loop {
            if !self.filled.contains(&(cx, y + 1)) {
                right = (Edge::FreeFall, cx);
                break;
            }
            if self.filled.contains(&(cx + 1, y)) {
                right = (Edge::Wall, cx);
                break;
            }
            cx += 1;
        }

        (left, right)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Water {
    Flowing,
    Stable,
}

pub struct FlowingWater {
    water: HashMap<(i64, i64), Water>,
    walls: Walls,
    queue: VecDeque<(i64, i64)>,
}

impl FlowingWater {
    fn new(walls: Walls, start: (i64, i64)) -> Self {
        let mut water: HashMap<(i64, i64), Water> = HashMap::new();
        if start.0 >= walls.top {
            water.insert(start, Water::Flowing);
        }
        let mut queue = VecDeque::new();
        queue.push_back(start);

        FlowingWater {
            water,
            walls,
            queue,
        }
    }

    fn step(&mut self) -> bool {
        let (x, y) = match self.queue.pop_front() {
            Some(v) => v,
            None => return false,
        };

        let bottom = match self.walls.find_bottom(x, y) {
            None => {
                for cy in (y + 1..=self.walls.bottom).rev() {
                    self.water.insert((x, cy), Water::Flowing);
                }
                return true;
            }
            Some(b) => b,
        };

        return true;
    }
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
