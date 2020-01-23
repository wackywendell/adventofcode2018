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
    IResult,
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

// Returns (finished, words)
#[allow(clippy::needless_lifetimes)]
fn parse_reaction<'a>(reaction: &'a str) -> impl Fn(&'a str) -> IResult<&str, HashSet<String>> {
    move |i: &str| {
        let (i, _) = tag(reaction)(i)?;
        let (i, _) = tag(" to ")(i)?;
        let (i, mut words) =
            separated_nonempty_list(tag(", "), recognize(many1(alphanumeric1)))(i)?;
        let wordset = words.drain(..).map(str::to_owned).collect();
        Ok((i, wordset))
    }
}

// Returns (finished, words)
#[allow(clippy::needless_lifetimes)]
fn parse_reaction_start<'a>(
    reaction: &'a str,
) -> impl Fn(&'a str) -> IResult<&str, (bool, HashSet<String>)> {
    move |i: &str| {
        let (i, wordset) = parse_reaction(reaction)(i)?;
        let (i, next) = alt((tag(")"), tag("; ")))(i)?;

        Ok((i, (next == ")", wordset)))
    }
}

fn parse_reactions(i: &str) -> IResult<&str, Reactions> {
    let (i, _) = tag("(")(i)?;

    let (i, weak_match) = opt(parse_reaction_start("weak"))(i)?;
    if let Some((finished, weaknesses)) = weak_match {
        let (i, immunities) = if finished {
            (i, HashSet::new())
        } else {
            let (i, imm) = parse_reaction("immune")(i)?;
            let (i, _) = tag(")")(i)?;
            (i, imm)
        };
        return Ok((
            i,
            Reactions {
                weaknesses,
                immunities,
            },
        ));
    };

    let (i, (finished, immunities)) = parse_reaction_start("immune")(i)?;
    let (i, weaknesses) = if finished {
        (i, HashSet::new())
    } else {
        let (i, wk) = parse_reaction("weak")(i)?;
        let (i, _) = tag(")")(i)?;
        (i, wk)
    };
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
        "Immune System:" => Ok(PossibleLine::Immune),
        "Infection:" => Ok(PossibleLine::Infection),
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

    let mut state = PossibleLine::Empty;

    for l in iter {
        let line = l.map_err(|e| e.into())?;
        let army = match parse_line(line.as_ref())? {
            PossibleLine::Empty => continue,
            PossibleLine::Infection => {
                state = PossibleLine::Infection;
                continue;
            }
            PossibleLine::Immune => {
                state = PossibleLine::Immune;
                continue;
            }
            PossibleLine::Army(army) => army,
        };

        if state == PossibleLine::Infection {
            eprintln!("Pushing infection: {:?}", army);
            battle.infection.push(army);
        } else if state == PossibleLine::Immune {
            eprintln!("Pushing immune: {:?}", army);
            battle.immune.push(army);
        } else {
            return Err(failure::err_msg("Expected it to start with army name"));
        }
    }

    // Highest initiative first
    battle.immune.sort_unstable_by_key(|a| -a.initiative);
    battle.infection.sort_unstable_by_key(|a| -a.initiative);

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

        assert_eq!(
            army,
            Army {
                initiative: 2,
                damage: 4507,
                hp: 5390,
                reactions: Reactions {
                    immunities: hs_from_arr(&[]),
                    weaknesses: hs_from_arr(&["radiation", "bludgeoning"]),
                },
                specialty: "fire".to_owned(),
                units: 17,
            }
        );
    }

    #[test]
    fn test_parse_lines() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok));
        let battle = maybe_battle.unwrap();

        eprintln!("Battle: {:?}", battle);

        assert_eq!(battle.immune.len(), 2);
        assert!(battle.immune[0].initiative > battle.immune[1].initiative);
        assert_eq!(battle.infection.len(), 2);
        assert!(battle.infection[0].initiative > battle.infection[1].initiative);

        // 17 units each with 5390 hit points (weak to radiation, bludgeoning) with an attack that does 4507 fire damage at initiative 2
        let army1 = Army {
            initiative: 2,
            damage: 4507,
            hp: 5390,
            reactions: Reactions {
                immunities: hs_from_arr(&[]),
                weaknesses: hs_from_arr(&["radiation", "bludgeoning"]),
            },
            specialty: "fire".to_owned(),
            units: 17,
        };
        // 989 units each with 1274 hit points (immune to fire; weak to bludgeoning, slashing) with an attack that does 25 slashing damage at initiative 3
        let army2 = Army {
            initiative: 3,
            damage: 25,
            hp: 1274,
            reactions: Reactions {
                immunities: hs_from_arr(&["fire"]),
                weaknesses: hs_from_arr(&["bludgeoning", "slashing"]),
            },
            specialty: "slashing".to_owned(),
            units: 989,
        };

        assert_eq!(battle.immune, vec!(army1, army2));
    }
}
