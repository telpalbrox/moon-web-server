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
    MustacheLikeNode::render_section(&nodes, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn render_test() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            MustacheLikeValue::String("value test".to_owned()),
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
        context.insert("person".to_owned(), MustacheLikeValue::Boolean(true));

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
            MustacheLikeValue::String("resque".to_owned()),
        );
        let mut repo2 = HashMap::new();
        repo2.insert(
            "name".to_owned(),
            MustacheLikeValue::String("hub".to_owned()),
        );
        let mut repo3 = HashMap::new();
        repo3.insert(
            "name".to_owned(),
            MustacheLikeValue::String("rip".to_owned()),
        );
        let repos = vec![
            MustacheLikeValue::Map(repo1),
            MustacheLikeValue::Map(repo2),
            MustacheLikeValue::Map(repo3),
        ];
        context.insert("repo".to_owned(), MustacheLikeValue::Array(repos));

        assert_eq!(
            render(
                "{{#repo}}\n  <b>{{name}}</b>\n{{/repo}}".to_owned(),
                &MustacheLikeValue::Map(context)
            ),
            "\n  <b>resque</b>\n\n  <b>hub</b>\n\n  <b>rip</b>\n"
        );
    }
}
