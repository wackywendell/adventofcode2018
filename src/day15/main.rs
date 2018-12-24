#![warn(clippy::all)]

use clap::{App, Arg};
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

// Location in the format (y, x) so that they sort naturally into reading order
type Location = (i16, i16);

trait Distancer<M> {
    fn dist(self, other: Self) -> M;
}

impl Distancer<i16> for Location {
    fn dist(self, other: Self) -> i16 {
        (self.0 - other.0).abs() + (self.1 - other.1).abs()
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
enum Side {
    Elf,
    Goblin,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
struct Character {
    location: Location,
    hp: i64,
    side: Side,
}

impl Character {
    fn new(location: Location, hp: i64, side: Side) -> Self {
        Character { location, hp, side }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Battle {
    squares: HashSet<Location>,
    occupied: HashSet<Location>,
    characters: Vec<Character>,
}

impl Battle {
    fn parse_lines<S, E, T>(iter: T, start_hp: i64) -> Result<Self, failure::Error>
    where
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        let mut squares = HashSet::new();
        let mut occupied = HashSet::new();
        let mut characters = Vec::new();

        for (y, l) in iter.into_iter().enumerate() {
            let line_ref = l.map_err(Into::into)?;
            let line = line_ref.as_ref();
            for (x, c) in line.chars().enumerate() {
                let side = match c {
                    '#' => continue,
                    '.' => None,
                    'E' => Some(Side::Elf),
                    'G' => Some(Side::Goblin),
                    _ => panic!("Character {} Not Recognized", c),
                };

                let loc = (y as i16, x as i16);
                squares.insert(loc);
                if let Some(s) = side {
                    characters.push(Character::new(loc, start_hp, s));
                    occupied.insert(loc);
                }

                // println!("Found {:?} at ({},{})", side, x, y);
            }
        }

        Ok(Battle {
            squares,
            characters,
            occupied,
        })
    }

    fn empty_neighbors(&self, loc: Location, allow: Option<Location>) -> Vec<Location> {
        let (y, x) = loc;
        let mut locs: Vec<Location> = vec![(y - 1, x), (y, x - 1), (y, x + 1), (y + 1, x)];
        // Keep neigbors that are (in allow) or (are viable squares and unoccupied)
        locs.retain(|&loc| {
            allow.map(|l| l == loc).unwrap_or(false)
                || (self.squares.contains(&loc) && !self.occupied.contains(&loc))
        });
        locs
    }

    // shortest_distance returns the (shortest distance, next step) from start to end,
    // if a path can be found.
    fn shortest_distance(&self, start: Location, end: Location) -> Option<(i16, Location)> {
        #[derive(PartialEq, PartialOrd, Eq, Ord)]
        struct PartialPath {
            dist: i16,
            loc: Location,
            first_step: Location,
            // path: Vec<Location>,
            covered: i16,
        };

        let mut partials: Vec<PartialPath> = vec![PartialPath {
            dist: start.dist(end),
            loc: start,
            first_step: start,
            // path: Vec::new(),
            covered: 0,
        }];

        let mut seen = HashSet::new();
        seen.insert(start);

        loop {
            partials.sort_by_key(|p| (-p.dist, p.first_step, p.loc));
            let popped = match partials.pop() {
                None => {
                    // All paths ended in dead ends. No good.
                    return None;
                }
                Some(p) => p,
            };

            if popped.dist == 0 {
                // println!("Found path from {:?} to {:?}:", start, end);
                // for &(y, x) in &popped.path {
                //     println!("..{},{}", y, x);
                // }
                return Some((popped.covered, popped.first_step));
            }

            for n in self.empty_neighbors(popped.loc, None) {
                if !seen.insert(n) {
                    // We've been here before
                    continue;
                }

                let first_step = if popped.first_step == start {
                    n
                } else {
                    popped.first_step
                };
                // let mut path = popped.path.clone();
                // path.push(n);
                let p = PartialPath {
                    dist: n.dist(end),
                    loc: n,
                    first_step,
                    covered: popped.covered + 1,
                    // path: path,
                };
                partials.push(p);
            }
        }
    }

    // Returns (next step, goal)
    fn find_target(&self, character: Character) -> Option<(Location, Location)> {
        let mut choices = Vec::with_capacity((self.characters.len() - 1) * 4);
        for target in &self.characters {
            if target.side == character.side {
                continue;
            }

            for empty in self.empty_neighbors(target.location, Some(character.location)) {
                let (dist, step) = match self.shortest_distance(character.location, empty) {
                    None => continue,
                    Some(sd) => sd,
                };

                choices.push((dist, empty, target.location, step));
            }
        }

        choices.sort();
        // println!("Choices:");
        // for (d, e, s) in &choices {
        //     println!("{} - {:?} - {:?}", d, e, s);
        // }

        choices.first().map(|&(_, _, t, s)| (s, t))
    }

    fn round(&mut self) {
        for ix in 0..self.characters.len() {
            let mut c = self.characters[ix];
            if c.hp <= 0 {
                // Dead characters don't move
                continue;
            }
            let (step, mut target) = match self.find_target(c) {
                None => {
                    // This character can't reach any enemies.
                    // No moving or attacking.
                    continue;
                }
                Some(st) => st,
            };
            // Move. This may be a no-op if we're already next to a target.
            c.location = step;
            self.characters[ix] = c;

            // Attack.
            if c.location.dist(target) == 1 {
                target.hp -= c.attack_power();
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 15")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day15.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let mut sum = 0;
    let mut values = vec![];
    for (_i, line) in buf_reader.lines().enumerate() {
        let s = line?;
        let n = s.trim().parse::<i64>().unwrap();
        sum += n;
        values.push(n);
    }

    println!("Final sum: {}", sum);

    let mut seen: HashSet<i64> = HashSet::new();
    sum = 0;
    'outer: loop {
        for &v in &values {
            sum += v;
            if seen.contains(&sum) {
                println!("Repeated: {}", sum);
                break 'outer;
            }
            seen.insert(sum);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_battle(s: &str) -> Battle {
        let lines: Vec<&str> = s.split('\n').skip(1).collect();
        fn ok(s: &str) -> Result<&str, failure::Error> {
            Ok(s)
        }

        Battle::parse_lines(lines.into_iter().map(ok), 200).unwrap()
    }

    #[test]
    fn test_targeting() {
        let test_input = r#"
#######
#E..G.#
#...#.#
#.G.#G#
#######"#;

        let battle = get_test_battle(test_input);

        assert_eq!(battle.characters.len(), 4);
        assert_eq!(battle.occupied.len(), battle.characters.len());
        assert_eq!(battle.squares.len(), 13);

        let &c = battle.characters.first().unwrap();
        assert_eq!(c.location, (1, 1));
        let (s, g) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (1, 2));
        assert_eq!(g, (1, 4));
    }

    #[test]
    fn test_far_targeting() {
        let test_input = r#"
#######
#.E...#
#..##.#
####..#
#G....#
#######"#;

        let battle = get_test_battle(test_input);
        assert_eq!(battle.characters.len(), 2);
        assert_eq!(battle.occupied.len(), battle.characters.len());
        assert_eq!(battle.squares.len(), 15);

        let &c = battle.characters.first().unwrap();
        assert_eq!(c.location, (1, 2));
        let (s, g) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (1, 3));
        assert_eq!(g, (4, 1));
    }

    #[test]
    fn test_near_targeting() {
        let test_input = r#"
#######
#.EG..#
#..G..#
#..#..#
#G....#
#######"#;

        let battle = get_test_battle(test_input);
        assert_eq!(battle.characters.len(), 4);
        assert_eq!(battle.occupied.len(), battle.characters.len());
        assert_eq!(battle.squares.len(), 19);

        let &c = battle.characters.first().unwrap();
        assert_eq!(c.location, (1, 2));
        let (s, g) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (1, 2));
        assert_eq!(g, (1, 3));
    }
}
