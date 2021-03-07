use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum MustacheLikeValue {
    String(String),
    Boolean(bool),
    Number(f64),
    Array(Vec<Box<MustacheLikeValue>>),
    Map(HashMap<String, Box<MustacheLikeValue>>),
}

#[derive(Debug, PartialEq)]
pub enum MustacheLikeNode {
    Text(String),
    Variable(String),
    Section(String, Vec<Box<MustacheLikeNode>>),
}

impl MustacheLikeNode {
    pub fn render_section(
        nodes: &Vec<Box<MustacheLikeNode>>,
        context: &MustacheLikeValue,
    ) -> String {
        let mut result = String::new();

        for node in nodes {
            result.push_str(&node.render(context));
        }

        result
    }

    pub fn render(&self, context: &MustacheLikeValue) -> String {
        match self {
            Self::Text(text) => {
                return String::from(text);
            }
            Self::Variable(name) => {
                match context {
                    MustacheLikeValue::Map(map) => {
                        let value = match map.get(name) {
                            None => return String::from(""),
                            Some(value) => value,
                        };
                        match &**value {
                            MustacheLikeValue::String(value) => return String::from(value),
                            MustacheLikeValue::Boolean(value) => return value.to_string(),
                            MustacheLikeValue::Number(value) => return value.to_string(),
                            _ => return String::from(""),
                        }
                    }
                    _ => todo!("Handle name for {:?} value", context),
                };
            }
            Self::Section(tag_name, nodes) => match context {
                MustacheLikeValue::Map(map) => {
                    let value = match map.get(tag_name) {
                        None => return String::from(""),
                        Some(value) => value,
                    };
                    match &**value {
                        MustacheLikeValue::Boolean(value) => {
                            if !value {
                                return String::from("");
                            }
                            return MustacheLikeNode::render_section(nodes, context);
                        }
                        MustacheLikeValue::Array(array) => {
                            let mut result = String::new();
                            for element in array {
                                result.push_str(&MustacheLikeNode::render_section(nodes, &element));
                            }
                            return result;
                        }
                        _ => todo!("Handle map section for {:?} value", &**value),
                    }
                }
                _ => todo!("Handle section for {:?} value", context),
            },
        };
    }
}
