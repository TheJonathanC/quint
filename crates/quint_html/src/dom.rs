#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}
