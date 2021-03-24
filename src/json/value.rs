use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
    Null,
}

impl JsonValue {
    pub fn stringify(&self) -> String {
        match self {
            Self::Number(number) => number.to_string(),
            Self::String(string) => string.to_owned(),
            Self::Boolean(boolean) => boolean.to_string(),
            Self::Null => String::from("null"),
            Self::Array(elements) => {
                let mut result = String::from("[");

                for (i, element) in elements.iter().enumerate() {
                    result.push_str(&element.stringify());
                    if i != elements.len() - 1 {
                        result.push(',');
                    }
                }

                result.push(']');
                result
            }
            Self::Object(object) => {
                let mut result = String::from("{");
                for (i, (key, value)) in object.iter().enumerate() {
                    result.push_str(&format!("\"{}\":{}", key, value.stringify()));
                    if i != object.len() - 1 {
                        result.push(',');
                    }
                }

                result.push('}');
                result
            }
        }
    }
}

impl JsonValue {
    pub fn as_object(&self) -> &HashMap<String, JsonValue> {
        match self {
            Self::Object(map) => {
                return map;
            },
            _ => panic!("Not an object")
        }
    }

    pub fn as_number(&self) -> f64 {
        match self {
            Self::Number(number) => {
                return *number;
            },
            _ => panic!("Not a number")
        }
    }
}

impl FromIterator<JsonValue> for JsonValue {
    fn from_iter<I: IntoIterator<Item=JsonValue>>(iter: I) -> Self {
        let mut array = Vec::new();

        for value in iter {
            array.push(value);
        }

        JsonValue::Array(array)
    }
}

impl From<&str> for JsonValue {
    fn from(string: &str) -> JsonValue {
        JsonValue::String(string.to_owned())
    }
}

impl From<&String> for JsonValue {
    fn from(string: &String) -> JsonValue {
        JsonValue::String(string.to_owned())
    }
}

impl From<String> for JsonValue {
    fn from(string: String) -> JsonValue {
        JsonValue::String(string)
    }
}

impl From<f64> for JsonValue {
    fn from(number: f64) -> JsonValue {
        JsonValue::Number(number)
    }
}

impl From<bool> for JsonValue {
    fn from(boolean: bool) -> JsonValue {
        JsonValue::Boolean(boolean)
    }
}

impl From<Vec<JsonValue>> for JsonValue {
    fn from(array: Vec<JsonValue>) -> JsonValue {
        JsonValue::Array(array)
    }
}

impl From<HashMap<String, JsonValue>> for JsonValue {
    fn from(object: HashMap<String, JsonValue>) -> JsonValue {
        JsonValue::Object(object)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stringify_number() {
        assert_eq!(JsonValue::Number(27_f64).stringify(), "27");
    }

    #[test]
    fn stringify_string() {
        assert_eq!(JsonValue::String("test".to_owned()).stringify(), "test");
    }

    #[test]
    fn stringify_false() {
        assert_eq!(JsonValue::Boolean(false).stringify(), "false");
    }

    #[test]
    fn stringify_true() {
        assert_eq!(JsonValue::Boolean(true).stringify(), "true");
    }

    #[test]
    fn stringify_null() {
        assert_eq!(JsonValue::Null.stringify(), "null");
    }

    #[test]
    fn stringify_array() {
        assert_eq!(
            JsonValue::Array(vec!(
                JsonValue::Number(1_f64),
                JsonValue::Number(2_f64),
                JsonValue::Number(3_f64)
            ))
            .stringify(),
            "[1,2,3]"
        );
    }

    fn _stringify_object() {
        let mut object = HashMap::new();
        object.insert("test".to_owned(), JsonValue::Boolean(true));
        object.insert("number".to_owned(), JsonValue::Number(1_f64));
        object.insert("missing".to_owned(), JsonValue::Null);
        assert_eq!(
            JsonValue::Object(object).stringify(),
            "{\"missing\":null,\"test\":true,\"number\":1}"
        );
    }
}
