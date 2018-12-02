use clap::{App, Arg};

fn main() {
    let matches = App::new("My Super Program")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap();

    println!("Using input {}", input_path);
}
