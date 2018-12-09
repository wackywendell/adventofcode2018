#![warn(clippy::all)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure;

use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
struct Point(i64, i64);

#[derive(Clone, Debug, Fail)]
enum ParseError {
    #[fail(display = "invalid line: {}", line)]
    LineError { line: String },
    #[fail(display = "invalid match for {}: {}", part, line)]
    MatchError { part: String, line: String },
}

impl ParseError {
    fn from_line<S: ToString>(s: S) -> ParseError {
        ParseError::LineError {
            line: s.to_string(),
        }
    }

    fn from_part<P: ToString, L: ToString>(part: P, line: L) -> ParseError {
        ParseError::MatchError {
            part: part.to_string(),
            line: line.to_string(),
        }
    }
}

impl FromStr for Point {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref re: regex::Regex = regex::Regex::new(r"^(\d+),\s*(\d+)$").unwrap();
        }

        let c = re.captures(s).ok_or_else(|| ParseError::from_line(s))?;

        let x = c
            .get(1)
            .ok_or_else(|| ParseError::from_line(s))?
            .as_str()
            .parse::<i64>()
            .or_else(|m| Err(ParseError::from_part(m, s)))?;
        let y = c
            .get(2)
            .ok_or_else(|| ParseError::from_line(s))?
            .as_str()
            .parse::<i64>()
            .or_else(|m| Err(ParseError::from_part(m, s)))?;

        Ok(Point(x, y))
    }
}

struct Points(Vec<Point>);

impl Points {
    fn parse_lines<S, E, T>(iter: T) -> Result<Self, failure::Error>
    where
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        let maybe_points: Result<Vec<Point>, failure::Error> = iter
            .into_iter()
            .map(|l| {
                let p: Result<Point, failure::Error> = match l {
                    Ok(s) => Point::from_str(s.as_ref()).map_err(|e| e.into()),
                    Err(e) => Err(e.into()),
                };
                p
            })
            .collect();

        Ok(Points(maybe_points?))
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 6")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day6.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let Points(points) = Points::parse_lines(buf_reader.lines())?;

    println!("Points: {}", points.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_from_str() {
        let p = Point::from_str("112, 3");
        assert_eq!(Point(112, 3), p.unwrap());
    }
}
