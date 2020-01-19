#![warn(clippy::all)]

use clap::{App, Arg};

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 25")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day25.txt");

    eprintln!("Using input {}", input_path);

    Ok(())
}
