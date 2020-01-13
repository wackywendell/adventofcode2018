use clap::{App, Arg};

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum Direction {
    N,
    E,
    W,
    S,
}

#[derive(Default, Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct Room {
    x: i64,
    y: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub distances: HashMap<Room, i64>,
    pub connections: HashMap<Room, HashSet<Room>>,
}
impl FromStr for Building {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut distances: HashMap<Room, i64> = HashMap::new();
        let mut connections: HashMap<Room, HashSet<Room>> = HashMap::new();

        let mut parents: Vec<Room> = Vec::new();

        let mut room: Room = Default::default();
        distances.insert(room, 0);
        connections.insert(room, Default::default());

        for (_ix, b) in s.bytes().enumerate() {
            // let read_so_far = || std::str::from_utf8(&s.as_bytes()[..=ix]).unwrap();
            let last = match b {
                b'^' => {
                    assert_eq!(distances.len(), 1);
                    continue;
                }
                b'$' => break,
                b'N' => {
                    let last = room;
                    room.y += 1;
                    last
                }
                b'E' => {
                    let last = room;
                    room.x += 1;
                    last
                }
                b'W' => {
                    let last = room;
                    room.x -= 1;
                    last
                }
                b'S' => {
                    let last = room;
                    room.y -= 1;
                    last
                }
                b'(' => {
                    parents.push(room);
                    continue;
                }
                b'|' => {
                    room = *parents.last().unwrap();
                    continue;
                }
                b')' => {
                    room = parents.pop().unwrap();
                    continue;
                }
                _ => panic!("unrecognized character: {}", b),
            };

            connections.entry(last).or_default().insert(room);
            connections.entry(room).or_default().insert(last);
            let dist = distances.get(&last).unwrap() + 1;
            // println!("At {:?}: {}  {}", room, dist, read_so_far());
            let mut to_update: VecDeque<(Room, i64)> = VecDeque::new();
            let old = match distances.entry(room) {
                Entry::Vacant(v) => {
                    v.insert(dist);
                    // println!("  Inserted");
                    continue;
                }
                Entry::Occupied(o) => *o.get(),
            };

            match dist.cmp(&old) {
                Ordering::Equal => {
                    // println!("  Equal: {}", dist);
                    continue;
                }
                Ordering::Less => {
                    // println!("  Less: {} < {}", dist, old);
                    to_update.push_back((room, dist));
                }
                Ordering::Greater => {
                    // println!("  Greater: {} > {}", dist, old);
                    for &r in connections.get(&room).iter().flat_map(|&c| c) {
                        to_update.push_back((r, old + 1));
                    }
                }
            };

            while !to_update.is_empty() {
                let (room, dist) = to_update.pop_front().unwrap();
                // println!("  Popped {:?}, {}", room, dist);

                match distances.entry(room) {
                    Entry::Vacant(v) => {
                        v.insert(dist);
                        // println!("    Vacant");
                        continue;
                    }
                    Entry::Occupied(mut o) => {
                        let old = o.get_mut();
                        // println!("    Occupied: {}", *old);
                        if *old <= dist {
                            continue;
                        }
                        // println!("    Updating {:?}: {} -> {}", room, old, dist);
                        *old = dist;
                        for &r in connections.get(&room).iter().flat_map(|&h| h) {
                            to_update.push_back((r, *old + 1));
                        }
                    }
                }
            }
        }

        Ok(Building {
            distances,
            connections,
        })
    }
}

impl Building {
    pub fn furthest(&self) -> i64 {
        self.distances.iter().map(|(_, &v)| v).max().unwrap()
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 20")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day20.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let some_lines: std::io::Result<Vec<String>> = buf_reader.lines().collect();
    let lines: Vec<String> = some_lines?;

    assert_eq!(lines.len(), 1);
    let line = &lines[0];

    eprintln!("Found string of length {}", line.as_bytes().len());

    let b = Building::from_str(line)?;
    println!("Furthest: {}", b.furthest());

    let mut over1000 = 0;
    for (_, d) in b.distances {
        if d >= 1000 {
            over1000 += 1;
        }
    }

    println!("Over 1000: {}", over1000);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic examples from the problem
    #[test]
    fn test_first() {
        let s = "^WNE$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 3);

        let s = "^ENWWW(NEEE|SSE(EE|N))$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 10);

        let s = "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 18);
    }

    /// More advanced examples from the problem
    #[test]
    fn test_more() {
        let s = "^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 23);

        let s = "^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 31);
    }

    /// Extra tests I added
    #[test]
    fn test_extra() {
        let s = "^WNE(NESW|)$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 5);

        let s = "^WNENES$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 6);

        let s = "^WNE(NESW|)S$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 3);
        let s = "^WNE(NESW|)SS$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 3);
        let s = "^WNE(NENSSW|)SS$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 4);
        let s = "^WNE(NESW|)SSS$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 3);
        let s = "^WNE(NESW|)SSSS$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 3);
        let s = "^WNE(NESW|)SSSSS$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 4);
    }
}
