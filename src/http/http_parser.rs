use super::HttpRequest;

pub struct HttpParser {
    input: String,
    index: usize,
}

impl HttpParser {
    pub fn new(input: String) -> HttpParser {
        HttpParser {
            index: 0,
            input
        }
    }

    fn expect_char(&self, ch: char) {
        match self.input.chars().nth(self.index) {
            Some(input_ch) => assert_eq!(input_ch, ch, "HttpParser: Expected char {:?}, got {:?}", ch, input_ch),
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

    fn parse_string_with_delimiter(&mut self, delimiter: Option<char>) -> String {
        let mut string = String::new();
        let mut peek_index = self.index;
        loop {
            if peek_index == self.input.len() {
                break;
            }
            let peeked_ch = self.peek_index(peek_index);
            if peeked_ch == '\0' {
                break;
            }
            if let Some(delimiter) = delimiter {
                if delimiter == peeked_ch {
                    break;
                }
            } else if peeked_ch.is_whitespace() {
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
            if let Some(delimiter) = delimiter {
                self.consume_specific(delimiter);
            } else {
                self.consume_whitespace();
            }
        }

        string
    }

    fn parse_string(&mut self) -> String {
        self.parse_string_with_delimiter(None)
    }

    fn parse_headers(&mut self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        loop {
            self.consume_whitespace();
            if self.index == self.input.len() {
                break;
            }
            let key = self.parse_string_with_delimiter(Some(':'));
            if key.is_empty() {
                break;
            }
            self.consume_whitespace();
            let value = self.parse_string_with_delimiter(Some('\r'));
            self.consume_specific('\n');
            if value.is_empty() {
                panic!("Header value for key '{:?}' is empty", key);
            }
            headers.push((key, value));
            if self.peek() == '\r' && self.peek_index(self.index + 1) == '\n' {
                break;
            }
        }

        headers
    }

    pub fn parse(&mut self) -> HttpRequest {
        let method = self.parse_string();
        self.consume_whitespace();
        let uri = self.parse_string();
        self.consume_whitespace();
        self.consume_specific_string("HTTP/");
        let version = self.parse_string();
        self.consume_whitespace();
        let headers = self.parse_headers();
        self.consume_specific('\r');
        self.consume_specific('\n');
        let body = self.parse_string_with_delimiter(Some('\0'));

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

    const BASIC_REQUEST: &str = "GET /test HTTP/1.1\r\nUserAgent: test rust\r\n\r\n";
    const POST_REQUEST: &str = "POST / HTTP/1.1\r\nUserAgent: test\r\n\r\ntest rust2";
    const FIREFOX_REQUEST: &str = "GET / HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r\nAccept-Language: en-US,es;q=0.8,ru;q=0.5,en;q=0.3\r\nAccept-Encoding: gzip, deflate\r\nConnection: keep-alive\r\nUpgrade-Insecure-Requests: 1\r\n\r\n";

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
        assert_eq!(request.body, "test rust2");
    }

    #[test]
    fn parse_firefox_request() {
        let mut parser = HttpParser::new(FIREFOX_REQUEST.to_owned());
        let request = parser.parse();
        assert_eq!(request.headers.len(), 7);
    }
}
