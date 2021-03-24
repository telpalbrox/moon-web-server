// https://github.com/SerenityOS/serenity/blob/6c087480cf0b179918bd7b2b8c7d2017553043ad/AK/URL.cpp

#[derive(PartialEq, Debug)]
pub struct URL {
    pub protocol: String,
    pub host: String,
    pub port: u16,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

#[derive(PartialEq)]
enum State {
    InProtocol,
    InHostname,
    InPort,
    InPath,
    InQuery,
    InFragment,
}

fn is_valid_protocol_character(ch: Option<char>) -> bool {
    match ch {
        Some(ch) => return ch >= 'a' && ch <= 'z',
        None => return false,
    }
}

fn is_valid_hostname_character(ch: Option<char>) -> bool {
    match ch {
        Some(ch) => return ch != '/' && ch != ':',
        None => return false,
    }
}

fn is_digit(ch: Option<char>) -> bool {
    match ch {
        Some(ch) => return ch >= '0' && ch <= '9',
        None => return false,
    }
}

pub struct URLParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> URLParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn peek(&mut self) -> Option<char> {
        self.input.chars().nth(self.index)
    }

    fn consume(&mut self) -> char {
        let ch = self.input.chars().nth(self.index);
        let ch = match ch {
            Some(ch) => ch,
            None => panic!(
                "URLParser: Expected char at index '{}' but input length is '{}'",
                self.index,
                self.input.len()
            ),
        };
        self.index = self.index + 1;
        ch
    }

    pub fn parse(&mut self) -> URL {
        let mut protocol: String = Default::default();
        let mut host: String = Default::default();
        let mut port: u16 = Default::default();
        let mut path: String = Default::default();
        let mut query: String = Default::default();
        let mut fragment: String = Default::default();

        let mut state = State::InProtocol;

        let mut buffer = String::with_capacity(256);

        while self.index < self.input.len() {
            match state {
                State::InProtocol => {
                    // println!("is_valid_protocol_character(peek()) {} peek() {:?} index {}", is_valid_protocol_character(peek()), peek(), index);
                    if is_valid_protocol_character(self.peek()) {
                        buffer.push(self.consume());
                        continue;
                    }
                    if self.peek() != Some(':') {
                        panic!("Expected ':' but found {:?}", self.peek())
                    }
                    self.consume();
                    protocol = buffer.clone();

                    if self.peek() != Some('/') {
                        panic!("Expected '/' but found {:?}", self.peek());
                    }
                    self.consume();
                    if self.consume() != '/' {
                        panic!("Expected '/'");
                    }
                    if protocol.is_empty() {
                        panic!("No protocol found!");
                    }

                    state = State::InHostname;
                    buffer.clear();
                    continue;
                }
                State::InHostname => {
                    if is_valid_hostname_character(self.peek()) {
                        buffer.push(self.consume());
                        continue;
                    }
                    host = buffer.clone();
                    buffer.clear();

                    if self.peek() == Some(':') {
                        self.consume();
                        state = State::InPort;
                        continue;
                    }
                    if self.peek() == Some('/') {
                        state = State::InPath;
                        continue;
                    }
                    panic!("Expected '/' or ':' but found {:?}", self.peek());
                }
                State::InPort => {
                    if is_digit(self.peek()) {
                        buffer.push(self.consume());
                        continue;
                    }
                    if buffer.is_empty() {
                        panic!("Port cannot be empty");
                    }
                    port = buffer.parse::<u16>().unwrap();
                    buffer.clear();

                    if self.peek() == Some('/') {
                        state = State::InPath;
                        continue;
                    }
                    panic!("Expected '/' but found {:?}", self.peek());
                }
                State::InPath => {
                    if self.peek() == Some('?') || self.peek() == Some('#') {
                        path = buffer.clone();
                        buffer.clear();
                        state = if self.peek() == Some('?') {
                            State::InQuery
                        } else {
                            State::InFragment
                        };
                        self.consume();
                        continue;
                    }
                    buffer.push(self.consume());
                    continue;
                }
                State::InQuery => {
                    if self.peek() == Some('#') {
                        query = buffer.clone();
                        buffer.clear();
                        self.consume();
                        state = State::InFragment;
                        continue;
                    }
                    buffer.push(self.consume());
                    continue;
                }
                State::InFragment => {
                    buffer.push(self.consume());
                    continue;
                }
            }
        }

        if state == State::InHostname {
            if buffer.is_empty() {
                panic!("URL finished and we are still in host name");
            }
            host = buffer.clone();
            path = String::from("/");
        }

        if state == State::InProtocol {
            panic!("URL finished and we are still in protocol");
        }
        if state == State::InPath {
            path = buffer.clone();
        }
        if state == State::InQuery {
            query = buffer.clone();
        }
        if state == State::InFragment {
            fragment = buffer.clone();
        }
        if state == State::InPort {
            port = buffer.parse::<u16>().unwrap();
        }

        if port == Default::default() {
            if protocol == "http" {
                port = 80
            } else if protocol == "https" {
                port = 443
            }
        }

        URL {
            protocol,
            host,
            port,
            path,
            query,
            fragment,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_url() {
        assert_eq!(
            URLParser::new("http://example.com/").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 80,
                path: String::from("/"),
                query: String::from(""),
                fragment: String::from("")
            }
        );
    }

    #[test]
    fn parse_query_and_fragment_url() {
        assert_eq!(
            URLParser::new("http://example.com/test?query=1#fragment").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 80,
                path: String::from("/test"),
                query: String::from("query=1"),
                fragment: String::from("fragment")
            }
        );
    }

    #[test]
    fn parse_path_url() {
        assert_eq!(
            URLParser::new("http://example.com/test").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 80,
                path: String::from("/test"),
                query: String::from(""),
                fragment: String::from("")
            }
        );
    }

    #[test]
    fn parse_query_url() {
        assert_eq!(
            URLParser::new("http://example.com/test?query=value&lol=good").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 80,
                path: String::from("/test"),
                query: String::from("query=value&lol=good"),
                fragment: String::from("")
            }
        );
    }

    #[test]
    fn parse_port_url() {
        assert_eq!(
            URLParser::new("http://example.com:8765/test").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 8765,
                path: String::from("/test"),
                query: String::from(""),
                fragment: String::from("")
            }
        );
    }

    #[test]
    fn parse_fragment_url() {
        assert_eq!(
            URLParser::new("http://example.com:8765/test#fragment").parse(),
            URL {
                protocol: String::from("http"),
                host: String::from("example.com"),
                port: 8765,
                path: String::from("/test"),
                query: String::from(""),
                fragment: String::from("fragment")
            }
        );
    }

    #[test]
    fn parse_default_https_port_url() {
        assert_eq!(
            URLParser::new("https://example.com/test").parse(),
            URL {
                protocol: String::from("https"),
                host: String::from("example.com"),
                port: 443,
                path: String::from("/test"),
                query: String::from(""),
                fragment: String::from("")
            }
        );
    }
}
