use super::parser::HttpParser;
use super::request::HttpRequest;
use super::response::HttpResponse;
use super::url::{URLParser, URL};
use super::HttpHeaders;
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;

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

pub fn send_http_request(url: &str) -> HttpResponse {
    send_http_request_impl(url, Vec::new())
}

pub fn send_http_request_with_headers(url: &str, headers: HttpHeaders) -> HttpResponse {
    send_http_request_impl(url, headers)
}

fn send_http_request_impl(url: &str, mut headers: HttpHeaders) -> HttpResponse {
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
    let mut stream = TcpStream::connect(host).unwrap();
    stream.write(request.to_string().as_bytes()).unwrap();
    let mut buffer = [0; 16384]; // 16K
    stream.read(&mut buffer).unwrap();
    let raw_response = String::from_utf8_lossy(&buffer).replace('\0', "");
    // println!("raw response: {:?}", raw_response);
    HttpParser::new(&raw_response).parse_response()
}
