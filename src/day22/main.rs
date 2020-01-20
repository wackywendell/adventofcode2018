#![warn(clippy::all)]

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use clap::{App, Arg};

const MODULUS: i64 = 20_183;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum Erosion {
    Rocky,
    Wet,
    Narrow,
}

impl Into<i64> for Erosion {
    fn into(self) -> i64 {
        match self {
            Erosion::Rocky => 0,
            Erosion::Wet => 1,
            Erosion::Narrow => 2,
        }
    }
}

impl Into<char> for Erosion {
    fn into(self) -> char {
        match self {
            Erosion::Rocky => '.',
            Erosion::Wet => '=',
            Erosion::Narrow => '|',
        }
    }
}

pub type Point = (i64, i64);

pub struct Cave {
    depth: i64,
    target: Point,

    geologies: Vec<Vec<i64>>,
}

impl Cave {
    pub fn new(depth: i64, target: Point) -> Cave {
        Cave {
            depth,
            target,
            geologies: Vec::new(),
        }
    }

    fn erosion_level(&mut self, x: i64, y: i64) -> i64 {
        // println!("erosion({}, {}); {}", x, y, self.geologies.len());
        // let rl = self.geologies[x as usize].len();
        // println!("erosion({}, {}); {}, {}", x, y, self.geologies.len(), rl);
        // let g = self.geologies[x as usize][y as usize];
        let g = self.geology(x, y);
        (g + self.depth) % MODULUS
    }

    fn unsafe_erosion_level(&mut self, x: i64, y: i64) -> i64 {
        // println!("erosion({}, {}); {}", x, y, self.geologies.len());
        // let rl = self.geologies[x as usize].len();
        // println!("erosion({}, {}); {}, {}", x, y, self.geologies.len(), rl);
        let g = self.geologies[x as usize][y as usize];
        // let g = self.geology(x, y);
        (g + self.depth) % MODULUS
    }

    pub fn erosion(&mut self, x: i64, y: i64) -> Erosion {
        match self.erosion_level(x, y) % 3 {
            0 => Erosion::Rocky,
            1 => Erosion::Wet,
            2 => Erosion::Narrow,
            _ => unreachable!(),
        }
    }

    fn geology_from_previous(&mut self, x: i64, y: i64) -> i64 {
        // eprintln!("Calling geology_from_previous({}, {})", x, y);
        if (x, y) == self.target {
            return 0;
        }
        if x == 0 {
            return ((y % MODULUS) * (48271 % MODULUS)) % MODULUS;
        } else if y == 0 {
            return ((x % MODULUS) * 16807) % MODULUS;
        }

        let e1: i64 = self.unsafe_erosion_level(x - 1, y);
        let e2: i64 = self.unsafe_erosion_level(x, y - 1);

        (e1 * e2) % MODULUS
    }

    fn geology(&mut self, target_x: i64, target_y: i64) -> i64 {
        let xlen = self.geologies.len();
        let ylen = self.geologies.get(0).map(|v| v.len()).unwrap_or(0);

        if xlen > target_x as usize && ylen > target_y as usize {
            return self.geologies[target_x as usize][target_y as usize];
        }

        if (target_x, target_y) == self.target {
            return 0;
        }

        if target_x <= 0 || target_y <= 0 {
            return self.geology_from_previous(target_x, target_y);
        }

        eprintln!("Calling geology({}, {})", target_x, target_y);

        // Fill existing rows out to target_y
        if (ylen as i64) < target_y + 1 {
            for x in 0..xlen as i64 {
                // eprintln!("Filling row {} from {}..={}", x, ylen, target_y);
                for y in (ylen as i64)..=target_y {
                    let value = self.geology_from_previous(x, y);
                    // println!(
                    //     "Adding value at ({}, {})",
                    //     x,
                    //     self.geologies[x as usize].len()
                    // );
                    self.geologies[x as usize].push(value);
                }
            }
        }

        let ylen2: i64 = std::cmp::max(ylen as i64, target_y + 1);

        // Fill the rest of the rows
        for x in (xlen as i64)..=target_x {
            // eprintln!("Filling new row {} from {}..{}", x, 0, ylen2);
            self.geologies.push(Vec::with_capacity(ylen + 1));
            for y in 0..ylen2 {
                // println!("Adding value at ({}, {})", x, y);
                let value = self.geology_from_previous(x, y);
                self.geologies[x as usize].push(value);
            }
            // println!(
            //     "Adding row {} ({})",
            //     self.geologies.len(),
            //     self.geologies[x as usize].len(),
            // );
        }

        // println!(
        //     "Expanded: ({}, {}) => ({}, {}) from ({}, {})",
        //     xlen,
        //     ylen,
        //     self.geologies.len(),
        //     self.geologies[0].len(),
        //     target_x,
        //     target_y,
        // );

        // for (i, row) in self.geologies.iter().enumerate() {
        //     println!("Row {}: Length {}", i, row.len());
        // }

        self.geologies[target_x as usize][target_y as usize]
    }

    pub fn risk(&mut self) -> i64 {
        let mut sum = 0;
        let (target_x, target_y) = self.target;

        for x in 0..=target_x {
            for y in 0..=target_y {
                let e = self.erosion(x, y);
                let risk: i64 = e.into();
                sum += risk;
            }
        }

        sum
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum Tool {
    Torch,
    ClimbingGear,
    Neither,
}

fn tools(erosion: Erosion) -> [Tool; 2] {
    match erosion {
        Erosion::Rocky => [Tool::Torch, Tool::ClimbingGear],
        Erosion::Wet => [Tool::ClimbingGear, Tool::Neither],
        Erosion::Narrow => [Tool::Torch, Tool::Neither],
    }
}

pub type Time = i64;

pub struct Routes {
    target: Point,
    // (location, tool in hand) -> (time taken, previous, previous tool)
    seen: HashMap<(Point, Tool), (Time, Point, Tool)>,
    // Time is expected arrival time
    queue: BinaryHeap<(Reverse<Time>, Point, Tool)>,
    fastest: Option<Time>,
}

impl Routes {
    fn heuristic(&self, point: Point, tool: Tool) -> i64 {
        let distance = (point.0 - self.target.0).abs() + (point.1 - self.target.1).abs();
        if tool == Tool::Torch {
            return distance;
        }
        distance + 7
    }

    fn push(&mut self, current: Time, pt: Point, tool: Tool, prev: Point, prev_tool: Tool) {
        if let Some(t) = self.fastest {
            if current > t {
                return;
            }
        }

        if let Some(&(existing_time, _, _)) = self.seen.get(&(pt, tool)) {
            if existing_time <= current {
                // We've already been here, and just as fast.
                return;
            }
        }

        self.seen.insert((pt, tool), (current, prev, prev_tool));

        let expected: Time = current + self.heuristic(pt, tool);
        if let Some(t) = self.fastest {
            if expected > t {
                return;
            }
        }
        self.queue.push((Reverse(expected), pt, tool));
        // println!("New Queue: {:?}", self.queue);
        // queue.sort_by_key(|(pt, tool)| {
        //     let h = Routes::heuristic(pt, tl, cave.target);
        //     let time = seen[(pt, tool)].0;
        //     h + time;
        // })
    }

    pub fn new(cave: &Cave) -> Routes {
        let mut seen = HashMap::new();
        let mut queue = BinaryHeap::new();

        let start = ((0, 0), Tool::Torch);
        seen.insert(start, (0, start.0, start.1));
        queue.push((Reverse(0), start.0, start.1));

        Routes {
            target: cave.target,
            seen,
            queue,
            fastest: None,
        }
    }

    pub fn step(&mut self, cave: &mut Cave) -> bool {
        let (_, (x, y), tool) = match self.queue.pop() {
            None => {
                return false;
            }
            Some(s) => s,
        };

        let (time, _, _) = self.seen[&((x, y), tool)];

        if (x, y) == self.target && tool == Tool::Torch {
            let is_faster = self.fastest.map(|f| f > time).unwrap_or(true);
            if is_faster {
                println!(
                    "Found fastest route: {} < {}",
                    time,
                    self.fastest.unwrap_or(0)
                );
                self.fastest = Some(time);
            } else {
                println!(
                    "Found slower route: {} > {}",
                    time,
                    self.fastest.unwrap_or(0)
                );
            }

            // We found a route, possibly the fastest route so far,
            // but there may be an even faster one still out there
            return true;
        }

        if let Some(f) = self.fastest {
            if time > f {
                // This path takes too long, let's go a different way
                return true;
            }
        }

        let dxys = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dy) in &dxys {
            let nx = x + dx;
            if nx < 0 {
                continue;
            }
            let ny = y + dy;
            if ny < 0 {
                continue;
            }

            let erosion = cave.erosion(nx, ny);

            for &next_tool in &tools(erosion) {
                let mut next_time = time + 1;
                if next_tool != tool {
                    next_time += 7;
                }

                self.push(next_time, (nx, ny), next_tool, (x, y), tool);
            }
        }

        true
    }

    pub fn route(&self) -> Vec<(Time, Point, Tool)> {
        if self.fastest.is_none() {
            return vec![];
        }

        let mut last = (self.target, Tool::Torch);
        let mut backtracked = Vec::new();
        while last != ((0, 0), Tool::Torch) {
            let (t, prev, prev_tool) = self.seen[&last];
            backtracked.push((t, last.0, last.1));
            last = (prev, prev_tool);
        }

        backtracked.reverse();
        backtracked
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 22")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("depth")
                .short("d")
                .long("depth")
                .value_name("DEPTH")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target-x")
                .short("x")
                .long("target-x")
                .value_name("TARGETX")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target-y")
                .short("y")
                .long("target-y")
                .value_name("TARGETY")
                .takes_value(true),
        )
        .get_matches();

    // let input_path = matches.value_of("INPUT").unwrap_or("inputs/day22.txt");
    let depth: i64 = matches.value_of("depth").unwrap_or("11991").parse()?;
    let target_x: i64 = matches.value_of("TARGETX").unwrap_or("6").parse()?;
    let target_y: i64 = matches.value_of("TARGETY").unwrap_or("797").parse()?;

    eprintln!("Using depth {}, target ({}, {})", depth, target_x, target_y);

    let mut c = Cave::new(depth, (target_x, target_y));
    println!("Risk: {}", c.risk());
    c.geology(target_x + 500, target_y + 500);

    let mut routes = Routes::new(&c);
    let mut step = 0;
    while routes.step(&mut c) {
        step += 1;
        if step % 10_000 == 0 {
            let &(expected_rev, pt, tool) = routes.queue.peek().unwrap();
            let expected = expected_rev.0;
            let (time, _, _) = routes.seen.get(&(pt, tool)).unwrap();
            println!(
                "Step {}: Seen {}, Queue {}, fastest: {:?}, at ({}, {}) with {:?}; time {} ({} / {})",
                step,
                routes.seen.len(),
                routes.queue.len(),
                routes.fastest,
                pt.0,
                pt.1,
                tool,
                expected - time,
                time,
                expected,
            );

            let ql = routes.queue.len();
            if ql > 10 {
                let mut all: Vec<_> = routes.queue.iter().collect();
                all.sort();
                let rem = &all[ql - 10..ql];
                println!("Remaining: {:?}", rem);
            }
        }
        if step >= 1_000_000 {
            break;
        }
    }

    let route = routes.route();
    for (time, pt, tool) in route {
        let state = c.erosion(pt.0, pt.1);
        println!("{}: {:?} {:?} {:?}", time, pt, tool, state);
    }

    let f = routes.fastest.unwrap();
    println!(
        "Fastest route: {} or with an extra 1 for no apparent reason: {}",
        f,
        f + 1
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r#"
M=.|=.|.|=.|=|=.
.|=|=|||..|.=...
.==|....||=..|==
=.|....|.==.|==.
=|..==...=.|==..
=||.=.=||=|=..|=
|.=.===|||..=..|
|..==||=.|==|===
.=..===..=|.|||.
.======|||=|=.|=
.===|=|===T===||
=|||...|==..|=.|
=.=|=.=..=.||==|
||=|=...|==.=|==
|=.=||===.|||===
||.|==.|.|.||=||
"#;

    fn char_to_erosion(c: char) -> Option<Erosion> {
        match c {
            'T' => None,
            'M' => None,
            '.' => Some(Erosion::Rocky),
            '=' => Some(Erosion::Wet),
            '|' => Some(Erosion::Narrow),
            _ => unreachable!(),
        }
    }

    fn get_example_geology() -> Vec<Vec<Option<Erosion>>> {
        let lines: Vec<&str> = EXAMPLE_INPUT.split('\n').collect();
        let mut rows: Vec<Vec<Option<Erosion>>> = Vec::new();

        for l in lines {
            let l = l.trim();
            if l.is_empty() {
                continue;
            };

            let row = l.chars().map(char_to_erosion).collect();
            rows.push(row);
        }

        rows
    }

    #[test]
    fn test_cave() {
        let mut c = Cave::new(510, (10, 10));

        let rows = get_example_geology();

        let mut sum = 0;

        for (y, row) in rows.iter().enumerate() {
            for (x, oe) in row.iter().enumerate() {
                let e = c.erosion(x as i64, y as i64);
                let &exp = match oe {
                    None => {
                        eprintln!("Skipping: ({}, {}): {:?}", x, y, e);
                        continue;
                    }
                    Some(e) => e,
                };

                let risk: i64 = e.into();
                if x <= 10 && y <= 10 {
                    sum += risk;
                }

                eprintln!("({}, {}): {:?} =? {:?}", x, y, e, exp,);
                assert_eq!(e, exp);
            }
        }

        assert_eq!(sum, 114);

        assert_eq!(c.risk(), 114);
    }

    #[test]
    fn test_routing() {
        let mut c = Cave::new(510, (10, 10));
        let mut routes = Routes::new(&c);

        let mut step = 0;
        while routes.step(&mut c) {
            step += 1;
            println!(
                "Step {}: Queue {}, fastest: {:?}, next: {:?}",
                step,
                routes.queue.len(),
                routes.fastest,
                routes.queue.peek(),
            );
        }

        assert_eq!(routes.fastest, Some(45));

        let route = routes.route();
        for (time, pt, tool) in route {
            let state = c.erosion(pt.0, pt.1);
            println!("{}: {:?} {:?} {:?}", time, pt, tool, state);
        }
    }
}
