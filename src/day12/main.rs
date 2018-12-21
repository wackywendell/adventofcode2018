#![warn(clippy::all)]

#[macro_use]
extern crate nom;
#[macro_use]
extern crate failure;

use clap::{App, Arg};
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum Pot {
    Plant,
    Empty,
}

struct PropagationRule {
    input: [Pot; 5],
    output: Pot,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Pots {
    pots: Vec<Pot>,
}

named!(parse_pot<&str, Pot>,
    alt!(value!(Pot::Plant, tag!("#")) | value!(Pot::Empty, tag!(".")))
);

named!(
    parse_input<&str, Pots>,
    do_parse!(
        tag!("initial state: ") >>
        pots: many1!(parse_pot) >>
        (Pots{pots})
    )
);

named!(
    parse_rule<&str, PropagationRule>,
    do_parse!(
        p0: parse_pot >>
        p1: parse_pot >>
        p2: parse_pot >>
        p3: parse_pot >>
        p4: parse_pot >>
        tag!(" => ") >>
        output: parse_pot >>
        (PropagationRule{input: [p0,p1,p2,p3,p4], output})
    )
);

fn parse_lines<S, E, T>(iter: T) -> Result<(Pots, Vec<PropagationRule>), failure::Error>
where
    S: AsRef<str>,
    E: Send + Sync + Into<failure::Error>,
    T: IntoIterator<Item = Result<S, E>>,
{
    let mut it = iter.into_iter();

    let pots = match it.next() {
        None => return Err(format_err!("No initial line")),
        Some(Err(e)) => {
            let err = e.into();
            println!("Failed to parse pots from {}", err);
            return Err(err);
        }
        Some(Ok(l)) => {
            let s: &str = l.as_ref();
            println!("Parsing pots from {}", l.as_ref());
            let pots: Pots = parse_input(s).map_err(aoc::convert_err)?.1;
            pots
        }
    };

    println!("Parsed pots: {:?}", pots);

    let mut rules = vec![];
    for l in it.next() {
        match l {
            Err(e) => return Err(e.into()),
            Ok(s) => {
                let rule = parse_rule(s.as_ref()).map_err(aoc::convert_err)?.1;
                rules.push(rule);
            }
        }
    }

    Ok((pots, rules))
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 12")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day12.txt");

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

    const TEST_INPUT: &'static str = r#"
initial state: #..#.#..##......###...###

...## => #
..#.. => #
.#... => #
.#.#. => #
.#.## => #
.##.. => #
.#### => #
#.#.# => #
#.### => #
##.#. => #
##.## => #
###.. => #
###.# => #
####. => #"#;

    fn get_test_lines() -> Vec<Result<&'static str, failure::Error>> {
        TEST_INPUT.split('\n').skip(1).map(Ok).collect()
    }

    #[test]
    fn test_parsing() {
        println!(r##"Test output: r#"{}"#"##, TEST_INPUT);

        let lines = get_test_lines();

        let (pots, rules) = parse_lines(lines).unwrap();

        assert_eq!(pots.pots.len(), 10);
        assert_eq!(rules.len(), 10);
    }
}
