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
    let matches = parse::parse(pattern);
    eprintln!("matches: {matches:#?}");

    let mut input = input_line.chars().collect::<VecDeque<_>>();

    // find first char that matches pattern, if any
    let first_match = matches.first().unwrap();
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

    for match_kind in matches.iter().skip(1) {
        eprint!("match: {match_kind:?} ");
        if let Some(current_char) = input.pop_front() {
            eprintln!("current_char: {current_char}");
            if !matches_char(current_char, match_kind) {
                eprintln!("doesn't match char: \'{current_char}\'");
                return false;
            } else {
                eprintln!("matches char: \'{current_char}\'");
            }
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
