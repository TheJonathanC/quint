mod parser;
mod types;

pub use parser::parse;
pub use types::{Declaration, Rule, Selector, Stylesheet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let ss = parse("");
        assert!(ss.rules.is_empty());
    }

    #[test]
    fn parse_single_rule() {
        let ss = parse("h1 { color: red; }");
        assert_eq!(ss.rules.len(), 1);
        assert_eq!(ss.rules[0].selectors.len(), 1);
        assert_eq!(ss.rules[0].selectors[0].tag, Some("h1".to_string()));
        assert_eq!(ss.rules[0].declarations.len(), 1);
        assert_eq!(ss.rules[0].declarations[0].property, "color");
        assert_eq!(ss.rules[0].declarations[0].value, "red");
    }

    #[test]
    fn parse_multiple_declarations() {
        let ss = parse("p { color: blue; font-size: 14px; margin: 10px; }");
        assert_eq!(ss.rules[0].declarations.len(), 3);
    }

    #[test]
    fn parse_multiple_selectors() {
        let ss = parse("h1, h2, h3 { font-weight: bold; }");
        assert_eq!(ss.rules[0].selectors.len(), 3);
    }

    #[test]
    fn parse_class_selector() {
        let ss = parse(".intro { color: green; }");
        assert_eq!(ss.rules[0].selectors[0].classes, vec!["intro"]);
        assert_eq!(ss.rules[0].selectors[0].tag, None);
    }

    #[test]
    fn parse_id_selector() {
        let ss = parse("#main { width: 100%; }");
        assert_eq!(ss.rules[0].selectors[0].id, Some("main".to_string()));
    }

    #[test]
    fn parse_combined_selector() {
        let ss = parse("div.highlight#hero { background: yellow; }");
        let sel = &ss.rules[0].selectors[0];
        assert_eq!(sel.tag, Some("div".to_string()));
        assert_eq!(sel.id, Some("hero".to_string()));
        assert_eq!(sel.classes, vec!["highlight"]);
    }

    #[test]
    fn parse_universal_selector() {
        let ss = parse("* { margin: 0; }");
        let sel = &ss.rules[0].selectors[0];
        assert_eq!(sel.tag, None);
        assert_eq!(sel.id, None);
        assert!(sel.classes.is_empty());
    }

    #[test]
    fn parse_with_comments() {
        let ss = parse("/* header styles */ h1 { color: red; /* TODO: change */ }");
        assert_eq!(ss.rules.len(), 1);
        assert_eq!(ss.rules[0].declarations[0].value, "red");
    }

    #[test]
    fn parse_no_trailing_semicolon() {
        let ss = parse("p { color: blue }");
        assert_eq!(ss.rules[0].declarations.len(), 1);
        assert_eq!(ss.rules[0].declarations[0].value, "blue");
    }

    #[test]
    fn parse_multiple_rules() {
        let ss = parse("h1 { color: red; } p { color: blue; }");
        assert_eq!(ss.rules.len(), 2);
    }

    #[test]
    fn specificity_tag() {
        let s = Selector {
            tag: Some("h1".into()),
            id: None,
            classes: vec![],
        };
        assert_eq!(s.specificity(), (0, 0, 1));
    }

    #[test]
    fn specificity_class() {
        let s = Selector {
            tag: None,
            id: None,
            classes: vec!["x".into()],
        };
        assert_eq!(s.specificity(), (0, 1, 0));
    }

    #[test]
    fn specificity_id() {
        let s = Selector {
            tag: None,
            id: Some("m".into()),
            classes: vec![],
        };
        assert_eq!(s.specificity(), (1, 0, 0));
    }

    #[test]
    fn specificity_combined() {
        let s = Selector {
            tag: Some("div".into()),
            id: Some("x".into()),
            classes: vec!["a".into(), "b".into()],
        };
        assert_eq!(s.specificity(), (1, 2, 1));
    }

    #[test]
    fn specificity_ordering() {
        let id = (1usize, 0usize, 0usize);
        let cls = (0usize, 1usize, 0usize);
        let tag = (0usize, 0usize, 1usize);
        assert!(id > cls);
        assert!(cls > tag);
    }

    #[test]
    fn value_with_colons() {
        let ss = parse("div { background: url(http://example.com/img.png); }");
        assert_eq!(
            ss.rules[0].declarations[0].value,
            "url(http://example.com/img.png)"
        );
    }
}
