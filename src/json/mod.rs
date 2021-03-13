mod parser;
mod value;
pub use parser::JsonParser;
pub use value::JsonValue;

pub trait ToJson {
    fn to_json(&self) -> JsonValue;
}
