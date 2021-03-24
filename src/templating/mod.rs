mod ast;
mod lexer;
mod parser;
use super::json::JsonValue;
pub use ast::MustacheLikeNode;
pub use lexer::{MustacheLikeLexer, MustacheLikeToken};
use parser::MustacheLikeParser;
use std::collections::HashMap;

pub fn render(input: String, context: &JsonValue) -> String {
    render_impl(input, context, &HashMap::new())
}

pub fn render_with_partials(input: String, context: &JsonValue, partials: &HashMap<String, String>) -> String {
    render_impl(input, context, partials)
}

fn render_impl(input: String, context: &JsonValue, partials: &HashMap<String, String>) -> String {
    let tokens = MustacheLikeLexer::new(input).run();
    assert_ne!(tokens.len(), 0, "No tokens were generated");
    let nodes = MustacheLikeParser::new(tokens).parse();
    assert_ne!(nodes.len(), 0, "No nodes were generated");
    MustacheLikeNode::render_section(&nodes, context, partials)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::json::JsonParser;
    use std::collections::HashMap;

    #[test]
    fn render_test() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            JsonValue::String("value test".to_owned()),
        );
        assert_eq!(
            render(
                "Input {{test}} more text".to_owned(),
                &JsonValue::Object(context)
            ),
            "Input value test more text"
        );
    }

    #[test]
    fn render_conditional() {
        let mut context = HashMap::new();
        context.insert("person".to_owned(), JsonValue::Boolean(true));

        assert_eq!(
            render(
                "Shown.\n{{#person}}\n  Never shown!\n{{/person}}".to_owned(),
                &JsonValue::Object(context)
            ),
            "Shown.\n\n  Never shown!\n"
        );
    }

    #[test]
    fn render_array() {
        let mut context = HashMap::new();
        let mut repo1 = HashMap::new();
        repo1.insert("name".to_owned(), JsonValue::String("resque".to_owned()));
        let mut repo2 = HashMap::new();
        repo2.insert("name".to_owned(), JsonValue::String("hub".to_owned()));
        let mut repo3 = HashMap::new();
        repo3.insert("name".to_owned(), JsonValue::String("rip".to_owned()));
        let repos = vec![
            JsonValue::Object(repo1),
            JsonValue::Object(repo2),
            JsonValue::Object(repo3),
        ];
        context.insert("repo".to_owned(), JsonValue::Array(repos));

        assert_eq!(
            render(
                "{{#repo}}\n  <b>{{name}}</b>\n{{/repo}}".to_owned(),
                &JsonValue::Object(context)
            ),
            "\n  <b>resque</b>\n\n  <b>hub</b>\n\n  <b>rip</b>\n"
        );
    }

    #[test]
    fn render_escaped_html() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            JsonValue::String("Colors <h1 class=\"test\" id='test'>Test & roll \\lol `lal`</h1>".to_owned()),
        );
        assert_eq!(
            render(
                "<div>{{test}}</div>".to_owned(),
                &JsonValue::Object(context)
            ),
            "<div>Colors &lt;h1 class=&quot;test&quot; id=&#39;test&#39;&gt;Test &amp; roll \\lol `lal`&lt;&#x2F;h1&gt;</div>"
        );
    }

    #[test]
    fn render_unescaped_html() {
        let mut context = HashMap::new();
        context.insert(
            "test".to_owned(),
            JsonValue::String("Colors <h1 class=\"test\" id='test'>Test & roll \\lol `lal`</h1>".to_owned()),
        );
        assert_eq!(
            render(
                "<div>{{&test}}</div>".to_owned(),
                &JsonValue::Object(context)
            ),
            "<div>Colors <h1 class=\"test\" id='test'>Test & roll \\lol `lal`</h1></div>"
        );
    }

    #[test]
    fn render_partials() {
        let context = JsonValue::Object(HashMap::new());
        let mut partials = HashMap::new();
        partials.insert("text".to_owned(), "from partial".to_owned());
        assert_eq!(render_with_partials("{{>text}}".to_owned(), &context, &partials), "from partial");
    }

    #[test]
    fn render_fail_lookup_partial() {
        let context = JsonValue::Object(HashMap::new());
        assert_eq!(render("{{>text}}".to_owned(), &context), "");
    }

    #[test]
    fn render_partial_context() {
        let mut context = HashMap::new();
        context.insert(
            "text".to_owned(),
            JsonValue::String("Content".to_owned()),
        );
        let mut partials = HashMap::new();
        partials.insert("partial".to_owned(), "*{{text}}*".to_owned());
        assert_eq!(render_with_partials("{{>partial}}".to_owned(), &JsonValue::Object(context), &partials), "*Content*");
    }

    #[test]
    fn render_recursive_partial() {
        let context = JsonParser::new("{ \"content\": \"X\", \"nodes\": [ { \"content\": \"Y\", \"nodes\": [] } ] }").parse();
        let mut partials = HashMap::new();
        partials.insert("node".to_owned(), "{{content}}<{{#nodes}}{{>node}}{{/nodes}}>".to_owned());
        assert_eq!(render_with_partials("{{>node}}".to_owned(), &context, &partials), "X<Y<>>");
    }
}
