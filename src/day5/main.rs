use clap::{App, Arg};
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Chemical {
    units: String,
}

impl Default for Chemical {
    fn default() -> Self {
        return Chemical {
            units: Default::default(),
        };
    }
}

impl Chemical {
    fn new() -> Self {
        Default::default()
    }

    fn react<I: IntoIterator<Item = char>>(&mut self, chars: I) {
        let mut last: Option<char> = self.units.pop();

        for c in chars {
            last = match last {
                None => Some(c),
                Some(l) if (l == c || l.to_ascii_lowercase() != c.to_ascii_lowercase()) => {
                    self.units.push(l);
                    Some(c)
                }
                Some(_) => {
                    // We drop both l and c
                    self.units.pop()
                }
            };
        }

        if let Some(l) = last {
            self.units.push(l);
        }
    }

    fn drop_react<I: IntoIterator<Item = char>>(chars: I, c: char) -> Self {
        let c = c.to_ascii_lowercase();
        let mut chem = Chemical::new();

        chem.react(chars.into_iter().filter(|l| l.to_ascii_lowercase() != c));
        return chem;
    }

    fn find_shortest_drop<S: AsRef<str>>(chars: S) -> (char, Self) {
        let a: &mut [u8] = &mut [0; 1];
        'a'.encode_utf8(a);
        let z: &mut [u8] = &mut [0; 1];
        'z'.encode_utf8(z);

        let inputs = (a[0]..z[0]).map(|b| char::from(b));

        let mut shortest = None;
        for c in inputs {
            let chem = Chemical::drop_react(chars.as_ref().chars(), c);
            shortest = match shortest {
                None => Some((c, chem)),
                Some((_, ref last_chem)) if last_chem.units.len() > chem.units.len() => {
                    Some((c, chem))
                }
                lc => lc,
            }
        }

        return shortest.unwrap();
    }
}

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

    let mut chem = Chemical::new();
    chem.react(contents.chars());
    println!("Units: {}", chem.units.len());

    let (c, short) = Chemical::find_shortest_drop(contents);
    println!("Shortest: {} {}", c, short.units.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_react() {
        let s = "dabAcCaCBAcCcaDA";
        let mut c = Chemical::new();
        c.react(s.chars());

        assert_eq!(c.units, "dabCBAcaDA");
    }

    #[test]
    fn test_drop_react() {
        let s = "dabAcCaCBAcCcaDA";

        let ac = Chemical::drop_react(s.chars(), 'a');
        assert_eq!(ac.units, "dbCBcD");

        let bc = Chemical::drop_react(s.chars(), 'b');
        assert_eq!(bc.units, "daCAcaDA");

        let cc = Chemical::drop_react(s.chars(), 'c');
        assert_eq!(cc.units, "daDA");

        let dc = Chemical::drop_react(s.chars(), 'd');
        assert_eq!(dc.units, "abCBAc");

        let (c, chem) = Chemical::find_shortest_drop(s);

        assert_eq!(c, 'c');
        assert_eq!(chem.units, cc.units);
    }
}
