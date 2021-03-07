use std::collections::HashMap;

const OPEN_TAG: &'static str = "{{";
const CLOSE_TAG: &'static str = "}}";

struct MustacheLikeLexer {
    input: String,
    index: usize,
    tokens: Vec<MustacheLikeToken>,
}

#[derive(Debug, PartialEq, Clone)]
enum MustacheLikeToken {
    Text(String),
    Name(String),
    OpenTag(String),
    CloseTag(String),
}

#[derive(Debug, PartialEq)]
enum MustacheLikeNode {
    Text(String),
    Variable(String),
    Section(String, Vec<Box<MustacheLikeNode>>),
}

impl MustacheLikeLexer {
    fn new(input: String) -> Self {
        Self {
            input,
            index: 0,
            tokens: Vec::new(),
        }
    }

    fn expect_char(&self, ch: char) {
        match self.input.chars().nth(self.index) {
            Some(input_ch) => assert_eq!(
                input_ch, ch,
                "MustacheLikeLexer: Expected char {:?}, got {:?} at index {}",
                ch, input_ch, self.index
            ),
            None => panic!(
                "MustacheLikeLexer: Expected char at index '{}' but input lenght is '{}'",
                self.index,
                self.input.len()
            ),
        }
    }

    fn consume_specific(&mut self, ch: char) {
        self.expect_char(ch);
        self.index = self.index + 1;
    }

    fn consume_specific_string(&mut self, string: &str) {
        string.chars().for_each(|ch| {
            self.consume_specific(ch);
        });
    }

    fn consume(&mut self) -> char {
        match self.input.chars().nth(self.index) {
            Some(input_ch) => {
                self.index = self.index + 1;
                input_ch
            }
            None => panic!(
                "MustacheLikeLexer: Expected char at index '{}' but input lenght is '{}'",
                self.index,
                self.input.len()
            ),
        }
    }

    fn peek_index(&self, index: usize) -> Option<char> {
        if index >= self.input.len() {
            return None;
        }
        self.input.chars().nth(index)
    }

    fn peek(&self) -> Option<char> {
        self.peek_index(self.index)
    }

    fn consume_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if !ch.is_whitespace() {
                break;
            }
            self.consume();
        }
    }

    fn peek_string(&self, delimiter: &str) -> Option<char> {
        if self.index >= self.input.len() {
            return None;
        }
        let delimiter_len = delimiter.len();
        let mut part = String::new();
        for i in 0..delimiter_len {
            if let Some(ch) = self.peek_index(self.index + i) {
                part.push(ch);
            } else {
                break;
            }
        }

        if part == delimiter {
            return None;
        }
        return Some(part.chars().nth(0).unwrap());
    }

    fn consume_until(&mut self, delimiter: &str) -> String {
        let mut string = String::new();
        while let Some(part) = self.peek_string(&delimiter) {
            string.push(part);
            self.index = self.index + 1;
            if self.eoi() {
                return string;
            }
        }
        string
    }

    fn eoi(&self) -> bool {
        self.index >= self.input.len()
    }

    fn run(mut self) -> Vec<MustacheLikeToken> {
        loop {
            if self.index >= self.input.len() {
                break;
            }

            let text_before_open_tag = self.consume_until(OPEN_TAG);
            if !text_before_open_tag.is_empty() {
                self.tokens
                    .push(MustacheLikeToken::Text(text_before_open_tag));
            }
            if self.eoi() {
                break;
            }
            self.consume_specific_string(OPEN_TAG);

            self.consume_whitespace();

            let text_inside_tag = self.consume_until(CLOSE_TAG).trim().to_owned();
            self.consume_specific_string(CLOSE_TAG);
            let first_char = match text_inside_tag.chars().nth(0) {
                Some(char) => char,
                None => panic!("First char not found"),
            };
            if first_char == '#' {
                let tag_name = text_inside_tag.chars().skip(1).collect();
                self.tokens.push(MustacheLikeToken::OpenTag(tag_name));
            } else if first_char == '/' {
                let tag_name = text_inside_tag.chars().skip(1).collect();
                self.tokens.push(MustacheLikeToken::CloseTag(tag_name));
            } else {
                self.tokens.push(MustacheLikeToken::Name(text_inside_tag));
            }
        }

        self.tokens
    }
}

#[derive(Debug, PartialEq)]
pub enum MustacheLikeValue {
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<Box<MustacheLikeValue>>),
    Map(HashMap<String, Box<MustacheLikeValue>>),
}

impl MustacheLikeNode {
    fn render_section(nodes: &Vec<Box<MustacheLikeNode>>, context: &MustacheLikeValue) -> String {
        let mut result = String::new();

        for node in nodes {
            result.push_str(&node.render(context));
        }

        result
    }

    fn render(&self, context: &MustacheLikeValue) -> String {
        match self {
            Self::Text(text) => {
                return String::from(text);
            }
            Self::Variable(name) => {
                match context {
                    MustacheLikeValue::Map(map) => {
                        let value = match map.get(name) {
                            None => return String::from(""),
                            Some(value) => value,
                        };
                        match &**value {
                            MustacheLikeValue::String(value) => return String::from(value),
                            MustacheLikeValue::Boolean(value) => return value.to_string(),
                            MustacheLikeValue::Number(value) => return value.to_string(),
                            _ => return String::from(""),
                        }
                    }
                    _ => todo!("Handle name for {:?} value", context),
                };
            }
            Self::Section(tag_name, nodes) => match context {
                MustacheLikeValue::Map(map) => {
                    let value = match map.get(tag_name) {
                        None => return String::from(""),
                        Some(value) => value,
                    };
                    match &**value {
                        MustacheLikeValue::Boolean(value) => {
                            if !value {
                                return String::from("");
                            }
                            return MustacheLikeNode::render_section(nodes, context);
                        }
                        MustacheLikeValue::Array(array) => {
                            let mut result = String::new();
                            for element in array {
                                result.push_str(&MustacheLikeNode::render_section(nodes, &element));
                            }
                            return result;
                        }
                        _ => todo!("Handle map section for {:?} value", &**value),
                    }
                }
                _ => todo!("Handle section for {:?} value", context),
            },
        };
    }
}

struct MustacheLikeParser {
    tokens: Vec<MustacheLikeToken>,
    index: usize,
}

impl MustacheLikeParser {
    fn new(tokens: Vec<MustacheLikeToken>) -> Self {
        Self { tokens, index: 0 }
    }

    fn has_finished(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn consume(&mut self) {
        self.index = self.index + 1;
    }

    fn consume_until(&mut self, token: &MustacheLikeToken) -> Vec<MustacheLikeToken> {
        let mut tokens = Vec::new();
        loop {
            let peeked_token = match self.tokens.get(self.index) {
                None => break,
                Some(token) => token,
            };
            if peeked_token == token {
                break;
            }
            tokens.push(peeked_token.to_owned());
            self.consume();
        }
        tokens
    }

    fn consume_specific(&mut self, token: &MustacheLikeToken) {
        let peeked_token = match self.tokens.get(self.index) {
            None => panic!(
                "Expected token {:?} at index {}, but list of tokens has size {}",
                token,
                self.index,
                self.tokens.len()
            ),
            Some(token) => token,
        };
        if peeked_token != token {
            panic!("Expected token {:?} but found {:?}", token, peeked_token);
        }
        self.consume();
    }

    fn parse(&mut self) -> Vec<Box<MustacheLikeNode>> {
        let mut nodes: Vec<Box<MustacheLikeNode>> = Vec::new();

        loop {
            if self.has_finished() {
                break;
            }

            let token = match self.tokens.get(self.index) {
                Some(token) => token,
                None => break,
            };

            match token {
                MustacheLikeToken::Text(text) => {
                    nodes.push(Box::new(MustacheLikeNode::Text(text.to_owned())));
                    self.consume();
                }
                MustacheLikeToken::Name(name) => {
                    nodes.push(Box::new(MustacheLikeNode::Variable(name.to_owned())));
                    self.consume();
                }
                MustacheLikeToken::OpenTag(tag_name) => {
                    let close_tag_token = MustacheLikeToken::CloseTag(tag_name.to_owned());
                    let tag_name = tag_name.clone();
                    self.consume();
                    let section_tokens = self.consume_until(&close_tag_token);
                    self.consume_specific(&close_tag_token);
                    let section_nodes = MustacheLikeParser::new(section_tokens).parse();
                    nodes.push(Box::new(MustacheLikeNode::Section(
                        tag_name.to_owned(),
                        section_nodes,
                    )));
                }
                MustacheLikeToken::CloseTag(tag_name) => {
                    panic!("Not expected close tag {:?}", tag_name);
                }
            }
        }

        nodes
    }
}

pub fn render(input: String, context: &MustacheLikeValue) -> String {
    let tokens = MustacheLikeLexer::new(input).run();
    assert_ne!(tokens.len(), 0, "No tokens were generated");
    let nodes = MustacheLikeParser::new(tokens).parse();
    assert_ne!(nodes.len(), 0, "No nodes were generated");
    let mut result = String::new();
    for node in nodes {
        result.push_str(&node.render(&context));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_until() {
        let mut lexer = MustacheLikeLexer::new("012{{".to_owned());
        assert_eq!(lexer.consume_until("{{"), "012");
        assert_eq!(lexer.index, 3);
    }

    #[test]
    fn lexer() {
        let lexer = MustacheLikeLexer::new("Input {{test}} more text".to_owned());
        assert_eq!(
            lexer.run(),
            vec!(
                MustacheLikeToken::Text(String::from("Input ")),
                MustacheLikeToken::Name(String::from("test")),
                MustacheLikeToken::Text(String::from(" more text"))
            )
        );
    }

    #[test]
    fn lexer_tags() {
        let lexer =
            MustacheLikeLexer::new("Shown.\n{{#person}}\n  Never shown!\n{{/person}}".to_owned());
        assert_eq!(
            lexer.run(),
            vec!(
                MustacheLikeToken::Text(String::from("Shown.\n")),
                MustacheLikeToken::OpenTag(String::from("person")),
                MustacheLikeToken::Text(String::from("\n  Never shown!\n")),
                MustacheLikeToken::CloseTag(String::from("person")),
            )
        )
    }

    fn expect_nodes(nodes: Vec<Box<MustacheLikeNode>>, expected_nodes: Vec<MustacheLikeNode>) {
        for index in 0..nodes.len() {
            let node = &*nodes[index];
            assert_eq!(node, &expected_nodes[index])
        }
    }

    #[test]
    fn parser() {
        let tokens = vec![
            MustacheLikeToken::Text(String::from("Input ")),
            MustacheLikeToken::Name(String::from("test")),
            MustacheLikeToken::Text(String::from(" more text")),
        ];
        let mut parser = MustacheLikeParser::new(tokens);
        let nodes = parser.parse();
        let expected_nodes = vec![
            MustacheLikeNode::Text(String::from("Input ")),
            MustacheLikeNode::Variable(String::from("test")),
            MustacheLikeNode::Text(String::from(" more text")),
        ];
        expect_nodes(nodes, expected_nodes);
    }

    #[test]
    fn parser_tree() {
        let tokens = vec![
            MustacheLikeToken::Text(String::from("Shown.\n")),
            MustacheLikeToken::OpenTag(String::from("person")),
            MustacheLikeToken::Text(String::from("\n  Never shown!\n")),
            MustacheLikeToken::CloseTag(String::from("person")),
        ];
        let nodes = MustacheLikeParser::new(tokens).parse();
        expect_nodes(
            nodes,
            vec![
                MustacheLikeNode::Text(String::from("Shown.\n")),
                MustacheLikeNode::Section(
                    String::from("person"),
                    vec![Box::new(MustacheLikeNode::Text(String::from(
                        "\n  Never shown!\n",
                    )))],
                ),
            ],
        );
    }

    #[test]
    fn render_test() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            Box::new(MustacheLikeValue::String("value test".to_owned())),
        );
        assert_eq!(
            render(
                "Input {{test}} more text".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "Input value test more text"
        );
    }

    #[test]
    fn render_conditional() {
        let mut context = HashMap::new();
        context.insert(
            "person".to_owned(),
            Box::new(MustacheLikeValue::Boolean(true)),
        );

        assert_eq!(
            render(
                "Shown.\n{{#person}}\n  Never shown!\n{{/person}}".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "Shown.\n\n  Never shown!\n"
        );
    }

    #[test]
    fn render_array() {
        let mut context = HashMap::new();
        let mut repo1 = HashMap::new();
        repo1.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("resque".to_owned())),
        );
        let mut repo2 = HashMap::new();
        repo2.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("hub".to_owned())),
        );
        let mut repo3 = HashMap::new();
        repo3.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("rip".to_owned())),
        );
        let repos = vec![
            Box::new(MustacheLikeValue::Map(repo1)),
            Box::new(MustacheLikeValue::Map(repo2)),
            Box::new(MustacheLikeValue::Map(repo3)),
        ];
        context.insert("repo".to_owned(), Box::new(MustacheLikeValue::Array(repos)));

        assert_eq!(
            render(
                "{{#repo}}\n  <b>{{name}}</b>\n{{/repo}}".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "\n  <b>resque</b>\n\n  <b>hub</b>\n\n  <b>rip</b>\n"
        );
    }
}
