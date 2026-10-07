#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Node {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}

impl Node {
    pub fn is_element(&self) -> bool {
        matches!(self, Node::Element { .. })
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Node::Text(_))
    }

    pub fn tag(&self) -> Option<&str> {
        match self {
            Node::Element { tag, .. } => Some(tag),
            Node::Text(_) => None,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        match self {
            Node::Element { attributes, .. } => {
                attributes.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
            }
            Node::Text(_) => None,
        }
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        if let Node::Element { attributes, .. } = self {
            if let Some(existing) = attributes.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(name)) {
                existing.1 = value.to_string();
            } else {
                attributes.push((name.to_string(), value.to_string()));
            }
        }
    }

    pub fn remove_attribute(&mut self, name: &str) {
        if let Node::Element { attributes, .. } = self {
            attributes.retain(|(k, _)| !k.eq_ignore_ascii_case(name));
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.get_attribute("id")
    }

    pub fn classes(&self) -> Vec<&str> {
        self.get_attribute("class")
            .map(|s| s.split_whitespace().collect())
            .unwrap_or_default()
    }

    pub fn has_class(&self, class: &str) -> bool {
        self.classes().contains(&class)
    }

    pub fn children(&self) -> &[Node] {
        match self {
            Node::Element { children, .. } => children,
            Node::Text(_) => &[],
        }
    }

    pub fn children_mut(&mut self) -> &mut Vec<Node> {
        match self {
            Node::Element { children, .. } => children,
            Node::Text(_) => panic!("Cannot get mutable children of a Text node"),
        }
    }

    pub fn text_content(&self) -> String {
        match self {
            Node::Text(t) => t.clone(),
            Node::Element { children, .. } => {
                let mut s = String::new();
                for child in children {
                    s.push_str(&child.text_content());
                }
                s
            }
        }
    }

    pub fn find_by_id(&self, target_id: &str) -> Option<&Node> {
        if let Node::Element { children, .. } = self {
            if self.id() == Some(target_id) {
                return Some(self);
            }
            for child in children {
                if let Some(found) = child.find_by_id(target_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_by_id_mut(&mut self, target_id: &str) -> Option<&mut Node> {
        if self.id() == Some(target_id) {
            return Some(self);
        }
        if let Node::Element { children, .. } = self {
            for child in children {
                if let Some(found) = child.find_by_id_mut(target_id) {
                    return Some(found);
                }
            }
        }
        None
    }

    pub fn find_all_by_tag(&self, target_tag: &str) -> Vec<&Node> {
        let mut results = Vec::new();
        self.collect_by_tag(target_tag, &mut results);
        results
    }

    fn collect_by_tag<'a>(&'a self, target_tag: &str, results: &mut Vec<&'a Node>) {
        if let Node::Element { tag, children, .. } = self {
            if tag.eq_ignore_ascii_case(target_tag) {
                results.push(self);
            }
            for child in children {
                child.collect_by_tag(target_tag, results);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_attributes_and_queries() {
        let mut node = Node::Element {
            tag: "div".to_string(),
            attributes: vec![
                ("id".to_string(), "main".to_string()),
                ("class".to_string(), "box active".to_string()),
            ],
            children: vec![
                Node::Element {
                    tag: "p".to_string(),
                    attributes: vec![("id".to_string(), "para1".to_string())],
                    children: vec![Node::Text("Hello world".to_string())],
                },
            ],
        };

        assert_eq!(node.tag(), Some("div"));
        assert_eq!(node.id(), Some("main"));
        assert!(node.has_class("box"));
        assert!(node.has_class("active"));
        assert!(!node.has_class("hidden"));
        assert_eq!(node.text_content(), "Hello world");

        let p_node = node.find_by_id("para1");
        assert!(p_node.is_some());
        assert_eq!(p_node.unwrap().tag(), Some("p"));

        node.set_attribute("data-test", "val");
        assert_eq!(node.get_attribute("data-test"), Some("val"));
        node.remove_attribute("data-test");
        assert_eq!(node.get_attribute("data-test"), None);

        let ps = node.find_all_by_tag("p");
        assert_eq!(ps.len(), 1);
    }
}
