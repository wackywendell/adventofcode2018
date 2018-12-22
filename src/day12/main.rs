#![warn(clippy::all)]

use clap::{App, Arg};
use combine::parser::char as c_char;
use combine::stream::state::State;
use combine::Parser;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
enum Pot {
    Plant,
    Empty,
}

impl Pot {
    fn parser<I>() -> impl combine::Parser<Input = I, Output = Self>
    where
        I: combine::Stream<Item = char>,
        // Necessary due to rust-lang/rust#24159
        I::Error: combine::ParseError<I::Item, I::Range, I::Position>,
    {
        let p = c_char::char('#').map(|_| Pot::Plant);
        let e = c_char::char('.').map(|_| Pot::Empty);

        p.or(e)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct PropagationRule {
    input: [Pot; 5],
    output: Pot,
}

impl PropagationRule {
    fn parser<I>() -> impl combine::Parser<Input = I, Output = Self>
    where
        I: combine::Stream<Item = char>,
        // Necessary due to rust-lang/rust#24159
        I::Error: combine::ParseError<I::Item, I::Range, I::Position>,
    {
        let inputs = combine::parser::repeat::count_min_max(5, 5, Pot::parser())
            .map(|v: Vec<Pot>| [v[0], v[1], v[2], v[3], v[4]]);
        let sep = combine::parser::char::string(" => ");
        let output = Pot::parser();

        (inputs, sep, output).map(|(i, _, o)| PropagationRule {
            input: i,
            output: o,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Pots {
    pots: Vec<Pot>,
    rules: Vec<PropagationRule>,
    start: usize,
}

impl Pots {
    fn new(pots: Vec<Pot>, rules: Vec<PropagationRule>) -> Self {
        Pots {
            pots,
            rules,
            start: 0,
        }
    }

    fn parser<I>() -> impl combine::Parser<Input = I, Output = Self>
    where
        I: combine::Stream<Item = char>,
        I::Error: combine::ParseError<I::Item, I::Range, I::Position>,
    {
        let pot = Pot::parser();
        let pots = combine::parser::repeat::many1(pot);

        (
            c_char::string("initial state: ").with(pots),
            combine::parser::repeat::many1(c_char::spaces().with(PropagationRule::parser())),
        )
            .map(|(pots, rules)| Pots::new(pots, rules))
    }

    fn rule_tuple(&self, ix: usize) -> [Pot; 5] {
        let mut arr = [Pot::Empty; 5];
        for i in 0..5 {
            match self.pots.get(ix - 2 + i) {
                None => {}
                Some(p) => arr[i] = *p,
            }
        }

        arr
    }

    fn get_rule(&self, ix: usize) -> Option<PropagationRule> {
        let arr = self.rule_tuple(ix);
        for &rule in &self.rules {
            if rule.input == arr {
                return Some(rule);
            }
        }

        None
    }
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

    let mut contents = String::new();
    let mut file = File::open(input_path)?;
    file.read_to_string(&mut contents)?;
    let s: &str = contents.as_ref();
    let stream = State::new(s);

    let mut parser = c_char::spaces().with(Pots::parser());
    let (pots, _) = parser.easy_parse(stream).unwrap();

    println!(
        "Parsed {} pots and {} rules",
        pots.pots.len(),
        pots.rules.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#"
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

    #[test]
    fn test_parsing() {
        println!(r##"Test output: r#"{}"#"##, TEST_INPUT);

        let mut parser = c_char::spaces().with(Pots::parser());
        let stream = State::new(TEST_INPUT);

        let (pots, _) = parser.easy_parse(stream).unwrap();

        assert_eq!(pots.pots.len(), 25);
        assert_eq!(pots.rules.len(), 14);
    }
}
