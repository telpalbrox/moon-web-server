use std::fs;
use std::sync::Arc;
use webserver::http::http_server::Route;
use webserver::http::HttpServer;
use webserver::http::{HttpRequest, HttpResponse};
use webserver::templating::render;

fn read_file(path: &'static str) -> String {
    fs::read_to_string(path).unwrap()
}

fn main() {
    let mut server = HttpServer::new();

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/"),
        handler: Arc::new(|request, response| {
            response.add_header("x-test".to_owned(), "more test".to_owned());
            response.set_body(format!("lol request to {}", request.uri));
        }),
    });

    server.get(
        "/id/:id",
        &|request: HttpRequest, response: &mut HttpResponse| {
            response.set_body(format!("url id: {}", request.params.get("id").unwrap()));
        },
    );

    server.get(
        "/query",
        &|request: HttpRequest, response: &mut HttpResponse| {
            response.set_body(format!(
                "query param key: {}",
                request
                    .query
                    .get("key")
                    .unwrap_or(&String::from("not present"))
            ));
        },
    );

    server.get(
        "/hello",
        &|request: HttpRequest, response: &mut HttpResponse| {
            response
                .headers_mut()
                .push(("Content-Type".to_owned(), "text/html".to_owned()));
            response.set_body(render(
                read_file("./src/templates/hello.html").to_owned(),
                &request.query,
            ));
        },
    );

    server.start();
}
