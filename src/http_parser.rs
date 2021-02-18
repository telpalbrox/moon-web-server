struct HttpParser {
    input: String,
    index: usize,
}

struct HttpRequest {
    pub method: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub uri: String,
    pub body: String
}

impl HttpParser {
    fn new(input: String) -> HttpParser {
        HttpParser {
            index: 0,
            input
        }
    }

    fn expect_char(&self, ch: char) {
        match self.input.chars().nth(self.index) {
            Some(input_ch) => assert_eq!(input_ch, ch, "HttpParser: Expected char '{}', got '{}'", ch, input_ch),
            None => panic!("HttpParser: Expected char at index '{}' but input lenght is '{}'", self.index, self.input.len())
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
            },
            None => panic!("HttpParser: Expected char at index '{}' but input lenght is '{}'", self.index, self.input.len())
        }
    }

    fn peek_index(&self, index: usize) -> char {
        if index >= self.input.len() {
            return '\0'
        }
        self.input.chars().nth(index).unwrap()
    }

    fn peek(&self) -> char {
        self.peek_index(self.index)
    }

    fn consume_whitespace(&mut self) {
        while self.peek().is_whitespace() {
            self.consume();
        }
    }

    fn parse_string_with_delimiter(&mut self, delimiter: char) -> String {
        let mut string = String::new();
        let mut peek_index = self.index;
        loop {
            if peek_index == self.input.len() {
                break;
            }
            let peeked_ch = self.peek_index(peek_index);
            if delimiter == '\0' && peeked_ch.is_whitespace() {
                break;
            }
            if delimiter != '\0' && delimiter == peeked_ch {
                break;
            }
            peek_index = peek_index + 1;
        }

        if peek_index > self.index {
            while peek_index != self.index {
                string.push(self.consume())
            }
        }

        if self.index < self.input.len() {
            if delimiter == '\0' {
                self.consume_whitespace();
            } else {
                self.consume_specific(delimiter);
            }
        }

        string
    }

    fn parse_string(&mut self) -> String {
        self.parse_string_with_delimiter('\0')
    }

    fn parse_headers(&mut self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        loop {
            if self.index == self.input.len() {
                break;
            }
            let key = self.parse_string_with_delimiter(':');
            if key.is_empty() {
                break;
            }
            self.consume_whitespace();
            let value = self.parse_string_with_delimiter('\n');
            if value.is_empty() {
                panic!("Header value for key '{}' is empty", key);
            }
            headers.push((key, value));
            if self.peek() == '\n' {
                break;
            }
        }

        headers
    }

    fn parse(&mut self) -> HttpRequest {
        let method = self.parse_string();
        self.consume_whitespace();
        let uri = self.parse_string();
        self.consume_whitespace();
        self.consume_specific_string("HTTP/");
        let version = self.parse_string();
        self.consume_whitespace();
        let headers = self.parse_headers();
        let mut body = String::new();
        if self.peek() == '\n' {
            self.consume_specific('\n');
            body = self.parse_string();
        }

        HttpRequest {
            method,
            headers,
            uri,
            version,
            body
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_REQUEST: &str = "GET /test HTTP/1.1\nUserAgent: test rust";
    const POST_REQUEST: &str = "POST / HTTP/1.1\nUserAgent: test\n\ntest";

    #[test]
    fn parse_basic_request() {
        let mut parser = HttpParser::new(BASIC_REQUEST.to_owned());
        let request = parser.parse();
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "/test");
        assert_eq!(request.version, "1.1");
        assert_eq!(request.headers.len(), 1);
        assert_eq!(request.headers[0].0, "UserAgent");
        assert_eq!(request.headers[0].1, "test rust");
    }

    #[test]
    fn parse_post_request() {
        let mut parser = HttpParser::new(POST_REQUEST.to_owned());
        let request = parser.parse();
        assert_eq!(request.body, "test");
    }
}
