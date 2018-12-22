#![warn(clippy::all)]

use clap::{App, Arg};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
enum Track {
    Intersection,
    DiagonalUp,
    DiagonalDown,
    Vertical,
    Horizontal,
}

impl Track {
    fn as_char(self) -> char {
        match self {
            Track::Intersection => '+',
            Track::DiagonalUp => '/',
            Track::DiagonalDown => '\\',
            Track::Vertical => '|',
            Track::Horizontal => '-',
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn left(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
    fn right(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
enum Turn {
    Left,
    Right,
    Straight,
}

impl Turn {
    fn next(self) -> Self {
        match self {
            Turn::Left => Turn::Straight,
            Turn::Straight => Turn::Right,
            Turn::Right => Turn::Left,
        }
    }

    fn apply(self, dir: Direction) -> Direction {
        match self {
            Turn::Straight => dir,
            Turn::Left => dir.left(),
            Turn::Right => dir.right(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
struct Cart {
    loc: (i64, i64),
    direction: Direction,
    next_turn: Turn,
}

impl Cart {
    fn new(loc: (i64, i64), direction: Direction, next_turn: Turn) -> Self {
        Cart {
            loc,
            direction,
            next_turn,
        }
    }

    fn as_char(self) -> char {
        match self.direction {
            Direction::Up => '^',
            Direction::Down => 'v',
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }

    fn step(&mut self) {
        let (x, y) = self.loc;
        self.loc = match self.direction {
            Direction::Up => (x, y - 1),
            Direction::Down => (x, y + 1),
            Direction::Left => (x - 1, y),
            Direction::Right => (x + 1, y),
        };
    }

    fn turn(&mut self, track: Track) {
        match (self.direction, track) {
            (Direction::Up, Track::Vertical) => {}
            (Direction::Down, Track::Vertical) => {}
            (d, Track::Vertical) => panic!("Can't move {:?} on a vertical track!", d),
            (Direction::Left, Track::Horizontal) => {}
            (Direction::Right, Track::Horizontal) => {}
            (d, Track::Horizontal) => panic!("Can't move {:?} on a sideways track!", d),
            //  ^
            // >/
            (Direction::Right, Track::DiagonalUp) => self.direction = Direction::Up,
            //  v
            // </
            (Direction::Down, Track::DiagonalUp) => self.direction = Direction::Left,
            // /<
            // v
            (Direction::Left, Track::DiagonalUp) => self.direction = Direction::Down,
            // />
            // ^
            (Direction::Up, Track::DiagonalUp) => self.direction = Direction::Right,
            // />
            // ^
            (Direction::Left, Track::DiagonalDown) => self.direction = Direction::Up,
            (Direction::Down, Track::DiagonalDown) => self.direction = Direction::Right,
            (Direction::Up, Track::DiagonalDown) => self.direction = Direction::Left,
            (Direction::Right, Track::DiagonalDown) => self.direction = Direction::Down,
            (d, Track::Intersection) => {
                self.direction = self.next_turn.apply(d);
                self.next_turn = self.next_turn.next();
            }
        }
    }
}

#[derive(Debug)]
struct Railway {
    tracks: HashMap<(i64, i64), Track>,
    carts: Vec<Cart>,
}

impl Railway {
    fn parse_lines<S, E, T>(iter: T) -> Result<Self, failure::Error>
    where
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        let mut tracks = HashMap::new();
        let mut carts = Vec::new();

        for (y, l) in iter.into_iter().enumerate() {
            let line_ref = l.map_err(Into::into)?;
            let line = line_ref.as_ref();
            for (x, c) in line.chars().enumerate() {
                let (cart_dir, track) = match c {
                    ' ' => continue,
                    '-' => (None, Track::Horizontal),
                    '|' => (None, Track::Vertical),
                    '/' => (None, Track::DiagonalUp),
                    '\\' => (None, Track::DiagonalDown),
                    '+' => (None, Track::Intersection),
                    '<' => (Some(Direction::Left), Track::Horizontal),
                    '>' => (Some(Direction::Right), Track::Horizontal),
                    '^' => (Some(Direction::Up), Track::Vertical),
                    'v' => (Some(Direction::Down), Track::Vertical),
                    _ => panic!("Character {} Not Recognized", c),
                };

                // println!(
                //     "Found ({}, {}) at ({},{})",
                //     cart_dir
                //         .map(|d| Cart::new((0, 0), d, Turn::Left).as_char())
                //         .unwrap_or('.'),
                //     track.as_char(),
                //     x,
                //     y
                // );

                let loc = (x as i64, y as i64);
                tracks.insert(loc, track);
                if let Some(dir) = cart_dir {
                    let cart = Cart::new(loc, dir, Turn::Left);
                    carts.push(cart);
                }
            }
        }

        Ok(Railway { tracks, carts })
    }

    fn step(&mut self) -> Vec<(i64, i64)> {
        self.carts.sort();
        // location -> cart index
        let mut occupied: HashMap<(i64, i64), usize> = HashMap::with_capacity(self.carts.len());
        let mut to_remove: HashSet<usize> = HashSet::new();

        for (i, c) in self.carts.iter().enumerate() {
            if let Some(_j) = occupied.insert(c.loc, i) {
                panic!("Collision not removed at ({}, {})", c.loc.0, c.loc.1);
            }
        }

        let mut collisions = Vec::new();
        for (i, c) in self.carts.iter_mut().enumerate() {
            occupied.remove(&c.loc);
            c.step();
            if let Some(j) = occupied.insert(c.loc, i) {
                to_remove.insert(i);
                to_remove.insert(j);
                collisions.push(c.loc);
            }
            let new_track = self.tracks[&c.loc];
            c.turn(new_track);
        }

        if to_remove.is_empty() {
            return collisions;
        }

        // Remove anything involved in a collision
        let new_carts: Vec<Cart> = self
            .carts
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| {
                if to_remove.contains(&i) {
                    None
                } else {
                    Some(c)
                }
            })
            .collect();
        self.carts = new_carts;

        collisions
    }
}

impl std::fmt::Display for Railway {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (mut max_x, mut max_y) = (0, 0);
        for &(x, y) in self.tracks.keys() {
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        let empty_row: Vec<char> = Vec::from_iter(std::iter::repeat(' ').take(max_x as usize + 1));
        let mut rows: Vec<Vec<char>> =
            Vec::from_iter(std::iter::repeat(empty_row).take(max_y as usize + 1));

        for (&(x, y), track) in &self.tracks {
            rows[y as usize][x as usize] = track.as_char();
        }

        let mut cart_locs = HashSet::with_capacity(self.carts.len());

        for cart in &self.carts {
            let (x, y) = cart.loc;
            let c = if cart_locs.insert(&cart.loc) {
                cart.as_char()
            } else {
                'X'
            };
            rows[y as usize][x as usize] = c;
        }

        for row in rows {
            let s: String = row.into_iter().collect();
            writeln!(f, "{}", s.trim_right())?;
        }

        Ok(())
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 13")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day13.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let mut railway = Railway::parse_lines(buf_reader.lines())?;

    eprintln!(
        "Found {} tracks and {} carts",
        railway.tracks.len(),
        railway.carts.len()
    );

    let mut n = 0;
    let (cx, cy) = loop {
        n += 1;
        let collisions = railway.step();
        if let Some(&c) = collisions.first() {
            break c;
        }
    };

    println!("Collision at ({},{}) after {} steps", cx, cy, n);

    let (cx, cy) = loop {
        n += 1;
        let _ = railway.step();
        if railway.carts.len() <= 1 {
            break railway.carts.first().unwrap().loc;
        }
    };

    println!("Last car at ({},{}) after {} steps", cx, cy, n);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#"
/->-\
|   |  /----\
| /-+--+-\  |
| | |  | v  |
\-+-/  \-+--/
  \------/"#;

    const TEST_INPUT2: &str = r#"
/>-<\
|   |
| /<+-\
| | | v
\>+</ |
  |   ^
  \<->/"#;

    fn get_test_railway(s: &str) -> Railway {
        let lines: Vec<&str> = s.split('\n').skip(1).collect();
        fn ok(s: &str) -> Result<&str, failure::Error> {
            Ok(s)
        }

        Railway::parse_lines(lines.into_iter().map(ok)).unwrap()
    }

    #[test]
    fn test_parsing() {
        let railway = get_test_railway(TEST_INPUT);
        assert_eq!(railway.carts.len(), 2);
        assert_eq!(railway.tracks.len(), 48);
    }

    #[test]
    fn test_advance() {
        let mut railway = get_test_railway(TEST_INPUT);
        println!("{}", railway);
        assert_eq!(railway.step(), vec![]);
        println!("{}", railway);
        assert_eq!(railway.carts.len(), 2);
        assert_eq!(railway.tracks.len(), 48);

        assert_eq!(
            railway.carts,
            vec![
                Cart::new((3, 0), Direction::Right, Turn::Left),
                Cart::new((9, 4), Direction::Right, Turn::Straight),
            ]
        );

        // Reset
        railway = get_test_railway(TEST_INPUT);
        for i in 0..13 {
            // We should have 13 collision-free steps
            println!("-- {} --\n{}", i, railway);
            assert_eq!(railway.step(), vec![]);
        }
        println!("-- == --\n{}", railway);

        assert_eq!(
            railway.carts,
            vec![
                Cart::new((7, 2), Direction::Down, Turn::Right),
                Cart::new((7, 4), Direction::Up, Turn::Left),
            ]
        );

        // Now we should have a collision
        let collision = railway.step();
        println!("-- XX --\n{}", railway);
        assert_eq!(collision, vec![(7, 3)]);
    }

    #[test]
    fn test_collision_removal() {
        let mut railway = get_test_railway(TEST_INPUT2);
        println!("{}", railway);
        assert_eq!(railway.carts.len(), 9);
        let collisions = railway.step();
        println!("{}", railway);
        assert_eq!(collisions, vec![(2, 0), (2, 4), (6, 4)]);
        assert_eq!(railway.carts.len(), 3);
        let collisions = railway.step();
        println!("{}", railway);
        assert_eq!(collisions, vec![]);
        assert_eq!(railway.carts.len(), 3);
        let collisions = railway.step();
        println!("{}", railway);
        assert_eq!(collisions, vec![(2, 4)]);
        assert_eq!(
            railway.carts,
            vec![Cart::new((6, 4), Direction::Up, Turn::Left)]
        );
    }
}
