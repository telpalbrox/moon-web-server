use super::super::json::JsonValue;

fn html_entity_map() -> Vec<(char, &'static str)> {
    vec![
        ('&', "&amp;"),
        ('<', "&lt;"),
        ('>', "&gt;"),
        ('"', "&quot;"),
        ('\'', "&#39;"),
        ('/', "&#x2F;"),
    ]
}

fn escape_html(string: &str) -> String {
    let mut result = String::from(string);
    for (ch_to_replace, str_to_replace_with) in html_entity_map() {
        result = result.replace(ch_to_replace, str_to_replace_with);
    }
    result
}

#[derive(Debug, PartialEq)]
pub enum MustacheLikeNode {
    Text(String),
    Variable(String, bool),
    Section(String, Vec<MustacheLikeNode>),
}

impl MustacheLikeNode {
    pub fn render_section(nodes: &Vec<MustacheLikeNode>, context: &JsonValue) -> String {
        let mut result = String::new();

        for node in nodes {
            result.push_str(&node.render(context));
        }

        result
    }

    pub fn render(&self, context: &JsonValue) -> String {
        match self {
            Self::Text(text) => {
                return String::from(text);
            }
            Self::Variable(name, escape) => {
                match context {
                    JsonValue::Object(map) => {
                        let value = match map.get(name) {
                            None => return String::from(""),
                            Some(value) => value,
                        };
                        match value {
                            JsonValue::String(value) => {
                                if *escape {
                                    return escape_html(value)
                                } else {
                                    return String::from(value)
                                }
                            },
                            JsonValue::Boolean(value) => return value.to_string(),
                            JsonValue::Number(value) => return value.to_string(),
                            _ => return String::from(""),
                        }
                    }
                    _ => todo!("Handle name for {:?} value", context),
                };
            }
            Self::Section(tag_name, nodes) => match context {
                JsonValue::Object(map) => {
                    let value = match map.get(tag_name) {
                        None => return String::from(""),
                        Some(value) => value,
                    };
                    match value {
                        JsonValue::Boolean(value) => {
                            if !value {
                                return String::from("");
                            }
                            return MustacheLikeNode::render_section(nodes, context);
                        }
                        JsonValue::Array(array) => {
                            let mut result = String::new();
                            for element in array {
                                result.push_str(&MustacheLikeNode::render_section(nodes, &element));
                            }
                            return result;
                        }
                        _ => todo!("Handle map section for {:?} value", value),
                    }
                }
                _ => todo!("Handle section for {:?} value", context),
            },
        };
    }
}
