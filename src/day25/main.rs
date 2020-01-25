#![warn(clippy::all)]

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;

use clap::{App, Arg};
use log::debug;
use text_io::try_scan;

use aoc::parse::parse_lines_err;

pub type Val = i64;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Vec4(Val, Val, Val, Val);

impl std::ops::Sub for Vec4 {
    type Output = Self;

    fn sub(self: Self, rhs: Self) -> Self {
        Vec4(
            self.0 - rhs.0,
            self.1 - rhs.1,
            self.2 - rhs.2,
            self.3 - rhs.3,
        )
    }
}

impl Vec4 {
    pub fn manhattan(self) -> Val {
        self.0.abs() + self.1.abs() + self.2.abs() + self.3.abs()
    }

    pub fn parse_line(line: &str) -> Result<Self, failure::Error> {
        let (x, y, z, t): (i64, i64, i64, i64);
        try_scan!(line.bytes() => "{},{},{},{}", x,y,z,t);
        Ok(Vec4(x, y, z, t))
    }
}

pub struct Constellations {
    // (constellation id, point)
    points: Vec<(usize, Vec4)>,
    // Maps constellation id -> Vec<Point id>
    constellations: HashMap<usize, Vec<usize>>,
}

impl Constellations {
    pub fn add(&mut self, v: Vec4) {
        let mut my_constellations: Vec<usize> = Vec::new();
        for &(c, v2) in &self.points {
            let d = (v2 - v).manhattan();
            if d <= 3 {
                my_constellations.push(c);
            }
        }

        let id = self.points.len();
        debug!("Adding point {}: {:?}", id, v);
        if my_constellations.is_empty() {
            debug!("  New constellation {} with point {}", id, id);
            self.points.push((id, v));
            self.constellations.insert(id, vec![id]);
            return;
        }

        my_constellations.sort();
        my_constellations.dedup();

        let mn: usize = my_constellations[0];
        debug!("  Joining constellation {}", mn);

        for c in &my_constellations[1..] {
            debug!("  Merging constellation {} -> constellation {}", c, mn);
            // Merge constellations
            let mut merging = self.constellations.remove(&c).unwrap();
            // Update each point in the main vec
            for &vid in &merging {
                debug!("    Merging point {} -> constellation {}", vid, mn);
                let (_, vc) = self.points[vid];
                self.points[vid] = (mn, vc);
            }
            // Merge maps in the main hashmap
            let new_c = self.constellations.get_mut(&mn).unwrap();
            new_c.append(&mut merging);
        }

        // Add current point to the constellation
        debug!("  Adding point {} -> constellation {}", id, mn);
        let new_c = self.constellations.get_mut(&mn).unwrap();
        new_c.push(id);
        new_c.sort();
        // Push point onto the main stack
        self.points.push((mn, v));
    }
}

impl FromIterator<Vec4> for Constellations {
    fn from_iter<T: IntoIterator<Item = Vec4>>(iter: T) -> Self {
        let it = iter.into_iter();
        let (mn, mx) = it.size_hint();
        let sz = if let Some(m) = mx { m } else { mn };

        let mut constellations = Constellations {
            points: Vec::with_capacity(sz),
            constellations: HashMap::new(),
        };

        for v in it {
            constellations.add(v)
        }

        constellations
    }
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();

    let matches = App::new("Day 25")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day25.txt");

    debug!("Using input {}", input_path);
    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let points = parse_lines_err(Vec4::parse_line, buf_reader.lines())?;
    let c = Constellations::from_iter(points);

    println!("Found {} constellations", c.constellations.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use test_env_log::test;

    use super::*;

    use aoc::parse::parse_str;

    const INPUT1: &str = r#"
        0,0,0,0
        3,0,0,0
        0,3,0,0
        0,0,3,0
        0,0,0,3
        0,0,0,6
        9,0,0,0
        12,0,0,0
    "#;

    #[test]
    fn test_parse() {
        let pt = Vec4::parse_line("10,-20,30,-44").unwrap();
        assert_eq!(pt, Vec4(10, -20, 30, -44));

        let pts = parse_str(Vec4::parse_line, INPUT1).unwrap();
        assert_eq!(pts.len(), 8);
        assert_eq!(
            pts,
            vec!(
                Vec4(0, 0, 0, 0),
                Vec4(3, 0, 0, 0),
                Vec4(0, 3, 0, 0),
                Vec4(0, 0, 3, 0),
                Vec4(0, 0, 0, 3),
                Vec4(0, 0, 0, 6),
                Vec4(9, 0, 0, 0),
                Vec4(12, 0, 0, 0),
            )
        );
    }

    const MORE_INPUTS: &[(usize, &str)] = &[
        (
            4,
            r#"
                -1,2,2,0
                0,0,2,-2
                0,0,0,-2
                -1,2,0,0
                -2,-2,-2,2
                3,0,2,-1
                -1,3,2,2
                -1,0,-1,0
                0,2,1,-2
                3,0,0,0
        "#,
        ),
        (
            3,
            r#"
                1,-1,0,1
                2,0,-1,0
                3,2,-1,0
                0,0,3,1
                0,0,-1,-1
                2,3,-2,0
                -2,2,0,0
                2,-2,0,-1
                1,-1,0,-1
                3,2,0,2
            "#,
        ),
        (
            8,
            r#"
                1,-1,-1,-2
                -2,-2,0,1
                0,2,1,3
                -2,3,-2,1
                0,2,3,-2
                -1,-1,1,-2
                0,-2,-1,0
                -2,2,3,-1
                1,2,2,0
                -1,-2,0,-2
            "#,
        ),
    ];

    #[test]
    fn test_constellation_creation() {
        let pts = parse_str(Vec4::parse_line, INPUT1).unwrap();
        let mut c = Constellations::from_iter(pts);

        assert_eq!(c.constellations.len(), 2);

        c.add(Vec4(6, 0, 0, 0));
        assert_eq!(c.constellations.len(), 1);

        for &(n, s) in MORE_INPUTS {
            let pts = parse_str(Vec4::parse_line, s).unwrap();
            let c = Constellations::from_iter(pts);
            assert_eq!(c.constellations.len(), n);
        }
    }
}
