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

#[derive(Debug, Clone, PartialEq, Eq)]
enum PossibleLine {
    Empty,
    Immune,
    Infection,
    Army(Army),
}

fn parse_line(s: &str) -> Result<PossibleLine, failure::Error> {
    let l = s.trim();
    match l {
        "" => Ok(PossibleLine::Empty),
        "Immune System:" => Ok(PossibleLine::Infection),
        "Infection:" => Ok(PossibleLine::Immune),
        t => {
            let army = match parse_army(t) {
                Err(e) => return Err(e.to_owned().into()),
                Ok((_, army)) => army,
            };
            Ok(PossibleLine::Army(army))
        }
    }
}

pub fn parse_lines<S, E, T>(iter: T) -> Result<Battle, failure::Error>
where
    S: AsRef<str>,
    E: Into<failure::Error>,
    T: IntoIterator<Item = Result<S, E>>,
{
    let mut battle: Battle = Default::default();

    let mut seen_infection = false;
    let mut seen_immune = false;

    for l in iter {
        let line = l.map_err(|e| e.into())?;
        let army = match parse_line(line.as_ref())? {
            PossibleLine::Empty => continue,
            PossibleLine::Infection => {
                seen_infection = true;
                continue;
            }
            PossibleLine::Immune => {
                seen_immune = true;
                continue;
            }
            PossibleLine::Army(army) => army,
        };

        if seen_infection {
            battle.infection.push(army);
        } else if seen_immune {
            battle.immune.push(army);
        } else {
            return Err(failure::err_msg("Expected it to start with army name"));
        }
    }

    Ok(battle)
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

    const TEST_INPUT: &str = r#"
        Immune System:
        17 units each with 5390 hit points (weak to radiation, bludgeoning) with an attack that does 4507 fire damage at initiative 2
        989 units each with 1274 hit points (immune to fire; weak to bludgeoning, slashing) with an attack that does 25 slashing damage at initiative 3

        Infection:
        801 units each with 4706 hit points (weak to radiation) with an attack that does 116 bludgeoning damage at initiative 1
        4485 units each with 2961 hit points (immune to radiation; weak to fire, cold) with an attack that does 12 slashing damage at initiative 4
    "#;

    #[test]
    fn test_parse_line() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let o = parse_line(lines[2]);

        let army = if let Ok(PossibleLine::Army(army)) = o {
            army
        } else {
            panic!("Failed to unwrap army: {:?}", o);
        };
    }

    #[test]
    fn test_parse_lines() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok));
        let battle = maybe_battle.unwrap();

        assert_eq!(battle.immune.len(), 2);
        assert_eq!(battle.infection.len(), 2);
    }
}
