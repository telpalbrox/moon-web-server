use webserver::http::{HttpServer};
use webserver::http::http_server::Route;
use std::sync::Arc;

fn main() {
    let mut server = HttpServer::new();

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/"),
        handler: Arc::new(|request, mut response| {
            response.add_header("x-test".to_owned(), "more test".to_owned());
            response.set_body(format!("lol request to {}", request.uri));
            response
        })
    });

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/id/:id"),
        handler: Arc::new(|request, mut response| {
            response.set_body(format!("url id: {}", request.params.get("id").unwrap()));
            response
        })
    });

    server.start();
}
