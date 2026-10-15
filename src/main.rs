use std::collections::VecDeque;
use std::env;
use std::io;
use std::process;

mod parse;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Atom {
    kind: AtomKind,
    quantifier_opt: Option<QuantifierKind>,
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
    Wildcard,
    Alternation(Vec<Vec<AtomKind>>),
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
    fn _max_input_length(&self) -> usize {
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

    fn _min_input_length(&self) -> usize {
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

    fn matches_str(current_str: &str, atom: &Atom) -> bool {
        assert!(!current_str.is_empty());

        match &atom.kind {
            AtomKind::SingleChar(ch) => {
                if current_str.len() != 1 || current_str.chars().next().unwrap() != *ch {
                    return false;
                }
            }
            AtomKind::Digit => {
                if current_str.len() != 1 || !current_str.chars().next().unwrap().is_numeric() {
                    return false;
                }
            }
            AtomKind::WordChar => {
                if current_str.len() != 1
                    || !current_str.chars().next().unwrap().is_alphanumeric()
                    || !current_str.starts_with('_')
                {
                    return false;
                }
            }
            AtomKind::PositiveCharGroup(group) => {
                if current_str.len() != 1 || !group.contains(current_str.chars().next().unwrap()) {
                    return false;
                }
            }
            AtomKind::NegativeCharGroup(group) => {
                if current_str.len() != 1 || group.contains(current_str.chars().next().unwrap()) {
                    return false;
                }
            }
            AtomKind::Wildcard => {
                if current_str.len() != 1 || current_str.starts_with('\n') {
                    return false;
                }
            }
            AtomKind::Alternation(patterns) => {
                if !patterns.iter().any(|pattern| {
                    let mut found_match = false;
                    for atom in pattern.iter().map(|atom_kind| Atom {
                        kind: atom_kind.clone(),
                        quantifier_opt: None,
                    }) {
                        if current_str
                            .chars()
                            .all(|ch| AtomMatcher::matches_str(&ch.to_string(), &atom))
                        {
                            found_match = true;
                        }
                    }

                    found_match
                }) {
                    return false;
                }
            }
        }

        true
    }

    fn match_dfs(&self, chars: Vec<char>) -> bool {
        // handle anchors and find first char
        let chars = chars.clone();

        let mut require_all_chars_match = false;
        let mut back_to_front = false;

        let first_char_idx = match (self.has_start_anchor, self.has_end_anchor) {
            // no anchors
            (false, false) => {
                // find first match
                let first_atom = self.atoms.first().unwrap();
                let mut idx = None;
                for (i, ch) in chars.clone().into_iter().enumerate() {
                    if AtomMatcher::matches_str(&ch.to_string(), first_atom) {
                        idx = Some(i);
                        break;
                    }
                }
                if let Some(idx) = idx {
                    idx
                } else {
                    // There is no matching first character
                    // TODO: handle zero or more
                    return false;
                }
            }
            // start and end anchor
            (true, true) => {
                require_all_chars_match = true;
                0
            }
            // start anchor only
            (true, false) => 0,
            // end anchor only
            (false, true) => {
                back_to_front = true;
                0
            }
        };

        let mut sequences = SequenceNode::build_all(&self.atoms, chars.len());

        sequences.reverse();

        dbg!(&sequences);

        'sequence: for sequence_node in sequences {
            let mut sequence = VecDeque::from(sequence_node.sequence);

            if back_to_front {
                sequence = sequence.into_iter().rev().collect();
            }

            let mut test_chars = VecDeque::from(chars.clone());

            if back_to_front {
                test_chars = test_chars.into_iter().rev().collect();
            }

            for _ in 0..first_char_idx {
                test_chars.pop_front();
            }

            while let Some(AtomNode {
                atom,
                num_chars_to_consume_opt: Some(num_chars_to_consume),
            }) = sequence.pop_front()
            {
                let mut chars_consumed = 0;
                for _ in 0..num_chars_to_consume {
                    if let Some(current_char) = test_chars.pop_front() {
                        if !AtomMatcher::matches_str(&current_char.to_string(), &atom) {
                            break;
                        } else {
                            chars_consumed += 1;
                        }
                    } else {
                        break;
                    }
                }

                eprintln!(
                    "require_all_chars_match: {}, chars_consumed: {}, num_chars_to_consume: {}, test_chars.len(): {}",
                    require_all_chars_match,
                    chars_consumed,
                    num_chars_to_consume,
                    test_chars.len()
                );

                if chars_consumed < num_chars_to_consume {
                    continue 'sequence;
                }
            }

            if sequence.is_empty() {
                if require_all_chars_match && !test_chars.is_empty() {
                    continue;
                } else {
                    return true;
                }
            }
        }

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
