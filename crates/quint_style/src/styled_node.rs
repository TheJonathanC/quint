use quint_html::Node;
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, String>;

#[derive(Debug, Clone, PartialEq)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub properties: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}
