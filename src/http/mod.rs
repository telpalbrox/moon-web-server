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

mod http_parser;
mod http_request;
mod http_response;
pub mod http_server;
pub use http_parser::HttpParser;
pub use http_request::HttpRequest;
pub use http_response::HttpResponse;
pub use http_server::HttpServer;
pub use http_server::HttpServer as Route;
