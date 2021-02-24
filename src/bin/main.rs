use std::sync::Arc;
use webserver::http::http_server::Route;
use webserver::http::HttpServer;

fn main() {
    let mut server = HttpServer::new();

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/"),
        handler: Arc::new(|request, mut response| {
            response.add_header("x-test".to_owned(), "more test".to_owned());
            response.set_body(format!("lol request to {}", request.uri));
            response
        }),
    });

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/id/:id"),
        handler: Arc::new(|request, mut response| {
            response.set_body(format!("url id: {}", request.params.get("id").unwrap()));
            response
        }),
    });

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/query"),
        handler: Arc::new(|request, mut response| {
            response.set_body(format!(
                "query param key: {}",
                request
                    .query
                    .get("key")
                    .unwrap_or(&String::from("not present"))
            ));
            response
        }),
    });

    server.start();
}
