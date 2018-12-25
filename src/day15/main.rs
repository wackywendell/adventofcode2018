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
    elf_power: i64,
}

impl Battle {
    fn parse_lines<S, E, T>(iter: T, start_hp: i64, elf_power: i64) -> Result<Self, failure::Error>
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
            elf_power,
        })
    }

    fn empty_neighbors(&self, loc: Location, allow: Option<Location>) -> Vec<Location> {
        let (y, x) = loc;
        let mut locs: Vec<Location> = vec![(y - 1, x), (y, x - 1), (y, x + 1), (y + 1, x)];
        // Keep neighbors that are (in allow) or (are viable squares and unoccupied)
        locs.retain(|&loc| {
            allow.map(|l| l == loc).unwrap_or(false)
                || (self.squares.contains(&loc) && !self.occupied.contains(&loc))
        });
        locs
    }

    // shortest_distance returns the (shortest distance, next step) from start to end,
    // if a path can be found.
    fn shortest_distance(&self, start: Location, end: Location) -> Option<(i16, Location)> {
        #[derive(PartialEq, PartialOrd, Eq, Ord, Debug)]
        struct PartialPath {
            covered: i16,
            dist: i16,
            first_step: Location,
            loc: Location,
            path: Vec<Location>,
        };

        let mut partials: Vec<PartialPath> = vec![PartialPath {
            dist: start.dist(end),
            loc: start,
            first_step: start,
            path: Vec::new(),
            covered: 0,
        }];

        let mut seen = HashSet::new();
        seen.insert(start);

        loop {
            partials.sort_by_key(|p| std::cmp::Reverse((p.covered, p.first_step, p.loc)));
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
                let mut path = popped.path.clone();
                path.push(n);
                let p = PartialPath {
                    dist: n.dist(end),
                    loc: n,
                    first_step,
                    covered: popped.covered + 1,
                    path,
                };
                partials.push(p);
            }
        }
    }

    // Returns (next step, goal, enemies_found)
    fn find_target(&self, character: Character) -> Option<(Location, Location, bool)> {
        let mut choices = Vec::with_capacity((self.characters.len() - 1) * 4);
        let mut enemies_found = 0;
        for target in &self.characters {
            if target.side == character.side || target.hp <= 0 {
                continue;
            }
            enemies_found += 1;

            for empty in self.empty_neighbors(target.location, Some(character.location)) {
                // println!("-- Checking empty at {:?}", empty);
                let (dist, step) = match self.shortest_distance(character.location, empty) {
                    None => continue,
                    Some(sd) => sd,
                };

                // println!(
                //     "Found target {:?}({}) at {:?} [{:?}]; distance {}, step {:?}",
                //     target.side, target.hp, target.location, empty, dist, step
                // );

                choices.push((dist, empty, target.location, step));
            }
        }

        if enemies_found == 0 {
            return Some((character.location, character.location, false));
        }

        choices.sort();
        // println!("Choices:");
        // for (d, e, s) in &choices {
        //     println!("{} - {:?} - {:?}", d, e, s);
        // }

        // if let Some((d, e, t, s)) = choices.first() {
        //     println!("Choosing {} - {:?} - {:?} - {:?}", d, e, t, s);
        // }

        choices.first().map(|&(_, _, t, s)| (s, t, true))
    }

    fn target_to_attack(&mut self, c: Character) -> Option<&mut Character> {
        let mut target = None;
        for t in self.characters.iter_mut() {
            if t.side == c.side || t.hp <= 0 || c.location.dist(t.location) != 1 {
                continue;
            }
            match target {
                None => {
                    target = Some(t);
                }
                Some(ref m) if m.hp <= t.hp => {}
                Some(_) => target = Some(t),
            };
        }

        target
    }

    fn attack_power(&self, character: Character) -> i64 {
        match character.side {
            Side::Goblin => 3,
            Side::Elf => self.elf_power,
        }
    }

    fn round(&mut self) -> bool {
        for ix in 0..self.characters.len() {
            let mut c = self.characters[ix];
            if c.hp <= 0 {
                // Dead characters don't move
                continue;
            }
            let (step, _goal, any_enemies) = match self.find_target(c) {
                None => {
                    // println!("Can't move {:?} at {:?}", c.side, c.location);
                    // This character can't reach any enemies.
                    // No moving or attacking.
                    continue;
                }
                Some(st) => st,
            };
            if !any_enemies {
                return false;
            }
            // Move. This may be a no-op if we're already next to a target.
            if c.location != step {
                // println!(
                //     "Moving {:?} at {:?} to {:?} (goal: {:?})",
                //     c.side, c.location, step, goal
                // );
                self.occupied.remove(&c.location);
                c.location = step;
                self.occupied.insert(c.location);
            }
            self.characters[ix] = c;

            // Attack.
            let ap = self.attack_power(c);
            let to_remove = if let Some(t) = self.target_to_attack(c) {
                // println!(
                //     "Attack by {:?}({}) at {:?} against {:?}({}) at {:?}",
                //     c.side, c.hp, c.location, t.side, t.hp, t.location
                // );
                t.hp -= ap;
                if t.hp <= 0 {
                    Some(t.location)
                } else {
                    None
                }
            } else {
                None
            };

            // Mark spots of dead characters as unoccupied.
            if let Some(loc) = to_remove {
                self.occupied.remove(&loc);
            }
        }

        self.characters.sort();

        true
    }

    // Run to completion. Returns (# of rounds, total hp, side that won)
    fn complete(&mut self) -> (usize, i64, Side) {
        let mut n = 0;
        while self.round() {
            n += 1;
        }

        let mut side = Side::Elf;
        let mut hp = 0;
        for c in &self.characters {
            if c.hp <= 0 {
                continue;
            }
            side = c.side;
            hp += c.hp;
        }

        (n, hp, side)
    }

    fn deaths(&self, side: Side) -> usize {
        self.characters
            .iter()
            .filter(|c| c.side == side && c.hp <= 0)
            .count()
    }
}

fn main() -> Result<(), failure::Error> {
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
    let initial = Battle::parse_lines(buf_reader.lines(), 200, 3)?;
    let mut battle = initial.clone();
    let (rounds, hp, side) = battle.complete();

    println!(
        "{:?} win after {} rounds with {} hp left. Score: {}",
        side,
        rounds,
        hp,
        hp * rounds as i64
    );

    let mut elf_power = 2;
    while battle.deaths(Side::Elf) > 0 {
        battle = initial.clone();
        elf_power += 1;
        battle.elf_power = elf_power;
        let (rounds, hp, side) = battle.complete();

        let elf_deaths = battle.deaths(Side::Elf);

        println!(
            "{}: {:?} win after {} rounds with {} hp left ({} elf deaths). Score: {}",
            elf_power,
            side,
            rounds,
            hp,
            elf_deaths,
            hp * rounds as i64
        );
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

        Battle::parse_lines(lines.into_iter().map(ok), 200, 3).unwrap()
    }

    fn get_characters(battle: &Battle) -> Vec<Character> {
        battle
            .characters
            .iter()
            .filter_map(|&c| if c.hp > 0 { Some(c) } else { None })
            .collect()
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
        let (s, g, _) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (1, 2));
        assert_eq!(g, (1, 4));
    }

    #[test]
    fn test_far_targeting() {
        let test_input = r#"
#######
#.....#
#..E..#
#.....#
#..####
#.....#
#..##.#
#####.#
#...G.#
#######"#;

        let battle = get_test_battle(test_input);
        assert_eq!(battle.characters.len(), 2);
        assert_eq!(battle.occupied.len(), battle.characters.len());

        let &c = battle.characters.first().unwrap();
        assert_eq!(c.location, (2, 3));
        println!("Find target!");
        let (s, g, _) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (2, 2));
        assert_eq!(g, (8, 4));
    }

    #[test]
    fn test_blocked_targeting() {
        let test_input = r#"
#######
#.E...#
#..##.#
#E##..#
#G....#
#######"#;

        let battle = get_test_battle(test_input);
        assert_eq!(battle.characters.len(), 3);
        assert_eq!(battle.occupied.len(), battle.characters.len());
        assert_eq!(battle.squares.len(), 16);

        let &c = battle.characters.first().unwrap();
        assert_eq!(c.location, (1, 2));
        let (s, g, _) = battle.find_target(c).unwrap();
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
        let (s, g, _) = battle.find_target(c).unwrap();
        println!("{:?}  {:?}", s, g);
        assert_eq!(s, (1, 2));
        assert_eq!(g, (1, 3));
    }

    fn get_test_battle_with_hps(s: &str, hps: &[i64]) -> Battle {
        let mut battle = get_test_battle(s);
        assert_eq!(battle.characters.len(), hps.len());
        for (mut c, &hp) in battle.characters.iter_mut().zip(hps) {
            c.hp = hp;
        }

        battle
    }

    #[test]
    fn test_rounds() {
        let initial = r"
#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######";

        let round1_str = r"
#######
#..G..#
#...EG#
#.#G#G#
#...#E#
#.....#
#######";

        let mut battle = get_test_battle(initial);
        println!("Running round 1");
        battle.round();
        let chars = get_characters(&battle);

        let expected_hps = &[200i64, 197, 197, 200, 197, 197];
        let round1 = get_test_battle_with_hps(round1_str, expected_hps);
        let exp_chars = get_characters(&round1);
        assert_eq!(chars, exp_chars);

        let round2_str = r"
#######
#...G.#
#..GEG#
#.#.#G#
#...#E#
#.....#
#######";

        println!("Running round 2");
        battle.round();
        let chars = get_characters(&battle);
        let expected_hps = &[200i64, 200, 188, 194, 194, 194];
        let round2 = get_test_battle_with_hps(round2_str, expected_hps);
        let exp_chars = get_characters(&round2);
        assert_eq!(chars, exp_chars);

        let round23_str = r"
#######
#...G.#
#..G.G#
#.#.#G#
#...#E#
#.....#
#######";
        let expected_hps = &[200i64, 200, 131, 131, 131];
        let round23 = get_test_battle_with_hps(round23_str, expected_hps);
        let exp_chars = get_characters(&round23);

        for n in 2..23 {
            println!("Running round {}", n + 1);
            battle.round();
        }

        let chars = get_characters(&battle);

        assert_eq!(chars, exp_chars);

        let round24_str = r"
#######
#..G..#
#...G.#
#.#G#G#
#...#E#
#.....#
#######";
        let expected_hps = &[200i64, 131, 200, 128, 128];
        let round24 = get_test_battle_with_hps(round24_str, expected_hps);
        let exp_chars = get_characters(&round24);

        for n in 23..24 {
            println!("Running round {}", n + 1);
            battle.round();
            println!("Occupied: {:?}", battle.occupied);
        }

        let chars = get_characters(&battle);

        assert_eq!(chars, exp_chars);

        let round25_str = r"
#######
#.G...#
#..G..#
#.#.#G#
#..G#E#
#.....#
#######";
        let expected_hps = &[200i64, 131, 125, 200, 125];
        let round25 = get_test_battle_with_hps(round25_str, expected_hps);
        let exp_chars = get_characters(&round25);

        for n in 24..25 {
            println!("Running round {}", n + 1);
            battle.round();
        }

        let chars = get_characters(&battle);

        assert_eq!(chars, exp_chars);

        let round47_str = r"
#######
#G....#
#.G...#
#.#.#G#
#...#.#
#....G#
#######";

        let expected_hps = &[200i64, 131, 59, 200];
        let round47 = get_test_battle_with_hps(round47_str, expected_hps);
        let exp_chars = get_characters(&round47);

        for n in 25..47 {
            println!("Running round {}", n + 1);
            let finished = battle.round();
            assert!(finished);
        }

        let chars = get_characters(&battle);

        assert_eq!(chars, exp_chars);

        println!("Running round 48");
        let finished = battle.round();
        assert!(!finished);
    }

    #[test]
    fn test_completion() {
        let initial = r"
#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######";

        let mut battle = get_test_battle(initial);
        let (rounds, hp, side) = battle.complete();

        assert_eq!(rounds, 47);
        assert_eq!(hp, 590);
        assert_eq!(side, Side::Goblin);
    }
}
