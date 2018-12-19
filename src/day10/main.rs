#![warn(clippy::all)]

#[macro_use]
extern crate nom;

use clap::{App, Arg};
use nom::types::CompleteStr;
use nom::digit;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::str::FromStr;

named!(integer<CompleteStr, i64>,
    map_res!(
        dbg!(recognize!(pair!(
            dbg!(opt!(alt!(tag_s!("+") | tag_s!("-")))),  // maybe sign?
            dbg_dmp!(digit)
        ))),
        |s:CompleteStr| { FromStr::from_str(s.0) }
    )
);

named!(integer_s<&str, i64>,
    map_res!(
        dbg!(recognize!(pair!(
            dbg!(opt!(alt!(tag_s!("+") | tag_s!("-")))),  // maybe sign?
            dbg_dmp!(digit)
        ))),
        FromStr::from_str
    )
);

#[derive(Debug,Clone,PartialEq,PartialOrd)]
struct Star {
    position: (i64, i64),
    velocity: (i64, i64),
}

named!(star_line<CompleteStr, Star>,
    do_parse!(
        tag!("position=<") >>
        x: ws!(integer) >>
        tag!(",") >>
        y: ws!(integer) >>
        tag!("> velocity=<") >>
        vx: ws!(integer) >>
        tag!(",") >>
        vy: ws!(integer) >>
        tag!(">") >>
        (Star{position: (x, y), velocity: (vx,vy)})
    )
);

named!(star_line_s<&str, Star>,
    do_parse!(
        tag!("position=<") >>
        x: ws!(integer_s) >>
        tag!(",") >>
        y: ws!(integer_s) >>
        tag!("> velocity=<") >>
        vx: ws!(integer_s) >>
        tag!(",") >>
        vy: ws!(integer_s) >>
        tag!(">") >>
        (Star{position: (x, y), velocity: (vx,vy)})
    )
);

fn convert_err<F>(err: nom::Err<&str, F>) -> nom::Err<String, F>{
    use nom::simple_errors::Context::Code;
    use nom::Err::{Incomplete, Error, Failure};
    match err {
        Incomplete(n) => Incomplete(n),
        Error(Code(s, ek)) => Error(Code(s.to_owned(), ek)),
        Failure(Code(s, ek)) => Failure(Code(s.to_owned(), ek)),
    }
}

struct Stars(Vec<Star>);

impl Stars {
    fn parse_line(s: &str) -> Result<Star, nom::Err<String, u32>> {
        star_line_s(s).map(|(_, s)| s).map_err(convert_err)
    }

    fn parse_lines<'a, S, E, T>(iter: T) -> Result<Self, failure::Error>
    where
        S: 'a,
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        let maybe_points: Result<Vec<Star>, failure::Error> = iter
            .into_iter()
            .map(|l| {
                let p: Result<Star, failure::Error> = match l {
                    Ok(s) => star_line_s(s.as_ref()).map(|(_, s)| s).map_err(|e| convert_err(e).into()),
                    Err(err) => Err(err.into()),
                };
                p
            })
            .collect();

        Ok(Stars(maybe_points?))
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 1")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day1.txt");

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

    #[test]
    fn test_integer_parse() {
        let parsed = integer(CompleteStr("-120"));
        println!("Parsed: {:?}", parsed);
        assert_eq!(parsed, Ok((CompleteStr(""), -120)));
    }

    #[test]
    fn test_integer_positive() {
        let parsed = integer(CompleteStr("+120"));
        println!("Parsed: {:?}", parsed);
        assert_eq!(parsed, Ok((CompleteStr(""), 120)));
    }

    #[test]
    fn test_parse_star() {
        let parsed = star_line(CompleteStr("position=< 9,  1> velocity=< 0,  2>"));
        println!("Parsed: {:?}", parsed);
        let s = Star{position: (9, 1), velocity: (0, 2)};
        assert_eq!(parsed, Ok((CompleteStr(""), s)));
    }
}