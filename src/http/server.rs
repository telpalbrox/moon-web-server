use super::super::thread_pool::ThreadPool;
use super::HttpParser;
use super::HttpResponse;
use std::env;
use std::io::prelude::*;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use super::HttpRequest;

type State<T> = Arc<Mutex<T>>;
type RouteHandler<T> = dyn Fn(&HttpRequest, &mut HttpResponse, State<T>) -> () + Send + Sync;
type Middleware<T> = dyn Fn(&HttpRequest, &mut HttpResponse, State<T>) -> bool + Send + Sync;

pub struct Route<T> {
    pub method: String,
    pub uri: String,
    pub middleware: Arc<Vec<Box<Middleware<T>>>>,
    pub handler: Arc<RouteHandler<T>>,
}

impl<T> Route<T> {
    pub fn matches_uri(&self, uri: &String) -> bool {
        // remove query part of the url
        let uri: String = uri.split('?').take(1).collect();
        if !self.uri.contains(':') {
            return self.uri == uri;
        }

        let colon_position = self.uri.find(':').unwrap();
        let route_before_color = self.uri.get(..colon_position).unwrap();
        if !uri.starts_with(route_before_color) {
            return false;
        }
        uri.get(colon_position + 1..).is_some()
    }

    pub fn add_params(&self, request: &mut HttpRequest) {
        let colon_position = match self.uri.find(':') {
            None => {
                return;
            }
            Some(position) => position,
        };

        let route_before_color = self.uri.get(..colon_position).unwrap();
        if !request.uri.starts_with(route_before_color) {
            return;
        }
        let param_key = self.uri.get(colon_position + 1..).unwrap();
        let param_value = match request.uri.get(colon_position..) {
            None => return,
            Some(value) => value,
        };
        request
            .params
            .insert(param_key.to_owned(), param_value.to_owned());
    }
}

pub struct HttpServer<T: Send + Sync + 'static> {
    routes: Arc<Vec<Route<T>>>,
    state: Arc<Mutex<T>>
}

impl<T: Send + Sync> HttpServer<T> {
    pub fn new(state: T) -> HttpServer<T> {
        HttpServer {
            routes: Arc::new(Vec::new()),
            state: Arc::new(Mutex::new(state))
        }
    }

    pub fn state(&self) -> State<T> {
        Arc::clone(&self.state)
    }

    pub fn add_route(&mut self, route: Route<T>) {
        Arc::get_mut(&mut self.routes).unwrap().push(route);
    }

    pub fn get(&mut self, uri: &str, handler: &'static RouteHandler<T>) {
        let route = Route {
            uri: uri.to_owned(),
            method: "GET".to_owned(),
            middleware: Arc::new(Vec::new()),
            handler: Arc::new(handler),
        };
        Arc::get_mut(&mut self.routes).unwrap().push(route);
    }

    pub fn start(&self) {
        let port = match env::var("PORT") {
            Ok(port) => port,
            Err(_) => "7878".to_owned(),
        };
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
        let pool = ThreadPool::new(4);

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let routes = Arc::clone(&self.routes);
            let mutex = Arc::clone(&self.state);

            pool.execute(move || {
                routes.len();
                let mut buffer = [0; 8192];
                stream.read(&mut buffer).unwrap();
                let raw_request = String::from_utf8_lossy(&buffer).replace('\0', "");
                // println!("raw_request: {:?}", raw_request);

                let mut request = HttpParser::new(&raw_request).parse_request();

                let mut found_route = None;
                for i in 0..routes.len() {
                    let route = routes.get(i).unwrap();
                    if route.matches_uri(&request.uri) {
                        found_route = Some(route);
                    }
                }

                let result: HttpResponse;
                result = match found_route {
                    Some(route) => {
                        let handler = &route.handler;
                        route.add_params(&mut request);
                        let mut response = HttpResponse::new();
                        let middlewares = Arc::clone(&route.middleware);
                        let mut should_handle = true;
                        for middleware in middlewares.iter() {
                            should_handle = middleware(&request, &mut response, mutex.clone());
                            if !should_handle {
                                break;
                            }
                        }
                        if should_handle {
                            handler(&request, &mut response, mutex);
                        }
                        response
                    }
                    None => {
                        let mut result = HttpResponse::new();
                        result.set_status_code(404);
                        result.set_body("Not found".to_owned());
                        result
                    }
                };

                let response = result.to_string();
                // println!("response: {:?}", response);
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::request::HttpRequest;
    use super::*;

    #[test]
    fn basic_route_match() {
        let route: Route<()> = Route {
            method: String::from("GET"),
            uri: String::from("/test"),
            middleware: Arc::new(Vec::new()),
            handler: Arc::new(|_, _, _| ()),
        };
        assert_eq!(route.matches_uri(&"/test?query=1".to_owned()), true);
    }

    #[test]
    fn route_with_parameter_match() {
        let route: Route<()> = Route {
            method: String::from("GET"),
            uri: String::from("/test/:test_param"),
            middleware: Arc::new(Vec::new()),
            handler: Arc::new(|_, _, _| ()),
        };
        assert_eq!(route.matches_uri(&"/test".to_owned()), false);
        assert_eq!(route.matches_uri(&"/test/test".to_owned()), true);
    }

    #[test]
    fn get_params_from_route() {
        let route: Route<()> = Route {
            method: String::from("GET"),
            uri: String::from("/test/:test_param"),
            middleware: Arc::new(Vec::new()),
            handler: Arc::new(|_, _, _| ()),
        };
        let mut request = HttpRequest::new_with_uri("/test/some_param".to_owned());
        route.add_params(&mut request);
        assert_eq!(request.params.get("test_param").is_some(), true);
        assert_eq!(request.params.get("test_param").unwrap(), "some_param");
    }
}
