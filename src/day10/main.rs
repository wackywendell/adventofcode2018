#![warn(clippy::all)]

#[macro_use]
extern crate nom;

use clap::{App, Arg};
use nom::digit;

use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn from_digits(input: &str) -> Result<i64, std::num::ParseIntError> {
  input.parse()
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
  c.is_digit(16)
}

named!(integer<&str, i64>,
  map_res!(digit, from_digits)
);

named!(hex_color<&str, (i64, u8)>,
  do_parse!(
    sign:       value!(-1, tag!("#"))   >>
    red:   digit >>
    (sign, red)
  )
);

// named!(integer<&str, i64>,
//   do_parse!(
//       sign: opt!(value!(tag!("-"), -1)) >>
//       digits: nom::digit >>
//       (sign, digits)
//   )
// );

// named!(hex_color<&str, Color>,
//   do_parse!(
//            tag!("#")   >>
//     red:   hex_primary >>
//     green: hex_primary >>
//     blue:  hex_primary >>
//     (Color { red, green, blue })
//   )
// );


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
