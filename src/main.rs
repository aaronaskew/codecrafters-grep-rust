use std::collections::HashSet;
use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

mod parse;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Atom {
    kind: AtomKind,
    quantifier_opt: Option<QuantifierKind>,
    minimum_satisfied: bool,
}

impl Atom {
    fn _match_required(&self) -> bool {
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

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
enum QuantifierKind {
    OneOrMore,
    ZeroOrMore,
    ZeroOrOne,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
enum AtomKind {
    SingleChar(char),
    Digit,
    WordChar,
    PositiveCharGroup(String),
    NegativeCharGroup(String),
}

#[derive(Debug)]
struct AtomMatcher {
    atoms: Vec<Atom>,
    has_start_anchor: bool,
    has_end_anchor: bool,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct SequenceNode {
    sequence: Vec<AtomNode>,
}

impl SequenceNode {
    /// Generates all valid combinations of AtomNodes for the given atoms
    /// `max_chars` is the total number of characters the entire sequence is allowed to consume.
    pub fn build_all(atoms: &[Atom], max_chars: usize) -> Vec<SequenceNode> {
        let mut results = Vec::new();
        let mut current_path = Vec::new();

        Self::backtrack(
            atoms,
            0,         // Start at the first atom
            max_chars, // Initial characters available to consume
            &mut current_path,
            &mut results,
        );

        results
    }

    fn backtrack(
        atoms: &[Atom],
        atom_idx: usize,
        remaining_chars: usize,
        current_path: &mut Vec<AtomNode>,
        results: &mut Vec<SequenceNode>,
    ) {
        // Base case: We have successfully assigned consumptions to all atoms
        if atom_idx == atoms.len() {
            results.push(SequenceNode {
                sequence: current_path.clone(),
            });
            return;
        }

        // Generate the possible configurations for the current Atom based on remaining space
        let base_node = AtomNode::from_atom(atoms[atom_idx].clone());
        let children = base_node.generate_children(remaining_chars);

        // DFS: Explore each possible consumption amount
        for child in children {
            // Determine how many chars this specific child consumes
            let consumed = child.num_chars_to_consume_opt.unwrap_or(0);

            // Push onto the sequence
            current_path.push(child);

            // Recurse to the next atom, subtracting the consumed characters from the limit
            Self::backtrack(
                atoms,
                atom_idx + 1,
                remaining_chars.saturating_sub(consumed),
                current_path,
                results,
            );

            // Pop off to evaluate the next sibling configuration
            current_path.pop();
        }
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct AtomNode {
    atom: Atom,
    num_chars_to_consume_opt: Option<usize>,
}

impl AtomNode {
    fn from_atom(atom: Atom) -> Self {
        Self {
            atom,
            num_chars_to_consume_opt: None,
        }
    }

    fn generate_children(&self, num_chars: usize) -> Vec<AtomNode> {
        let mut children = vec![];

        match self.atom.quantifier_opt {
            Some(q) => match q {
                QuantifierKind::OneOrMore => {
                    for i in 1..=num_chars {
                        children.push(AtomNode {
                            atom: self.atom.clone(),
                            num_chars_to_consume_opt: Some(i),
                        })
                    }
                }
                QuantifierKind::ZeroOrMore => {
                    for i in 0..=num_chars {
                        children.push(AtomNode {
                            atom: self.atom.clone(),
                            num_chars_to_consume_opt: Some(i),
                        })
                    }
                }
                QuantifierKind::ZeroOrOne => {
                    for i in 0..=1 {
                        children.push(AtomNode {
                            atom: self.atom.clone(),
                            num_chars_to_consume_opt: Some(i),
                        })
                    }
                }
            },
            None => children.push(AtomNode {
                atom: self.atom.clone(),
                num_chars_to_consume_opt: Some(1),
            }),
        }

        // eprintln!("children: {children:?}\n");

        children.into_iter().rev().collect()
    }
}

impl AtomMatcher {
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

    // /// Consumes first characters of `chars` until it matches (if at all) the first atom.
    // /// Consumes the first atom if the quantifier is None or ZeroOrOne
    // /// Returns whether or not there was a match.
    // fn find_first_match(&mut self, chars: &mut VecDeque<char>) -> bool {
    //     // TODO: test for multiple starting characters

    //     // find first char that matches pattern, if any
    //     let first_atom = self.atoms.first().unwrap();
    //     eprint!("first atom: {first_atom:?} ");
    //     loop {
    //         if let Some(current_char) = chars.pop_front() {
    //             if Self::matches_char(current_char, first_atom) {
    //                 eprintln!("matches char: \'{current_char}\'");
    //                 eprintln!("remaining chars: {chars:?}");
    //                 break;
    //             }
    //         } else {
    //             eprintln!("doesn't match any chars");
    //             return false;
    //         }
    //     }

    //     match first_atom.quantifier_opt {
    //         None | Some(QuantifierKind::ZeroOrOne) => {
    //             self.atoms.remove(0);
    //         }
    //         _ => {}
    //     }

    //     eprintln!("remaining atoms: {:?}", self.atoms);

    //     true
    // }

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

    fn match_dfs(&self, chars: Vec<char>) -> bool {
        //let mut seq_visited: HashSet<SequenceNode> = HashSet::new();
        //let mut seq_stack: Vec<SequenceNode> = vec![];

        // TODO: build sequence stack

        let mut sequences = SequenceNode::build_all(&self.atoms, chars.len());

        sequences.reverse();

        dbg!(&sequences);

        'sequence: for sequence_node in sequences {
            let mut sequence = VecDeque::from(sequence_node.sequence);

            let mut test_chars = VecDeque::from(chars.clone());

            while let Some(AtomNode {
                atom,
                num_chars_to_consume_opt: Some(num_chars_to_consume),
            }) = sequence.pop_front()
            {
                let mut chars_consumed = 0;
                for _ in 0..num_chars_to_consume {
                    if let Some(current_char) = test_chars.pop_front() {
                        if !AtomMatcher::matches_char(current_char, &atom) {
                            break;
                        } else {
                            chars_consumed += 1;
                        }
                    } else {
                        break;
                    }
                }

                if chars_consumed < num_chars_to_consume {
                    continue 'sequence;
                }
            }

            if sequence.is_empty() {
                return true;
            }
        }

        // while let Some(mut curr_sequence_node) = sequences.pop() {
        //     // if seq_visited.contains(&curr_sequence_node) {
        //     //     continue;
        //     // }
        //     curr_sequence_node.sequence.reverse();

        //     let mut test_chars = chars.clone();
        //     // test_chars.reverse();
        //     let mut node_visited: HashSet<AtomNode> = HashSet::new();
        //     let mut node_stack: Vec<AtomNode> = vec![];
        //     while let Some(curr_atom_node) = curr_sequence_node.sequence.pop() {
        //         if node_visited.contains(&curr_atom_node) {
        //             continue;
        //         }

        //         if let Some(curr_char) = test_chars.pop() {
        //             eprintln!(
        //                 "Visiting Node -> Atom: {:?}, num_chars_to_consume: {:?}, curr_char: {curr_char}\n",
        //                 curr_atom_node.atom, curr_atom_node.num_chars_to_consume_opt
        //             );

        //             if AtomMatcher::matches_char(curr_char, &curr_atom_node.atom) {
        //                 continue;
        //             } else {
        //                 return false;
        //             }

        //             node_visited.insert(curr_atom_node.clone());

        //             let children = curr_atom_node.generate_children(chars.len());
        //             for child in children.into_iter() {
        //                 if !node_visited.contains(&child) {
        //                     node_stack.push(child);
        //                 }
        //             }
        //         } else {
        //             let remaining_chars_to_consume = curr_sequence_node
        //                 .sequence
        //                 .iter()
        //                 .map(|a| a.num_chars_to_consume_opt.unwrap_or(0))
        //                 .sum::<usize>();

        //             if remaining_chars_to_consume > 0 {
        //                 continue;
        //             }
        //         }
        //     }

        //     if curr_sequence_node.sequence.is_empty() {
        //         return true;
        //     }
        // }

        false
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let atom_matcher = parse::parse(pattern);
    // eprintln!("atom_matcher: {atom_matcher:#?}");

    // let chars = input_line.chars().collect::<VecDeque<_>>();
    let chars = input_line.chars().collect();

    // handle_anchors(atom_matcher, chars)

    atom_matcher.match_dfs(chars)
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
