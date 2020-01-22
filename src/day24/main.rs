#![warn(clippy::all)]

use std::collections::HashSet;

use clap::{App, Arg};
use nom5::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1},
    combinator::{opt, recognize},
    multi::{many1, separated_nonempty_list},
    sequence::{pair, tuple},
    Err, IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reactions {
    weaknesses: HashSet<String>,
    immunities: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Army {
    initiative: i64,
    units: i64,
    hp: i64,
    damage: i64,
    specialty: String,
    reactions: Reactions,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Battle {
    infection: Vec<Army>,
    immune: Vec<Army>,
}

fn parse_reaction<'a>(reaction: &str, s: &'a str) -> IResult<&'a str, HashSet<String>> {
    let (i, _) = tag(reaction)(s)?;
    let (i, _) = tag(" to ")(i)?;
    let (i, o) = separated_nonempty_list(tag(", "), recognize(many1(alphanumeric1)))(i)?;
    let reactions: HashSet<String> = o.iter().map(|&sr: &&str| String::from(sr)).collect();
    Ok((i, reactions))
}

fn parse_reactions(i: &str) -> IResult<&str, Reactions> {
    let (i, _) = tag("(")(i)?;
    let (mut i, weaknesses) = match parse_reaction("weak", i) {
        Err(Err::Error(_)) => (i, HashSet::new()),
        Err(e) => return Err(e),
        Ok((i, h)) => (i, h),
    };

    if !weaknesses.is_empty() {
        i = match tag("; ")(i) {
            Ok((i, _)) => i,
            Err(Err::Error(_)) => {
                // We only have weaknesses
                let (i, _) = tag(")")(i)?;
                return Ok((
                    i,
                    Reactions {
                        weaknesses,
                        immunities: HashSet::new(),
                    },
                ));
            }
            Err(e) => return Err(e),
        };
    }

    let (i, immunities) = parse_reaction("immune", i)?;

    let (i, _) = tag(")")(i)?;

    Ok((
        i,
        Reactions {
            weaknesses,
            immunities,
        },
    ))
}

fn parse_integer(i: &str) -> IResult<&str, i64> {
    let (i, o) = recognize(pair(opt(alt((tag("+"), tag("-")))), digit1))(i)?;

    let n = str::parse(o).expect("Should be a real number");

    Ok((i, n))
}

pub fn parse_army(i: &str) -> IResult<&str, Army> {
    let (i, (units, _, hp, _)) = tuple((
        parse_integer,
        tag(" units each with "),
        parse_integer,
        tag(" hit points "),
    ))(i)?;
    let (i, reactions) = parse_reactions(i)?;

    let (i, (_, damage, _, specialty, _, initiative)) = tuple((
        tag(" with an attack that does "),
        parse_integer,
        tag(" "),
        recognize(many1(alphanumeric1)),
        tag(" damage at initiative "),
        parse_integer,
    ))(i)?;

    Ok((
        i,
        Army {
            initiative,
            units,
            hp,
            damage,
            specialty: specialty.to_owned(),
            reactions,
        },
    ))
}

pub fn parse_lines<'a, S, E, T>(iter: T) -> Result<Battle, failure::Error>
where
    S: AsRef<str> + 'a,
    E: Into<failure::Error>,
    T: IntoIterator<Item = Result<S, E>> + 'a,
{
    let mut seen_infection = false;
    let mut seen_immune = false;

    let mut battle: Battle = Default::default();

    for rl in iter {
        let line = match rl {
            Err(e) => return Err(e.into()),
            Ok(ref l) => l.as_ref().trim(),
        };
        if line.is_empty() {
            continue;
        };

        if !seen_immune {
            if line != "Immune System:" {
                return Err(failure::err_msg("Expected line 'Immune System:', got {}"));
            }
            seen_immune = true;
            continue;
        }
        if line == "Infection:" {
            seen_infection = true;
            continue;
        }

        let (i, army) = parse_army(line)?;

        if seen_infection {
            battle.infection.push(army);
        } else {
            battle.immune.push(army);
        }
    }

    return Ok(battle);
}

fn main() -> Result<(), failure::Error> {
    let matches = App::new("Day 24")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day24.txt");

    eprintln!("Using input {}", input_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = r#""#;

    fn hs_from_arr(strings: &[&str]) -> HashSet<String> {
        strings.iter().map(|&s: &&str| s.to_owned()).collect()
    }

    #[test]
    fn test_parse_specialties() {
        let s = "(weak to fire)";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire"]));
        assert_eq!(o.immunities, HashSet::new());
        assert_eq!(i, "");

        let s = "(weak to fire, cold)";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire", "cold"]));
        assert_eq!(o.immunities, HashSet::new());
        assert_eq!(i, "");

        let s = "(weak to fire; immune to cold, slashing)";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire"]));
        assert_eq!(o.immunities, hs_from_arr(&["cold", "slashing"]));
        assert_eq!(i, "");

        let s = "(immune to cold, slashing)";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, HashSet::new());
        assert_eq!(o.immunities, hs_from_arr(&["cold", "slashing"]));
        assert_eq!(i, "");

        let s = "(immune to cold)";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, HashSet::new());
        assert_eq!(o.immunities, hs_from_arr(&["cold"]));
        assert_eq!(i, "");
    }
}
