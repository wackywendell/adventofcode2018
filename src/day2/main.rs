use clap::{App, Arg};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

fn find_multiples(s: &str) -> (bool, bool) {
    let mut counts: HashMap<char, i8> = HashMap::new();

    for c in s.chars() {
        let v = counts.entry(c).or_insert(0);
        *v += 1;
    }

    let (mut doubles, mut triples) = (false, false);
    for &v in counts.values() {
        match v {
            3 => triples = true,
            2 => doubles = true,
            _ => {}
        }
    }

    return (doubles, triples);
}

fn main() -> std::io::Result<()> {
    let matches = App::new("My Super Program")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day2.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let mut doubles = 0;
    let mut triples = 0;

    let mut seen_minus_char: HashMap<String, HashSet<String>> = HashMap::new();
    for (_i, line) in buf_reader.lines().enumerate() {
        let s = line?;
        let (double, triple) = find_multiples(s.trim());
        if double {
            doubles += 1
        }
        if triple {
            triples += 1
        }

        for ix in 0..s.len() {
            let mut buf = String::new();
            buf.push_str(&s[..ix]);
            buf.push_str(&s[ix + 1..]);

            let entry = seen_minus_char.entry(buf);
            let value = entry.or_default();
            value.insert(s.clone());

            if value.len() >= 2 {
                println!("Found: {}{}", &s[..ix], &s[ix + 1..]);
            }
        }
    }

    println!(
        "Checksum: {} * {} = {}",
        doubles,
        triples,
        doubles * triples
    );

    Ok(())
}
