use crate::styled_node::PropertyMap;
use quint_css::{Combinator, Declaration, Selector, Stylesheet};
use quint_html::Node;
use std::collections::HashMap;

fn matches_simple(node: &Node, selector: &Selector) -> bool {
    match node {
        Node::Text(_) => false,
        Node::Element {
            tag, attributes, ..
        } => {
            if let Some(sel_tag) = &selector.tag
                && !sel_tag.eq_ignore_ascii_case(tag)
            {
                return false;
            }

            if let Some(sel_id) = &selector.id {
                let elem_id = attributes
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("id"))
                    .map(|(_, v)| v.as_str());
                if elem_id != Some(sel_id.as_str()) {
                    return false;
                }
            }

            if !selector.classes.is_empty() {
                let elem_classes: Vec<&str> = attributes
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("class"))
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

pub fn matches(node: &Node, selector: &Selector) -> bool {
    matches_with_ancestors(node, selector, &[])
}

pub fn matches_with_ancestors(node: &Node, selector: &Selector, ancestors: &[&Node]) -> bool {
    if !matches_simple(node, selector) {
        return false;
    }

    if let Some((combinator, prev_selector)) = &selector.relation {
        match combinator {
            Combinator::Child => {
                if let Some(parent) = ancestors.last() {
                    let parent_ancestors = if ancestors.len() > 1 {
                        &ancestors[..ancestors.len() - 1]
                    } else {
                        &[]
                    };
                    matches_with_ancestors(parent, prev_selector, parent_ancestors)
                } else {
                    false
                }
            }
            Combinator::Descendant => {
                for (i, ancestor) in ancestors.iter().enumerate().rev() {
                    let anc_ancestors = &ancestors[..i];
                    if matches_with_ancestors(ancestor, prev_selector, anc_ancestors) {
                        return true;
                    }
                }
                false
            }
        }
    } else {
        true
    }
}

struct MatchedDeclaration {
    specificity: (usize, usize, usize),
    declaration: Declaration,
    source_order: usize,
}

pub fn cascade(node: &Node, stylesheet: &Stylesheet, ancestors: &[&Node]) -> HashMap<String, String> {
    let mut matched: Vec<MatchedDeclaration> = Vec::new();

    for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
        let best_match = rule
            .selectors
            .iter()
            .filter(|sel| matches_with_ancestors(node, sel, ancestors))
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

    // Inline style="..." attribute support
    if let Node::Element { attributes, .. } = node
        && let Some((_, style_val)) = attributes.iter().find(|(k, _)| k.eq_ignore_ascii_case("style"))
    {
        let inline_decls = quint_css::parse_declarations_from_str(style_val);
        for decl in inline_decls {
            properties.insert(decl.property, decl.value);
        }
    }

    expand_shorthands(&mut properties);

    properties
}

fn expand_shorthands(properties: &mut PropertyMap) {
    // 1. margin shorthand
    if let Some(margin) = properties.get("margin").cloned() {
        let parts: Vec<&str> = margin.split_whitespace().collect();
        let (top, right, bottom, left) = match parts.len() {
            1 => (parts[0], parts[0], parts[0], parts[0]),
            2 => (parts[0], parts[1], parts[0], parts[1]),
            3 => (parts[0], parts[1], parts[2], parts[1]),
            4 => (parts[0], parts[1], parts[2], parts[3]),
            _ => ("", "", "", ""),
        };
        if !top.is_empty() {
            properties.entry("margin-top".to_string()).or_insert_with(|| top.to_string());
            properties.entry("margin-right".to_string()).or_insert_with(|| right.to_string());
            properties.entry("margin-bottom".to_string()).or_insert_with(|| bottom.to_string());
            properties.entry("margin-left".to_string()).or_insert_with(|| left.to_string());
        }
    }

    // 2. padding shorthand
    if let Some(padding) = properties.get("padding").cloned() {
        let parts: Vec<&str> = padding.split_whitespace().collect();
        let (top, right, bottom, left) = match parts.len() {
            1 => (parts[0], parts[0], parts[0], parts[0]),
            2 => (parts[0], parts[1], parts[0], parts[1]),
            3 => (parts[0], parts[1], parts[2], parts[1]),
            4 => (parts[0], parts[1], parts[2], parts[3]),
            _ => ("", "", "", ""),
        };
        if !top.is_empty() {
            properties.entry("padding-top".to_string()).or_insert_with(|| top.to_string());
            properties.entry("padding-right".to_string()).or_insert_with(|| right.to_string());
            properties.entry("padding-bottom".to_string()).or_insert_with(|| bottom.to_string());
            properties.entry("padding-left".to_string()).or_insert_with(|| left.to_string());
        }
    }

    // 3. border shorthand (e.g. "1px solid red", "2px black")
    if let Some(border) = properties.get("border").cloned() {
        let parts: Vec<&str> = border.split_whitespace().collect();
        for part in &parts {
            if part.ends_with("px")
                || part.ends_with("pt")
                || part.ends_with("em")
                || part.ends_with("rem")
                || part.chars().all(|c| c.is_ascii_digit())
            {
                properties.entry("border-top-width".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-right-width".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-bottom-width".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-left-width".to_string()).or_insert_with(|| part.to_string());
            } else if ["solid", "dashed", "dotted", "double", "none", "hidden"].contains(part) {
                properties.entry("border-style".to_string()).or_insert_with(|| part.to_string());
            } else {
                properties.entry("border-top-color".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-right-color".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-bottom-color".to_string()).or_insert_with(|| part.to_string());
                properties.entry("border-left-color".to_string()).or_insert_with(|| part.to_string());
            }
        }
    }

    // 4. border-width shorthand
    if let Some(bw) = properties.get("border-width").cloned() {
        let parts: Vec<&str> = bw.split_whitespace().collect();
        let (top, right, bottom, left) = match parts.len() {
            1 => (parts[0], parts[0], parts[0], parts[0]),
            2 => (parts[0], parts[1], parts[0], parts[1]),
            3 => (parts[0], parts[1], parts[2], parts[1]),
            4 => (parts[0], parts[1], parts[2], parts[3]),
            _ => ("", "", "", ""),
        };
        if !top.is_empty() {
            properties.entry("border-top-width".to_string()).or_insert_with(|| top.to_string());
            properties.entry("border-right-width".to_string()).or_insert_with(|| right.to_string());
            properties.entry("border-bottom-width".to_string()).or_insert_with(|| bottom.to_string());
            properties.entry("border-left-width".to_string()).or_insert_with(|| left.to_string());
        }
    }

    // 5. border-color shorthand
    if let Some(bc) = properties.get("border-color").cloned() {
        let parts: Vec<&str> = bc.split_whitespace().collect();
        let (top, right, bottom, left) = match parts.len() {
            1 => (parts[0], parts[0], parts[0], parts[0]),
            2 => (parts[0], parts[1], parts[0], parts[1]),
            3 => (parts[0], parts[1], parts[2], parts[1]),
            4 => (parts[0], parts[1], parts[2], parts[3]),
            _ => ("", "", "", ""),
        };
        if !top.is_empty() {
            properties.entry("border-top-color".to_string()).or_insert_with(|| top.to_string());
            properties.entry("border-right-color".to_string()).or_insert_with(|| right.to_string());
            properties.entry("border-bottom-color".to_string()).or_insert_with(|| bottom.to_string());
            properties.entry("border-left-color".to_string()).or_insert_with(|| left.to_string());
        }
    }

    // 6. background shorthand
    if let Some(bg) = properties.get("background").cloned()
        && !properties.contains_key("background-color")
    {
        properties.insert("background-color".to_string(), bg);
    }

    // 7. gap shorthand
    if let Some(gap) = properties.get("gap").cloned() {
        let parts: Vec<&str> = gap.split_whitespace().collect();
        if parts.len() == 1 {
            properties.entry("row-gap".to_string()).or_insert_with(|| parts[0].to_string());
            properties.entry("column-gap".to_string()).or_insert_with(|| parts[0].to_string());
        } else if parts.len() >= 2 {
            properties.entry("row-gap".to_string()).or_insert_with(|| parts[0].to_string());
            properties.entry("column-gap".to_string()).or_insert_with(|| parts[1].to_string());
        }
    }

    // 8. flex shorthand
    if let Some(flex) = properties.get("flex").cloned() {
        let parts: Vec<&str> = flex.split_whitespace().collect();
        if parts.len() == 1 {
            if parts[0] == "none" {
                properties.entry("flex-grow".to_string()).or_insert_with(|| "0".to_string());
                properties.entry("flex-shrink".to_string()).or_insert_with(|| "0".to_string());
                properties.entry("flex-basis".to_string()).or_insert_with(|| "auto".to_string());
            } else if parts[0] == "auto" {
                properties.entry("flex-grow".to_string()).or_insert_with(|| "1".to_string());
                properties.entry("flex-shrink".to_string()).or_insert_with(|| "1".to_string());
                properties.entry("flex-basis".to_string()).or_insert_with(|| "auto".to_string());
            } else {
                properties.entry("flex-grow".to_string()).or_insert_with(|| parts[0].to_string());
                properties.entry("flex-shrink".to_string()).or_insert_with(|| "1".to_string());
                properties.entry("flex-basis".to_string()).or_insert_with(|| "0%".to_string());
            }
        }
    }
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
            && let Some(value) = parent_props.get(prop)
        {
            child_props.insert(prop.to_string(), value.clone());
        }
    }
}
