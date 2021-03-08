use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use webserver::http::http_server::Route;
use webserver::http::HttpServer;
use webserver::http::{HttpRequest, HttpResponse};
use webserver::json::JsonValue;
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
            let mut context = HashMap::new();
            let name = request
                .query
                .get("name")
                .unwrap_or(&String::from(""))
                .to_owned();
            if name == "victoria" {
                context.insert(String::from("beloved"), JsonValue::Boolean(true));
            }
            context.insert("name".to_owned(), JsonValue::String(name));
            response.set_body(render(
                read_file("./src/templates/hello.html").to_owned(),
                &JsonValue::Object(context),
            ));
        },
    );

    server.get(
        "/headers",
        &|request: HttpRequest, response: &mut HttpResponse| {
            response
                .headers_mut()
                .push(("Content-Type".to_owned(), "text/html".to_owned()));

            let mut context = HashMap::new();
            context.insert("headers".to_owned(), JsonValue::from(request.headers));
            response.set_body(render(
                read_file("./src/templates/headers.html").to_owned(),
                &JsonValue::from(context),
            ));
        },
    );

    server.start();
}
