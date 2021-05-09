use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use webserver::http::send_http_request;
use webserver::http::server::Route;
use webserver::http::HttpServer;
use webserver::http::{HttpRequest, HttpResponse};
use webserver::json::JsonValue;
use webserver::templating::render;

fn read_file(path: &'static str) -> String {
    fs::read_to_string(path).unwrap()
}

fn main() {
    let mut server = HttpServer::new(HashMap::new());

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/"),
        middleware: Arc::new(vec![]),
        handler: Arc::new(|request, response, _| {
            response.add_header("x-test".to_owned(), "more test".to_owned());
            response.set_body(format!("lol request to {}", request.uri));
        }),
    });

    server.add_route(Route {
        method: String::from("GET"),
        uri: String::from("/middleware"),
        middleware: Arc::new(vec![Box::new(|_, response, _| {
            response.set_body(String::from("Set from middelware"));
            return false;
        })]),
        handler: Arc::new(|_, _, _| {
            panic!("This doesn't run");
        }),
    });

    server.get("/id/:id", &|request: &HttpRequest,
                            response: &mut HttpResponse,
                            _| {
        response.set_body(format!("url id: {}", request.params.get("id").unwrap()));
    });

    server.get("/query", &|request: &HttpRequest,
                           response: &mut HttpResponse,
                           _| {
        response.set_body(format!(
            "query param key: {}",
            request
                .query
                .get("key")
                .unwrap_or(&String::from("not present"))
        ));
    });

    server.get("/hello", &|request: &HttpRequest,
                           response: &mut HttpResponse,
                           _| {
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
            &read_file("./examples/templates/hello.html").to_owned(),
            &JsonValue::Object(context),
        ));
    });

    server.get("/headers", &|request: &HttpRequest,
                             response: &mut HttpResponse,
                             _| {
        response
            .headers_mut()
            .push(("Content-Type".to_owned(), "text/html".to_owned()));

        let mut context = HashMap::new();
        context.insert(
            "headers".to_owned(),
            JsonValue::from(request.headers.clone()),
        );
        response.set_body(render(
            &read_file("./examples/templates/headers.html").to_owned(),
            &JsonValue::from(context),
        ));
    });

    let mutex = server.state();
    let mut state = mutex.lock().unwrap();
    state.insert(String::from("visits"), String::from("0"));
    drop(state);
    drop(mutex);

    server.get("/httpreq", &|_request: &HttpRequest,
                             response: &mut HttpResponse,
                             _| {
        response
            .headers_mut()
            .push(("Content-Type".to_owned(), "application/json".to_owned()));
        let bin_response = send_http_request("http://httpbin.org/get");
        response.set_body(bin_response.body);
    });

    server.get(
        "/visits",
        &|_request: &HttpRequest,
          response: &mut HttpResponse,
          state: Arc<Mutex<HashMap<String, String>>>| {
            let mut state = state.lock().unwrap();
            let mut visits: u64 = state.get("visits").unwrap().parse().unwrap();
            visits = visits + 1;
            state.insert(String::from("visits"), visits.to_string());

            response.set_body(format!("visits: {}", visits.to_string()));
        },
    );

    server.start();
}
