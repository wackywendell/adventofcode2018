#![warn(clippy::all)]

use std::cmp::{max, min};
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::ops::Sub;

use clap::{App, Arg};

use text_io::try_scan;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point(i64, i64, i64);

impl Point {
    pub fn distance(self, other: Self) -> i64 {
        let dx = (self.0 - other.0).abs();
        let dy = (self.1 - other.1).abs();
        let dz = (self.2 - other.2).abs();

        dx + dy + dz
    }
}

impl Sub for Point {
    type Output = (i64, i64, i64);

    fn sub(self, other: Point) -> (i64, i64, i64) {
        (self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nanobot {
    signal: i64,
    loc: Point,
}

impl Nanobot {
    pub fn in_range(&self, other: &Self) -> bool {
        let dist = self.loc.distance(other.loc);

        dist <= self.signal
    }

    pub fn parse_line(line: &str) -> Result<Self, failure::Error> {
        let (x, y, z, signal): (i64, i64, i64, i64);
        try_scan!(line.bytes() => "pos=<{},{},{}>, r={}", x,y,z,signal);
        Ok(Nanobot {
            loc: Point(x, y, z),
            signal,
        })
    }

    fn parse_lines<S, E, T>(iter: T) -> Result<Vec<Self>, failure::Error>
    where
        S: AsRef<str>,
        E: Into<failure::Error>,
        T: IntoIterator<Item = Result<S, E>>,
    {
        iter.into_iter()
            .filter_map(|rl| match rl {
                Err(e) => Some(Err(e.into())),
                Ok(l) => {
                    let trimmed = l.as_ref().trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(Self::parse_line(trimmed))
                    }
                }
            })
            .collect()
    }
}

// strongest_range finds the nanobot with the strongest signal, and calculates
// what the
pub fn strongest_range(bots: &[Nanobot]) -> Option<(Nanobot, isize)> {
    if bots.is_empty() {
        return None;
    }

    let strongest = bots.iter().max_by_key(|b| b.signal)?;

    let in_range: isize = bots
        .iter()
        .filter(|b| strongest.in_range(b))
        .map(|_| 1)
        .sum();

    Some((strongest.clone(), in_range))
}

pub fn min_dist(v: i64, start: i64, end: i64) -> i64 {
    if v < start {
        return start - v;
    }
    if v > end {
        return v - end;
    }

    0
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region(Point, Point);

impl Region {
    pub fn size(&self) -> i64 {
        let &Region(p1, p2) = self;
        let (dx, dy, dz) = p2 - p1;
        (dx + 1) * (dy + 1) + (dz + 1)
    }

    pub fn min_distance(&self, point: Point) -> i64 {
        let &Region(p1, p2) = self;
        let dx = min_dist(point.0, p1.0, p2.0);
        let dy = min_dist(point.1, p1.1, p2.1);
        let dz = min_dist(point.2, p1.2, p2.2);

        dx + dy + dz
    }
    pub fn max_distance(&self, point: Point) -> i64 {
        let &Region(p1, p2) = self;
        let (d1, d2) = (p1 - point, p2 - point);

        let dx = max(d1.0.abs(), d2.0.abs());
        let dy = max(d1.1.abs(), d2.1.abs());
        let dz = max(d1.2.abs(), d2.2.abs());

        dx + dy + dz
    }

    pub fn possible_range(&self, bot: &Nanobot) -> bool {
        self.min_distance(bot.loc) <= bot.signal
    }

    pub fn split(&self, n: usize) -> Vec<Region> {
        if n <= 1 {
            return vec![self.clone()];
        }
        let Region(p1, p2) = self;

        fn split_value(v1: i64, v2: i64, n: usize) -> Vec<i64> {
            let dv = v2 - v1;
            let mut values: Vec<i64> = (0..=n).map(|s| v1 + (s as i64) * dv / (n as i64)).collect();
            values[0] -= 1;
            values.dedup();
            values
        }

        let xs = split_value(p1.0, p2.0, n);
        let ys = split_value(p1.1, p2.1, n);
        let zs = split_value(p1.2, p2.2, n);

        let mut regions = Vec::with_capacity((xs.len() - 1) * (ys.len() - 1) * (zs.len() - 1));

        for (&x1, &x2) in xs.iter().zip(&xs[1..]) {
            for (&y1, &y2) in ys.iter().zip(&ys[1..]) {
                for (&z1, &z2) in zs.iter().zip(&zs[1..]) {
                    regions.push(Region(Point(x1 + 1, y1 + 1, z1 + 1), Point(x2, y2, z2)));
                }
            }
        }

        regions
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct BotRegion {
    in_range: usize,
    area: Region,
}

pub struct BotMaximizer {
    bots: Vec<Nanobot>,
    // area: Region,
    queue: BinaryHeap<BotRegion>,
    strongest: Option<(usize, Point)>,
}

impl BotMaximizer {
    pub fn new(bots: Vec<Nanobot>) -> Self {
        if bots.is_empty() {
            panic!("Can't maximize over empty bots");
        }
        let someps = bots
            .iter()
            .fold(None, |extrema: Option<(Point, Point)>, b| {
                let points = match extrema {
                    None => (b.loc, b.loc),
                    Some((minp, maxp)) => (
                        Point(
                            min(minp.0, b.loc.0),
                            min(minp.1, b.loc.1),
                            min(minp.2, b.loc.2),
                        ),
                        Point(
                            max(maxp.0, b.loc.0),
                            max(maxp.1, b.loc.1),
                            max(maxp.2, b.loc.2),
                        ),
                    ),
                };

                Some(points)
            });

        let (minp, maxp) = someps.unwrap();

        let initial = BotRegion {
            in_range: bots.len(),
            area: Region(minp, maxp),
        };
        let queue = BinaryHeap::from(vec![initial]);

        BotMaximizer {
            bots,
            // area: Region(minp, maxp),
            queue,
            strongest: None,
        }
    }

    fn calculate_in_range(&self, region: &Region) -> usize {
        let mut sum = 0;
        for b in &self.bots {
            let ranged = region.possible_range(b);
            // println!(
            //     "Range: {:?} - {:?}: {}, {}",
            //     b,
            //     region,
            //     region.min_distance(b.loc),
            //     ranged
            // );

            if ranged {
                sum += 1;
            }
        }
        sum
    }

    // Step forward, and return 'true' if more work needs to be done.
    pub fn step(&mut self, n: usize) -> bool {
        if self.strongest.is_some() {
            return false;
        }

        if self.queue.is_empty() {
            panic!("Should not have an empty queue");
        }

        let next = self.queue.pop().unwrap();
        let splits = next.area.split(n);
        if splits.len() == 1 {
            println!("Found maximal region: {:?}", next);
            self.strongest = Some((next.in_range, next.area.0));
            return false;
        }

        for r in splits {
            let in_range = self.calculate_in_range(&r);
            let br = BotRegion { in_range, area: r };
            self.queue.push(br);
        }

        true
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 23")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day23.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let bots = Nanobot::parse_lines(buf_reader.lines())?;

    let (strongest, in_range) = strongest_range(&bots).unwrap();

    println!("Strongest bot: {:?}", strongest);
    println!("In range: {}", in_range);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#"
        pos=<0,0,0>, r=4
        pos=<1,0,0>, r=1
        pos=<4,0,0>, r=3
        pos=<0,2,0>, r=1
        pos=<0,5,0>, r=3
        pos=<0,0,3>, r=1
        pos=<1,1,1>, r=1
        pos=<1,1,2>, r=1
        pos=<1,3,1>, r=1
        "#;

    fn get_test_bots(s: &str) -> Result<Vec<Nanobot>, failure::Error> {
        let lines: Vec<&str> = s.split('\n').collect();
        let bots: Vec<Nanobot> =
            Nanobot::parse_lines::<_, failure::Error, _>(lines.iter().map(Ok))?;

        Ok(bots)
    }

    #[test]
    fn test_splits() {
        let r = Region(Point(10, 20, 40), Point(12, 21, 40));
        let mut splits = r.split(3);
        splits.sort();

        assert_eq!(
            splits,
            vec![
                Region(Point(10, 20, 40), Point(10, 20, 40)),
                Region(Point(10, 21, 40), Point(10, 21, 40)),
                Region(Point(11, 20, 40), Point(11, 20, 40)),
                Region(Point(11, 21, 40), Point(11, 21, 40)),
                Region(Point(12, 20, 40), Point(12, 20, 40)),
                Region(Point(12, 21, 40), Point(12, 21, 40)),
            ]
        )
    }

    #[test]
    fn test_parse() {
        let bots = get_test_bots(TEST_INPUT).unwrap();
        assert_eq!(bots.len(), 9);

        let (strongest, in_range) = strongest_range(&bots).unwrap();

        assert_eq!(
            strongest,
            Nanobot {
                loc: Point(0, 0, 0),
                signal: 4
            }
        );

        assert_eq!(in_range, 7);
    }

    const TEST_INPUT2: &str = r#"
    pos=<10,12,12>, r=2
    pos=<12,14,12>, r=2
    pos=<16,12,12>, r=4
    pos=<14,14,14>, r=6
    pos=<50,50,50>, r=200
    pos=<10,10,10>, r=5"#;

    #[test]
    fn test_maximizer() {
        let bots = get_test_bots(TEST_INPUT2).unwrap();
        assert_eq!(bots.len(), 6);

        let mut maximizer = BotMaximizer::new(bots);
        let r = maximizer.queue.peek().unwrap().area.clone();
        let ir = maximizer.calculate_in_range(&r);
        assert_eq!(ir, maximizer.bots.len());

        let r = Region(Point(12, 12, 12), Point(12, 12, 12));
        let ir = maximizer.calculate_in_range(&r);
        assert_eq!(ir, 5);

        let r = Region(Point(10, 10, 10), Point(23, 23, 23));
        let ir = maximizer.calculate_in_range(&r);
        assert_eq!(ir, 6);

        println!("Looking at {:?}", maximizer.queue.peek());

        while maximizer.step(3) {
            println!("Looking at {:?}", maximizer.queue.peek());
            let queued = maximizer.queue.clone().into_sorted_vec();
            println!("Queue: {:?}", &queued[queued.len() - 10..]);
        }

        let (d, p) = maximizer.strongest.unwrap();

        assert_eq!(d, 5);
        assert_eq!(p, Point(12, 12, 12));
    }
}
