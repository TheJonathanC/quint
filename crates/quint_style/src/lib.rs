mod matcher;
mod styled_node;

pub use matcher::{matches, matches_with_ancestors};
pub use styled_node::{PropertyMap, StyledNode};

use quint_css::Stylesheet;
use quint_html::Node;

pub fn style_tree<'a>(
    nodes: &'a [Node],
    stylesheet: &Stylesheet,
    parent_properties: &PropertyMap,
) -> Vec<StyledNode<'a>> {
    style_tree_with_ancestors(nodes, stylesheet, parent_properties, &mut Vec::new())
}

fn style_tree_with_ancestors<'a>(
    nodes: &'a [Node],
    stylesheet: &Stylesheet,
    parent_properties: &PropertyMap,
    ancestors: &mut Vec<&'a Node>,
) -> Vec<StyledNode<'a>> {
    nodes
        .iter()
        .map(|node| {
            let mut properties = match node {
                Node::Element { .. } => matcher::cascade(node, stylesheet, ancestors),
                Node::Text(_) => PropertyMap::new(),
            };

            matcher::inherit(&mut properties, parent_properties);

            let children = match node {
                Node::Element { children, .. } => {
                    ancestors.push(node);
                    let res = style_tree_with_ancestors(children, stylesheet, &properties, ancestors);
                    ancestors.pop();
                    res
                }
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

pub fn default_ua_css() -> &'static str {
    r#"
head, title, style, script, link, meta, noscript { display: none; }
body { margin: 8px; font-size: 16px; color: black; background-color: white; }
h1 { font-size: 32px; font-weight: bold; margin: 21px 0; }
h2 { font-size: 24px; font-weight: bold; margin: 19px 0; }
h3 { font-size: 19px; font-weight: bold; margin: 16px 0; }
h4 { font-size: 16px; font-weight: bold; margin: 21px 0; }
h5 { font-size: 13px; font-weight: bold; margin: 22px 0; }
h6 { font-size: 11px; font-weight: bold; margin: 25px 0; }
p { margin: 16px 0; }
a { color: blue; }
b, strong { font-weight: bold; }
i, em { font-style: italic; }
button { padding: 4px 8px; border: 1px solid black; background: #e0e0e0; }
"#
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
        assert!(!styled[0].children[0].properties.contains_key("margin"));
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

    #[test]
    fn descendant_selector_matches() {
        let css = parse_css("div p { color: purple; }");
        let dom = vec![elem("div", vec![], vec![elem("section", vec![], vec![elem("p", vec![], vec![])])])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(
            styled[0].children[0].children[0].properties.get("color").unwrap(),
            "purple"
        );
    }

    #[test]
    fn child_selector_matches() {
        let css = parse_css("div > p { color: orange; }");
        let dom = vec![
            elem("div", vec![], vec![elem("p", vec![], vec![])]),
            elem("section", vec![], vec![elem("div", vec![], vec![elem("span", vec![], vec![elem("p", vec![], vec![])])])]),
        ];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        // Direct child matches
        assert_eq!(styled[0].children[0].properties.get("color").unwrap(), "orange");
        // Grandchild does not match child selector
        assert_ne!(
            styled[1].children[0].children[0].children[0].properties.get("color"),
            Some(&"orange".to_string())
        );
    }

    #[test]
    fn inline_style_overrides_css() {
        let css = parse_css("p { color: red; }");
        let dom = vec![elem("p", vec![("style", "color: blue; margin: 20px;")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("color").unwrap(), "blue");
        assert_eq!(styled[0].properties.get("margin-top").unwrap(), "20px");
        assert_eq!(styled[0].properties.get("margin-left").unwrap(), "20px");
    }

    #[test]
    fn shorthand_margin_and_padding_expanded() {
        let css = parse_css(".box { margin: 10px 20px; padding: 5px; }");
        let dom = vec![elem("div", vec![("class", "box")], vec![])];
        let styled = style_tree(&dom, &css, &PropertyMap::new());
        assert_eq!(styled[0].properties.get("margin-top").unwrap(), "10px");
        assert_eq!(styled[0].properties.get("margin-bottom").unwrap(), "10px");
        assert_eq!(styled[0].properties.get("margin-right").unwrap(), "20px");
        assert_eq!(styled[0].properties.get("margin-left").unwrap(), "20px");
        assert_eq!(styled[0].properties.get("padding-top").unwrap(), "5px");
        assert_eq!(styled[0].properties.get("padding-right").unwrap(), "5px");
    }
}
