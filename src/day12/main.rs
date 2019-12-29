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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PotState {
    pots: VecDeque<Pot>,
    start: isize,
}

impl PotState {
    fn index_sum(&self) -> i64 {
        self.pots
            .iter()
            .enumerate()
            .filter_map(|(ix, p)| {
                if p.full() {
                    Some((ix as i64) - (self.start as i64))
                } else {
                    None
                }
            })
            .sum()
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

        arr
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pots {
    state: PotState,
    rules: HashMap<[Pot; 5], Pot>,
}

impl Pots {
    fn new<P, R>(pots: P, rules: R) -> Self
    where
        P: Iterator<Item = Pot>,
        R: Iterator<Item = PropagationRule>,
    {
        let rule_map = HashMap::from_iter(rules.map(|r| (r.input, r.output)));
        Pots {
            state: PotState {
                pots: pots.collect(),
                start: 0,
            },
            rules: rule_map,
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

    fn get_rule(&self, ix: isize) -> Pot {
        let arr = self.state.rule_tuple(ix);
        *self.rules.get(&arr).unwrap_or(&Pot::Empty)
    }

    fn advance(&mut self) {
        // Find the first two
        let first_pair = [self.get_rule(-2), self.get_rule(-1)];
        let mut last_pair = first_pair;
        let ln = self.state.pots.len() as isize;

        // transform the existing ones
        for ix in 0..ln + 2 {
            let transformed = self.get_rule(ix);
            if ix >= 2 {
                self.state.pots[(ix - 2) as usize] = last_pair[0];
            }
            last_pair = [last_pair[1], transformed];
        }

        // Add the first two if necessary
        if first_pair[0].full() || first_pair[1].full() {
            self.state.pots.push_front(first_pair[1]);
            self.state.start += 1;
        }
        if first_pair[0].full() {
            self.state.pots.push_front(first_pair[0]);
            self.state.start += 1;
        }

        // And append the last two, if necessary
        if last_pair[0].full() || last_pair[1].full() {
            self.state.pots.push_back(last_pair[0]);
        }
        if last_pair[1].full() {
            self.state.pots.push_back(last_pair[1]);
        }

        // Pop any empty ones from the end
        while let Some(&p) = self.state.pots.back() {
            if p.full() {
                break;
            }
            self.state.pots.pop_back();
        }

        // Pop any empty ones from the beginning
        while let Some(&p) = self.state.pots.front() {
            if p.full() {
                break;
            }
            self.state.pots.pop_front();
            self.state.start -= 1;
        }
    }
}

impl std::fmt::Display for PotState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for p in self.pots.iter().take(self.start.max(0) as usize) {
            write!(f, "{}", p.as_char())?;
        }
        write!(f, "|")?;
        if self.start < 0 {
            write!(f, "({})", -self.start)?;
        }
        // for _ in 0..(-self.start.min(0)) {
        //     write!(f, "{}", Pot::Empty.as_char())?;
        // }
        for p in self.pots.iter().skip(self.start.max(0) as usize) {
            write!(f, "{}", p.as_char())?;
        }

        Ok(())
    }
}

struct PotAdvancer {
    pots: Pots,
    // Pots -> (start, generation)
    seen: HashMap<VecDeque<Pot>, (isize, isize)>,
    repeats: Vec<PotState>,
    // Start_shift, first generation
    first: Option<(isize, isize)>,
    index: isize,
}

impl PotAdvancer {
    fn new(p: Pots) -> Self {
        let mut seen = HashMap::new();
        seen.insert(p.state.pots.clone(), (p.state.start, 0));
        PotAdvancer {
            pots: p.clone(),
            seen,
            repeats: vec![p.state],
            first: None,
            index: 0,
        }
    }

    fn advance(&mut self, dist: isize) {
        let target_ix = self.index + dist;
        while self.index != target_ix && self.first.is_none() {
            self.simple_step();
        }
        if self.index == target_ix {
            return;
        }

        self.index = target_ix;
        let (start_shift, first_gen) = self.first.unwrap();
        let len = self.repeats.len() as isize;
        let skipped = (self.index - first_gen) / len;
        let r_ix = (self.index - first_gen) % len;
        let mut state = self.repeats[r_ix as usize].clone();
        state.start += skipped * start_shift;
        self.pots.state = state;
    }

    fn simple_step(&mut self) {
        self.index += 1;
        self.pots.advance();
        let state = self.pots.state.clone();

        let (start_ix, generation): (isize, isize) = match self.seen.entry(state.pots) {
            std::collections::hash_map::Entry::Vacant(v) => {
                v.insert((self.pots.state.start, self.index));
                self.repeats.push(self.pots.state.clone());
                return;
            }
            std::collections::hash_map::Entry::Occupied(o) => *o.get(),
        };

        let shift = self.pots.state.start - start_ix;
        println!(
            "Found repeat at indices {} - {} with shift {}",
            generation, self.index, shift,
        );

        let just_repeats = Vec::from_iter(self.repeats.drain((generation as usize)..));
        self.repeats = just_repeats;
        self.first = Some((shift, generation));
        self.seen.clear();
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
    let (mut pots, _) = parser.easy_parse(stream).unwrap();

    println!(
        "Parsed {} pots and {} rules",
        pots.state.pots.len(),
        pots.rules.len()
    );

    for _ in 0..20 {
        pots.advance();
        println!("{}", pots.state);
    }
    println!("Index sum: {}", pots.state.index_sum());

    let mut a = PotAdvancer::new(pots);

    a.advance(50_000_000_000 - 20);
    println!("Index sum: {}", a.pots.state.index_sum());

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

        assert_eq!(pots.state.pots.len(), 25);
        assert_eq!(pots.rules.len(), 14);
    }

    #[test]
    fn test_advance() {
        println!(r##"Test output: r#"{}"#"##, TEST_INPUT);

        let mut parser = c_char::spaces().with(Pots::parser());
        let stream = State::new(TEST_INPUT);
        let (mut pots, _) = parser.easy_parse(stream).unwrap();
        for i in 0..20 {
            println!("Pots {:2}: {}", i, pots.state);
            pots.advance();
        }

        let state_str = format!("{}", pots.state);
        assert_eq!(state_str, "#.|...##....#####...#######....#.#..##");
        assert_eq!(pots.state.index_sum(), 325);
    }
}
