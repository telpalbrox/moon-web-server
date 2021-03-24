use super::http::{HttpServer, HttpResponse, HttpHeaders};
use super::http::send_http_request_with_headers;
use super::json::{JsonValue};
use super::templating::render;
use std::fs;
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::channel;

fn read_file(path: &'static str) -> String {
    fs::read_to_string(path).unwrap()
}

fn html(res: &mut HttpResponse) {
    res
        .headers_mut()
        .push(("Content-Type".to_owned(), "text/html".to_owned()));
}

const HN_API_URL: &str = "http://0.0.0.0:8081/https://hacker-news.firebaseio.com/v0";

fn headers() -> HttpHeaders {
    vec![("x-requested-with".to_owned(), ("rust lol".to_owned()))]
}

fn request(url: &str) -> JsonValue {
    let response = send_http_request_with_headers(url, headers());
    response.json()
}

fn get_item(id: f64) -> JsonValue {
    let response = send_http_request_with_headers(&format!("{}/item/{}.json", HN_API_URL, id), headers());
    response.json()
}

fn get_top_stories() -> JsonValue {
    let stories_ids = match request(&format!("{}/topstories.json", HN_API_URL)) {
        JsonValue::Array(ids) => ids,
        _ => panic!("expected array from api")
    };
    let stories: Vec<f64> = stories_ids.into_iter().take(10).map(|id| {
        return match id {
            JsonValue::Number(id) => id,
            _ => panic!("expected ids to be numbers")
        }
    }).collect();
    let (tx, rx) = channel();
    let mut joins = Vec::new();
    let stories_len = stories.len();
    for (i, id) in stories.into_iter().enumerate() {
        let tx = tx.clone();
        joins.push(thread::spawn(move || {
            let item = get_item(id);
            tx.send((i, item)).expect("Error sending item");
        }));
    }
    let mut stories_results = Vec::with_capacity(stories_len);
    for _ in 0..stories_len {
        let (i, item) = rx.recv().expect("Error receiving item");
        stories_results.push((i, item));
    }

    for child in joins {
        child.join().unwrap();
    }

    stories_results.sort_by_key(|(i, _)| {
        *i
    });

    let stories_results = stories_results.into_iter().map(|(_, value)| value).collect();

    JsonValue::Array(stories_results)
}

pub fn oldweb(server: &mut HttpServer) {
    server.get("/hn", &|_req, mut res| {
        html(&mut res);
        let hn_response = get_top_stories();
        let mut context = HashMap::new();
        context.insert("stories".to_owned(), hn_response);
        res.set_body(render(read_file("./src/templates/oldweb/hn.hbs"), &JsonValue::Object(context)));
    });
}
