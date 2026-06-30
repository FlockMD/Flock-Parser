use std::{char, collections::{HashSet, HashMap, VecDeque}, i32, cmp};
use std::collections::BTreeSet;
use crate::utils::matrix::Matrix;
use crate::parser::parser::Node;



struct CharMap {
    // associative mapping to map char <--> int and allow for special characters withaccept gaps (eg.
    // if our alphabet is only 'a' and '$', our mapping is 'a' --> 0, '$' --> 1
    encoder: HashMap<char, usize>,
    decoder: Vec<char>,
}



// need a function that takes an array of strings (terms) that are defined, and from them get a
// list of the unique characters these strings use, sort them, enumerate them, and return a map of
// char <--> int encoding
// we need this since we are not just constraining our potentially defined terms to just the
// lowercase alphabet (26 values), we want them to be able to include numbers, special characters
// ($, %, -, etc.)

impl CharMap {
    pub fn new<S: AsRef<str>>(terms: &[S]) -> Self {
        let mut chars: BTreeSet<char> = BTreeSet::new();

        for term in terms {
            let s: &str = term.as_ref();
            for c in s.chars() {
                chars.insert(c);
            }
        }

        let decoder: Vec<char> = chars.into_iter().collect();

        let mut encoder: HashMap<char, usize> = HashMap::new();
        for (i, c) in decoder.iter().enumerate() {
            encoder.entry(*c).or_insert(i);
        }

        Self {
            encoder,
            decoder,
        }
    }

    fn forward(&self, k: char) -> usize {
        match self.encoder.get(&k) {
            Some(i) => *i,
            None => panic!("key not found"),
        }
    }
    
    fn backward(&self, k: usize) -> char {
        self.decoder[k]
    }

}

pub struct AhoCorasick {
    max_s: usize, // sum of lengths of all terms
    max_c: usize, // number of characters in input alphabet
    
    // OUTPUT FUNCTION:
    // Bit i in this mask is one if the word with index i appears when the machine enters this state
    charmap: CharMap,
    accept: Vec<i32>, // accept[i] = 'What term (in terms of its idx in terms array) does the ith node correspond to (if it is an accept state, else -1)'
    failure: Vec<i32>, // failure[i] = index of node to fallback to`
                       // eg. if string 'ab' at index i and next character 'c', and we have term 'bc' but not term 'abc', failure[i] would be index of string 'b', then from there reading 'c' will transition to state 'bc'
    goto: Matrix<i32>, // for each node i, trie[i] contains a list of indices of all adjacent nodes
                       // eg. if we are currently at node 'ab' (suppose it is at index i), and we have terms 'abc' and 'abed', then we will have trie[i]['c'] --> (index of node for 'abc'), and trie[i]['e'] --> (index of node for 'abe')
}

impl AhoCorasick {
    fn new(terms: &[Node]) -> Self {
        let term_idents: Vec<String> = terms.iter().map(|term| {
            match term {
                Node::Def {id, ident, body} | Node::Local {id, ident, body} => ident.clone(),
                _ => panic!("Must supply definition-like!"),
            }
        }).collect();
        let charmap: CharMap = CharMap::new(&term_idents);
        let max_c: usize = charmap.decoder.len();
        let mut sum: usize = 0;
        
        for term in term_idents {
            let count = term.chars().count();
            sum += count;
        }
        let max_s: usize = sum;

        let mut accept: Vec<i32> = vec![-1; max_s];
        let mut failure: Vec<i32> = Vec::new();
        let mut goto: Matrix<i32> = Matrix::new(max_s as usize, max_c as usize, -1);

        // construct goto and accept: 
        // for each string, loop through all characters and, if one doesnt currently exist, create an edge from the previous node to the next
        // once at end of string, set 'accept state' flag to true
        for (idx, term) in terms.iter().enumerate() { 
            let term = match term {
                Node::Def { id, ident, body } | Node::Local { id, ident, body } => {
                    let mut state: i32 = 0;
                    //let s: &str = term.as_ref();
                    for c in ident.chars() {
                        let c_enc: usize = charmap.forward(c);
                        let next = goto.get(state as usize, c_enc);
                        if next == -1 {
                            // next state does not exist in Trie, create new node as first free entry in goto, and create edge from prev to next node
                            match goto.first_free_row() {
                                None => panic!("goto function filled"),
                                Some(i) => {
                                    // create new node at goto[i] representing current string
                                    // create edge from state to this new node
                                    goto.put(state as usize, c_enc, i as i32);
                                    state = i as i32;
                                }
                            }
                        }
                        else {
                            // state already exists in Trie, continue
                            state = next;
                        }

                    }
                    accept[state as usize] = idx as i32;


                },
                _ => panic!("Must supply definition-like!"),
            };
            
        }

        // construct failure:
        let mut q = VecDeque::new();
        
        // handle 'base cases': root failure points to root, roots children failure all point to root
        // --> already set to 0 (root) by default

        // for each child of root, enqueue
        for i in 0..goto.cols {
            let c = goto.get(0, i);
            q.push_back(c);
        }

        // level order traversal
        while !q.is_empty() {
            let o = q.pop_front();
            match o {
                None => panic!("Queue should be non-empty"), // might want to do something better than panic here
                Some(curr) => {
                    // for each child of curr, enqueue
                    for c in 0..goto.cols { // c is key of edge from curr --> child
                        let child = goto.get(0, c);
                        if child != -1 {
                            q.push_back(child);

                            // we construct the failure link for each child of curr
                            // each failure link is built by moving up to the failure links until we find a node that contains an edge corresponding to the last character of curr, then traversing that edge and setting the failure link to point to that resultant node

                            let mut n = failure[curr as usize]; // traverse failure link of curr (child's parent)
                            while goto.get(n as usize, c) != -1 && n != 0 { // while failure node doesn't have the next character as a child, continue moving up the tree via failure links
                                n = failure[n as usize];
                            }
                            // since we've found the earliest viable node to fail back to, set child's failure link (or root of n is a leaf)
                            let new_fail = cmp::max(goto.get(n as usize, c), 0);
                            failure[child as usize] = new_fail;
                        }
                    }
                }
            }
        }
        

        Self {
            max_s,
            max_c,
            charmap,
            accept,
            failure,
            goto,
        }
    }

    /**
     returns tuples (starting position, term idx)
     */
    fn search(&self, text: &str) -> Vec<(usize, usize)> {
        // we will store found instances as tuple (idx of start of matched word, idx of term in terms)
        let found: Vec<(usize, usize)> = Vec::new();
        let state = 0;
        let i = 0;
        let text_chars: Vec<char> = text.chars().collect();
        while i < text_chars.len() {
            let c = self.charmap.forward(text_chars[i]);
            if (self.goto.get(state, c) >= 0) {
                
            }

        }
        found
    }


}

