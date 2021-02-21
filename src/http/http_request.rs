#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub uri: String,
    pub body: String
}
