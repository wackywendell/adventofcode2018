#![warn(clippy::all)]

use clap::{App, Arg};

const MODULUS: i64 = 20_183;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq)]
pub enum Erosion {
    Rocky,
    Wet,
    Narrow,
}

impl Into<i64> for Erosion {
    fn into(self) -> i64 {
        match self {
            Erosion::Rocky => 0,
            Erosion::Wet => 1,
            Erosion::Narrow => 2,
        }
    }
}

impl Into<char> for Erosion {
    fn into(self) -> char {
        match self {
            Erosion::Rocky => '.',
            Erosion::Wet => '=',
            Erosion::Narrow => '|',
        }
    }
}

pub struct Cave {
    depth: i64,
    target: (i64, i64),

    geologies: Vec<Vec<i64>>,
}

impl Cave {
    pub fn new(depth: i64, target: (i64, i64), calculate_to: (i64, i64)) -> Cave {
        let mut c = Cave {
            depth,
            target,
            geologies: Vec::new(),
        };

        c.calculate(calculate_to.0, calculate_to.1);

        c
    }

    fn erosion_level(&self, x: i64, y: i64) -> i64 {
        // println!("erosion({}, {}); {}", x, y, self.geologies.len());
        // let rl = self.geologies[x as usize].len();
        // println!("erosion({}, {}); {}, {}", x, y, self.geologies.len(), rl);
        let g = self.geologies[x as usize][y as usize];
        (g + self.depth) % MODULUS
    }

    pub fn erosion(&self, x: i64, y: i64) -> Erosion {
        match self.erosion_level(x, y) % 3 {
            0 => Erosion::Rocky,
            1 => Erosion::Wet,
            2 => Erosion::Narrow,
            _ => unreachable!(),
        }
    }

    fn geology(&self, x: i64, y: i64) -> i64 {
        if (x, y) == self.target {
            return 0;
        }
        if x == 0 {
            return ((y % MODULUS) * (48271 % MODULUS)) % MODULUS;
        } else if y == 0 {
            return ((x % MODULUS) * 16807) % MODULUS;
        }

        let e1: i64 = self.erosion_level(x - 1, y);
        let e2: i64 = self.erosion_level(x, y - 1);

        (e1 * e2) % MODULUS
    }

    fn calculate(&mut self, target_x: i64, target_y: i64) {
        for x in 0..=target_x {
            while self.geologies.len() < x as usize + 1 {
                self.geologies.push(Vec::new());
            }
            // let gl = self.geologies.len();
            // println!("Length: {}, x: {}", gl, x);

            for y in 0..=target_y {
                // println!("calculate at ({}, {})", x, y);
                let value = self.geology(x, y);
                let row = self.geologies.get_mut(x as usize).unwrap();

                // println!("calculate at ({}, {}); lengths {}, {}", x, y, gl, row.len());

                match row.get_mut(y as usize) {
                    None => row.push(value),
                    Some(r) => *r = value,
                }
            }
        }
    }

    pub fn risk(&self) -> i64 {
        let mut sum = 0;
        let (target_x, target_y) = self.target;

        if target_x >= self.geologies.len() as i64 {
            panic!(
                "Can't reach {} with only {} rows",
                target_x,
                self.geologies.len()
            )
        }

        for x in 0..=target_x {
            for y in 0..=target_y {
                let e = self.erosion(x, y);
                let risk: i64 = e.into();
                sum += risk;
            }
        }

        sum
    }
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 22")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("depth")
                .short("d")
                .long("depth")
                .value_name("DEPTH")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target-x")
                .short("x")
                .long("target-x")
                .value_name("TARGETX")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("target-y")
                .short("y")
                .long("target-y")
                .value_name("TARGETY")
                .takes_value(true),
        )
        .get_matches();

    // let input_path = matches.value_of("INPUT").unwrap_or("inputs/day22.txt");
    let depth: i64 = matches.value_of("depth").unwrap_or("11991").parse()?;
    let target_x: i64 = matches.value_of("TARGETX").unwrap_or("6").parse()?;
    let target_y: i64 = matches.value_of("TARGETY").unwrap_or("797").parse()?;

    eprintln!("Using depth {}, target ({}, {})", depth, target_x, target_y);

    let c = Cave::new(depth, (target_x, target_y), (target_x + 50, target_y + 50));
    println!("Risk: {}", c.risk());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_INPUT: &str = r#"
M=.|=.|.|=.|=|=.
.|=|=|||..|.=...
.==|....||=..|==
=.|....|.==.|==.
=|..==...=.|==..
=||.=.=||=|=..|=
|.=.===|||..=..|
|..==||=.|==|===
.=..===..=|.|||.
.======|||=|=.|=
.===|=|===T===||
=|||...|==..|=.|
=.=|=.=..=.||==|
||=|=...|==.=|==
|=.=||===.|||===
||.|==.|.|.||=||
"#;

    fn char_to_erosion(c: char) -> Option<Erosion> {
        match c {
            'T' => None,
            'M' => None,
            '.' => Some(Erosion::Rocky),
            '=' => Some(Erosion::Wet),
            '|' => Some(Erosion::Narrow),
            _ => unreachable!(),
        }
    }

    fn get_example_geology() -> Vec<Vec<Option<Erosion>>> {
        let lines: Vec<&str> = EXAMPLE_INPUT.split('\n').collect();
        let mut rows: Vec<Vec<Option<Erosion>>> = Vec::new();

        for l in lines {
            let l = l.trim();
            if l.is_empty() {
                continue;
            };

            let row = l.chars().map(char_to_erosion).collect();
            rows.push(row);
        }

        rows
    }

    #[test]
    fn test_cave() {
        let c = Cave::new(510, (10, 10), (20, 20));

        let rows = get_example_geology();

        let mut sum = 0;

        for (y, row) in rows.iter().enumerate() {
            for (x, oe) in row.iter().enumerate() {
                let e = c.erosion(x as i64, y as i64);
                let &exp = match oe {
                    None => {
                        eprintln!("Skipping: ({}, {}): {:?}", x, y, e);
                        continue;
                    }
                    Some(e) => e,
                };

                let risk: i64 = e.into();
                if x <= 10 && y <= 10 {
                    sum += risk;
                }

                eprintln!("({}, {}): {:?} =? {:?}", x, y, e, exp,);
                assert_eq!(e, exp);
            }
        }

        assert_eq!(sum, 114);

        assert_eq!(c.risk(), 114);
    }
}
