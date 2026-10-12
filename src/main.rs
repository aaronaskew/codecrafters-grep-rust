use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

mod parse;

#[derive(Debug, Clone)]
struct Atom {
    kind: AtomKind,
    quantifier_opt: Option<QuantifierKind>,
    minimum_satisfied: bool,
}

impl Atom {
    fn match_required(&self) -> bool {
        // - if current atom has no quantifier & minimum_satisfied is no, yes, otherwise no
        // - if OneOrMore, if minimum already satisfied, no, else yes
        // - if ZeroOrMore, no
        // - if ZeroOrOne, no

        if let Some(quantifier) = self.quantifier_opt {
            match quantifier {
                QuantifierKind::OneOrMore => !self.minimum_satisfied,
                QuantifierKind::ZeroOrMore | QuantifierKind::ZeroOrOne => false,
            }
        } else {
            !self.minimum_satisfied
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum QuantifierKind {
    OneOrMore,
    ZeroOrMore,
    ZeroOrOne,
}

#[derive(Debug, Clone)]
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
    fn max_input_length(&self) -> usize {
        if self.atoms.iter().any(|atom| {
            let Some(quantifier) = &atom.quantifier_opt else {
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

    fn min_input_length(&self) -> usize {
        self.atoms
            .iter()
            .map(|atom| {
                if let Some(quantifier) = &atom.quantifier_opt {
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

    /// Consumes first characters of `chars` until it matches (if at all) the first atom.
    /// Consumes the first atom if the quantifier is None or ZeroOrOne
    /// Returns whether or not there was a match.
    fn find_first_match(&mut self, chars: &mut VecDeque<char>) -> bool {
        // TODO: test for multiple starting characters

        // find first char that matches pattern, if any
        let first_atom = self.atoms.first().unwrap();
        eprint!("first atom: {first_atom:?} ");
        loop {
            if let Some(current_char) = chars.pop_front() {
                if Self::matches_char(current_char, first_atom) {
                    eprintln!("matches char: \'{current_char}\'");
                    eprintln!("remaining chars: {chars:?}");
                    break;
                }
            } else {
                eprintln!("doesn't match any chars");
                return false;
            }
        }

        match first_atom.quantifier_opt {
            None | Some(QuantifierKind::ZeroOrOne) => {
                self.atoms.remove(0);
            }
            _ => {}
        }

        eprintln!("remaining atoms: {:?}", self.atoms);

        true
    }

    fn matches_char(current_char: char, atom: &Atom) -> bool {
        match &atom.kind {
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

    fn match_atoms(atoms: Vec<Atom>, chars: Vec<char>) -> bool {
        let mut atom_iter = atoms.into_iter().enumerate().peekable();
        let mut char_iter = chars.iter().enumerate().peekable();

        let mut advance_char = Some(false);
        let mut current_char_opt = char_iter.next();

        let mut advance_atom = Some(false);
        let mut current_atom_opt = atom_iter.next();

        loop {
            if advance_char.is_some_and(|advance| advance) {
                eprintln!("advance_char=true, current_char before: {current_char_opt:?}");
                current_char_opt = char_iter.next();
                eprintln!("advance_char=true, current_char after: {current_char_opt:?}");
            }

            if advance_atom.is_some_and(|advance| advance) {
                eprintln!("advance_atom=true, current_atom before: {current_atom_opt:?}");
                current_atom_opt = atom_iter.next();
                eprintln!("advance_atom=true, current_atom after: {current_atom_opt:?}");
            }

            // dbg!(&current_char_opt, &current_atom_opt);

            advance_atom = None;
            advance_char = None;

            match (current_atom_opt.as_mut(), current_char_opt) {
                // All atoms satisfied, return true
                (None, _) => {
                    return true;
                }
                // No more characters, check that no match is required of this and subsequent atoms
                (Some((_, current_atom)), None) => {
                    if current_atom.match_required() {
                        return false;
                    } else {
                        advance_atom = Some(true);
                        advance_char = Some(false);
                        continue;
                    }
                }
                // Character & Atom available
                (
                    Some(&mut (_current_atom_pos, ref mut current_atom)),
                    Some((_current_char_pos, current_char)),
                ) => {
                    if current_atom.match_required() {
                        eprintln!("match required");

                        if !Self::matches_char(*current_char, current_atom) {
                            return false;
                        }

                        // Match required and matched - advance char do not advance Atom
                        current_atom.minimum_satisfied = true;
                        advance_char = Some(true);
                        advance_atom = Some(false);
                    } else {
                        // Match not required
                        match current_atom.quantifier_opt {
                            // Match not required and quantifier exists
                            Some(quantifier) => match quantifier {
                                QuantifierKind::OneOrMore => {
                                    // TODO: Do some depth/breadth first search madness

                                    // test if next atom will match this char. if so, advance atom but not char
                                    if let Some((_, next_atom)) = atom_iter.peek() {
                                        if Self::matches_char(*current_char, next_atom) {
                                            advance_atom = Some(true);
                                            advance_char = Some(false);
                                        }
                                    } else {
                                        // Next atom doesn't match this char
                                        if !Self::matches_char(*current_char, current_atom) {
                                            return false;
                                        }

                                        // This atom matches this char. Advance char but not atom.
                                        advance_char = Some(true);
                                        advance_atom = Some(false);
                                    }
                                }
                                QuantifierKind::ZeroOrMore => todo!(),
                                QuantifierKind::ZeroOrOne => todo!(),
                            },
                            // no quantifier
                            None => {
                                // Match not required and no quantifier: advance atom. do not advance char
                                advance_atom = Some(true);
                                advance_char = Some(false);
                            }
                        }
                    }
                }
            }

            assert!(advance_atom.is_some() && advance_char.is_some());

            eprintln!(
                "Match!: Atom: {:?} char: {}",
                current_atom_opt
                    .clone()
                    .expect("current_atom_opt should be some if there was a match")
                    .1,
                current_char_opt
                    .expect("current_char_opt should be some if there was a match")
                    .1
            );

            eprintln!(
                "advance_atom: {} advance_char: {}",
                advance_atom.unwrap(),
                advance_char.unwrap()
            );
        }
    }
}

fn handle_anchors(mut matcher: Matcher, mut chars: VecDeque<char>) -> bool {
    match (matcher.has_start_anchor, matcher.has_end_anchor) {
        (true, true) => {
            if chars.len() > matcher.max_input_length() || chars.len() < matcher.min_input_length()
            {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, Vec::from(chars)) {
                return false;
            }
        }
        (true, false) => {
            if chars.len() < matcher.min_input_length() {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, Vec::from(chars)) {
                return false;
            }
        }
        (false, true) => {
            if chars.len() < matcher.min_input_length() {
                return false;
            }

            // If only end anchor, work from back to front.
            matcher.atoms.reverse();
            matcher.has_start_anchor = !matcher.has_start_anchor;
            matcher.has_end_anchor = !matcher.has_end_anchor;
            let chars = chars.into_iter().rev().collect::<Vec<_>>();

            if !Matcher::match_atoms(matcher.atoms, chars) {
                return false;
            }
        }
        (false, false) => {
            dbg!(&chars.len(), &matcher.min_input_length());

            if chars.len() < matcher.min_input_length() {
                return false;
            }

            if !matcher.find_first_match(&mut chars) {
                return false;
            }

            dbg!(&chars.len(), &matcher.min_input_length());

            if chars.len() < matcher.min_input_length() {
                return false;
            }

            if !Matcher::match_atoms(matcher.atoms, Vec::from(chars)) {
                return false;
            }
        }
    }

    true
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let matcher = parse::parse(pattern);
    eprintln!("matcher: {matcher:#?}");

    let chars = input_line.chars().collect::<VecDeque<_>>();

    handle_anchors(matcher, chars)
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
