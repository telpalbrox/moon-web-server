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

impl ToString for HttpRequest {
    fn to_string(&self) -> String {
        let mut result = String::new();
        let request_line = format!("{} {} HTTP/{}\r\n", self.method, self.uri, self.version);
        result.push_str(&request_line);
        for (key, value) in &self.headers {
            let header = format!("{}:{}\r\n", key, value);
            result.push_str(&header);
        }
        result.push_str("\r\n");
        result.push_str(&self.body);
        result
    }
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
