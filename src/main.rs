use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern.chars().count() == 1 {
        input_line.contains(pattern)
    } else if pattern == "\\d" {
        input_line.contains(|c: char| c.is_ascii_digit())
    } else if pattern == "\\w" {
        input_line.contains(|c: char| c.is_ascii_alphanumeric() || c == '_')
    } else if pattern.starts_with('[') && pattern.ends_with(']') && pattern.chars().count() > 2 {
        let mut group = pattern.chars().collect::<VecDeque<_>>();
        group.pop_front();
        group.pop_back();

        let is_negative_group = group[0] == '^';

        if is_negative_group {
            group.pop_front();
        }

        input_line.contains(|c| group.contains(&c)) && !is_negative_group
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
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

    // TODO: Uncomment the code below to pass the first stage
    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
