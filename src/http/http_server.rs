use std::io::prelude::*;
use std::net::TcpListener;
use super::super::thread_pool::ThreadPool;
use super::HttpParser;
use std::sync::Arc;

use super::HttpRequest;

type RouteHandler = dyn Fn(HttpRequest) -> String + Send + Sync;

pub struct Route {
    pub method: String,
    pub uri: String,
    pub handler: Arc<RouteHandler>
}

pub struct HttpServer {
    routes: Arc<Vec<Route>>
}

impl HttpServer {
    pub fn new() -> HttpServer {
        HttpServer {
            routes: Arc::new(Vec::new())
        }
    }

    pub fn add_route(&mut self, route: Route) {
        Arc::get_mut(&mut self.routes).unwrap().push(route);
    }

    pub fn start(&self) {
        let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
        let pool = ThreadPool::new(4);
    
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let routes = Arc::clone(&self.routes);
    
            pool.execute(move || {
                routes.len();
                let mut buffer = [0; 8192];
                stream.read(&mut buffer).unwrap();
                let raw_request = String::from_utf8_lossy(&buffer);
                // println!("raw_request: {:?}", raw_request);
            
                let request = HttpParser::new(raw_request.as_ref().to_owned()).parse();

                let mut found_route = None;
                for i in 0..routes.len() {
                    let route = routes.get(i).unwrap();
                    if route.uri == request.uri {
                        found_route = Some(route);
                    }
                }

                let route = match found_route {
                    Some(route) => route,
                    None => return
                };

                let handler = &route.handler;
                let result = handler(request);

                let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", result);
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            });
        }
    }
}
