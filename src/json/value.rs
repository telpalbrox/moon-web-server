use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
}
