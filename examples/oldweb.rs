use webserver::http::{HttpServer, HttpResponse, HttpHeaders};
use webserver::http::send_http_request_with_headers;
use webserver::json::{JsonValue};
use webserver::templating::render_with_partials;
use std::fs;
use std::collections::HashMap;
use std::thread;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

fn read_file(path: &'static str) -> String {
    fs::read_to_string(path).unwrap()
}

fn html(res: &mut HttpResponse) {
    res
        .headers_mut()
        .push(("Content-Type".to_owned(), "text/html".to_owned()));
}

const HN_API_URL: &str = "http://0.0.0.0:8081/https://hacker-news.firebaseio.com/v0";

type ItemsCache = HashMap<u64, JsonValue>;
type ItemsCacheMutex = Arc<Mutex<ItemsCache>>;

fn headers() -> HttpHeaders {
    vec![("x-requested-with".to_owned(), ("rust lol".to_owned()))]
}

fn request(url: &str) -> JsonValue {
    let response = send_http_request_with_headers(url, headers());
    response.json()
}

fn get_max_id() -> u64 {
    let response = send_http_request_with_headers(&format!("{}/maxitem.json", HN_API_URL), headers());
    response.json().as_number() as u64
}

fn get_updates() -> JsonValue {
    let response = send_http_request_with_headers(&format!("{}/updates.json", HN_API_URL), headers());
    response.json()
}

fn get_changed_items() -> Vec<u64> {
    get_updates().as_object().get("items").unwrap().as_array().into_iter().map(|value| value.as_number() as u64).collect()
}

fn get_time_ago(time: f64) -> String {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as f64;
    let minutes = (now - time) / 60f64;
    if minutes < 60f64 {
        return format!("{} minutes ago", minutes.round());
    }
    let hours = minutes / 60f64;
    if hours < 24f64 {
        return format!("{} hours ago", hours.round());
    }
    let days = hours / 24f64;
    if days < 30f64 {
        return format!("{} days ago", days.round());
    }
    let years = days / 365f64;
    return format!("{} years ago", years.round());
}

fn fetch_item(items_cache: &ItemsCacheMutex, id: u64) -> JsonValue {
    let item_cache_read = items_cache.lock().unwrap();
    if item_cache_read.contains_key(&id) {
        println!("Cache hit for {}", id);
        return item_cache_read.get(&id).unwrap().clone();
    }
    drop(item_cache_read);
    let response = send_http_request_with_headers(&format!("{}/item/{}.json", HN_API_URL, id), headers());
    let item = response.json();
    println!("fetched item {}", id);
    let clone = item.clone();
    let mut item_cache = items_cache.lock().unwrap();
    item_cache.insert(id, item);
    drop(item_cache);
    clone
}

fn get_item(items_cache: ItemsCacheMutex, id: u64) -> JsonValue {
    let mut item = fetch_item(&items_cache, id);

    let item_map = item.as_object_mut();
    let time = item_map.get(&"time".to_owned()).expect("item doesn't have time").as_number();
    item_map.insert("relative_time".to_owned(),JsonValue::String(get_time_ago(time)));
    let kids = match item_map.get("kids") {
        Some(value) => value.as_array(),
        None => return item
    };
    if kids.is_empty() {
        return item;
    }
    let kids_ids: Vec<u64> = kids.into_iter().map(|id| {
        return match id {
            JsonValue::Number(id) => *id as u64,
            _ => panic!("expected ids to be numbers")
        }
    }).collect();
    let kid_items = map_id_to_objects(&items_cache, kids_ids, true);
    item_map.insert("kids".to_owned(), kid_items);

    item
}

fn get_items(items_cache: &ItemsCacheMutex, ids: &Vec<u64>) -> JsonValue {
    let (tx, rx) = channel();
    let mut joins = Vec::new();
    let stories_len = ids.len();
    for (i, id) in ids.into_iter().enumerate() {
        let tx = tx.clone();
        let id = id.clone(); 
        let mutex = items_cache.clone();
        joins.push(thread::spawn(move || {
            let item = fetch_item(&mutex, id);
            drop(mutex);
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

    let stories_results = stories_results.into_iter().map(|(_, value)| value.clone()).collect();

    JsonValue::Array(stories_results)
}

fn map_id_to_objects(items_cache: &ItemsCacheMutex, ids: Vec<u64>, fetch_kids: bool) -> JsonValue {
    if ids.is_empty() {
        return JsonValue::Null;
    }
    let mut items = get_items(&items_cache, &ids);
    for item in items.as_array_mut() {
        let item = item.as_object_mut();

        let time = item.get(&"time".to_owned()).expect("item doesn't have time").as_number();
        item.insert("relative_time".to_owned(),JsonValue::String(get_time_ago(time)));

        if !fetch_kids {
            continue;
        }
        
        let kids = match item.get("kids") {
            Some(value) => value.as_array(),
            None => continue
        };
        if kids.is_empty() {
            continue;
        }
        let ids: Vec<u64> = kids.into_iter().map(|id| {
            return match id {
                JsonValue::Number(id) => *id as u64,
                _ => panic!("expected ids to be numbers")
            }
        }).collect();
        let items = map_id_to_objects(items_cache, ids, true);
        item.insert("kids".to_owned(), items);
    }

    return items;
}

fn get_stories(items_cache: &ItemsCacheMutex, path: &str) -> JsonValue {
    let stories_ids = match request(&format!("{}/{}.json", HN_API_URL, path)) {
        JsonValue::Array(ids) => ids,
        _ => panic!("expected array from api")
    };
    let stories_ids: Vec<u64> = stories_ids.into_iter().take(30).map(|id| {
        return match id {
            JsonValue::Number(id) => id as u64,
            _ => panic!("expected ids to be numbers")
        }
    }).collect();
    map_id_to_objects(items_cache, stories_ids, false)
}

fn get_top_stories(items_cache: &ItemsCacheMutex) -> JsonValue {
    get_stories(items_cache, "topstories")
}

fn warmup(server: &HttpServer<ItemsCache>) {
    let items_cache = server.state().clone();
    thread::spawn(move || {
        let max_id = get_max_id();
        let low_id = max_id - 4000;
        for i in low_id..max_id {
            fetch_item(&items_cache, i);
        }
    });
}

fn watch_changed_items(server: &HttpServer<ItemsCache>) {
    let items_cache = server.state().clone();
    thread::spawn(move || {
        loop {
            println!("fetching updates");
            for id in get_changed_items() {
                fetch_item(&items_cache, id);
            }
            thread::sleep(Duration::from_secs(60));
        }
    });
}

fn oldweb(server: &mut HttpServer<ItemsCache>) {
    server.get("/hn", &|_req, mut res, items_cache| {
        html(&mut res);
        let hn_response = get_top_stories(&items_cache);
        let mut context = HashMap::new();
        context.insert("stories".to_owned(), hn_response);
        let layout = read_file("./examples/templates/oldweb/layout.hbs");
        let hn = read_file("./examples/templates/oldweb/hn.hbs");
        let hnitemsummary = read_file("./examples/templates/oldweb/partials/hnitemsummary.hbs");
        let mut partials = HashMap::new();
        partials.insert("body".to_owned(), hn);
        partials.insert("hnitemsummary".to_owned(), hnitemsummary);
        res.set_body(render_with_partials(&layout, &JsonValue::Object(context), &partials));
    });

    server.get("/hn/:id", &|req, res, items_cache| {
        let id = match req.params.get("id") {
            Some(id) => id,
            None => panic!("hn id not found")
        };
        let id = match id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                res.set_status_code(400);
                res.set_body(String::from("Item id has to be a number"));
                return;
            }
        };
        let item = get_item(items_cache, id);
        let layout = read_file("./examples/templates/oldweb/layout.hbs");
        let hnitem = read_file("./examples/templates/oldweb/hnitem.hbs");
        let hncomment = read_file("./examples/templates/oldweb/partials/hncomment.hbs");
        let hnitemsummary = read_file("./examples/templates/oldweb/partials/hnitemsummary.hbs");
        let mut partials = HashMap::new();
        partials.insert("body".to_owned(), hnitem);
        partials.insert("hncomment".to_owned(), hncomment);
        partials.insert("hnitemsummary".to_owned(), hnitemsummary);
        res.set_body(render_with_partials(&layout, &item, &partials));
    });
}

fn main() {
    let items_cache = HashMap::new();
    let mut server = HttpServer::new(items_cache);
    watch_changed_items(&server);
    warmup(&server);
    oldweb(&mut server);
    server.start();
}
