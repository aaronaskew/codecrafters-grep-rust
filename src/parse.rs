use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::character::complete::alpha1;
use nom::character::complete::char;
use nom::combinator::opt;
use nom::multi::many1;

use crate::Atom;
use crate::AtomKind;
use crate::Matcher;
use crate::QuantifierKind;

fn negative_group(input: &str) -> IResult<&str, AtomKind> {
    (tag("[^"), alpha1, tag("]"))
        .map(|(_, chars, _)| AtomKind::NegativeCharGroup(String::from(chars)))
        .parse(input)
}

fn positive_group(input: &str) -> IResult<&str, AtomKind> {
    (tag("["), alpha1, tag("]"))
        .map(|(_, chars, _)| AtomKind::PositiveCharGroup(String::from(chars)))
        .parse(input)
}

fn digit(input: &str) -> IResult<&str, AtomKind> {
    tag("\\d").map(|_| AtomKind::Digit).parse(input)
}

fn word_char(input: &str) -> IResult<&str, AtomKind> {
    tag("\\w").map(|_| AtomKind::WordChar).parse(input)
}

fn single_char(input: &str) -> IResult<&str, AtomKind> {
    take(1u8)
        .map(|s: &str| AtomKind::SingleChar(s.chars().next().unwrap()))
        .parse(input)
}

fn quantifier(input: &str) -> IResult<&str, QuantifierKind> {
    alt((char('+'), char('*'), char('?')))
        .map(|c| match c {
            '+' => QuantifierKind::OneOrMore,
            '?' => QuantifierKind::ZeroOrOne,
            '*' => QuantifierKind::ZeroOrMore,
            _ => panic!(),
        })
        .parse(input)
}

fn parse_pattern(input: &str) -> IResult<&str, (bool, Vec<Atom>, bool)> {
    let char_vec = input.chars().collect::<Vec<_>>();

    // Determine is there is a start anchor and remove the '^' if so
    let start_anchor = char_vec.first().unwrap() == &'^';
    let input = if start_anchor {
        input
            .char_indices()
            .nth(1) // Get the byte index of the 2nd character
            .map(|(i, _)| &input[i..]) // Slice from that index to the end
            .unwrap_or("") // Return empty string if string had 0 or 1 chars
    } else {
        input
    };

    // Determine if there is an end anchor and remove the '$' if so
    let end_anchor = char_vec.last().unwrap() == &'$';
    let input = if end_anchor {
        input
            .char_indices()
            .next_back() // Get the byte index of the last character
            .map(|(i, _)| &input[..i]) // Slice from the start to that index
            .unwrap_or(input) // Return original string if empty
    } else {
        input
    };

    let Ok((input, atom_vec)) = many1(
        (
            alt((
                negative_group,
                positive_group,
                digit,
                word_char,
                single_char,
            )),
            opt(quantifier),
        )
            .map(|(kind, quantifier)| Atom { kind, quantifier }),
    )
    .parse(input) else {
        panic!()
    };

    Ok((input, (start_anchor, atom_vec, end_anchor)))
}

pub fn parse(input: &str) -> Matcher {
    let Ok((_, (has_start_anchor, atoms, has_end_anchor))) = parse_pattern(input) else {
        panic!("Cannot parse");
    };

    Matcher {
        atoms,
        has_start_anchor,
        has_end_anchor,
    }
}
