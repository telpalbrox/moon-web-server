use super::parser::HttpParser;
use super::request::HttpRequest;
use super::response::HttpResponse;
use super::url::{URLParser, URL};
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
    let url = URLParser::new(url).parse();
    let host = format!("{}:{}", url.host, url.port);
    let request = HttpRequest {
        method: String::from("GET"),
        body: String::new(),
        uri: get_uri(&url),
        version: String::from("1.1"),
        headers: vec![(String::from("Host"), String::from(&host))],
        query: HashMap::new(),
        params: HashMap::new(),
    };
    let mut stream = TcpStream::connect(host).unwrap();
    stream.write(request.to_string().as_bytes()).unwrap();
    let mut buffer = [0; 8192];
    stream.read(&mut buffer).unwrap();
    let raw_response = String::from_utf8_lossy(&buffer).replace('\0', "");
    HttpParser::new(raw_response).parse_response()
}
