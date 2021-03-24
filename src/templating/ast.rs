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

#[derive(Debug, PartialEq)]
pub enum MustacheLikeNode {
    Text(String),
    Variable(String, bool),
    Section(String, Vec<MustacheLikeNode>),
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
                            return MustacheLikeNode::render_section(nodes, context, partials);
                        }
                        JsonValue::Array(array) => {
                            let mut result = String::new();
                            for element in array {
                                result.push_str(&MustacheLikeNode::render_section(nodes, &element, partials));
                            }
                            return result;
                        },
                        JsonValue::String(value) => {
                            if value.is_empty() {
                                return String::default();
                            }
                            return MustacheLikeNode::render_section(nodes, context, partials);
                        },
                        JsonValue::Number(_) => {
                            return MustacheLikeNode::render_section(nodes, context, partials);
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
