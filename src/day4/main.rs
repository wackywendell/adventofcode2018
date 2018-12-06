#![warn(clippy::all)]

#[macro_use]
extern crate lazy_static;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::Timelike;
use clap::{App, Arg};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::iter::FromIterator;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Date {
    year: u16,
    month: u16,
    day: u16,
    hour: u16,
    minute: u16,
}

impl Into<NaiveDateTime> for Date {
    fn into(self) -> NaiveDateTime {
        let date = NaiveDate::from_ymd(self.year as i32, self.month as u32, self.day as u32);
        let time = NaiveTime::from_hms(self.hour as u32, self.minute as u32, 0);

        NaiveDateTime::new(date, time)
    }
}

#[derive(Copy, Clone, Debug)]
enum LogLine {
    WakeUp(Date),
    FallAsleep(Date),
    Guard(Date, u16),
}

impl Ord for LogLine {
    fn cmp(&self, other: &LogLine) -> Ordering {
        let date: NaiveDateTime = (*self).into();
        let other_date: NaiveDateTime = (*other).into();
        date.cmp(&other_date)
    }
}

impl PartialOrd for LogLine {
    fn partial_cmp(&self, other: &LogLine) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for LogLine {
    fn eq(&self, other: &LogLine) -> bool {
        let date: NaiveDateTime = (*self).into();
        let other_date: NaiveDateTime = (*other).into();
        date == other_date
    }
}

impl Eq for LogLine {}

impl Into<NaiveDateTime> for LogLine {
    fn into(self) -> NaiveDateTime {
        let d = match self {
            LogLine::WakeUp(d) => d,
            LogLine::FallAsleep(d) => d,
            LogLine::Guard(d, _) => d,
        };

        d.into()
    }
}

#[derive(Copy, Clone, Debug)]
struct LogLineError {}

impl FromStr for LogLine {
    type Err = LogLineError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref re_shift: regex::Regex = regex::Regex::new(
                r"^\[(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})\] Guard #(\d+) begins shift$"
            )
            .unwrap();
            static ref re_falls_asleep: regex::Regex =
                regex::Regex::new(r"^\[(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})\] falls asleep$")
                    .unwrap();
            static ref re_wakeup: regex::Regex =
                regex::Regex::new(r"^\[(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2})\] wakes up$")
                    .unwrap();
        }

        let to_u16 = |i: usize, c: &regex::Captures| -> u16 {
            c.get(i)
                .expect(&format!("Group {} not found", i))
                .as_str()
                .parse()
                .expect(&format!("Couldn't parse group {}", i))
        };

        let to_date = |c: &regex::Captures| -> Date {
            Date {
                year: to_u16(1, c),
                month: to_u16(2, c),
                day: to_u16(3, c),
                hour: to_u16(4, c),
                minute: to_u16(5, c),
            }
        };

        if let Some(c) = re_shift.captures(s) {
            let guard_id = to_u16(6, &c);
            Ok(LogLine::Guard(to_date(&c), guard_id))
        } else if let Some(c) = re_falls_asleep.captures(s) {
            Ok(LogLine::FallAsleep(to_date(&c)))
        } else if let Some(c) = re_wakeup.captures(s) {
            Ok(LogLine::WakeUp(to_date(&c)))
        } else {
            Err(LogLineError {})
        }
    }
}

struct Log {
    lines: Vec<LogLine>,
}

impl<'a, S: AsRef<str>> FromIterator<S> for Log {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut v = Vec::from_iter(
            iter.into_iter()
                .map(|s| LogLine::from_str(s.as_ref()).expect("ugh")),
        );
        v.sort();
        Log { lines: v }
    }
}

impl<'a> IntoIterator for &'a Log {
    type Item = &'a LogLine;
    type IntoIter = ::std::slice::Iter<'a, LogLine>;

    fn into_iter(self) -> Self::IntoIter {
        return (&self.lines).into_iter();
    }
}

struct Shift {
    _start: NaiveDateTime,
    guard: u16,
    naps: Vec<(NaiveDateTime, NaiveDateTime)>,
}

impl Shift {
    fn new(guard: u16, start: NaiveDateTime) -> Shift {
        return Shift {
            _start: start,
            guard: guard,
            naps: vec![],
        };
    }
}

struct Shifts(Vec<Shift>);

impl std::ops::Deref for Shifts {
    type Target = [Shift];

    fn deref(&self) -> &[Shift] {
        let Shifts(ref v) = self;
        return v.deref();
    }
}

impl Shifts {
    fn guard_times(&self) -> HashMap<u16, [u16; 60]> {
        let mut m = HashMap::new();
        let Shifts(ref v) = self;
        for shift in v {
            let entry = m.entry(shift.guard);
            let arr = entry.or_insert_with(|| [0; 60]);
            for (s, e) in &shift.naps {
                for i in s.minute()..e.minute() {
                    arr[i as usize] += 1;
                }
            }
        }

        return m;
    }

    fn max_guard_time(&self) -> (u16, u32, u8) {
        let mut max_guard: u16 = 0;
        let mut max_guard_total: u32 = 0;
        let mut max_guard_minute: u8 = 0;

        let m = self.guard_times();
        for (&k, v) in &m {
            let mut max_min: usize = 0;
            let mut total = 0;
            for ix in 0..60 {
                let kc = v[ix];
                if kc > v[max_min] {
                    max_min = ix;
                }
                total += kc as u32;
            }
            if total >= max_guard_total {
                max_guard = k;
                max_guard_total = total;
                max_guard_minute = max_min as u8;
            }
        }

        return (max_guard, max_guard_total, max_guard_minute);
    }

    fn max_guard_minute(&self) -> (u16, u32, u8) {
        let mut guard: u16 = 0;
        let mut count: u32 = 0;
        let mut minute: u8 = 0;

        let m = self.guard_times();
        for (&g, v) in &m {
            for min in 0..60 {
                if v[min] as u32 > count {
                    guard = g;
                    count = v[min] as u32;
                    minute = min as u8;
                }
            }
        }

        return (guard, count, minute);
    }
}

impl Into<Shifts> for Log {
    fn into(mut self) -> Shifts {
        let mut shifts = vec![];
        let mut guard: Option<(NaiveDateTime, u16)> = None;
        let mut last: Option<NaiveDateTime> = None;
        let mut naps = vec![];
        for line in self.lines.drain(..) {
            match (line, guard, last) {
                (LogLine::Guard(d, g), None, None) => {
                    // First line
                    guard = Some((d.into(), g))
                }
                (LogLine::Guard(d, g), Some((d0, g0)), None) => {
                    // Finished last day...
                    let mut shift = Shift::new(g0, d0);
                    shift.naps.append(&mut naps);
                    shifts.push(shift);
                    // Starting a new day
                    guard = Some((d.into(), g));
                }
                (LogLine::FallAsleep(d), Some(_), None) => {
                    last = Some(d.into());
                }
                (LogLine::WakeUp(d), Some(_), Some(d0)) => {
                    naps.push((d0, d.into()));
                    last = None;
                }
                _ => {
                    panic!("Log lines out of order!");
                }
            }
        }

        if let (Some((d, g)), None) = (guard, last) {
            let mut shift = Shift::new(g, d);
            shift.naps.append(&mut naps);
            shifts.push(shift);
        } else {
            panic!("Leftover log lines");
        }

        return Shifts(shifts);
    }
}

fn main() -> std::io::Result<()> {
    let matches = App::new("Day 4")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT")
                .takes_value(true),
        )
        .get_matches();

    let input_path = matches.value_of("INPUT").unwrap_or("inputs/day4.txt");

    eprintln!("Using input {}", input_path);

    let file = File::open(input_path)?;
    let buf_reader = BufReader::new(file);

    let lines = buf_reader.lines().filter_map(|l| l.ok());
    let log = Log::from_iter(lines);

    println!("Lines: {}", log.lines.len());

    let shifts: Shifts = log.into();
    let Shifts(ref shifts_vec) = shifts;
    println!("Shifts: {}", shifts_vec.len());

    let (guard, total, minute) = shifts.max_guard_time();
    println!(
        "Guard: {}, Total: {}, Minute: {} => {}",
        guard,
        total,
        minute,
        (guard as u64) * (minute as u64)
    );

    let (guard, count, minute) = shifts.max_guard_minute();
    println!(
        "Guard: {}, Count: {}, Minute: {} => {}",
        guard,
        count,
        minute,
        (guard as u64) * (minute as u64)
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() {
        let lines = vec![
            "[1518-11-01 00:00] Guard #10 begins shift",
            "[1518-11-01 00:05] falls asleep",
            "[1518-11-01 00:25] wakes up",
            "[1518-11-01 00:30] falls asleep",
            "[1518-11-01 00:55] wakes up",
            "[1518-11-01 23:58] Guard #99 begins shift",
            "[1518-11-02 00:40] falls asleep",
            "[1518-11-02 00:50] wakes up",
            "[1518-11-03 00:05] Guard #10 begins shift",
            "[1518-11-03 00:24] falls asleep",
            "[1518-11-03 00:29] wakes up",
            "[1518-11-04 00:02] Guard #99 begins shift",
            "[1518-11-04 00:36] falls asleep",
            "[1518-11-04 00:46] wakes up",
            "[1518-11-05 00:03] Guard #99 begins shift",
            "[1518-11-05 00:45] falls asleep",
            "[1518-11-05 00:55] wakes up",
        ];

        let log = Log::from_iter(&lines);
        assert_eq!(log.lines.len(), lines.len());

        let shifts: Shifts = log.into();

        assert_eq!(shifts.len(), 5);

        let (max_guard, max_guard_total, max_guard_minute) = shifts.max_guard_time();

        assert_eq!(10, max_guard);
        assert_eq!(50, max_guard_total);
        assert_eq!(24, max_guard_minute);

        let (guard, count, minute) = shifts.max_guard_minute();

        assert_eq!(99, guard);
        assert_eq!(3, count);
        assert_eq!(45, minute);
    }
}
