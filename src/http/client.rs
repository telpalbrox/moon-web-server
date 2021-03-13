use super::parser::HttpParser;
use super::response::HttpResponse;
use super::request::HttpRequest;
use std::io::prelude::*;
use std::collections::HashMap;
use std::net::TcpStream;

pub fn send_http_request(method: &str, host: &str, port: u16, path: &str) -> HttpResponse {
    let host = format!("{}:{}", host, port);
    let request = HttpRequest {
        method: String::from(method),
        body: String::new(),
        uri: String::from(path),
        version: String::from("1.1"),
        headers: vec!(
            (String::from("Host"), String::from(&host))
        ),
        query: HashMap::new(),
        params: HashMap::new(),
    };
    let mut stream = TcpStream::connect(host).unwrap();
    println!("write {:?}", request.to_string());
    stream.write(request.to_string().as_bytes()).unwrap();
    let mut buffer = [0; 8192];
    print!("read");
    stream.read(&mut buffer).unwrap();
    let raw_response = String::from_utf8_lossy(&buffer).replace('\0', "");
    HttpParser::new(raw_response).parse_response()
}
