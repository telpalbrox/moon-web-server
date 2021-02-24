pub type HttpHeaders = Vec<(String, String)>;

mod http_parser;
mod http_request;
mod http_response;
pub mod http_server;
pub use http_parser::HttpParser as HttpParser;
pub use http_request::HttpRequest as HttpRequest;
pub use http_server::HttpServer as HttpServer;
pub use http_server::HttpServer as Route;
pub use http_response::HttpResponse as HttpResponse;