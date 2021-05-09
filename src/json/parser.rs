use super::JsonValue;
use std::collections::HashMap;

pub struct JsonParser {
    input: Vec<char>,
    index: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            index: 0,
        }
    }

    fn expect_char(&self, ch: char) {
        match self.input.get(self.index) {
            Some(input_ch) => assert_eq!(
                *input_ch, ch,
                "JsonParser: Expected char {:?}, got {:?} at index {}",
                ch, input_ch, self.index
            ),
            None => panic!(
                "JsonParser: Expected char at index '{}' but input lenght is '{}'",
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
        match self.input.get(self.index) {
            Some(input_ch) => {
                self.index = self.index + 1;
                *input_ch
            }
            None => panic!(
                "JsonParser: Expected char at index '{}' but input lenght is '{}'",
                self.index,
                self.input.len()
            ),
        }
    }

    fn peek_index(&self, index: usize) -> Option<&char> {
        if index >= self.input.len() {
            return None;
        }
        self.input.get(index)
    }

    fn peek(&self) -> Option<&char> {
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

    fn consume_quoted_string(&mut self) -> String {
        self.consume_specific('"');
        let mut result = String::new();

        loop {
            let mut peek_index = self.index;
            let mut ch = None;

            loop {
                if peek_index >= self.input.len() {
                    break;
                }
                ch = self.peek_index(peek_index);
                match ch {
                    None => panic!("JsonParser::consume_quoted_string cannot peek"),
                    Some('"') | Some('\\') => {
                        break;
                    }
                    Some(_) => {
                        peek_index = peek_index + 1;
                    }
                }
            }

            let ch = *ch.unwrap();

            if peek_index != self.index {
                while peek_index != self.index {
                    result.push(self.consume());
                }
            }

            if self.index == self.input.len() {
                break;
            }

            if ch == '"' {
                break;
            }

            if ch != '\\' {
                result.push(self.consume());
                continue;
            }

            self.consume_specific('\\');
            let escaped_ch = self.consume();
            match escaped_ch {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                _ => {
                    // TODO: handel \f \b \u
                    result.push(escaped_ch);
                }
            }
        }

        self.consume_specific('"');

        result
    }

    fn parse_string(&mut self) -> JsonValue {
        let result = self.consume_quoted_string();
        JsonValue::String(result)
    }

    fn parse_true(&mut self) -> JsonValue {
        self.consume_specific_string("true");
        JsonValue::Boolean(true)
    }

    fn parse_false(&mut self) -> JsonValue {
        self.consume_specific_string("false");
        JsonValue::Boolean(false)
    }

    fn parse_null(&mut self) -> JsonValue {
        self.consume_specific_string("null");
        JsonValue::Null
    }

    fn parse_number(&mut self) -> JsonValue {
        let mut number_str = String::new();
        let mut fraction_str = String::new();
        let mut is_double = false;

        loop {
            let ch = match self.peek() {
                None => break,
                Some(ch) => *ch,
            };

            if ch == '.' {
                is_double = true;
                self.consume();
                continue;
            }

            if ch == '-' || (ch >= '0' && ch <= '9') {
                if is_double {
                    fraction_str.push(ch);
                } else {
                    number_str.push(ch);
                }
                self.consume();
                continue;
            }
            break;
        }
        if number_str.len() == 0 || (is_double && fraction_str.len() == 0) {
            panic!("sonParser::parse_number Error parsing number: no numbers were found");
        }

        let number;
        if is_double {
            let final_number_str = format!("{}.{}", number_str, fraction_str);
            number = final_number_str.parse().expect(&format!(
                "JsonParser::parse_number Error parsing number: invalid number {:?}",
                final_number_str
            ));
        } else {
            number = number_str.parse().expect(&format!(
                "JsonParser::parse_number Error parsing number: invalid number {:?}",
                number_str
            ));
        }

        JsonValue::Number(number)
    }

    fn parse_array(&mut self) -> JsonValue {
        self.consume_specific('[');
        let mut array = vec![];

        loop {
            self.consume_whitespace();

            if self.peek() == Some(&']') {
                break;
            }

            let element = self.parse_value();
            array.push(element);
            self.consume_whitespace();

            if self.peek() == Some(&']') {
                break;
            }
            self.consume_specific(',');
            if self.peek() == Some(&']') {
                panic!("JsonParser::parse_array cannot close array after ','");
            }
        }

        self.consume_whitespace();
        self.consume_specific(']');

        JsonValue::Array(array)
    }

    fn parse_object(&mut self) -> JsonValue {
        self.consume_specific('{');
        let mut object = HashMap::new();

        loop {
            self.consume_whitespace();
            if self.peek() == Some(&'}') {
                break;
            }

            let key = self.consume_quoted_string();
            if key.is_empty() {
                panic!("JsonParser::parse_object Empty object key");
            }

            self.consume_whitespace();
            self.consume_specific(':');
            self.consume_whitespace();

            let value = self.parse_value();
            object.insert(key, value);

            self.consume_whitespace();
            if self.peek() == Some(&'}') {
                break;
            }

            self.consume_specific(',');
            self.consume_whitespace();
            if self.peek() == Some(&'}') {
                panic!("JsonParser::parse_object cannot close object after ','");
            }
        }

        self.consume_whitespace();
        self.consume_specific('}');

        JsonValue::Object(object)
    }

    fn parse_value(&mut self) -> JsonValue {
        self.consume_whitespace();
        let type_hint = match self.peek() {
            Some(ch) => ch,
            None => panic!("JsonParser::parse_value nothing to peek!"),
        };
        match type_hint {
            '"' => self.parse_string(),
            't' => self.parse_true(),
            'f' => self.parse_false(),
            'n' => self.parse_null(),
            '-' | '0'..='9' => self.parse_number(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            _ => panic!("JsonParser::parse_value unknown type hint {:?}", type_hint),
        }
    }

    pub fn parse(&mut self) -> JsonValue {
        self.parse_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string() {
        let value = JsonParser::new("\"test\"").parse();
        assert_eq!(value, JsonValue::String("test".to_owned()));
    }

    #[test]
    fn parse_true() {
        let value = JsonParser::new("true").parse();
        assert_eq!(value, JsonValue::Boolean(true));
    }

    #[test]
    fn parse_false() {
        let value = JsonParser::new("false").parse();
        assert_eq!(value, JsonValue::Boolean(false));
    }

    #[test]
    fn parse_null() {
        let value = JsonParser::new("null").parse();
        assert_eq!(value, JsonValue::Null);
    }

    #[test]
    fn parse_number() {
        let value = JsonParser::new("27").parse();
        assert_eq!(value, JsonValue::Number(27_f64));
    }

    #[test]
    fn parse_fraction_number() {
        let value = JsonParser::new("27.02").parse();
        assert_eq!(value, JsonValue::Number(27.02_f64));
    }

    #[test]
    fn parse_array() {
        let value = JsonParser::new("[1, \"dos\", null]").parse();
        assert_eq!(
            value,
            JsonValue::Array(vec!(
                JsonValue::Number(1_f64),
                JsonValue::String("dos".to_owned()),
                JsonValue::Null
            ))
        );
    }

    #[test]
    fn parse_object() {
        let value = JsonParser::new("{\"test\": true, \"number\": 1, \"missing\": null}").parse();
        let mut object = HashMap::new();
        object.insert("test".to_owned(), JsonValue::Boolean(true));
        object.insert("number".to_owned(), JsonValue::Number(1_f64));
        object.insert("missing".to_owned(), JsonValue::Null);
        assert_eq!(value, JsonValue::Object(object));
    }
}
