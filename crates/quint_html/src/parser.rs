use crate::dom::Node;

pub fn parse(input: &str) -> Vec<Node> {
    let mut chars = input.chars().peekable();
    parse_nodes(&mut chars)
}

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(tag: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    VOID_ELEMENTS.contains(&lower.as_str())
}

fn parse_nodes(chars: &mut std::iter::Peekable<std::str::Chars>) -> Vec<Node> {
    let mut nodes = Vec::new();

    loop {
        skip_whitespace(chars);
        if chars.peek().is_none() {
            break;
        }
        if chars.peek() == Some(&'<') {
            // Clone the iterator to peek ahead
            let mut lookahead = chars.clone();
            lookahead.next(); // consume '<'
            if lookahead.peek() == Some(&'/') {
                break; // Closing tag — let the parent handle it
            }
            if lookahead.peek() == Some(&'!') {
                // Doctype or comment
                chars.next(); // consume '<'
                chars.next(); // consume '!'
                if chars.peek() == Some(&'-') {
                    let mut check = chars.clone();
                    check.next();
                    if check.peek() == Some(&'-') {
                        // Skip HTML comment <!-- ... -->
                        chars.next(); // consume '-'
                        chars.next(); // consume '-'
                        loop {
                            match chars.next() {
                                Some('-') if chars.peek() == Some(&'-') => {
                                    chars.next();
                                    if chars.peek() == Some(&'>') {
                                        chars.next();
                                        break;
                                    }
                                }
                                Some(_) => continue,
                                None => break,
                            }
                        }
                        continue;
                    }
                }
                // Skip <!DOCTYPE ...> or other declarations until '>'
                while let Some(c) = chars.next() {
                    if c == '>' {
                        break;
                    }
                }
                continue;
            }
            if let Some(node) = parse_element(chars) {
                nodes.push(node);
            }
        } else {
            nodes.push(parse_text(chars));
        }
    }
    nodes
}

fn parse_element(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<Node> {
    chars.next(); // Consume '<'

    let tag = parse_tag_name(chars);
    let attributes = parse_attributes(chars);

    skip_whitespace(chars);
    let self_closing = chars.peek() == Some(&'/') || is_void_element(&tag);
    if chars.peek() == Some(&'/') {
        chars.next();
    }

    if chars.peek() == Some(&'>') {
        chars.next();
    }

    if self_closing {
        return Some(Node::Element {
            tag,
            attributes,
            children: vec![],
        });
    }

    let children = parse_nodes(chars);

    if chars.peek() == Some(&'<') {
        chars.next();
        if chars.peek() == Some(&'/') {
            chars.next();
            let _closing_tag = parse_tag_name(chars);
            while chars.peek().is_some() && chars.peek() != Some(&'>') {
                chars.next();
            }
            chars.next(); // consume >
        }
    }

    Some(Node::Element {
        tag,
        attributes,
        children,
    })
}

fn parse_tag_name(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut name = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            name.push(c);
            chars.next();
        } else {
            break;
        }
    }
    name
}

fn parse_attributes(chars: &mut std::iter::Peekable<std::str::Chars>) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    loop {
        skip_whitespace(chars);
        match chars.peek() {
            Some(&'>') | Some(&'/') | None => break,
            _ => {}
        }
        let key = parse_tag_name(chars);
        if key.is_empty() {
            break;
        }
        skip_whitespace(chars);
        let value = if chars.peek() == Some(&'=') {
            chars.next();
            skip_whitespace(chars);
            parse_attribute_value(chars)
        } else {
            String::new()
        };
        attrs.push((key, value));
    }
    attrs
}

fn parse_attribute_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let quote = match chars.peek() {
        Some(&'"') | Some(&'\'') => {
            let q = *chars.peek().unwrap();
            chars.next();
            Some(q)
        }
        _ => None,
    };

    let mut value = String::new();
    while let Some(&c) = chars.peek() {
        if let Some(q) = quote {
            if c == q {
                chars.next();
                break;
            }
        } else if c == '>' || c == ' ' {
            break;
        }
        value.push(c);
        chars.next();
    }
    value
}

fn parse_text(chars: &mut std::iter::Peekable<std::str::Chars>) -> Node {
    let mut text = String::new();
    while let Some(&c) = chars.peek() {
        if c == '<' {
            break;
        }
        text.push(c);
        chars.next();
    }
    Node::Text(text)
}

fn skip_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_element() {
        let nodes = parse("<p>hello</p>");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "p");
                assert_eq!(children.len(), 1);
                assert!(matches!(&children[0], Node::Text(t) if t == "hello"));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parse_nested_elements() {
        let nodes = parse("<div><p>text</p></div>");
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn parse_self_closing() {
        let nodes = parse("<br/>");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "br");
                assert!(children.is_empty());
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parse_attributes() {
        let nodes = parse(r#"<a href="https://example.com" class="link">click</a>"#);
        match &nodes[0] {
            Node::Element { attributes, .. } => {
                assert_eq!(attributes[0], ("href".into(), "https://example.com".into()));
                assert_eq!(attributes[1], ("class".into(), "link".into()));
            }
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn parse_empty_input() {
        let nodes = parse("");
        assert!(nodes.is_empty());
    }

    #[test]
    fn parse_text_only() {
        let nodes = parse("just text");
        assert_eq!(nodes.len(), 1);
        assert!(matches!(&nodes[0], Node::Text(t) if t == "just text"));
    }

    #[test]
    fn parse_doctype_and_void_elements() {
        let nodes = parse("<!doctype html><link rel=\"icon\" href=\"data:,\"><p>hi</p>");
        assert_eq!(nodes.len(), 2);
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "link");
                assert!(children.is_empty());
            }
            _ => panic!("expected link element"),
        }
        match &nodes[1] {
            Node::Element { tag, .. } => {
                assert_eq!(tag, "p");
            }
            _ => panic!("expected p element"),
        }
    }

    #[test]
    fn parse_comments() {
        let nodes = parse("<!-- comment --><h1>title</h1>");
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { tag, .. } => assert_eq!(tag, "h1"),
            _ => panic!("expected h1 element"),
        }
    }
}
