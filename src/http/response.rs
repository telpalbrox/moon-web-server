use super::HttpHeaders;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpResponse {
    pub headers: HttpHeaders,
    pub status_code: u16,
    pub body: String,
    pub version: String,
    pub reason: String
}

const HTTP_VERSION: &str = "HTTP/1.1";

impl HttpResponse {
    fn http_reasons() -> HashMap<u16, &'static str> {
        [
            (200, "Ok"),
            (400, "Bad request"),
            (404, "Not found"),
            (500, "Internal server error"),
        ]
        .iter()
        .cloned()
        .collect()
    }

    pub fn new() -> Self {
        HttpResponse {
            headers: Vec::new(),
            status_code: 200,
            body: String::new(),
            version: String::from("1.1"),
            reason: String::from("")
        }
    }

    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    pub fn add_header(&mut self, header_key: String, header_value: String) {
        self.headers_mut().push((header_key, header_value));
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code;
    }

    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    fn get_status_line(&self) -> String {
        let reasons = HttpResponse::http_reasons();
        let reason_phrase = reasons
            .get(&self.status_code())
            .unwrap_or(&"Something happened");
        format!("{} {} {}", HTTP_VERSION, self.status_code(), reason_phrase)
    }

    fn headers_to_string(&self) -> String {
        let mut headers_string = String::new();

        for (header_key, header_value) in self.headers() {
            headers_string.push_str(&format!("{}:{}\r\n", header_key, header_value));
        }

        headers_string
    }

    pub fn to_string(&self) -> String {
        let status_line = self.get_status_line();
        let headers = self.headers_to_string();
        format!("{}\r\n{}\r\n{}", status_line, headers, self.body())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_response() {
        let default_response = HttpResponse::new();
        assert_eq!(default_response.to_string(), "HTTP/1.1 200 Ok\r\n\r\n");
    }

    #[test]
    fn headers_response() {
        let mut response = HttpResponse::new();
        response.add_header("x-test".to_owned(), "more test".to_owned());
        response.set_body("test body".to_owned());
        assert_eq!(
            response.to_string(),
            "HTTP/1.1 200 Ok\r\nx-test:more test\r\n\r\ntest body"
        )
    }
}
