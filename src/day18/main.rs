use clap::{App, Arg};

use std::cmp::{max, min};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Acre {
    Open,
    Trees,
    Lumberyard,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Area {
    acres: Vec<Vec<Acre>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct State {
    pub width: usize,
    pub height: usize,
    pub trees: usize,
    pub lumberyards: usize,
}

impl Area {
    fn parse_line<S>(line: S) -> Result<Vec<Acre>, failure::Error>
    where
        S: AsRef<str>,
    {
        let bytes = line.as_ref().as_bytes();

        bytes
            .iter()
            .map(|b| match b {
                b'.' => Ok(Acre::Open),
                b'|' => Ok(Acre::Trees),
                b'#' => Ok(Acre::Lumberyard),
                c => Err(failure::format_err!("Unrecognized character {}", c)),
            })
            .collect()
    }

    pub fn parse_lines<I, S>(lines: I) -> Result<Self, failure::Error>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let result: Result<Vec<Vec<Acre>>, failure::Error> = lines
            .into_iter()
            .filter_map(|l| {
                let trimmed = l.as_ref().trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(Area::parse_line(trimmed))
                }
            })
            .collect();

        Ok(Area { acres: result? })
    }

    pub fn state(&self) -> State {
        let height = self.acres.len();
        if height == 0 {
            return State {
                height: 0,
                width: 0,
                trees: 0,
                lumberyards: 0,
            };
        }

        let width = self.acres[0].len();

        let mut trees = 0;
        let mut lumberyards = 0;

        for row in &self.acres {
            for a in row {
                match a {
                    Acre::Open => {}
                    Acre::Trees => trees += 1,
                    Acre::Lumberyard => lumberyards += 1,
                }
            }
        }

        State {
            height,
            width,
            trees,
            lumberyards,
        }
    }

    fn get_neighbors(&self, row: usize, col: usize) -> (usize, usize) {
        if self.acres.is_empty() {
            return (0, 0);
        }
        let width = self.acres[0].len();
        let (mut trees, mut lumberyards) = (0, 0);
        let row_start_ix = max(row, 1) - 1;
        let row_end_ix = min(row + 2, self.acres.len());
        let col_start_ix = max(col, 1) - 1;
        let col_end_ix = min(col + 2, width);

        // println!("Neighbors ({}, {}):", row, col);

        for (rix, acres) in self
            .acres
            .iter()
            .enumerate()
            .skip(row_start_ix)
            .take(row_end_ix - row_start_ix)
        {
            for (cix, acre) in acres
                .iter()
                .enumerate()
                .skip(col_start_ix)
                .take(col_end_ix - col_start_ix)
            {
                if (rix == row) && (cix == col) {
                    continue;
                }

                // println!("          ({}, {}): {:?}", rix, cix, acre);

                match acre {
                    Acre::Open => {}
                    Acre::Trees => trees += 1,
                    Acre::Lumberyard => lumberyards += 1,
                }
            }
        }

        (trees, lumberyards)
    }

    pub fn advance(&mut self) -> bool {
        let height = self.acres.len();
        if height == 0 {
            return false;
        }
        let width = self.acres[0].len();

        let mut new_acres: Vec<Vec<Acre>> = Vec::with_capacity(height);
        let mut changed: bool = false;

        for (rix, row) in self.acres.iter().enumerate() {
            let mut new_row: Vec<Acre> = Vec::with_capacity(width);
            for (cix, &acre) in row.iter().enumerate() {
                let (trees, lumberyards) = self.get_neighbors(rix, cix);

                let new_acre = match acre {
                    Acre::Open if trees >= 3 => Acre::Trees,
                    Acre::Open => Acre::Open,
                    Acre::Trees if lumberyards >= 3 => Acre::Lumberyard,
                    Acre::Trees => Acre::Trees,
                    Acre::Lumberyard if lumberyards >= 1 && trees >= 1 => Acre::Lumberyard,
                    Acre::Lumberyard => Acre::Open,
                };

                changed = changed || (acre != new_acre);

                // println!(
                //     "advance ({}, {}) ({} trees, {} lumberyards): {:?} => {:?}",
                //     rix, cix, trees, lumberyards, acre, new_acre
                // );

                new_row.push(new_acre);
            }
            new_acres.push(new_row);
        }

        self.acres = new_acres;

        changed
    }
}

pub struct Tracker {
    time: usize,
    area: Area,
    seen: HashMap<Area, usize>,
    history: Vec<Area>,
    repeats: Option<(usize, Vec<Area>)>,
}

impl Tracker {
    pub fn new(area: Area) -> Self {
        Tracker {
            area,
            time: Default::default(),
            seen: Default::default(),
            history: Default::default(),
            repeats: None,
        }
    }

    fn advance(&mut self) {
        if self.time == 0 {
            self.history.push(self.area.clone());
        }

        self.time += 1;
        if let Some((start, reps)) = &self.repeats {
            let ix = (self.time - start) % reps.len();
            self.area = reps[ix].clone();
            return;
        }

        self.area.advance();
        let cloned = self.area.clone();
        let repeat_time = match self.seen.entry(cloned) {
            Entry::Vacant(v) => {
                v.insert(self.time);
                self.history.push(self.area.clone());
                return;
            }
            Entry::Occupied(o) => *o.get(),
        };

        // So we know that repeat_time == self.time
        println!(
            "History len {}, repeat_time {}",
            self.history.len(),
            repeat_time
        );
        let reps = self.history.split_off(repeat_time);
        self.history.clear();
        self.seen.clear();
        println!(
            "Found repeat, {} -> {} ({})",
            repeat_time,
            self.time,
            reps.len()
        );
        self.repeats = Some((repeat_time, reps));
    }

    pub fn advance_to(&mut self, t: usize) {
        while self.repeats.is_none() {
            if t <= self.time {
                return;
            }

            self.advance();
        }

        if let Some((start, reps)) = &self.repeats {
            let ix = (t - start) % reps.len();
            self.area = reps[ix].clone();
            self.time = t;
        } else {
            unreachable!()
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 18")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day18.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let some_lines: std::io::Result<VecDeque<String>> = buf_reader.lines().collect();
    let mut lines: VecDeque<String> = some_lines?;
    let area = Area::parse_lines(&mut lines)?;

    let mut tracker = Tracker::new(area);
    tracker.advance_to(10);
    let state = tracker.area.state();

    println!(
        "At time {}: with {} trees, {} lumberyards = {} value",
        tracker.time,
        state.trees,
        state.lumberyards,
        state.trees * state.lumberyards
    );

    tracker.advance_to(1_000_000_000);

    let state = tracker.area.state();
    println!(
        "Finished at {} with {} trees, {} lumberyards = {} value",
        tracker.time,
        state.trees,
        state.lumberyards,
        state.trees * state.lumberyards
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUTS: [&str; 11] = [
        // Start
        r#"
            .#.#...|#.
            .....#|##|
            .|..|...#.
            ..|#.....#
            #.#|||#|#|
            ...#.||...
            .|....|...
            ||...#|.#|
            |.||||..|.
            ...#.|..|.
        "#,
        // 1 Minute
        r#"
            .......##.
            ......|###
            .|..|...#.
            ..|#||...#
            ..##||.|#|
            ...#||||..
            ||...|||..
            |||||.||.|
            ||||||||||
            ....||..|.
        "#,
        // 2 Minutes
        r#"
            .......#..
            ......|#..
            .|.|||....
            ..##|||..#
            ..###|||#|
            ...#|||||.
            |||||||||.
            ||||||||||
            ||||||||||
            .|||||||||
        "#,
        // 3 Minutes
        r#"
            .......#..
            ....|||#..
            .|.||||...
            ..###|||.#
            ...##|||#|
            .||##|||||
            ||||||||||
            ||||||||||
            ||||||||||
            ||||||||||
    "#,
        // 4 Minutes
        r#"
            .....|.#..
            ...||||#..
            .|.#||||..
            ..###||||#
            ...###||#|
            |||##|||||
            ||||||||||
            ||||||||||
            ||||||||||
            ||||||||||
    "#,
        // 5 Minutes
        r#"
            ....|||#..
            ...||||#..
            .|.##||||.
            ..####|||#
            .|.###||#|
            |||###||||
            ||||||||||
            ||||||||||
            ||||||||||
            ||||||||||
    "#,
        // 6 Minutes
        r#"
            ...||||#..
            ...||||#..
            .|.###|||.
            ..#.##|||#
            |||#.##|#|
            |||###||||
            ||||#|||||
            ||||||||||
            ||||||||||
            ||||||||||
    "#,
        // 7 Minutes
        r#"
            ...||||#..
            ..||#|##..
            .|.####||.
            ||#..##||#
            ||##.##|#|
            |||####|||
            |||###||||
            ||||||||||
            ||||||||||
            ||||||||||
    "#,
        // 8 Minutes
        r#"
            ..||||##..
            ..|#####..
            |||#####|.
            ||#...##|#
            ||##..###|
            ||##.###||
            |||####|||
            ||||#|||||
            ||||||||||
            ||||||||||
    "#,
        // 9 Minutes
        r#"
            ..||###...
            .||#####..
            ||##...##.
            ||#....###
            |##....##|
            ||##..###|
            ||######||
            |||###||||
            ||||||||||
            ||||||||||
    "#,
        // 10 Minutes
        r#"
            .||##.....
            ||###.....
            ||##......
            |##.....##
            |##.....##
            |##....##|
            ||##.####|
            ||#####|||
            ||||#|||||
            ||||||||||
    "#,
    ];

    fn get_test_area(s: &str) -> Result<Area, failure::Error> {
        let lines: Vec<&str> = s.split('\n').collect();
        Area::parse_lines(lines)
    }

    #[test]
    fn test_parse() {
        let maybe_area = get_test_area(TEST_INPUTS[0]);
        let area = maybe_area.unwrap();

        let s = area.state();

        assert_eq!(s.height, 10);
        assert_eq!(s.width, 10);
        assert_eq!(s.trees, 27);
        assert_eq!(s.lumberyards, 17);
    }

    #[test]
    fn test_neighbors() {
        let area = get_test_area(TEST_INPUTS[0]).unwrap();

        let nbr = area.get_neighbors(0, 0);
        assert_eq!(nbr, (0, 1));
        let nbr = area.get_neighbors(0, 1);
        assert_eq!(nbr, (0, 0));
        let nbr = area.get_neighbors(1, 0);
        assert_eq!(nbr, (1, 1));
        let nbr = area.get_neighbors(1, 1);
        assert_eq!(nbr, (1, 1));

        let nbr = area.get_neighbors(0, 7);
        assert_eq!(nbr, (1, 3));
        let nbr = area.get_neighbors(0, 8);
        assert_eq!(nbr, (2, 2));
    }

    #[test]
    fn test_advance() {
        let mut area = get_test_area(TEST_INPUTS[0]).unwrap();

        let mut min = 0;
        for input in TEST_INPUTS.iter().skip(1) {
            area.advance();
            min += 1;
            let stepped = get_test_area(input).unwrap();
            assert_eq!(area.acres, stepped.acres);
        }

        assert_eq!(min, 10);

        let expected_state = State {
            height: 10,
            width: 10,
            trees: 37,
            lumberyards: 31,
        };
        assert_eq!(area.state(), expected_state);
    }

    #[test]
    fn test_tracker() {
        let area = get_test_area(TEST_INPUTS[0]).unwrap();

        let mut tracker = Tracker::new(area);
        let mut area = get_test_area(TEST_INPUTS[0]).unwrap();

        while tracker.repeats.is_none() {
            area.advance();
            tracker.advance();
        }

        let (start, reps) = (tracker.repeats).as_ref().unwrap();

        println!("Repeats with loop {} after {}", reps.len(), start);

        for _ in 0..=reps.len() * 2 {
            area.advance();
            tracker.advance();

            assert_eq!(area, tracker.area);

            let area0 = get_test_area(TEST_INPUTS[0]).unwrap();
            let mut new_tracker = Tracker::new(area0);
            new_tracker.advance_to(tracker.time);

            assert_eq!(area, new_tracker.area);
        }
    }
}
