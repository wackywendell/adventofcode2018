#![warn(clippy::all)]

#[macro_use]
extern crate nom;

use nom::digit;

use std::str::FromStr;

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
        recognize!(pair!(opt!(alt!(tag_s!("+") | tag_s!("-"))), digit)),
        |s: T| FromStr::from_str(s.as_ref())
    )
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
