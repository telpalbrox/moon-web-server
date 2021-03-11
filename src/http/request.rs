use super::HttpHeaders;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub version: String,
    pub headers: HttpHeaders,
    pub uri: String,
    pub body: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

#[cfg(test)]
impl HttpRequest {
    pub fn new_with_uri(uri: String) -> Self {
        Self {
            method: "GET".to_owned(),
            uri,
            headers: Vec::new(),
            version: "1.1".to_owned(),
            body: String::new(),
            params: HashMap::new(),
            query: HashMap::new(),
        }
    }
}
