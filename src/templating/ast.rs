use super::super::json::JsonValue;
use super::render_with_partials;
use std::collections::HashMap;

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

fn is_value_truthy(value: Option<&JsonValue>) -> bool {
    let value = match value {
        None => return false,
        Some(value) => value
    };

    match value {
        JsonValue::Boolean(value) => *value,
        JsonValue::Array(value) => !value.is_empty(),
        JsonValue::String(value) => !value.is_empty(),
        JsonValue::Null => false,
        JsonValue::Object(value) => !value.is_empty(),
        JsonValue::Number(value) => *value != 0f64
    }
}

#[derive(Debug, PartialEq)]
pub enum MustacheLikeNode {
    Text(String),
    Variable(String, bool),
    Section(String, Vec<MustacheLikeNode>, bool),
    Partial(String),
}

impl MustacheLikeNode {
    pub fn render_section(nodes: &Vec<MustacheLikeNode>, context: &JsonValue, partials: &HashMap<String, String>) -> String {
        let mut result = String::new();

        for node in nodes {
            result.push_str(&node.render(context, partials));
        }

        result
    }

    pub fn render(&self, context: &JsonValue, partials: &HashMap<String, String>) -> String {
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
                                    return escape_html(value);
                                } else {
                                    return String::from(value);
                                }
                            },
                            JsonValue::Boolean(value) => return value.to_string(),
                            JsonValue::Number(value) => return value.to_string(),
                            _ => return String::from(""),
                        }
                    }
                    _ => todo!("Handle name {:?} for {:?} value", name, context),
                };
            }
            Self::Section(tag_name, nodes, inverted) => match context {
                JsonValue::Object(map) => {
                    let value = map.get(tag_name);
                    let truthy = is_value_truthy(value);
                    let should_render = truthy != *inverted;
                    if !should_render {
                        return String::default();
                    }

                    let render = || {
                        return MustacheLikeNode::render_section(nodes, context, partials);
                    };

                    let value = match value {
                        None => return render(),
                        Some(value) => value
                    };

                    match value {
                        JsonValue::Boolean(_) => {
                            return render();
                        }
                        JsonValue::Array(array) => {
                            if array.is_empty() {
                                return render();
                            }
                            let mut result = String::new();
                            for element in array {
                                result.push_str(&MustacheLikeNode::render_section(nodes, &element, partials));
                            }
                            return result;
                        },
                        JsonValue::String(_) => {
                            return render();
                        },
                        JsonValue::Number(_) => {
                            return render();
                        },
                        _ => todo!("Handle map section for {:?} value", value),
                    }
                },
                _ => todo!("Handle section for {:?} value", context),
            },
            Self::Partial(name) => {
                let partial_src = partials.get(name);
                match partial_src {
                    None => return String::from(""),
                    Some(partial_src) => {
                        return render_with_partials(partial_src.to_owned(), context, partials);
                    }
                }
            }
        };
    }
}
