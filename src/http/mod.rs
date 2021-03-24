use super::json::JsonValue;
use std::collections::HashMap;

pub type HttpHeaders = Vec<(String, String)>;

impl From<HttpHeaders> for JsonValue {
    fn from(headers: HttpHeaders) -> JsonValue {
        let mut result: Vec<JsonValue> = Vec::with_capacity(headers.len());

        for (key, value) in headers {
            let mut object = HashMap::with_capacity(2);
            object.insert("key".to_owned(), JsonValue::from(value));
            object.insert("value".to_owned(), JsonValue::from(key));
            result.push(JsonValue::from(object));
        }

        JsonValue::from(result)
    }
}

mod client;
mod parser;
mod request;
mod response;
pub mod server;
mod url;
pub use client::send_http_request;
pub use parser::HttpParser;
pub use request::HttpRequest;
pub use response::HttpResponse;
pub use server::HttpServer;
pub use server::HttpServer as Route;
