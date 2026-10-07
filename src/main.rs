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

    let mut input_vec = input_line.chars().collect::<VecDeque<_>>();

    // // Determine is there is a start anchor and remove the '^' if so
    // let start_anchor = char_vec.first().unwrap() == &'^';
    // let input = if start_anchor {
    //     input
    //         .char_indices()
    //         .nth(1) // Get the byte index of the 2nd character
    //         .map(|(i, _)| &input[i..]) // Slice from that index to the end
    //         .unwrap_or("") // Return empty string if string had 0 or 1 chars
    // } else {
    //     input
    // };

    // // Determine if there is 
    // let end_anchor = char_vec.last().unwrap() == &'$';
    // let input = if end_anchor {
    //     input
    //         .char_indices()
    //         .next_back() // Get the byte index of the last character
    //         .map(|(i, _)| &input[..i]) // Slice from the start to that index
    //         .unwrap_or(input) // Return original string if empty
    // } else {
    //     input
    // };
    
    if matcher.end_anchor {
        input_vec.pop_back();
    }

    let continue_if_fail;

    match (matcher.start_anchor, matcher.end_anchor) {
        (true, true) => {
            input_vec.pop_front();
            input_vec.pop_back();
            continue_if_fail = false;
        }
        (true, false) => {
            input_vec.pop_front();
            continue_if_fail = false;
        }
        (false, true) => {
            input_vec.pop_back();
            continue_if_fail = false;
        }
        (false, false) => {
            continue_if_fail=true;
        }
    }

         

    if matcher.start_anchor {
        if !matches_char(input_vec.pop_front().unwrap(), matcher.matches.first().unwrap()) {
            return false;
        }
    } else {
        // find first char that matches pattern, if any
        let first_match = matcher.matches.first().unwrap();
        eprint!("first match: {first_match:?} ");
        loop {
            if let Some(current_char) = input_vec.pop_front() {
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

    if matcher.matches.iter().skip(1).count() > input_vec.len() {
        return false;
    }

    if matcher.end_anchor && matcher.matches.iter().skip(1).count() != input_vec.len() {
        return false;
    }

    for (match_kind, current_char) in matcher.matches.iter().skip(1).zip(input_vec.iter()) {
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
