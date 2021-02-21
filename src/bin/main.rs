use webserver::http::{HttpServer};
use webserver::http::http_server::Route;
use std::sync::Arc;

fn main() {
    let mut server = HttpServer::new();

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/"),
        handler: Arc::new(|request| {
            String::from(format!("lol request to {}", request.uri))
        })
    });

    server.start();
}
