use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;
use webserver::thread_pool::ThreadPool;
use webserver::http_parser::HttpParser;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let raw_request = String::from_utf8_lossy(&buffer);
    // println!("raw_request: {:?}", raw_request);

    let request = HttpParser::new(raw_request.as_ref().to_owned()).parse();
    // println!("request: {:?}", request);

    let (status_line, filename) = if request.uri == "/" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if request.uri == "/sleep" {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!("{}\r\n\r\n{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
