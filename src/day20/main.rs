use clap::{App, Arg};

use std::cmp::max;
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

        for (ix, b) in s.bytes().enumerate() {
            let read_so_far = || std::str::from_utf8(&s.as_bytes()[..=ix]).unwrap();
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
            println!("At {:?}: {}  {}", room, dist, read_so_far());
            let mut to_update: VecDeque<(Room, i64)> = VecDeque::new();
            // to_update.push_back((room, dist));

            let old = match distances.entry(room) {
                Entry::Vacant(v) => {
                    v.insert(dist);
                    println!("  Inserted");
                    continue;
                }
                Entry::Occupied(o) => *o.get(),
            };

            match dist.cmp(&old) {
                std::cmp::Ordering::Equal => {
                    println!("  Equal: {}", dist);
                    continue;
                }
                std::cmp::Ordering::Less => {
                    println!("  Less: {} < {}", dist, old);
                    to_update.push_back((room, dist));
                }
                std::cmp::Ordering::Greater => {
                    println!("  Greater: {} > {}", dist, old);
                    for &r in connections.get(&room).iter().flat_map(|&c| c) {
                        to_update.push_back((r, old + 1));
                    }
                }
            };

            while !to_update.is_empty() {
                let (room, dist) = to_update.pop_front().unwrap();
                println!("  Popped {:?}, {}", room, dist);

                match distances.entry(room) {
                    Entry::Vacant(v) => {
                        v.insert(dist);
                        println!("    Vacant");
                        continue;
                    }
                    Entry::Occupied(mut o) => {
                        let old = o.get_mut();
                        println!("    Occupied: {}", *old);
                        if *old <= dist {
                            continue;
                        }
                        println!("    Updating {:?}: {} -> {}", room, old, dist);
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

fn furthest_node(s: &str) -> isize {
    let mut furthest_seen = 0isize;

    let mut parents: Vec<isize> = Vec::new();
    let mut current_distance = 0isize;
    let mut alternatives: Vec<isize> = Vec::new();
    for (_ix, b) in s.bytes().enumerate() {
        // let read_so_far = || std::str::from_utf8(&s.as_bytes()[..=ix]).unwrap();
        match b {
            b'^' => assert_eq!(furthest_seen, 0),
            b'$' => break,
            b'N' => current_distance += 1,
            b'E' => current_distance += 1,
            b'W' => current_distance += 1,
            b'S' => current_distance += 1,
            b'(' => {
                parents.push(current_distance);
            }
            b'|' => {
                alternatives.push(current_distance);
                current_distance = *parents.last().unwrap();
            }
            b')' => {
                alternatives.push(current_distance);
                let parent_distance = *parents.last().unwrap();
                let max_child = alternatives.iter().copied().max();
                let mx = max_child.expect("Expected alternatives not to be empty");
                let min_child = alternatives.iter().copied().min();
                let mn = min_child.expect("Expected alternatives not to be empty");

                if current_distance == *parents.last().unwrap() {
                    // Detour
                    let max_in_detour = parent_distance + (mx - parent_distance) / 2;
                    // println!(
                    //     "Detour at {}: ({}, {})",
                    //     read_so_far(),
                    //     furthest_seen,
                    //     max_in_detour
                    // );
                    furthest_seen = max(furthest_seen, max_in_detour);
                } else {
                    // dbg!((furthest_seen, mx));
                    furthest_seen = max(furthest_seen, mx);
                    // println!(
                    //     "Alternative at {}: ({}, {})",
                    //     read_so_far(),
                    //     furthest_seen,
                    //     mx
                    // );
                }
                // println!("Resetting to {}", mn);
                current_distance = mn;
                parents.pop();
                alternatives.clear();

                continue;
            }
            _ => panic!("unrecognized character: {}", b),
        }

        if parents.is_empty() {
            // println!(
            //     "Furthest at {}: {}, {}",
            //     read_so_far(),
            //     furthest_seen,
            //     current_distance
            // );
            furthest_seen = max(furthest_seen, current_distance);
        }
    }

    furthest_seen
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

    let furthest = furthest_node(&line);

    println!("Furthest: {}", furthest);

    let b = Building::from_str(line)?;
    println!("Furthest: {}", b.furthest());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic examples from the problem
    #[test]
    fn test_first() {
        let s = "^WNE$";
        assert_eq!(furthest_node(s), 3);

        let s = "^ENWWW(NEEE|SSE(EE|N))$";
        assert_eq!(furthest_node(s), 10);

        let s = "^ENNWSWW(NEWS|)SSSEEN(WNSE|)EE(SWEN|)NNN$";
        assert_eq!(furthest_node(s), 18);
    }

    /// More advanced examples from the problem
    #[test]
    fn test_more() {
        let s = "^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$";
        assert_eq!(furthest_node(s), 23);

        let s = "^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$";
        assert_eq!(furthest_node(s), 31);
    }

    /// Extra tests I added
    #[test]
    fn test_extra() {
        let s = "^WNE(NESW|)$";
        assert_eq!(furthest_node(s), 5);

        let s = "^WNENES$";
        assert_eq!(furthest_node(s), 6);

        let s = "^WNE(NESW|)S$";
        assert_eq!(furthest_node(s), 5);
        let s = "^WNE(NESW|)SS$";
        assert_eq!(furthest_node(s), 5);
        let s = "^WNE(NENSSW|)SS$";
        assert_eq!(furthest_node(s), 6);
        let s = "^WNE(NESW|)SSS$";
        assert_eq!(furthest_node(s), 6);
    }

    /// Basic examples from the problem
    #[test]
    fn test_building_first() {
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
    fn test_building_more() {
        let s = "^ESSWWN(E|NNENN(EESS(WNSE|)SSS|WWWSSSSE(SW|NNNE)))$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 23);

        let s = "^WSSEESWWWNW(S|NENNEEEENN(ESSSSW(NWSW|SSEN)|WSWWN(E|WWS(E|SS))))$";
        let b = Building::from_str(s).unwrap();
        assert_eq!(b.furthest(), 31);
    }

    /// Extra tests I added
    #[test]
    fn test_building_extra() {
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
