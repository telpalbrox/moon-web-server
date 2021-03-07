mod ast;
mod lexer;
mod parser;
pub use ast::{MustacheLikeNode, MustacheLikeValue};
pub use lexer::{MustacheLikeLexer, MustacheLikeToken};
use parser::MustacheLikeParser;

pub fn render(input: String, context: &MustacheLikeValue) -> String {
    let tokens = MustacheLikeLexer::new(input).run();
    assert_ne!(tokens.len(), 0, "No tokens were generated");
    let nodes = MustacheLikeParser::new(tokens).parse();
    assert_ne!(nodes.len(), 0, "No nodes were generated");
    let mut result = String::new();
    for node in nodes {
        result.push_str(&node.render(&context));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn expect_nodes(nodes: Vec<Box<MustacheLikeNode>>, expected_nodes: Vec<MustacheLikeNode>) {
        for index in 0..nodes.len() {
            let node = &*nodes[index];
            assert_eq!(node, &expected_nodes[index])
        }
    }

    #[test]
    fn parser() {
        let tokens = vec![
            MustacheLikeToken::Text(String::from("Input ")),
            MustacheLikeToken::Name(String::from("test")),
            MustacheLikeToken::Text(String::from(" more text")),
        ];
        let mut parser = MustacheLikeParser::new(tokens);
        let nodes = parser.parse();
        let expected_nodes = vec![
            MustacheLikeNode::Text(String::from("Input ")),
            MustacheLikeNode::Variable(String::from("test")),
            MustacheLikeNode::Text(String::from(" more text")),
        ];
        expect_nodes(nodes, expected_nodes);
    }

    #[test]
    fn parser_tree() {
        let tokens = vec![
            MustacheLikeToken::Text(String::from("Shown.\n")),
            MustacheLikeToken::OpenTag(String::from("person")),
            MustacheLikeToken::Text(String::from("\n  Never shown!\n")),
            MustacheLikeToken::CloseTag(String::from("person")),
        ];
        let nodes = MustacheLikeParser::new(tokens).parse();
        expect_nodes(
            nodes,
            vec![
                MustacheLikeNode::Text(String::from("Shown.\n")),
                MustacheLikeNode::Section(
                    String::from("person"),
                    vec![Box::new(MustacheLikeNode::Text(String::from(
                        "\n  Never shown!\n",
                    )))],
                ),
            ],
        );
    }

    #[test]
    fn render_test() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            Box::new(MustacheLikeValue::String("value test".to_owned())),
        );
        assert_eq!(
            render(
                "Input {{test}} more text".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "Input value test more text"
        );
    }

    #[test]
    fn render_conditional() {
        let mut context = HashMap::new();
        context.insert(
            "person".to_owned(),
            Box::new(MustacheLikeValue::Boolean(true)),
        );

        assert_eq!(
            render(
                "Shown.\n{{#person}}\n  Never shown!\n{{/person}}".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "Shown.\n\n  Never shown!\n"
        );
    }

    #[test]
    fn render_array() {
        let mut context = HashMap::new();
        let mut repo1 = HashMap::new();
        repo1.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("resque".to_owned())),
        );
        let mut repo2 = HashMap::new();
        repo2.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("hub".to_owned())),
        );
        let mut repo3 = HashMap::new();
        repo3.insert(
            "name".to_owned(),
            Box::new(MustacheLikeValue::String("rip".to_owned())),
        );
        let repos = vec![
            Box::new(MustacheLikeValue::Map(repo1)),
            Box::new(MustacheLikeValue::Map(repo2)),
            Box::new(MustacheLikeValue::Map(repo3)),
        ];
        context.insert("repo".to_owned(), Box::new(MustacheLikeValue::Array(repos)));

        assert_eq!(
            render(
                "{{#repo}}\n  <b>{{name}}</b>\n{{/repo}}".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "\n  <b>resque</b>\n\n  <b>hub</b>\n\n  <b>rip</b>\n"
        );
    }
}
