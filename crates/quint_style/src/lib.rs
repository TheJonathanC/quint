mod matcher;
mod styled_node;

pub use styled_node::{PropertyMap, StyledNode};

use quint_css::Stylesheet;
use quint_html::Node;

pub fn style_tree<'a>(
    nodes: &'a [Node],
    stylesheet: &Stylesheet,
    parent_properties: &PropertyMap,
) -> Vec<StyledNode<'a>> {
    nodes
        .iter()
        .map(|node| {
            let mut properties = match node {
                Node::Element { .. } => matcher::cascade(node, stylesheet),
                Node::Text(_) => PropertyMap::new(),
            };

            matcher::inherit(&mut properties, parent_properties);

            let children = match node {
                Node::Element { children, .. } => style_tree(children, stylesheet, &properties),
                Node::Text(_) => vec![],
            };

            StyledNode {
                node,
                properties,
                children,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quint_css::parse as parse_css;
    use quint_html::Node;

    fn elem(tag: &str, attrs: Vec<(&str, &str)>, children: Vec<Node>) -> Node {
        Node::Element {
            tag: tag.to_string(),
            attributes: attrs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            children,
        }
    }

    fn text(s: &str) -> Node {
        Node::Text(s.to_string())
    }

    #[test]
    fn tag_selector_matches() {
        let css = parse_css("h1 { color: red; }");
        let dom = vec![elem("h1", vec![], vec![text("hello")])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("color").unwrap(), "red");
    }

    #[test]
    fn class_selector_matches() {
        let css = parse_css(".highlight { background: yellow; }");
        let dom = vec![elem("p", vec![("class", "highlight")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("background").unwrap(), "yellow");
    }

    #[test]
    fn id_selector_matches() {
        let css = parse_css("#main { width: 960px; }");
        let dom = vec![elem("div", vec![("id", "main")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("width").unwrap(), "960px");
    }

    #[test]
    fn no_match_returns_empty_properties() {
        let css = parse_css("h1 { color: red; }");
        let dom = vec![elem("p", vec![], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert!(styled[0].properties.is_empty());
    }

    #[test]
    fn specificity_wins() {
        let css = parse_css("p { color: blue; } .special { color: red; }");
        let dom = vec![elem("p", vec![("class", "special")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("color").unwrap(), "red");
    }

    #[test]
    fn source_order_breaks_ties() {
        let css = parse_css("p { color: red; } p { color: blue; }");
        let dom = vec![elem("p", vec![], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("color").unwrap(), "blue");
    }

    #[test]
    fn inheritance_flows_down() {
        let css = parse_css("div { color: green; }");
        let dom = vec![elem("div", vec![], vec![elem("p", vec![], vec![])])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(
            styled[0].children[0].properties.get("color").unwrap(),
            "green"
        );
    }

    #[test]
    fn child_overrides_inherited() {
        let css = parse_css("div { color: green; } p { color: red; }");
        let dom = vec![elem("div", vec![], vec![elem("p", vec![], vec![])])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(
            styled[0].children[0].properties.get("color").unwrap(),
            "red"
        );
    }

    #[test]
    fn non_inherited_property_does_not_flow() {
        let css = parse_css("div { margin: 10px; }");
        let dom = vec![elem("div", vec![], vec![elem("p", vec![], vec![])])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert!(styled[0].children[0].properties.get("margin").is_none());
    }

    #[test]
    fn text_node_inherits() {
        let css = parse_css("p { color: blue; }");
        let dom = vec![elem("p", vec![], vec![text("hello")])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(
            styled[0].children[0].properties.get("color").unwrap(),
            "blue"
        );
    }

    #[test]
    fn universal_selector_matches_all() {
        let css = parse_css("* { margin: 0; }");
        let dom = vec![elem("div", vec![], vec![elem("p", vec![], vec![])])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("margin").unwrap(), "0");
        assert_eq!(styled[0].children[0].properties.get("margin").unwrap(), "0");
    }

    #[test]
    fn multiple_classes_on_element() {
        let css = parse_css(".a { color: red; } .b { font-size: 14px; }");
        let dom = vec![elem("p", vec![("class", "a b")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("color").unwrap(), "red");
        assert_eq!(styled[0].properties.get("font-size").unwrap(), "14px");
    }
}
