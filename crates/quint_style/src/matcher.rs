use crate::styled_node::PropertyMap;
use quint_css::{Declaration, Selector, Stylesheet};
use quint_html::Node;
use std::collections::HashMap;

pub fn matches(node: &Node, selector: &Selector) -> bool {
    match node {
        Node::Text(_) => false,
        Node::Element {
            tag, attributes, ..
        } => {
            if let Some(sel_tag) = &selector.tag
                && sel_tag != tag {
                    return false;
                }

            if let Some(sel_id) = &selector.id {
                let elem_id = attributes
                    .iter()
                    .find(|(k, _)| k == "id")
                    .map(|(_, v)| v.as_str());
                if elem_id != Some(sel_id.as_str()) {
                    return false;
                }
            }

            if !selector.classes.is_empty() {
                let elem_classes: Vec<&str> = attributes
                    .iter()
                    .find(|(k, _)| k == "class")
                    .map(|(_, v)| v.split_whitespace().collect())
                    .unwrap_or_default();
                for cls in &selector.classes {
                    if !elem_classes.contains(&cls.as_str()) {
                        return false;
                    }
                }
            }

            true
        }
    }
}

struct MatchedDeclaration {
    specificity: (usize, usize, usize),
    declaration: Declaration,
    source_order: usize,
}

pub fn cascade(node: &Node, stylesheet: &Stylesheet) -> HashMap<String, String> {
    let mut matched: Vec<MatchedDeclaration> = Vec::new();

    for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
        let best_match = rule
            .selectors
            .iter()
            .filter(|sel| matches(node, sel))
            .map(|sel| sel.specificity())
            .max();

        if let Some(specificity) = best_match {
            for decl in &rule.declarations {
                matched.push(MatchedDeclaration {
                    specificity,
                    declaration: decl.clone(),
                    source_order: rule_index,
                });
            }
        }
    }

    matched.sort_by_key(|m| (m.specificity, m.source_order));

    let mut properties = HashMap::new();
    for m in matched {
        properties.insert(m.declaration.property, m.declaration.value);
    }
    properties
}

const INHERITED_PROPERTIES: &[&str] = &[
    "color",
    "font-family",
    "font-size",
    "font-style",
    "font-weight",
    "line-height",
    "text-align",
    "visibility",
    "white-space",
    "word-spacing",
    "letter-spacing",
    "text-indent",
    "text-transform",
];

pub fn inherit(child_props: &mut PropertyMap, parent_props: &PropertyMap) {
    for &prop in INHERITED_PROPERTIES {
        if !child_props.contains_key(prop)
            && let Some(value) = parent_props.get(prop) {
                child_props.insert(prop.to_string(), value.clone());
            }
    }
}
