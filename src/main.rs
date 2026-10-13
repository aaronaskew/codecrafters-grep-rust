use std::collections::HashSet;
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
struct AtomNode {
    atom: Atom,
    num_chars_to_consume_opt: Option<usize>,
    start_char_idx: usize,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct SequenceNode {
    seqeunce: Vec<AtomNode>,
}

impl AtomNode {
    fn from_atom(atom: Atom) -> Self {
        Self {
            atom,
            num_chars_to_consume_opt: None,
            start_char_idx: 0,
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
                            start_char_idx: 0,
                        })
                    }
                }
                QuantifierKind::ZeroOrMore => {
                    for i in 0..=num_chars {
                        children.push(AtomNode {
                            atom: self.atom.clone(),
                            num_chars_to_consume_opt: Some(i),
                            start_char_idx: 0,
                        })
                    }
                }
                QuantifierKind::ZeroOrOne => {
                    for i in 0..=1 {
                        children.push(AtomNode {
                            atom: self.atom.clone(),
                            num_chars_to_consume_opt: Some(i),
                            start_char_idx: 0,
                        })
                    }
                }
            },
            None => children.push(AtomNode {
                atom: self.atom.clone(),
                num_chars_to_consume_opt: Some(1),
                start_char_idx: 0,
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
        let mut seq_visited: HashSet<Vec<AtomNode>> = HashSet::new();
        let mut seq_stack: Vec<Vec<AtomNode>> = vec![];

        // TODO: build sequence stack
        // maybe create another node type for the sequence of nodes

        // self.atoms
        //     .clone()
        //     .into_iter()
        //     .rev()
        //     .flat_map(|atom| Node::from_atom(atom).generate_children(chars.len()))
        //     .collect();

        while let Some(mut curr_sequence) = seq_stack.pop() {
            if seq_visited.contains(&curr_sequence) {
                continue;
            }

            let mut node_visited: HashSet<AtomNode> = HashSet::new();
            let mut node_stack: Vec<AtomNode> = vec![];
            while let Some(curr_node) = curr_sequence.pop() {
                if node_visited.contains(&curr_node) {
                    continue;
                }

                eprintln!(
                    "Visiting Node -> Atom: {:?}, num_chars_to_consume: {:?}\n",
                    curr_node.atom, curr_node.num_chars_to_consume_opt
                );
                node_visited.insert(curr_node.clone());

                let children = curr_node.generate_children(chars.len());
                for child in children.into_iter() {
                    if !node_visited.contains(&child) {
                        node_stack.push(child);
                    }
                }
            }
        }

        todo!()
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let atom_matcher = parse::parse(pattern);
    eprintln!("atom_matcher: {atom_matcher:#?}");

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
