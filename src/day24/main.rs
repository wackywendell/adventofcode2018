#![warn(clippy::all)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use clap::{App, Arg};
use log::{debug, info};
use nom5::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1},
    combinator::{opt, recognize},
    multi::{many1, separated_nonempty_list},
    sequence::{pair, tuple},
    IResult,
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Index {
    value: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Reactions {
    weaknesses: HashSet<String>,
    immunities: HashSet<String>,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Unknown,
    Infection,
    Immune,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Army {
    name: String,
    side: Side,
    initiative: i64,
    units: i64,
    hp: i64,
    damage: i64,
    specialty: String,
    reactions: Reactions,
}

impl Army {
    fn index(&self) -> Index {
        Index {
            value: self.initiative,
        }
    }

    fn effective_power(&self, boost: i64) -> i64 {
        self.units * (self.damage + boost)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Battle {
    // Maps initiative -> Army
    armies: HashMap<Index, Army>,
    boost: i64,
}

impl Battle {
    // (Immune, Infection)
    pub fn units(&self) -> (i64, i64) {
        let (mut imm, mut inf) = (0, 0);
        for a in self.armies.values() {
            match a.side {
                Side::Immune => imm += a.units,
                Side::Infection => inf += a.units,
                _ => panic!("Unknown side in army {:?}", a),
            }
        }

        (imm, inf)
    }

    pub fn effective_power(&self, ix: Index) -> i64 {
        let army = &self.armies[&ix];

        let boost = if army.side == Side::Immune {
            self.boost
        } else {
            0
        };

        (army.damage + boost) * army.units
    }

    fn target_order(&self) -> VecDeque<Index> {
        let mut queue: Vec<Index> = self.armies.iter().map(|(_, a)| a.index()).collect();

        queue.sort_unstable_by_key(|&ix| (-self.effective_power(ix), -self[ix].initiative));

        debug!("Target Order:");
        for &ix in &queue {
            debug!("  Index: {}, power: {}", ix.value, self.effective_power(ix));
        }

        VecDeque::from(queue)
    }

    fn attack_order(&self) -> VecDeque<Index> {
        let mut queue: Vec<Index> = self.armies.iter().map(|(_, a)| a.index()).collect();

        queue.sort_unstable_by_key(|&ix| std::cmp::Reverse(self[ix].initiative));

        VecDeque::from(queue)
    }

    fn potential_damage(&self, attack: Index, defend: Index) -> i64 {
        let d = &self[defend];

        if d.units == 0 {
            return 0;
        }

        let a = &self[attack];
        if d.reactions.immunities.contains(&a.specialty) {
            return 0;
        }

        let damage = self.effective_power(attack);

        if d.reactions.weaknesses.contains(&a.specialty) {
            return damage * 2;
        }

        damage
    }

    fn choose_target(&self, attacker: Index, ignore: &HashSet<Index>) -> Option<(i64, Index)> {
        let att_army = &self.armies[&attacker];
        if att_army.units == 0 {
            // Dead
            return None;
        }

        debug!(
            "Choosing {} ({})",
            attacker.value, self.armies[&attacker].name
        );

        // ((Damage, Effective Power, Initiative), Defender)
        let mut best: Option<((i64, i64, i64), Index)> = None;
        for (&ix, def) in &self.armies {
            debug!("  {} ({}):", ix.value, def.name);
            // Same side
            if att_army.side == def.side {
                debug!("    Same side.");
                continue;
            }

            if ignore.contains(&ix) {
                // Already attacked
                debug!("    Already attacked.");
                continue;
            }

            let damage = self.potential_damage(attacker, ix);
            if damage == 0 {
                debug!("    Damage zero.");
                continue;
            }

            let new_key = (damage, self.effective_power(ix), def.initiative);

            let (last_key, _) = match best {
                None => {
                    debug!("    Attacking!");
                    best = Some((new_key, ix));
                    continue;
                }
                Some(d) => d,
            };

            debug!(
                "    Comparing: ({}, {}, {}) > ({}, {}, {}) = {}",
                damage,
                self.effective_power(ix),
                def.initiative,
                last_key.0,
                last_key.1,
                last_key.2,
                new_key > last_key,
            );
            if new_key > last_key {
                // debug!("      Replacing!");
                best = Some((new_key, ix));
            }
        }

        best.map(|((dmg, _, _), ix)| (dmg, ix))
    }

    // Returns total units killed
    pub fn fight(&mut self) -> i64 {
        let order = self.target_order();
        let mut attacks: HashMap<Index, Index> = HashMap::new();
        let mut attacked: HashSet<Index> = HashSet::new();

        let mut deaths = 0;

        for ix in order {
            let (dmg, def) = match self.choose_target(ix, &attacked) {
                None => {
                    debug!("Target {}: Skipping", ix.value);
                    continue;
                }
                Some(d) => d,
            };

            attacks.insert(ix, def);
            debug!(
                "Target {} ({}): Targeting {} ({}) for {} damage",
                ix.value, self.armies[&ix].name, def.value, self.armies[&def].name, dmg
            );
            attacked.insert(def);
        }

        let order = self.attack_order();

        for att in order {
            let def = match attacks.get(&att) {
                None => {
                    debug!(
                        "Attack from {} ({}): Skipping",
                        att.value, self.armies[&att].name
                    );
                    continue;
                }
                Some(&d) => d,
            };

            let dmg = self.potential_damage(att, def);

            let army = &self.armies[&def];
            let units_killed = std::cmp::min(army.units, dmg / army.hp);

            debug!(
                "Attack from {} ({}) against {} ({}): Killed {} / {} units ({} damage / {} hp)",
                att.value,
                self.armies[&att].name,
                def.value,
                self.armies[&def].name,
                units_killed,
                army.units,
                dmg,
                army.hp
            );
            let army = self.armies.get_mut(&def).unwrap();
            army.units -= units_killed;
            deaths += units_killed;
        }

        deaths
    }
}

impl std::ops::Index<Index> for Battle {
    type Output = Army;

    fn index(&self, key: Index) -> &Army {
        &self.armies[&key]
    }
}

impl std::ops::IndexMut<Index> for Battle {
    // type Output = Army;

    fn index_mut(&mut self, key: Index) -> &mut Army {
        self.armies.get_mut(&key).unwrap()
    }
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
        let (i, next) = alt((tag(") "), tag("; ")))(i)?;

        Ok((i, (next == ") ", wordset)))
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
            let (i, _) = tag(") ")(i)?;
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
        let (i, _) = tag(") ")(i)?;
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
    let (i, opt_reactions) = opt(parse_reactions)(i)?;
    let reactions = opt_reactions.unwrap_or_default();

    let (i, (_, damage, _, specialty, _, initiative)) = tuple((
        tag("with an attack that does "),
        parse_integer,
        tag(" "),
        recognize(many1(alphanumeric1)),
        tag(" damage at initiative "),
        parse_integer,
    ))(i)?;

    Ok((
        i,
        Army {
            name: "Unknown".to_owned(),
            side: Side::Unknown,
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

pub fn parse_lines<S, E, T>(iter: T, boost: i64) -> Result<Battle, failure::Error>
where
    S: AsRef<str>,
    E: Into<failure::Error>,
    T: IntoIterator<Item = Result<S, E>>,
{
    let mut battle: Battle = Battle {
        armies: Default::default(),
        boost,
    };

    let mut state = PossibleLine::Empty;
    let mut immune_seen = 0;
    let mut infection_seen = 0;

    for l in iter {
        let line = l.map_err(|e| e.into())?;
        let mut army = match parse_line(line.as_ref())? {
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
            infection_seen += 1;
            army.name = format!("Infection {}", infection_seen);
            army.side = Side::Infection;
            debug!(
                "Pushing infection: {:?} power: {}",
                army,
                army.effective_power(0)
            );
        } else if state == PossibleLine::Immune {
            immune_seen += 1;
            army.name = format!("Immune {}", immune_seen);
            army.side = Side::Immune;
            debug!(
                "Pushing immune: {:?} power: {}",
                army,
                army.effective_power(0)
            );
        } else {
            return Err(failure::err_msg("Expected it to start with army name"));
        }

        battle.armies.insert(army.index(), army);
    }

    Ok(battle)
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();

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

    debug!("Using input {}", input_path);
    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);
    let mut battle = parse_lines(buf_reader.lines(), 0)?;

    loop {
        let killed = battle.fight();
        let (imm, inf) = battle.units();
        info!(
            "{} Units Killed. {} Immune, {} Infection remain",
            killed, imm, inf
        );
        if killed == 0 {
            break;
        }
    }

    let (imm, inf) = battle.units();
    println!("Battle complete. {} Immune, {} Infection remain", imm, inf);

    Ok(())
}

#[cfg(test)]
mod tests {
    use test_env_log::test;

    use super::*;

    fn hs_from_arr(strings: &[&str]) -> HashSet<String> {
        strings.iter().map(|&s: &&str| s.to_owned()).collect()
    }

    #[test]
    fn test_parse_specialties() {
        let s = "(weak to fire) ";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire"]));
        assert_eq!(o.immunities, HashSet::new());
        assert_eq!(i, "");

        let s = "(weak to fire, cold) ";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire", "cold"]));
        assert_eq!(o.immunities, HashSet::new());
        assert_eq!(i, "");

        let s = "(weak to fire; immune to cold, slashing) ";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, hs_from_arr(&["fire"]));
        assert_eq!(o.immunities, hs_from_arr(&["cold", "slashing"]));
        assert_eq!(i, "");

        let s = "(immune to cold, slashing) ";
        let (i, o) = parse_reactions(s).unwrap();
        assert_eq!(o.weaknesses, HashSet::new());
        assert_eq!(o.immunities, hs_from_arr(&["cold", "slashing"]));
        assert_eq!(i, "");

        let s = "(immune to cold) ";
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
                name: "Unknown".to_owned(),
                side: Side::Unknown,
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

        let o = parse_line("10 units each with 20 hit points with an attack that does 40 fire damage at initiative 1");
        let army = if let Ok(PossibleLine::Army(army)) = o {
            army
        } else {
            panic!("Failed to unwrap army: {:?}", o);
        };

        assert_eq!(
            army,
            Army {
                name: "Unknown".to_owned(),
                side: Side::Unknown,
                initiative: 1,
                damage: 40,
                hp: 20,
                reactions: Reactions {
                    immunities: hs_from_arr(&[]),
                    weaknesses: hs_from_arr(&[]),
                },
                specialty: "fire".to_owned(),
                units: 10,
            }
        );
    }

    #[test]
    fn test_parse_lines() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok), 0);
        let battle = maybe_battle.unwrap();

        info!("Battle: {:?}", battle);

        assert_eq!(battle.armies.len(), 4);

        // 17 units each with 5390 hit points (weak to radiation, bludgeoning) with an attack that does 4507 fire damage at initiative 2
        let army1 = Army {
            name: "Immune 1".to_owned(),
            side: Side::Immune,
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
            name: "Immune 2".to_owned(),
            side: Side::Immune,
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

        // 801 units each with 4706 hit points (weak to radiation) with an attack that does 116 bludgeoning damage at initiative 1
        let army3 = Army {
            name: "Infection 1".to_owned(),
            side: Side::Infection,
            initiative: 1,
            damage: 116,
            hp: 4706,
            reactions: Reactions {
                immunities: hs_from_arr(&[]),
                weaknesses: hs_from_arr(&["radiation"]),
            },
            specialty: "bludgeoning".to_owned(),
            units: 801,
        };
        // 4485 units each with 2961 hit points (immune to radiation; weak to fire, cold) with an attack that does 12 slashing damage at initiative 4
        let army4 = Army {
            name: "Infection 2".to_owned(),
            side: Side::Infection,
            initiative: 4,
            damage: 12,
            hp: 2961,
            reactions: Reactions {
                immunities: hs_from_arr(&["radiation"]),
                weaknesses: hs_from_arr(&["fire", "cold"]),
            },
            specialty: "slashing".to_owned(),
            units: 4485,
        };

        assert_eq!(battle[army1.index()], army1);
        assert_eq!(battle[army2.index()], army2);
        assert_eq!(battle[army3.index()], army3);
        assert_eq!(battle[army4.index()], army4);
    }

    #[test]
    fn test_target_order() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok), 0);
        let battle = maybe_battle.unwrap();

        let order: Vec<i64> = battle.target_order().iter().map(|i| i.value).collect();
        assert_eq!(order, vec![1, 2, 4, 3]);
    }

    #[test]
    fn test_attack_order() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok), 0);
        let battle = maybe_battle.unwrap();

        let order: Vec<i64> = battle.attack_order().iter().map(|i| i.value).collect();
        assert_eq!(order, vec![4, 3, 2, 1]);
    }

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_fight() {
        let lines: Vec<&str> = TEST_INPUT.split('\n').collect();
        let maybe_battle = parse_lines::<_, failure::Error, _>(lines.iter().map(Ok), 0);
        let mut battle = maybe_battle.unwrap();

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 905);
        assert_eq!(battle[Index { value: 1 }].units, 797);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 761);
        assert_eq!(battle[Index { value: 1 }].units, 793);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 618);
        assert_eq!(battle[Index { value: 1 }].units, 789);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 475);
        assert_eq!(battle[Index { value: 1 }].units, 786);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 333);
        assert_eq!(battle[Index { value: 1 }].units, 784);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 191);
        assert_eq!(battle[Index { value: 1 }].units, 783);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 49);
        assert_eq!(battle[Index { value: 1 }].units, 782);
        assert_eq!(battle[Index { value: 4 }].units, 4434);

        battle.fight();
        assert_eq!(battle[Index { value: 2 }].units, 0);
        assert_eq!(battle[Index { value: 3 }].units, 0);
        assert_eq!(battle[Index { value: 1 }].units, 782);
        assert_eq!(battle[Index { value: 4 }].units, 4434);
    }

    #[test]
    fn test_boost_fight() {
        unimplemented!()
    }
}
