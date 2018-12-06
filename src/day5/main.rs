use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 5")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day5.txt");

    eprintln!("Using input {}", input_path);

    let mut contents = String::new();
    let mut file = File::open(input_path)?;
    file.read_to_string(&mut contents)?;

    Ok(())
}
