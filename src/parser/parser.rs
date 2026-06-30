use std::{collections::{HashMap, HashSet}, u32, fs};

use serde::Serialize;

#[derive(Debug)]
pub struct Document {
    parser: Parser,
    pub content: Block,
}

// easy api through Document to interface with, acts as a 'controller' to interface with different parts of the code (parser, resolver, etc)
impl Document {
    pub fn new(source_path: &str) -> Self {
        let contents = match fs::read_to_string(source_path) {
            Ok(s) => s,
            Err(e) => {
                panic!("Failed to read file: {}", e)
            }
        };
        let mut parser = Parser::new(contents);
        let content: Block = parser.parse_until(None);

        Self {
            parser,
            content,
        }
    }

    pub fn write(&self, parser_output_path: &str, defs_output_path: &str) {
        let content = self.content.to_json();
        match fs::write(parser_output_path, content) {
            Ok(()) => println!("File written"),
            Err(e) => println!("Error: {}", e),
        }
        let defs_json = self.parser.serialize_defs();
        match fs::write(defs_output_path, defs_json) {
            Ok(()) => println!("Definitions written"),
            Err(e) => println!("Error: {}", e),
        }
    }
}

#[derive(Debug, Clone, Serialize)]

pub struct Block {
    pub id: Vec<u32>,
    pub nodes: Vec<Node>,
}

impl Block {
    fn new() -> Self {
        Self {
            id: Vec::new(),
            nodes: Vec::new(),
        }
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}


#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Node {
    Text {
        id: Vec<u32>,
        text: String,
    },
    
    // Command Types 
    Def {
        id: Vec<u32>,
        ident: String,
        body: Block,
    },
    Local {
        id: Vec<u32>,
        ident: String,
        body: Block,
    },
    Section {
        id: Vec<u32>,
        ident: String,
        body: Block,
    },
    Img {
        id: Vec<u32>,
        path: String,
    },
    Latex {
        id: Vec<u32>,
        expr: String,
    },
    Custom {
        id: Vec<u32>,
        name: String,
        args: Vec<Block>,
    },
}

#[derive(Debug)]
pub struct Parser {
    input: Vec<char>,
    pos: usize,
    defs: HashMap<String, Vec<Node>>,
    id_stack: Vec<u32>
}

impl Parser {

    pub fn new(input: String) -> Self {
        let normalized = input.replace("\r\n", "\n").replace('\r', "\n");

        Self {
            input: normalized.chars().collect(),
            pos: 0,
            defs: HashMap::new(),
            id_stack: Vec::new()
        }
    }

    // ----- ID Functions ---- //

    fn current_id(&self) -> Vec<u32> {
        self.id_stack.clone()
    }

    fn with_child_id<T>(&mut self, child_index: u32, f: impl FnOnce(&mut Self) -> T) -> T {
        self.id_stack.push(child_index);
        let result = f(self);
        self.id_stack.pop();
        result
    }

    // ----- Helpers ----- //

    fn serialize_defs(&self) -> String {
        serde_json::to_string(&self.defs).unwrap()
    }
    
    // ----- Parser Helpers ----- //

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.pos += 1;
        ch
    }

    /**
     Advances one word until white-space (or supplied break tokens if any) encountered
     */
    fn advance_word(&mut self, break_tokens: Option<&HashSet<char>>) -> Option<String> {
        // TODO: probably want to add optional list of characters to break at. 
        // Consider use case of trying to parse command name, and you've written:
        // --> @img(path here)
        // It would make sense to only parse 'img', but currently it parses '@img(path here)'
        // So, you would currently need to write:
        // --> @img (path here)
        // Note the space
        let mut s = String::new();

        while let Some(c) = self.peek() {
            if c.is_whitespace() 
                || break_tokens.is_some_and(|tokens| tokens.contains(&c)) 
            {
                break
            }
            s.push(c);
            self.advance();
        }

        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
    
    // advances until provided token break_token is found, and return consumed tokens
    fn advance_until(&mut self, break_token: char) -> Option<String> {

        let mut s = String::new();

        while let Some(c) = self.peek() {
            if c == '\\' {
                self.advance();
                let next = match self.advance() {
                    Some(ch) => match ch {
                        'n' => break,
                        _ => ch
                    }
                    None => panic!("finished parsing before encountering expected token {}", break_token)
                };
                s.push(next);
            }
            else if c == break_token {
                break;
            }
            s.push(c);
            self.advance();
        }

        if s.is_empty() {
            None
        } else {
            Some(s)
        } 
    }

    // consumes characters until provided token break_token is found
    fn consume_until(&mut self, break_token: char) {
        while let Some(c) = self.peek() {
            if c == '\\' {
                self.advance();
                self.advance();
            }
            else if c == break_token {
                break;
            }
            self.advance();
        }
    }

    // consumes until non-whitespace (space, tab, newline, etc) is found
    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    // ----- Command Parsers ----- //
    
    // would be nice to have a generic command parser that lets you auto parse (...) {...} without
    // having to manually repeat every time

    // Generic function to parse all text of form ("text here") { "block stuff here" }
    // Want to extend to make Block part optional to just parse form ("text here")
    fn parse_command_structure(&mut self) -> (String, Block) {

        self.consume_until('(');
        self.advance(); // consume '('
        self.consume_whitespace(); // ignore leading whitespaces

        let ident = self
            .advance_until(')')
            .expect("Must define on some term (cannot be empty)");

        self.advance(); // consume ')'

        self.consume_until('{');
        self.advance(); // consume '{'

        let body = self.parse_until(Some('}'));

        self.advance(); // consume '}'

        (ident, body)
    }

    fn parse_def(&mut self) -> Node {

        let (ident, body) = self.parse_command_structure();
        let node = Node::Def { id: self.current_id(), ident: ident.clone(), body };

        self.defs.entry(ident)
            .or_insert_with(Vec::new)
            .push(node.clone());

        node
    }

    fn parse_local(&mut self) -> Node {

        let (ident, body) = self.parse_command_structure();
        let node = Node::Local { id: self.current_id(), ident: ident.clone(), body };

        self.defs.entry(ident)
            .or_insert_with(Vec::new)
            .push(node.clone());

        node
    }

    fn parse_section(&mut self) -> Node {
        let (ident, body) = self.parse_command_structure();

        Node::Section { id: self.current_id(), ident, body }
    }

    fn parse_img(&mut self) -> Node {

        self.consume_until('(');
        self.advance(); // consume '('

        let path = self
            .advance_until(')')
            .expect("Must define on some term (cannot be empty)");
        // do some extra validation for valid path structure here

        self.advance(); // consume ')'

        Node::Img { id: self.current_id(), path }

    }

    fn parse_latex(&mut self) -> Node {

        self.consume_until('(');
        self.advance(); // consume '('

        let expr = self
            .advance_until(')')
            .expect("Must define on some term (cannot be empty)");
        // need to implement avoid breaking on '@' here, also may want to fully parse latex here?
        // (Check to see if there are any rust latex parser crates?)

        self.advance(); // consume ')'

        Node::Latex { id: self.current_id(), expr }
    }


    fn parse_command(&mut self) -> Node {
        let s = self.advance_word(None);

        match s {
            Some(command) => match command.as_str() {
                "def" => self.parse_def(),
                "local" => self.parse_local(),
                "section" => self.parse_section(),
                "img" => self.parse_img(),
                "latex" => self.parse_latex(),

                _ => panic!("Unrecognized function name {}", command)
            
            }

            None => panic!("command cannot have value 'None'")
        }
    }
    
    // NOTE: Only used for Node::Text blocks (in order to allow for interruption with special chars), NOT used for parsing text making up idents (currently using consume_until and advance_until for that)
    fn parse_text(&mut self, break_token: Option<char>) -> Node {
        let mut text = String::new();

        while let Some(c) = self.peek() {
            match c {
                
                '\\' => {
                    self.advance();
                    if let Some(next) = self.advance() {
                        text.push(next);
                    }
                }

                '\n' => {
                    text.push('\n');
                    self.advance();

                    while matches!(self.peek(), Some(' ' | '\t')) {
                        self.advance();
                    }
                }

                c if break_token.is_some_and(|bt| c == bt) => break,

                '@' => break,
                
                
                _ => {
                    text.push(c);
                    self.advance();
                }
            }
        }
        Node::Text { id: self.current_id(), text }
    }

    pub fn parse_until(&mut self, break_token: Option<char>) -> Block {
        let mut block = Block {
            id: self.current_id(),
            nodes: Vec::new()
        };

        while let Some(c) = self.peek() {
            if break_token.is_some_and(|bt| c == bt) {
                break;
            }

            match c {
                '@' => {
                    self.advance();
                    
                    let child_index = block.nodes.len() as u32;
                    let node = self.with_child_id(child_index, |parser| {
                        parser.parse_command()
                    });

                    block.nodes.push(node);
                }

                _ => { // if no special character encountered, proceed to parse as normal text
                    let child_index = block.nodes.len() as u32;
                    let node = self.with_child_id(child_index, |parser| {
                        parser.parse_text(None)
                    });
                    block.nodes.push(node);
                }
            }
        }

        block
    }
}
