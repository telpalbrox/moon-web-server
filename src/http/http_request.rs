use std::collections::HashMap;
use super::HttpHeaders;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub version: String,
    pub headers: HttpHeaders,
    pub uri: String,
    pub body: String,
    pub params: HashMap<String, String>
}
