use std::collections::HashMap;

const OPEN_TAG: &'static str = "{{";
const CLOSE_TAG: &'static str = "}}";

struct MustacheLikeLexer {
    input: String,
    index: usize,
    tokens: Vec<MustacheLikeToken>,
}

#[derive(Debug, PartialEq)]
enum MustacheLikeToken {
    Text(String),
    Name(String),
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
            if text_before_open_tag.is_empty() {
                break;
            }
            self.tokens
                .push(MustacheLikeToken::Text(text_before_open_tag));
            if self.eoi() {
                break;
            }
            self.consume_specific_string(OPEN_TAG);

            self.consume_whitespace();

            let text_inside_tag = self.consume_until(CLOSE_TAG).trim().to_owned();
            self.consume_specific_string(CLOSE_TAG);
            self.tokens.push(MustacheLikeToken::Name(text_inside_tag));
        }

        self.tokens
    }
}

impl MustacheLikeToken {
    fn render(&self, context: &HashMap<String, String>) -> String {
        match self {
            Self::Text(text) => {
                return String::from(text);
            }
            Self::Name(name) => {
                if let Some(value) = context.get(name) {
                    return String::from(value);
                }
                return String::from("");
            }
        };
    }
}

pub fn render(input: String, context: &HashMap<String, String>) -> String {
    let tokens = MustacheLikeLexer::new(input).run();
    let mut result = String::new();
    for token in tokens {
        result.push_str(&token.render(&context));
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
    fn render_test() {
        let mut context = HashMap::new();
        context.insert("test".to_owned(), "value test".to_owned());
        assert_eq!(
            render("Input {{test}} more text".to_owned(), &context),
            "Input value test more text"
        );
    }
}
