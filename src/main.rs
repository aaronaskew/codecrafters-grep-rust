use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

mod parse;

#[derive(Debug)]
enum MatchKind {
    SingleChar(char),
    Digit,
    WordChar,
    PositiveCharGroup(String),
    NegativeCharGroup(String),
}

#[derive(Debug)]
struct Matcher {
    matches: Vec<MatchKind>,
    start_anchor: bool,
    end_anchor: bool,
}

fn matches_char(current_char: char, match_kind: &MatchKind) -> bool {
    match match_kind {
        MatchKind::SingleChar(ch) => {
            if current_char != *ch {
                return false;
            }
        }
        MatchKind::Digit => {
            if !current_char.is_numeric() {
                return false;
            }
        }
        MatchKind::WordChar => {
            if !(current_char.is_alphanumeric() || current_char == '_') {
                return false;
            }
        }
        MatchKind::PositiveCharGroup(group) => {
            if !group.contains(current_char) {
                return false;
            }
        }
        MatchKind::NegativeCharGroup(group) => {
            if group.contains(current_char) {
                return false;
            }
        }
    }

    true
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let matcher = parse::parse(pattern);
    eprintln!("matcher: {matcher:#?}");

    let mut input = input_line.chars().collect::<VecDeque<_>>();

    if matcher.end_anchor {
        input.pop_back();
    }

    if matcher.start_anchor {
        if !matches_char(input.pop_front().unwrap(), matcher.matches.first().unwrap()) {
            return false;
        }
    } else {
        // find first char that matches pattern, if any
        let first_match = matcher.matches.first().unwrap();
        eprint!("first match: {first_match:?} ");
        loop {
            if let Some(current_char) = input.pop_front() {
                if matches_char(current_char, first_match) {
                    eprintln!("matches char: \'{current_char}\'");
                    break;
                }
            } else {
                eprintln!("doesn't match any chars");
                return false;
            }
        }
    }

    if matcher.matches.iter().skip(1).count() > input.len() {
        return false;
    }

    if matcher.end_anchor && matcher.matches.iter().skip(1).count() != input.len() {
        return false;
    }

    for (match_kind, current_char) in matcher.matches.iter().skip(1).zip(input.iter()) {
        eprintln!("match: {match_kind:?} current_char: {current_char}");

        if !matches_char(*current_char, match_kind) {
            eprintln!("doesn't match char: \'{current_char}\'");
            return false;
        } else {
            eprintln!("matches char: \'{current_char}\'");
        }
    }

    true
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
