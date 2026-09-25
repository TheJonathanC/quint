#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}
