use super::http::{HttpServer, HttpResponse, HttpHeaders};
use super::http::send_http_request_with_headers;
use super::json::{JsonValue};
use super::templating::render_with_partials;
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

fn fetch_item(id: f64) -> JsonValue {
    let response = send_http_request_with_headers(&format!("{}/item/{}.json", HN_API_URL, id), headers());
    response.json()
}

fn get_item(id: f64) -> JsonValue {
    let mut item = fetch_item(id);
    let item_map = item.as_object_mut();
    let kids = match item_map.get("kids") {
        Some(value) => value.as_array(),
        None => return item
    };
    if kids.is_empty() {
        return item;
    }
    let kids_ids: Vec<f64> = kids.into_iter().map(|id| {
        return match id {
            JsonValue::Number(id) => *id,
            _ => panic!("expected ids to be numbers")
        }
    }).collect();
    let kid_items = map_id_to_objects(kids_ids, true);
    item_map.insert("kids".to_owned(), kid_items);
    item_map.insert("hasKids".to_owned(), JsonValue::Boolean(true));

    item
}

fn get_items(ids: &Vec<f64>) -> JsonValue {
    let (tx, rx) = channel();
    let mut joins = Vec::new();
    let stories_len = ids.len();
    for (i, id) in ids.into_iter().enumerate() {
        let tx = tx.clone();
        let id = id.clone();
        joins.push(thread::spawn(move || {
            let item = fetch_item(id);
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

fn map_id_to_objects(ids: Vec<f64>, fetch_kids: bool) -> JsonValue {
    if ids.is_empty() {
        return JsonValue::Null;
    }
    let mut items = get_items(&ids);
    if !fetch_kids {
        return items;
    }
    for item in items.as_array_mut() {
        let item = item.as_object_mut();
        let kids = match item.get("kids") {
            Some(value) => value.as_array(),
            None => continue
        };
        if kids.is_empty() {
            continue;
        }
        let ids: Vec<f64> = kids.into_iter().map(|id| {
            return match id {
                JsonValue::Number(id) => *id,
                _ => panic!("expected ids to be numbers")
            }
        }).collect();
        let items = map_id_to_objects(ids, true);
        item.insert("kids".to_owned(), items);
        item.insert("hasKids".to_owned(), JsonValue::Boolean(true));
    }

    return items;
}

fn get_stories(path: &str) -> JsonValue {
    let stories_ids = match request(&format!("{}/{}.json", HN_API_URL, path)) {
        JsonValue::Array(ids) => ids,
        _ => panic!("expected array from api")
    };
    let stories_ids: Vec<f64> = stories_ids.into_iter().take(30).map(|id| {
        return match id {
            JsonValue::Number(id) => id,
            _ => panic!("expected ids to be numbers")
        }
    }).collect();
    map_id_to_objects(stories_ids, false)
}

fn get_top_stories() -> JsonValue {
    get_stories("topstories")
}

pub fn oldweb(server: &mut HttpServer) {
    server.get("/hn", &|_req, mut res| {
        html(&mut res);
        let hn_response = get_top_stories();
        let mut context = HashMap::new();
        context.insert("stories".to_owned(), hn_response);
        let layout = read_file("./src/templates/oldweb/layout.hbs");
        let hn = read_file("./src/templates/oldweb/hn.hbs");
        let hnitemsummary = read_file("./src/templates/oldweb/partials/hnitemsummary.hbs");
        let mut partials = HashMap::new();
        partials.insert("body".to_owned(), hn);
        partials.insert("hnitemsummary".to_owned(), hnitemsummary);
        res.set_body(render_with_partials(layout, &JsonValue::Object(context), &partials));
    });

    server.get("/hn/:id", &|req, res| {
        let id = match req.params.get("id") {
            Some(id) => id,
            None => panic!("hn id not found")
        };
        let id = match id.parse::<f64>() {
            Ok(id) => id,
            Err(_) => {
                res.set_status_code(400);
                res.set_body(String::from("Item id has to be a number"));
                return;
            }
        };
        let item = get_item(id);
        let layout = read_file("./src/templates/oldweb/layout.hbs");
        let hnitem = read_file("./src/templates/oldweb/hnitem.hbs");
        let hncomment = read_file("./src/templates/oldweb/partials/hncomment.hbs");
        let hnitemsummary = read_file("./src/templates/oldweb/partials/hnitemsummary.hbs");
        let mut partials = HashMap::new();
        partials.insert("body".to_owned(), hnitem);
        partials.insert("hncomment".to_owned(), hncomment);
        partials.insert("hnitemsummary".to_owned(), hnitemsummary);
        res.set_body(render_with_partials(layout, &item, &partials));
    });
}
