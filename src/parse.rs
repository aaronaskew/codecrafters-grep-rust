use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::character::complete::alpha1;
use nom::multi::many1;

use crate::MatchKind;
use crate::Matcher;

fn negative_group(input: &str) -> IResult<&str, MatchKind> {
    (tag("[^"), alpha1, tag("]"))
        .map(|(_, chars, _)| MatchKind::NegativeCharGroup(String::from(chars)))
        .parse(input)
}

fn positive_group(input: &str) -> IResult<&str, MatchKind> {
    (tag("["), alpha1, tag("]"))
        .map(|(_, chars, _)| MatchKind::PositiveCharGroup(String::from(chars)))
        .parse(input)
}

fn digit(input: &str) -> IResult<&str, MatchKind> {
    tag("\\d").map(|_| MatchKind::Digit).parse(input)
}

fn word_char(input: &str) -> IResult<&str, MatchKind> {
    tag("\\w").map(|_| MatchKind::WordChar).parse(input)
}

fn single_char(input: &str) -> IResult<&str, MatchKind> {
    take(1u8)
        .map(|s: &str| MatchKind::SingleChar(s.chars().next().unwrap()))
        .parse(input)
}

fn parse_pattern(input: &str) -> IResult<&str, (bool, Vec<MatchKind>, bool)> {
    let char_vec = input.chars().collect::<Vec<_>>();

    let start_anchor = char_vec.first().unwrap() == &'^';
    let end_anchor = char_vec.last().unwrap() == &'$';

    let Ok((input, match_vec)) = many1(alt((
        negative_group,
        positive_group,
        digit,
        word_char,
        single_char,
    )))
    .parse(input) else {
        panic!()
    };

    Ok((input, (start_anchor, match_vec, end_anchor)))
}

pub fn parse(input: &str) -> Matcher {
    let Ok((_, (start_anchor, matches, end_anchor))) = parse_pattern(input) else {
        panic!("Cannot parse");
    };

    Matcher {
        matches,
        start_anchor,
        end_anchor,
    }
}
