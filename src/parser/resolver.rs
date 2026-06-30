/*
use aho_corasick::AhoCorasick;

struct TermMatch {
    term_id: usize,
    start: usize,
    end: usize,
}


fn is_word_boundary(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().last();
    let after = text[end..].chars().next();

    let is_word_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';

    (before.map_or(true, |c| !is_word_char(c))) &&
    (after.map_or(true, |c| !is_word_char(c)))
}


fn find_matches(text: &str, ) {
    for mat in ac.find_iter(text) {
        if is_word_boundary(text, mat.start(), mat.end()) {
            // accept match
        }
    }
}

fn main() {    
    // temp, will feed in from elsewhere
    let patterns = vec![
        "electron",
        "string.len()",
    ];

    let ac = AhoCorasick::new(patterns).unwrap();

    let text = "my favourite particle is an electron. string.len() is useful";
}
*/
