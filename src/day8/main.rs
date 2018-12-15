#![warn(clippy::all)]

use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

fn parse_vec<F: FromStr>(s: &str) -> Result<Vec<F>, <F as FromStr>::Err> {
    let splits = s.trim().split(' ');
    splits.map(|s| F::from_str(s)).collect()
}

#[derive(Clone, Debug)]
struct Parsed {
    metadata: Vec<i64>,
    children: Vec<Parsed>,
}

impl Parsed {
    fn parse(nums: &[i64]) -> Parsed {
        let (p, r) = Parsed::parse_single(nums);
        if !r.is_empty() {
            panic!("Remaining: {:?}", r);
        }

        p
    }

    fn parse_single(nums: &[i64]) -> (Parsed, &[i64]) {
        let nchildren = nums[0] as usize;
        let nmetadata = nums[1] as usize;

        let mut remaining = &nums[2..];
        let mut children = vec![];
        for _ in 0..nchildren {
            let (child, r) = Parsed::parse_single(remaining);
            children.push(child);
            remaining = r;
        }

        let (metadata, remaining) = remaining.split_at(nmetadata);

        let p = Parsed {
            children,
            metadata: metadata.to_vec(),
        };

        (p, remaining)
    }

    fn sum_metadata(&self) -> i64 {
        let child_sum: i64 = self.children.iter().map(|c| c.sum_metadata()).sum();

        let n: i64 = self.metadata.iter().sum::<i64>();

        n + child_sum
    }

    fn value(&self) -> i64 {
        if self.children.is_empty() {
            return self.sum_metadata();
        }

        let mut sum = 0;
        for &n in &self.metadata {
            if n > (self.children.len() as i64) {
                continue
            }

            sum += self.children[(n-1) as usize].value();
        }

        sum
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 8")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day8.txt");

    eprintln!("Using input {}", input_path);

    let mut contents = String::new();
    let mut file = File::open(input_path)?;
    file.read_to_string(&mut contents)?;

    let v: Vec<i64> = parse_vec(&contents).unwrap();
    let p = Parsed::parse(&v);

    println!("Final sum: {}", p.sum_metadata());
    println!("Value: {}", p.value());

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_sum() {
        let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
        let nums: Vec<i64> = parse_vec(input).unwrap();
        let p = Parsed::parse(&nums);

        assert_eq!(p.sum_metadata(), 138);
    }


    #[test]
    fn test_value() {
        let input = "2 3 0 3 10 11 12 1 1 0 1 99 2 1 1 2";
        let nums: Vec<i64> = parse_vec(input).unwrap();
        let p = Parsed::parse(&nums);

        assert_eq!(p.value(), 66);
    }
}
