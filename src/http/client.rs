use super::parser::HttpParser;
use super::request::HttpRequest;
use super::response::HttpResponse;
use super::url::{URLParser, URL};
use super::HttpHeaders;
use super::HttpParserError;
use std::{collections::HashMap, str};
use std::io::prelude::*;
use std::net::TcpStream;
use std::fmt;

#[derive(Debug)]
pub struct ConnectionError {
    host: String,
    tcp_error: std::io::Error
}

#[derive(Debug)]
pub enum HttpClientError {
    ConnectionError(ConnectionError),
    ParseHttpResponseError(HttpParserError),
    ReadResponseError(std::io::Error),
    WriteResponseError(std::io::Error),
}

impl fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectionError(connection_error) => {
                write!(f, "Error connecting to {:?}: {}", connection_error.host, connection_error.tcp_error)
            },
            Self::ParseHttpResponseError(err) => {
                err.fmt(f)
            },
            Self::ReadResponseError(err) => {
                write!(f, "Error reading server response: {}", err)
            },
            Self::WriteResponseError(err) => {
                write!(f, "Error writing request: {}", err)
            }
        }
    }
}

type Result<T> = std::result::Result<T, HttpClientError>;

fn get_uri(url: &URL) -> String {
    let mut result = url.path.clone();
    if !url.path.is_empty() {
        result.push('?');
        result.push_str(&url.query);
    }
    if !url.fragment.is_empty() {
        result.push('#');
        result.push_str(&url.fragment);
    }
    result
}

pub fn send_http_request(url: &str) -> Result<HttpResponse> {
    send_http_request_impl(url, Vec::new())
}

pub fn send_http_request_with_headers(url: &str, headers: HttpHeaders) -> Result<HttpResponse> {
    send_http_request_impl(url, headers)
}

fn send_http_request_impl(url: &str, mut headers: HttpHeaders) -> Result<HttpResponse> {
    let url = URLParser::new(url).parse();
    let host = format!("{}:{}", url.host, url.port);
    let mut req_headers = vec![(String::from("Host"), String::from(&host))];
    req_headers.append(&mut headers);
    let request = HttpRequest {
        method: String::from("GET"),
        body: String::new(),
        uri: get_uri(&url),
        version: String::from("1.1"),
        headers: req_headers,
        query: HashMap::new(),
        params: HashMap::new(),
    };
    let mut stream = match TcpStream::connect(&host) {
        Ok(stream) => stream,
        Err(err) => return Err(HttpClientError::ConnectionError(ConnectionError { tcp_error: err, host: host.to_owned() }))
    };
    match stream.write(request.to_string().as_bytes()) {
        Ok(_) => {},
        Err(err) => return Err(HttpClientError::WriteResponseError(err))
    }
    let mut buffer = [0; 16384]; // 16K
    match stream.read(&mut buffer) {
        Ok(_) => {},
        Err(err) => return Err(HttpClientError::ReadResponseError(err))
    };
    let raw_response = String::from_utf8_lossy(&buffer).replace('\0', "");
    // println!("raw response: {:?}", raw_response);
    match HttpParser::new(&raw_response).parse_response() {
        Ok(response) => return Ok(response),
        Err(err) => {
            return Err(HttpClientError::ParseHttpResponseError(err));
        }
    }
}
