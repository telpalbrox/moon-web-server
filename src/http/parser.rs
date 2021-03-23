use super::{HttpRequest, HttpResponse};
use std::collections::HashMap;

pub struct HttpParser {
    input: String,
    index: usize,
}

impl HttpParser {
    pub fn new(input: String) -> HttpParser {
        HttpParser { index: 0, input }
    }

    fn expect_char(&self, ch: char) {
        match self.input.chars().nth(self.index) {
            Some(input_ch) => assert_eq!(
                input_ch, ch,
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
        match self.input.chars().nth(self.index) {
            Some(input_ch) => {
                self.index = self.index + 1;
                input_ch
            }
            None => panic!(
                "JsonParser: Expected char at index '{}' but input lenght is '{}'",
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

    fn parse_string_with_delimiter(&mut self, delimiter: Option<char>) -> String {
        let mut string = String::new();
        let mut peek_index = self.index;
        loop {
            if peek_index == self.input.len() {
                break;
            }
            let peeked_ch = self.peek_index(peek_index);
            if peeked_ch == None {
                break;
            }
            let peeked_ch = peeked_ch.unwrap();
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
            }
        }

        string
    }

    fn parse_string(&mut self) -> String {
        self.parse_string_with_delimiter(None)
    }

    fn parse_headers(&mut self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if self.peek() == Some('\r') {
            return headers;
        }

        loop {
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
            if self.peek() == Some('\r') && self.peek_index(self.index + 1) == Some('\n') {
                break;
            }
        }

        self.consume_specific('\r');
        self.consume_specific('\n');

        headers
    }

    fn parse_query_params(uri: &String) -> HashMap<String, String> {
        let mut query_params = HashMap::new();

        let uri_without_fragment: String = uri.split('#').take(1).collect();
        let uri_parts: Vec<&str> = uri_without_fragment.split('?').take(2).collect();
        let query = match uri_parts.get(1) {
            Some(query) => query,
            None => return query_params,
        };

        for query_param in query.split('&') {
            let param_parts: Vec<&str> = query_param.split('=').collect();
            let query_key = match param_parts.get(0) {
                Some(key) => key,
                None => continue,
            };
            let query_value = match param_parts.get(1) {
                Some(value) => value,
                None => continue,
            };
            query_params.insert(
                query_key.to_owned().to_owned(),
                query_value.to_owned().to_owned(),
            );
        }

        query_params
    }

    fn parse_request_line(&mut self) -> (String, String, String) {
        let method = self.parse_string();
        self.consume_whitespace();
        let uri = self.parse_string();
        self.consume_whitespace();
        self.consume_specific_string("HTTP/");
        let version = self.parse_string();
        self.consume_specific('\r');
        self.consume_specific('\n');
        (method, uri, version)
    }

    fn parse_message(&mut self) -> (Vec<(String, String)>, String) {
        let headers = self.parse_headers();
        let body = self.parse_string_with_delimiter(Some('\0'));
        (headers, body)
    }

    fn parse_status_line(&mut self) -> (String, u16, String) {
        self.consume_specific_string("HTTP/");
        let version = self.parse_string();
        self.consume_specific(' ');
        let status_code_str = self.parse_string();
        let status_code = status_code_str.parse::<u16>().unwrap();
        self.consume_specific(' ');
        let reason = self.parse_string_with_delimiter(Some('\r'));
        self.consume_specific('\n');
        (version, status_code, reason)
    }

    pub fn parse_request(&mut self) -> HttpRequest {
        let (method, uri, version) = self.parse_request_line();
        let (headers, body) = self.parse_message();

        HttpRequest {
            method,
            headers,
            query: HttpParser::parse_query_params(&uri),
            uri,
            version,
            body,
            params: HashMap::new(),
        }
    }

    pub fn parse_response(&mut self) -> HttpResponse {
        let (version, status_code, reason) = self.parse_status_line();
        let (headers, body) = self.parse_message();

        HttpResponse {
            version,
            status_code,
            reason,
            headers,
            body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_REQUEST: &str = "GET /test HTTP/1.1\r\nUserAgent: test rust\r\n\r\n";
    const EMPTY_HEADER_REQUEST: &str = "GET /test HTTP/1.1\r\n\r\n";
    const POST_REQUEST: &str = "POST / HTTP/1.1\r\nUserAgent: test\r\n\r\ntest rust2";
    const FIREFOX_REQUEST: &str = "GET / HTTP/1.1\r\nHost: localhost:7878\r\nUser-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:85.0) Gecko/20100101 Firefox/85.0\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r\nAccept-Language: en-US,es;q=0.8,ru;q=0.5,en;q=0.3\r\nAccept-Encoding: gzip, deflate\r\nConnection: keep-alive\r\nUpgrade-Insecure-Requests: 1\r\n\r\n";
    const QUERY_REQUEST: &str =
        "GET /test?query=1&query2=2 HTTP/1.1\r\nUserAgent: test rust\r\n\r\n";
    const SINGLE_QUERY_REQUEST: &str = "GET /test?query=1 HTTP/1.1\r\nUserAgent: test rust\r\n\r\n";
    const EMPTY_QUERY_REQUEST: &str = "GET /test?query= HTTP/1.1\r\nUserAgent: test rust\r\n\r\n";

    const BASIC_RESPONSE: &str = "HTTP/1.1 200 Ok\r\nx-test:more test\r\n\r\nlol request to /";

    #[test]
    fn parse_basic_request() {
        let mut parser = HttpParser::new(BASIC_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "/test");
        assert_eq!(request.version, "1.1");
        assert_eq!(request.headers.len(), 1);
        assert_eq!(request.headers[0].0, "UserAgent");
        assert_eq!(request.headers[0].1, "test rust");
    }

    #[test]
    fn parse_empty_header_request() {
        let mut parser = HttpParser::new(EMPTY_HEADER_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.method, "GET");
        assert_eq!(request.uri, "/test");
        assert_eq!(request.version, "1.1");
        assert_eq!(request.headers.len(), 0);
    }

    #[test]
    fn parse_post_request() {
        let mut parser = HttpParser::new(POST_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.body, "test rust2");
    }

    #[test]
    fn parse_firefox_request() {
        let mut parser = HttpParser::new(FIREFOX_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.headers.len(), 7);
    }

    #[test]
    fn parse_query_params() {
        let mut parser = HttpParser::new(SINGLE_QUERY_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.query.len(), 1);
        assert_eq!(request.query.get("query"), Some(&String::from("1")));
        assert_eq!(request.query.get("query2"), None);
        let mut parser = HttpParser::new(QUERY_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.query.len(), 2);
        assert_eq!(request.query.get("query"), Some(&String::from("1")));
        assert_eq!(request.query.get("query2"), Some(&String::from("2")));
        let mut parser = HttpParser::new(EMPTY_QUERY_REQUEST.to_owned());
        let request = parser.parse_request();
        assert_eq!(request.query.len(), 1);
        assert_eq!(request.query.get("query"), Some(&String::from("")));
        assert_eq!(request.query.get("query2"), None);
    }

    #[test]
    fn parse_response() {
        let mut parser = HttpParser::new(BASIC_RESPONSE.to_owned());
        let response = parser.parse_response();
        assert_eq!(response.version, "1.1");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.reason, "Ok");
        assert_eq!(response.headers.len(), 1);
        assert_eq!(response.headers[0].0, "x-test");
        assert_eq!(response.headers[0].1, "more test");
        assert_eq!(response.body, "lol request to /");
    }
}
