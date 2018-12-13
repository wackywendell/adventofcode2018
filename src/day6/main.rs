#![warn(clippy::all)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure;

use clap::{App, Arg};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug, Default)]
struct Point(i64, i64);

#[derive(Clone, Debug, Fail)]
enum ParseError {
    #[fail(display = "invalid line: {}", line)]
    LineError { line: String },
    #[fail(display = "invalid match for {}: {}", part, line)]
    MatchError { part: String, line: String },
}

impl ParseError {
    fn from_line<S: ToString>(s: &S) -> ParseError {
        ParseError::LineError {
            line: s.to_string(),
        }
    }

    fn from_part<P: ToString, L: ToString>(part: &P, line: &L) -> ParseError {
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

        let c = re.captures(s).ok_or_else(|| ParseError::from_line(&s))?;

        let x = c
            .get(1)
            .ok_or_else(|| ParseError::from_line(&s))?
            .as_str()
            .parse::<i64>()
            .or_else(|m| Err(ParseError::from_part(&m, &s)))?;
        let y = c
            .get(2)
            .ok_or_else(|| ParseError::from_line(&s))?
            .as_str()
            .parse::<i64>()
            .or_else(|m| Err(ParseError::from_part(&m, &s)))?;

        Ok(Point(x, y))
    }
}

impl Point {
    fn manhattan(self, other: Point) -> i64 {
        let Point(x, y) = self;
        let Point(x2, y2) = other;

        (x - x2).abs() + (y - y2).abs()
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

    fn find_closest(&self, p: Point) -> Option<Point> {
        let Points(ref ps) = self;

        let (mut d, mut closest): (i64, Option<Point>) = (-1, None);
        for &p2 in ps {
            if closest.is_none() && d < 0 {
                closest = Some(p2);
                d = p.manhattan(p2);
                continue;
            }
            let d2 = p2.manhattan(p);

            match d2.cmp(&d) {
                std::cmp::Ordering::Greater => continue,
                std::cmp::Ordering::Equal => closest = None,
                std::cmp::Ordering::Less => {
                    d = d2;
                    closest = Some(p2);
                }
            }
        }

        closest
    }

    fn count_distances(&self) -> HashMap<Point, Option<i64>> {
        let mut h = HashMap::new();
        let Points(ref ps) = self;
        if ps.is_empty() {
            return h;
        }

        let Point(x0, y0) = ps[0];

        let (mut minx, mut maxx, mut miny, mut maxy): (i64, i64, i64, i64) = (x0, x0, y0, y0);

        for &Point(x, y) in ps {
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
        }

        for x in minx..=maxx {
            for y in miny..=maxy {
                let p = match self.find_closest(Point(x, y)) {
                    None => continue,
                    Some(p) => p,
                };

                let is_edge = x == minx || x == maxx || y == miny || y == maxy;

                if is_edge {
                    h.insert(p, None);
                    continue;
                }
                h.entry(p)
                    .and_modify(|o| *o = o.map(|n| n + 1))
                    .or_insert(Some(1));
            }
        }

        h
    }

    fn find_area(&self, distance: i64) -> i64 {
        let Points(ref ps) = self;
        if ps.is_empty() {
            return 0;
        }

        let Point(x0, y0) = ps[0];

        let (mut minx, mut maxx, mut miny, mut maxy): (i64, i64, i64, i64) = (x0, x0, y0, y0);

        for &Point(x, y) in ps {
            minx = minx.min(x);
            maxx = maxx.max(x);
            miny = miny.min(y);
            maxy = maxy.max(y);
        }

        let point_count = ps.len();
        // Any point more than max_reach from an "outermost" point
        // can't possibly be within a total distance of `distance`
        // from all points.
        // This is an overestimate - we could cut this down to something
        // more circular - but there's no need.
        let max_reach = distance / (point_count as i64);

        let mut area: i64 = 0;
        for x in minx - max_reach..=maxx + max_reach {
            for y in miny - max_reach..=maxy + max_reach {
                let total_distance: i64 = ps.iter().map(|p| p.manhattan(Point(x, y))).sum();

                if total_distance >= distance {
                    continue;
                }
                area += 1;
            }
        }

        area
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

    let points = Points::parse_lines(buf_reader.lines())?;

    let ds = points.count_distances();
    let max_a = ds.values().filter_map(|&v| v).max();
    match max_a {
        None => println!("Max area: Not found"),
        Some(a) => println!("Max area: {}", a),
    }

    let total_a = points.find_area(10000);
    println!("Total area: {}", total_a);

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

    fn str_ok(s: &str) -> Result<&str, failure::Error> {
        Ok(s)
    }

    #[test]
    fn test_area() {
        let test_input = vec!["1, 1", "1, 6", "8, 3", "3, 4", "5, 5", "8, 9"];

        let points = Points::parse_lines(test_input.iter().map(|&s| str_ok(s))).unwrap();
        let ds = points.count_distances();
        let max_a = ds.values().filter_map(|&v| v).max();
        assert_eq!(Some(17), max_a);

        let total_a = points.find_area(32);
        assert_eq!(16, total_a);
    }
}
