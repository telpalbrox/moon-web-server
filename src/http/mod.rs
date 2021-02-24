pub type HttpHeaders = Vec<(String, String)>;

mod http_parser;
mod http_request;
mod http_response;
pub mod http_server;
pub use http_parser::HttpParser;
pub use http_request::HttpRequest;
pub use http_response::HttpResponse;
pub use http_server::HttpServer;
pub use http_server::HttpServer as Route;
