#![warn(clippy::all)]

#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;

#[derive(Default)]
struct Claims {
    non_overlaps: HashMap<usize, Claim>,
    claims: Vec<Claim>,
    overlaps: Vec<Rectangle>,
}

impl<'a, S: AsRef<str>> FromIterator<S> for Claims {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut c: Claims = Default::default();
        for l in iter {
            c.add_claim(Claim::from_line(l.as_ref()));
        }
        c
    }
}

impl Claims {
    fn add_overlap(&mut self, rect: Rectangle) {
        let mut queue = vec![rect];
        'outer: while let Some(r) = queue.pop() {
            for &o in &self.overlaps {
                if r.overlap(o).is_some() {
                    queue.extend(r.difference(o));
                    continue 'outer;
                }
            }
            self.overlaps.push(r);
        }
    }

    fn add_claim(&mut self, claim: Claim) {
        let mut overlapped: bool = false;
        let mut overlaps = vec![];
        for other in &self.claims {
            if let Some(o) = claim.rect.overlap(other.rect) {
                self.non_overlaps.remove(&other.id);
                overlapped = true;
                overlaps.push(o);
            }
        }

        for o in overlaps {
            self.add_overlap(o);
        }

        self.claims.push(claim);
        if !overlapped {
            self.non_overlaps.insert(claim.id, claim);
        }
    }

    fn overlap_area(&self) -> i64 {
        self.overlaps.iter().map(|o| o.area()).sum()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Rectangle {
    top: i16,
    bottom: i16,
    left: i16,
    right: i16,
}

impl fmt::Display for Rectangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "R({}-{}, {}-{})",
            self.left, self.right, self.top, self.bottom
        )
    }
}

impl Rectangle {
    fn new(left: i16, right: i16, top: i16, bottom: i16) -> Rectangle {
        Rectangle {
            left,
            right,
            top,
            bottom,
        }
    }

    fn area(self) -> i64 {
        i64::from(self.right - self.left) * i64::from(self.bottom - self.top)
    }

    fn overlap(self, other: Rectangle) -> Option<Rectangle> {
        let left = max(self.left, other.left);
        let right = min(self.right, other.right);
        if left >= right {
            return None;
        }
        let top = max(self.top, other.top);
        let bottom = min(self.bottom, other.bottom);
        if top >= bottom {
            return None;
        }

        Some(Rectangle::new(left, right, top, bottom))
    }

    fn difference(self, other: Rectangle) -> Vec<Rectangle> {
        let mut v = vec![];

        let xs = &[
            min(self.left, other.left),
            max(self.left, other.left),
            min(self.right, other.right),
            max(self.right, other.right),
        ];
        let ys = &[
            min(self.top, other.top),
            max(self.top, other.top),
            min(self.bottom, other.bottom),
            max(self.bottom, other.bottom),
        ];

        for (i, (&l, &r)) in xs.iter().zip(xs[1..].iter()).enumerate() {
            for (j, (&t, &b)) in ys.iter().zip(ys[1..].iter()).enumerate() {
                if i == 1 && j == 1 {
                    continue;
                }
                let rect = Rectangle::new(l, r, t, b);
                if l >= r || t >= b {
                    continue;
                }

                if self.overlap(rect).is_some() {
                    v.push(rect);
                }
            }
        }

        v
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug)]
struct Claim {
    id: usize,
    rect: Rectangle,
}

impl Claim {
    fn from_line(line: &str) -> Claim {
        lazy_static! {
            static ref re: regex::Regex =
                regex::Regex::new(r"^#(\d+) @ (\d+),(\d+): (\d+)x(\d+)$").unwrap();
        }

        let captures = re.captures(line).expect("Match not found");

        let parse = |i: usize| -> i16 {
            captures
                .get(i)
                .expect("Group 1 not found")
                .as_str()
                .parse()
                .expect("Couldn't parse group 1")
        };

        let id = parse(1) as usize;
        let x = parse(2);
        let y = parse(3);
        let w = parse(4);
        let h = parse(5);

        Claim {
            id,
            rect: Rectangle {
                top: y,
                bottom: y + h,
                left: x,
                right: x + w,
            },
        }
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 3")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day3.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let lines = buf_reader.lines().filter_map(|l| l.ok());
    let claims = Claims::from_iter(lines);

    println!("Overlap areas: {}", claims.overlap_area());
    for id in claims.non_overlaps.keys() {
        println!("No overlap: {}", id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        let first = Rectangle::new(1, 5, 3, 7);
        let second = Rectangle::new(3, 7, 1, 5);

        let o = first.overlap(second);

        assert_eq!(o, Some(Rectangle::new(3, 5, 3, 5)));
    }

    #[test]
    fn test_difference() {
        let first = Rectangle::new(1, 5, 3, 7);
        let second = Rectangle::new(3, 7, 1, 5);

        let mut v = first.difference(second);
        v.sort();

        let mut expected: Vec<Rectangle> = vec![
            Rectangle::new(1, 3, 3, 5),
            Rectangle::new(1, 3, 5, 7),
            Rectangle::new(3, 5, 5, 7),
        ];
        expected.sort();

        assert_eq!(expected, v);
    }

    #[test]
    fn test_overlap_area() {
        let inputs = vec![
            "#1 @ 1,3: 4x4",
            "#2 @ 3,1: 4x4",
            "#3 @ 5,5: 2x2",
            "#4 @ 3,3: 2x2",
        ];
        let claims = Claims::from_iter(inputs);
        assert_eq!(claims.overlap_area(), 4);
    }
}
