use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

mod parse;

#[derive(Debug)]
struct Atom {
    kind: AtomKind,
    quantifier: Option<QuantifierKind>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum QuantifierKind {
    OneOrMore,
    ZeroOrMore,
    ZeroOrOne,
}

#[derive(Debug)]
enum AtomKind {
    SingleChar(char),
    Digit,
    WordChar,
    PositiveCharGroup(String),
    NegativeCharGroup(String),
}

#[derive(Debug)]
struct Matcher {
    atoms: Vec<Atom>,
    has_start_anchor: bool,
    has_end_anchor: bool,
}

impl Matcher {
    fn max_length(&self) -> usize {
        if self.atoms.iter().any(|atom| {
            let Some(quantifier) = &atom.quantifier else {
                return false;
            };

            matches!(
                quantifier,
                QuantifierKind::OneOrMore | QuantifierKind::ZeroOrMore
            )
        }) {
            return usize::MAX;
        }

        self.atoms.len()
    }

    fn min_length(&self) -> usize {
        self.atoms
            .iter()
            .map(|atom| {
                if let Some(quantifier) = &atom.quantifier {
                    if matches!(
                        quantifier,
                        QuantifierKind::ZeroOrMore | QuantifierKind::ZeroOrOne
                    ) {
                        0
                    } else {
                        1
                    }
                } else {
                    1
                }
            })
            .sum()
    }

    /// Consumes first characters of `input_vec_deque` until it matches (if at all) the first atom.
    /// Consumes the first atom if the quantifier is None or ZeroOrOne
    /// Returns whether or not there was a match.
    fn find_first_match(&mut self, input_vec_deque: &mut VecDeque<char>) -> bool {
        // find first char that matches pattern, if any
        let first_atom = self.atoms.first().unwrap();
        eprint!("first atom: {first_atom:?} ");
        loop {
            if let Some(current_char) = input_vec_deque.pop_front() {
                if matches_char(current_char, &first_atom.kind) {
                    eprintln!("matches char: \'{current_char}\'");
                    break;
                }
            } else {
                eprintln!("doesn't match any chars");
                return false;
            }
        }

        match first_atom.quantifier {
            None | Some(QuantifierKind::ZeroOrOne) => {
                self.atoms.remove(0);
            }
            _ => {}
        }

        true
    }

    fn match_atoms(atoms: Vec<Atom>, input_vec_deque: VecDeque<char>) -> bool {
        // for char_i in 0..input_vec_deque.len() {
        //     for atom_i in 0..atoms.len() {
        //         if char_i > 0 && atom_i > 0 {
        //             if matches_char(input_vec_deque[char_i], &atoms[atom_i].kind)
        //                 && matches_char(input_vec_deque[char_i], &atoms[atom_i - 1].kind)
        //             {
        //                 // Previous
        //             }
        //         }
        //     }
        // }

        for (atom, current_char) in atoms.iter().zip(input_vec_deque.iter()) {
            eprintln!("match: {atom:?} current_char: {current_char}");

            if !matches_char(*current_char, &atom.kind) {
                eprintln!("doesn't match char: \'{current_char}\'");
                return false;
            } else {
                eprintln!("matches char: \'{current_char}\'");
            }
        }

        true
    }
}

fn matches_char(current_char: char, atom_kind: &AtomKind) -> bool {
    match atom_kind {
        AtomKind::SingleChar(ch) => {
            if current_char != *ch {
                return false;
            }
        }
        AtomKind::Digit => {
            if !current_char.is_numeric() {
                return false;
            }
        }
        AtomKind::WordChar => {
            if !(current_char.is_alphanumeric() || current_char == '_') {
                return false;
            }
        }
        AtomKind::PositiveCharGroup(group) => {
            if !group.contains(current_char) {
                return false;
            }
        }
        AtomKind::NegativeCharGroup(group) => {
            if group.contains(current_char) {
                return false;
            }
        }
    }

    true
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let mut matcher = parse::parse(pattern);
    eprintln!("matcher: {matcher:#?}");

    let mut input_vec_deque = input_line.chars().collect::<VecDeque<_>>();

    match (matcher.has_start_anchor, matcher.has_end_anchor) {
        (true, true) => {
            if input_vec_deque.len() > matcher.max_length()
                || input_vec_deque.len() < matcher.min_length()
            {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, input_vec_deque) {
                return false;
            }
        }
        (true, false) => {
            if input_vec_deque.len() < matcher.min_length() {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, input_vec_deque) {
                return false;
            }
        }
        (false, true) => {
            if input_vec_deque.len() < matcher.min_length() {
                return false;
            }

            let input_vec_deque_rev = input_vec_deque.into_iter().rev().collect::<VecDeque<_>>();
            matcher.atoms.reverse();

            if !Matcher::match_atoms(matcher.atoms, input_vec_deque_rev) {
                return false;
            }
        }
        (false, false) => {
            dbg!(&input_vec_deque.len(), &matcher.min_length());

            if input_vec_deque.len() < matcher.min_length() {
                return false;
            }

            if !matcher.find_first_match(&mut input_vec_deque) {
                return false;
            }

            if input_vec_deque.len() < matcher.min_length() {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, input_vec_deque) {
                return false;
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
