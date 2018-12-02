use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

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

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day1.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let mut sum = 0;
    for (_i, line) in buf_reader.lines().enumerate() {
        let s = line?;
        let n = s.trim().parse::<i64>().unwrap();
        sum += n;
    }

    println!("Final sum: {}", sum);

    Ok(())
}
