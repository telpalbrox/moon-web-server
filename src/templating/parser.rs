use super::{MustacheLikeNode, MustacheLikeToken};

pub struct MustacheLikeParser {
    tokens: Vec<MustacheLikeToken>,
    index: usize,
}

impl MustacheLikeParser {
    pub fn new(tokens: Vec<MustacheLikeToken>) -> Self {
        Self { tokens, index: 0 }
    }

    fn has_finished(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn consume(&mut self) {
        self.index = self.index + 1;
    }

    fn consume_until(&mut self, token: &MustacheLikeToken) -> Vec<MustacheLikeToken> {
        let mut tokens = Vec::new();
        loop {
            let peeked_token = match self.tokens.get(self.index) {
                None => break,
                Some(token) => token,
            };
            if peeked_token == token {
                break;
            }
            tokens.push(peeked_token.to_owned());
            self.consume();
        }
        tokens
    }

    fn consume_specific(&mut self, token: &MustacheLikeToken) {
        let peeked_token = match self.tokens.get(self.index) {
            None => panic!(
                "Expected token {:?} at index {}, but list of tokens has size {}",
                token,
                self.index,
                self.tokens.len()
            ),
            Some(token) => token,
        };
        if peeked_token != token {
            panic!("Expected token {:?} but found {:?}", token, peeked_token);
        }
        self.consume();
    }

    pub fn parse(&mut self) -> Vec<Box<MustacheLikeNode>> {
        let mut nodes: Vec<Box<MustacheLikeNode>> = Vec::new();

        loop {
            if self.has_finished() {
                break;
            }

            let token = match self.tokens.get(self.index) {
                Some(token) => token,
                None => break,
            };

            match token {
                MustacheLikeToken::Text(text) => {
                    nodes.push(Box::new(MustacheLikeNode::Text(text.to_owned())));
                    self.consume();
                }
                MustacheLikeToken::Name(name) => {
                    nodes.push(Box::new(MustacheLikeNode::Variable(name.to_owned())));
                    self.consume();
                }
                MustacheLikeToken::OpenTag(tag_name) => {
                    let close_tag_token = MustacheLikeToken::CloseTag(tag_name.to_owned());
                    let tag_name = tag_name.clone();
                    self.consume();
                    let section_tokens = self.consume_until(&close_tag_token);
                    self.consume_specific(&close_tag_token);
                    let section_nodes = MustacheLikeParser::new(section_tokens).parse();
                    nodes.push(Box::new(MustacheLikeNode::Section(
                        tag_name.to_owned(),
                        section_nodes,
                    )));
                }
                MustacheLikeToken::CloseTag(tag_name) => {
                    panic!("Not expected close tag {:?}", tag_name);
                }
            }
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
