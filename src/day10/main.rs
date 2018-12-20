#![warn(clippy::all)]

#[macro_use]
extern crate nom;

use clap::{App, Arg};

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Star {
    position: (i64, i64),
    velocity: (i64, i64),
}

named!(star_line_s<&str, Star>,
    do_parse!(
        tag!("position=<") >>
        x: ws!(aoc::parse_integer) >>
        tag!(",") >>
        y: ws!(aoc::parse_integer) >>
        tag!("> velocity=<") >>
        vx: ws!(aoc::parse_integer) >>
        tag!(",") >>
        vy: ws!(aoc::parse_integer) >>
        tag!(">") >>
        (Star{position: (x, y), velocity: (vx,vy)})
    )
);

fn convert_err<F>(err: nom::Err<&str, F>) -> nom::Err<String, F> {
    use nom::simple_errors::Context::Code;
    use nom::Err::{Error, Failure, Incomplete};
    match err {
        Incomplete(n) => Incomplete(n),
        Error(Code(s, ek)) => Error(Code(s.to_owned(), ek)),
        Failure(Code(s, ek)) => Failure(Code(s.to_owned(), ek)),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Stars(Vec<Star>);

impl Stars {
    fn parse_lines<S, E, T>(iter: T) -> Result<Self, failure::Error>
    where
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        let maybe_points: Result<Vec<Star>, failure::Error> = iter
            .into_iter()
            .map(|l| {
                let p: Result<Star, failure::Error> = match l {
                    Ok(s) => star_line_s(s.as_ref())
                        .map(|(_, s)| s)
                        .map_err(|e| convert_err(e).into()),
                    Err(err) => Err(err.into()),
                };
                p
            })
            .collect();

        Ok(Stars(maybe_points?))
    }

    fn advance(&mut self, time: i64) {
        for s in self.0.iter_mut() {
            let (x, y) = s.position;
            let (vx, vy) = s.velocity;
            s.position = (x + vx * time, y + vy * time);
        }
    }

    fn step(&mut self) {
        self.advance(1)
    }

    fn minimals(&self) -> Option<(i64, i64, i64, i64)> {
        let Stars(ref stars) = self;
        if stars.is_empty() {
            return None;
        }
        let (sx, sy) = stars[0].position;
        let (mut x_min, mut x_max, mut y_min, mut y_max) = (sx, sx, sy, sy);
        for s in stars {
            let (x, y) = s.position;
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }

        Some((x_min, x_max, y_min, y_max))
    }

    fn area(&self) -> i64 {
        let Stars(ref stars) = self;
        if stars.len() <= 1 {
            return 1;
        }
        let (x_min, x_max, y_min, y_max) = self.minimals().unwrap();

        (x_max + 1 - x_min) * (y_max + 1 - y_min)
    }

    fn minimize(&mut self) -> i64 {
        let mut last = self.area();
        let mut steps = 0;
        loop {
            self.step();
            steps += 1;
            let area = self.area();
            if area >= last {
                break;
            }
            last = area;
        }
        steps -= 1;
        self.advance(-1);

        steps
    }

    fn to_strings(&self) -> Vec<String> {
        if self.0.is_empty() {
            return vec![];
        }
        let (x_min, x_max, y_min, y_max) = self.minimals().unwrap();
        let (w, h) = ((x_max - x_min + 1) as usize, (y_max - y_min + 1) as usize);

        let row: String = std::iter::repeat('.').take(w).collect();

        let mut strings: Vec<String> = std::iter::repeat(row).take(h).collect();
        for s in &self.0 {
            let (x, y) = s.position;
            let (row, col) = ((x - x_min) as usize, (y - y_min) as usize);
            strings[col].replace_range(row..=row, "#");
        }
        strings
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 10")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day10.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let mut stars = Stars::parse_lines(buf_reader.lines())?;

    println!("Found stars: {}", stars.0.len());
    let steps = stars.minimize();
    println!("Minimized after {} steps:", steps);
    for s in &stars.to_strings() {
        println!("{}", s);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_STARS: [&str; 31] = [
        "position=< 9,  1> velocity=< 0,  2>",
        "position=< 7,  0> velocity=<-1,  0>",
        "position=< 3, -2> velocity=<-1,  1>",
        "position=< 6, 10> velocity=<-2, -1>",
        "position=< 2, -4> velocity=< 2,  2>",
        "position=<-6, 10> velocity=< 2, -2>",
        "position=< 1,  8> velocity=< 1, -1>",
        "position=< 1,  7> velocity=< 1,  0>",
        "position=<-3, 11> velocity=< 1, -2>",
        "position=< 7,  6> velocity=<-1, -1>",
        "position=<-2,  3> velocity=< 1,  0>",
        "position=<-4,  3> velocity=< 2,  0>",
        "position=<10, -3> velocity=<-1,  1>",
        "position=< 5, 11> velocity=< 1, -2>",
        "position=< 4,  7> velocity=< 0, -1>",
        "position=< 8, -2> velocity=< 0,  1>",
        "position=<15,  0> velocity=<-2,  0>",
        "position=< 1,  6> velocity=< 1,  0>",
        "position=< 8,  9> velocity=< 0, -1>",
        "position=< 3,  3> velocity=<-1,  1>",
        "position=< 0,  5> velocity=< 0, -1>",
        "position=<-2,  2> velocity=< 2,  0>",
        "position=< 5, -2> velocity=< 1,  2>",
        "position=< 1,  4> velocity=< 2,  1>",
        "position=<-2,  7> velocity=< 2, -2>",
        "position=< 3,  6> velocity=<-1, -1>",
        "position=< 5,  0> velocity=< 1,  0>",
        "position=<-6,  0> velocity=< 2,  0>",
        "position=< 5,  9> velocity=< 1, -2>",
        "position=<14,  7> velocity=<-2,  0>",
        "position=<-3,  6> velocity=< 2, -1>",
    ];

    #[test]
    fn test_parse_star() {
        let parsed = star_line_s("position=< 9,  1> velocity=< 0,  2>");
        println!("Parsed: {:?}", parsed);
        let s = Star {
            position: (9, 1),
            velocity: (0, 2),
        };
        assert_eq!(parsed, Ok(("", s)));
    }

    #[test]
    fn test_parse_stars() {
        let parsed = Stars::parse_lines::<_, failure::Error, _>(TEST_STARS.iter().map(Ok));
        println!("Parsed: {:?}", parsed);
        let mut stars = parsed.expect("Parse error");
        let first = Star {
            position: (9, 1),
            velocity: (0, 2),
        };
        let second = Star {
            position: (7, 0),
            velocity: (-1, 0),
        };
        let last = Star {
            position: (-3, 6),
            velocity: (2, -1),
        };

        {
            let Stars(ref star_v) = stars;
            assert_eq!(star_v.len(), TEST_STARS.len());
            assert_eq!(star_v[0], first);
            assert_eq!(star_v[1], second);
            assert_eq!(star_v[star_v.len() - 1], last);
        }

        assert_eq!(stars.area(), 22 * 16);
        stars.step();
        assert_eq!(stars.area(), 18 * 12);
        stars.advance(-1);
        assert_eq!(stars.area(), 22 * 16);

        let steps = stars.minimize();
        assert_eq!(steps, 3);
        assert_eq!(stars.area(), 10 * 8);
    }

    #[test]
    fn test_stars_drawn() {
        let parsed = Stars::parse_lines::<_, failure::Error, _>(TEST_STARS.iter().map(Ok));
        println!("Parsed: {:?}", parsed);
        let mut stars = parsed.expect("Parse error");
        stars.minimize();
        let strung = stars.to_strings();
        for s in &strung {
            println!("{}", s);
        }
        assert_eq!(
            strung,
            vec![
                "#...#..###",
                "#...#...#.",
                "#...#...#.",
                "#####...#.",
                "#...#...#.",
                "#...#...#.",
                "#...#...#.",
                "#...#..###",
            ]
        );
    }
}
