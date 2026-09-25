use crate::dom::Node;
use scraper::{Html, node::Node as ScraperNode};

pub fn parse(input: &str) -> Vec<Node> {
    let document = Html::parse_document(input);
    let mut root_nodes = Vec::new();

    for child_ref in document.tree.root().children() {
        if let Some(node) = convert_node(child_ref) {
            root_nodes.push(node);
        }
    }

    root_nodes
}

fn convert_node(node_ref: ego_tree::NodeRef<scraper::node::Node>) -> Option<Node> {
    let node_data = node_ref.value();

    match node_data {
        ScraperNode::Element(el) => {
            let tag = el.name().to_string();
            let attributes = el
                .attrs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            
            let mut children = Vec::new();
            for child_ref in node_ref.children() {
                if let Some(child_node) = convert_node(child_ref) {
                    children.push(child_node);
                }
            }
            
            Some(Node::Element { tag, attributes, children })
        }
        ScraperNode::Text(text) => {
            let t = text.text.to_string();
            if t.trim().is_empty() {
                None // Skip pure whitespace text nodes to simplify layout
            } else {
                Some(Node::Text(t))
            }
        }
        _ => None, // Skip comments, doctypes, etc.
    }
}
