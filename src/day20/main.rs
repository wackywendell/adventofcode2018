use petgraph::Graph;

use clap::{App, Arg};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use std::cmp::max;
// use std::str::FromStr;

pub enum Direction {
    N,
    E,
    W,
    S,
}

pub enum Node {
    Direction(Direction),
    Alternatives,
    Detours,
}

pub struct Route {
    pub graph: Graph<Node, (), petgraph::Directed>,
}

// impl FromStr for Route {
//     type Err = failure::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let g: Graph<Node, (), petgraph::Directed> = Graph::default();

//         Ok(Route { graph: g })
//     }
// }

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
}
