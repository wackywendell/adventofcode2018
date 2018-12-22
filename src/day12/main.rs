#![warn(clippy::all)]

use clap::{App, Arg};
use combine::parser::char as c_char;
use combine::stream::state::State;
use combine::Parser;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq)]
enum Pot {
    Plant,
    Empty,
}

impl Pot {
    fn full(self) -> bool {
        self == Pot::Plant
    }

    fn as_char(self) -> char {
        match self {
            Pot::Empty => '.',
            Pot::Plant => '#',
        }
    }

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

#[derive(Debug, Clone)]
struct Pots {
    pots: VecDeque<Pot>,
    rules: HashMap<[Pot; 5], Pot>,
    start: usize,
}

impl Pots {
    fn new<P, R>(pots: P, rules: R) -> Self
    where
        P: Iterator<Item = Pot>,
        R: Iterator<Item = PropagationRule>,
    {
        let rule_map = HashMap::from_iter(rules.map(|r| (r.input, r.output)));
        Pots {
            pots: pots.collect(),
            rules: rule_map,
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
            .map(|(pots, rules)| {
                let _: Vec<Pot> = pots;
                let _: Vec<PropagationRule> = rules;
                Pots::new(pots.into_iter(), rules.into_iter())
            })
    }

    fn rule_tuple(&self, ix: isize) -> [Pot; 5] {
        let mut arr = [Pot::Empty; 5];
        fn get(ps: &VecDeque<Pot>, j: isize) -> Pot {
            if j >= 0 && j < ps.len() as isize {
                ps[j as usize]
            } else {
                Pot::Empty
            }
        }
        for i in -2..=2isize {
            arr[(i + 2) as usize] = get(&self.pots, ix + i)
        }
        // println!(
        //     "ix: {} -> {}{}{}{}{}",
        //     ix,
        //     arr[0].as_char(),
        //     arr[1].as_char(),
        //     arr[2].as_char(),
        //     arr[3].as_char(),
        //     arr[4].as_char()
        // );

        arr
    }

    fn get_rule(&self, ix: isize) -> Pot {
        let arr = self.rule_tuple(ix);
        *self.rules.get(&arr).unwrap_or(&Pot::Empty)
    }

    fn advance(&mut self) {
        // Find the first two
        let first_pair = [self.get_rule(-2), self.get_rule(-1)];
        let mut last_pair = first_pair;
        let ln = self.pots.len() as isize;

        // transform the existing ones
        for ix in 0..ln + 2 {
            let transformed = self.get_rule(ix);
            if ix >= 2 {
                self.pots[(ix - 2) as usize] = last_pair[0];
            }
            last_pair = [last_pair[1], transformed];
        }

        // Add the first two if necessary
        if first_pair[0].full() || first_pair[1].full() {
            self.pots.push_front(first_pair[1]);
            self.start += 1;
        }
        if first_pair[0].full() {
            self.pots.push_front(first_pair[0]);
            self.start += 1;
        }

        // And append the last two, if necessary
        if last_pair[0].full() || last_pair[1].full() {
            self.pots.push_back(last_pair[0]);
        }
        if last_pair[1].full() {
            self.pots.push_back(last_pair[1]);
        }
    }
}

impl std::fmt::Display for Pots {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for p in self.pots.iter().take(self.start) {
            write!(f, "{}", p.as_char())?;
        }
        write!(f, "|")?;
        for p in self.pots.iter().skip(self.start) {
            write!(f, "{}", p.as_char())?;
        }

        Ok(())
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

    #[test]
    fn test_advance() {
        println!(r##"Test output: r#"{}"#"##, TEST_INPUT);

        let mut parser = c_char::spaces().with(Pots::parser());
        let stream = State::new(TEST_INPUT);
        let (mut pots, _) = parser.easy_parse(stream).unwrap();
        for i in 0..20 {
            println!("Pots {:2}: {}", i, pots);
            pots.advance();
        }

        let state_str = format!("{}", pots);
        assert_eq!(state_str, "#.|...##....#####...#######....#.#..##")
    }
}
