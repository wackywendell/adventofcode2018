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
    left: i64,
    right: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Edge {
    Wall,
    FreeFall,
    Water,
}

impl Walls {
    fn parse_lines<I, S>(lines: I) -> Result<Walls, failure::Error>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let some_walls: Result<Vec<Wall>, failure::Error> = lines
            .into_iter()
            .filter_map(|l| {
                let trimmed = l.as_ref().trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Wall::parse_line(trimmed))
                }
            })
            .collect();
        let wall_vec: Vec<Wall> = some_walls?;

        let mut filled = HashSet::new();
        let (mut top, mut bottom) = (None, None);
        let (mut left, mut right) = (None, None);
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
                left = Some(match left {
                    None => px,
                    Some(leftx) => std::cmp::min(leftx, px),
                });
                right = Some(match right {
                    None => px,
                    Some(rightx) => std::cmp::max(rightx, px),
                });
                filled.insert((px, py));
            }
        }

        Ok(Walls {
            filled,
            top: top.unwrap(),
            bottom: bottom.unwrap(),
            left: left.unwrap(),
            right: right.unwrap(),
        })
    }

    fn to_bytes(&self) -> Vec<Vec<u8>> {
        let s: Vec<u8> = std::iter::repeat(b'.')
            .take(((self.right + 1) - (self.left - 1) + 1) as usize)
            .collect();

        let mut lines: Vec<Vec<u8>> = std::iter::repeat(s)
            .take((self.bottom + 1) as usize)
            .collect();

        for &(x, y) in &self.filled {
            let rel_y = y as usize;
            let rel_x = (x - self.left + 1) as usize;
            lines[rel_y][rel_x] = b'#';
        }

        lines
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
    seen: HashSet<(i64, i64)>,
}

pub struct Progress {
    pub bottom: i64,
    pub lowest: i64,
    pub waters: Vec<(i64, i64)>,
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
            seen: Default::default(),
        }
    }

    pub fn progress(&self) -> Progress {
        let mut waters: Vec<(i64, i64)> = self.queue.iter().copied().collect();
        waters.sort_by_key(|&(x, y)| (y, x));
        let lowest = self
            .water
            .keys()
            .fold(0, |old, &(_, new)| std::cmp::max(old, new));
        Progress {
            bottom: self.walls.bottom,
            lowest,
            waters,
        }
    }

    fn find_bottom(&self, x: i64, y: i64) -> Option<(Edge, i64)> {
        for cy in y + 1..=self.walls.bottom {
            if self.water.get(&(x, cy)) == Some(&Water::Stable) {
                return Some((Edge::Water, cy - 1));
            }
            if self.walls.filled.contains(&(x, cy)) {
                return Some((Edge::Wall, cy - 1));
            }
        }

        None
    }

    fn find_sides(&self, x: i64, y: i64) -> ((Edge, i64), (Edge, i64)) {
        let left: (Edge, i64);
        let right: (Edge, i64);

        let mut cx = x;
        loop {
            let below = (cx, y + 1);
            if self.water.get(&below) != Some(&Water::Stable) && !self.walls.filled.contains(&below)
            {
                // No stable water or wall (floor) underneath, so its a freefall edge
                left = (Edge::FreeFall, cx);
                break;
            }
            if self.walls.filled.contains(&(cx - 1, y)) {
                left = (Edge::Wall, cx);
                break;
            }
            cx -= 1;
        }

        cx = x;
        loop {
            let below = (cx, y + 1);
            if self.water.get(&below) != Some(&Water::Stable) && !self.walls.filled.contains(&below)
            {
                right = (Edge::FreeFall, cx);
                break;
            }
            if self.walls.filled.contains(&(cx + 1, y)) {
                right = (Edge::Wall, cx);
                break;
            }
            cx += 1;
        }

        (left, right)
    }

    fn step(&mut self) -> bool {
        let (x, y) = match self.queue.pop_front() {
            Some(v) => v,
            None => return false,
        };

        let (bottom_type, bottom) = match self.find_bottom(x, y) {
            None => {
                // println!(
                //     "No bottom found, inserting water from ({}, {}) to ({}, {})",
                //     x,
                //     y + 1,
                //     x,
                //     self.walls.bottom,
                // );
                for cy in (y + 1..=self.walls.bottom).rev() {
                    self.water.insert((x, cy), Water::Flowing);
                }
                return true;
            }
            Some(b) => b,
        };

        if bottom_type == Edge::Water && bottom > y && self.seen.contains(&(x, y)) {
            println!("Seen ({}, {})", x, y);
            return true;
        }

        self.seen.insert((x, y));

        // println!("Bottom found: ({}, {}) -> ({}, {})", x, y, x, bottom);

        for cy in (y + 1..=bottom).rev() {
            self.water.insert((x, cy), Water::Flowing);
        }

        let sides = self.find_sides(x, bottom);
        if let ((Edge::Wall, lx), (Edge::Wall, rx)) = sides {
            // println!("       found double wall");
            for sx in (lx..=rx).rev() {
                self.water.insert((sx, bottom), Water::Stable);
            }
            self.queue.push_back((x, bottom - 1));
            return true;
        }

        let ((left_edge, lx), (right_edge, rx)) = sides;
        // println!(
        //     "       found: {:?}: {}, {:?}: {}",
        //     left_edge, lx, right_edge, rx
        // );
        for sx in lx..=rx {
            self.water.insert((sx, bottom), Water::Flowing);
        }

        match left_edge {
            Edge::Wall => {}
            Edge::FreeFall => {
                // println!("Pushing left edge ({}, {})", lx, bottom);
                self.queue.push_back((lx, bottom));
            }
            Edge::Water => panic!("This shouldn't happen"),
        }

        if (left_edge, lx) == (right_edge, rx) {
            // TODO: Panic?
            return true;
        }

        match right_edge {
            Edge::Wall => {}
            Edge::FreeFall => {
                self.queue.push_back((rx, bottom));
            }
            Edge::Water => panic!("This shouldn't happen"),
        }

        true
    }

    fn to_bytes(&self) -> Vec<Vec<u8>> {
        let mut lines = self.walls.to_bytes();

        for (&(x, y), water) in &self.water {
            assert!(x >= self.walls.left - 1);
            assert!(x <= self.walls.right + 1);
            assert!(y >= 0, "{} >= {}", y, self.walls.top);
            let rel_y = y as usize;
            let rel_x = (x - self.walls.left + 1) as usize;

            let c: char = match water {
                Water::Flowing => '|',
                Water::Stable => '~',
            };
            lines[rel_y][rel_x] = c as u8;
        }

        lines
    }

    fn print(&self) {
        for l in self.to_bytes() {
            let s = String::from_utf8(l).unwrap();
            println!("{}", s);
        }
    }

    /// water_count returns a count of (stable, flowing) water squares
    fn water_count(&self) -> (i64, i64) {
        let (mut stable, mut flowing) = (0, 0);

        for (&(_, y), water) in &self.water {
            if y < self.walls.top {
                // These aren't counted
                continue;
            }
            match water {
                Water::Flowing => flowing += 1,
                Water::Stable => stable += 1,
            }
        }

        (stable, flowing)
    }
}

fn print_progress(step: i64, progress: Progress) {
    println!(
        "-- Flowed {}: lowest {}, high flow {}, low flow {}, bottom {}, {} remaining",
        step,
        progress.lowest,
        progress.waters.first().unwrap_or(&(0, 0)).0,
        progress.waters.last().unwrap_or(&(0, 0)).0,
        progress.bottom,
        progress.waters.len(),
    );
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

    let mut flow = FlowingWater::new(walls, (500, 0));
    print_progress(0, flow.progress());
    flow.print();

    let mut steps = 0;
    while flow.step() {
        steps += 1;
        if steps % 100 == 0 {
            print_progress(steps, flow.progress());
            if steps % 10_000 == 0 {
                flow.print();
            }
        }

        if steps > 100_000 {
            break;
        }
    }

    flow.print();
    let (s, f) = flow.water_count();
    println!("Finished after {} steps.", steps);
    println!("{} stable + {} flowing = {} water squares", s, f, s + f);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#"
x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504"#;

    fn get_test_walls(s: &str) -> Result<Walls, failure::Error> {
        let lines: Vec<&str> = s.split('\n').collect();
        Walls::parse_lines(lines)
    }

    #[test]
    fn test_parse() {
        println!("Getting test input...");
        let maybe_walls = get_test_walls(TEST_INPUT);
        let walls = match maybe_walls {
            Err(e) => {
                println!("Error: {}", e);
                panic!("Error getting test input: {}", e);
            }
            Ok(w) => w,
        };
        println!("Creating flow...");
        let flow = FlowingWater::new(walls, (500, 0));
        println!("Start:");
        flow.print();
    }

    #[test]
    fn test_run() {
        println!("Getting test input...");
        let walls = get_test_walls(TEST_INPUT).unwrap();
        println!("Creating flow...");
        let mut flow = FlowingWater::new(walls, (500, 0));

        println!("Start:");
        flow.print();
        let mut i = 0;
        while flow.step() {
            println!();
            println!();
            flow.print();

            i += 1;
            if i > 12 {
                panic!("Didn't finish in 10 steps");
            }
        }

        let (s, f) = flow.water_count();

        assert_eq!(28, f);
        assert_eq!(29, s);
    }
}
