#![warn(clippy::all)]

use log::{debug, warn};
use nom::{alt, digit, map_res, opt, pair, recognize, tag};

use std::str::FromStr;

// convert_err converts a nom::Err<&str, ..> into a nom::Err<String, ..> by
// cloning. This allows code to read and parse without allocating until
// an error is hit, and only allocate then - so that the resulting value-or-error
// can be passed up the chain independently of the input.
pub fn convert_err<F>(err: nom::Err<&str, F>) -> nom::Err<String, F> {
    use nom::simple_errors::Context::Code;
    use nom::Err::{Error, Failure, Incomplete};
    match err {
        Incomplete(n) => Incomplete(n),
        Error(Code(s, ek)) => Error(Code(s.to_owned(), ek)),
        Failure(Code(s, ek)) => Failure(Code(s.to_owned(), ek)),
    }
}

// Parse an integer from an input.
// There are a lot of requirements on the input, but
// both &str and CompleteStr ought to work here.
pub fn parse_integer<'a, T>(input: T) -> nom::IResult<T, i64>
where
    T: Clone
        + AsRef<str>
        + nom::InputTake
        + nom::InputTakeAtPosition
        + nom::Slice<std::ops::Range<usize>>
        + nom::Slice<std::ops::RangeFrom<usize>>
        + nom::Slice<std::ops::RangeTo<usize>>
        + nom::Offset
        + nom::AtEof
        + nom::Compare<&'a str>,
    <T as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
{
    map_res!(
        input,
        recognize!(pair!(opt!(alt!(tag!("+") | tag!("-"))), digit)),
        |s: T| FromStr::from_str(s.as_ref())
    )
}

// Parse a series of items from iterator.
pub fn parse_lines_err<E1, E2, S, T, F, Item>(f: F, iter: T) -> Result<Vec<Item>, failure::Error>
where
    E1: Into<failure::Error>,
    E2: Into<failure::Error>,
    F: Fn(&str) -> Result<Item, E1>,
    S: AsRef<str>,
    T: IntoIterator<Item = Result<S, E2>>,
    Item: std::fmt::Debug,
{
    iter.into_iter()
        .filter_map(|rl| match rl {
            Err(e) => {
                let e = e.into();
                warn!("  Error getting line: {}", e);
                Some(Err(e))
            }
            Ok(l) => {
                let trimmed = l.as_ref().trim();
                if trimmed.is_empty() {
                    None
                } else {
                    let fd = f(trimmed).map_err(|e| e.into());
                    match fd {
                        Ok(ref i) => debug!("  Parsed line '{}' -> {:?}", trimmed, i),
                        Err(ref e) => warn!("  Error parsing line '{}': {}", trimmed, e),
                    }
                    Some(fd)
                }
            }
        })
        .collect()
}

pub fn parse_lines<E, S, T, F, Item>(f: F, iter: T) -> Result<Vec<Item>, failure::Error>
where
    E: Into<failure::Error>,
    F: Fn(&str) -> Result<Item, E>,
    S: AsRef<str>,
    T: IntoIterator<Item = S>,
    Item: std::fmt::Debug,
{
    parse_lines_err::<_, failure::Error, _, _, _, _>(f, iter.into_iter().map(Ok))
}

pub fn parse_str<E, F, S, Item>(f: F, s: S) -> Result<Vec<Item>, failure::Error>
where
    E: Into<failure::Error>,
    F: Fn(&str) -> Result<Item, E>,
    S: AsRef<str>,
    Item: std::fmt::Debug,
{
    parse_lines(f, s.as_ref().split('\n'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::types::CompleteStr;

    #[test]
    fn test_integer_parse() {
        let parsed = parse_integer(CompleteStr("-120"));
        println!("Parsed: {:?}", parsed);
        assert_eq!(parsed, Ok((CompleteStr(""), -120)));
    }

    #[test]
    fn test_integer_positive() {
        let parsed = parse_integer(CompleteStr("+120"));
        println!("Parsed: {:?}", parsed);
        assert_eq!(parsed, Ok((CompleteStr(""), 120)));
    }
}
