use crate::types::{Declaration, Rule, Selector, Stylesheet};
use std::iter::Peekable;
use std::str::Chars;

pub fn parse(input: &str) -> Stylesheet {
    let mut chars = input.chars().peekable();
    let mut rules = Vec::new();

    loop {
        skip_whitespace_and_comments(&mut chars);
        if chars.peek().is_none() {
            break;
        }

        // skip at-rules
        if chars.peek() == Some(&'@') {
            skip_at_rule(&mut chars);
            continue;
        }

        if let Some(rule) = parse_rule(&mut chars) {
            rules.push(rule);
        }
    }

    Stylesheet { rules }
}

fn skip_whitespace_and_comments(chars: &mut Peekable<Chars>) {
    loop {
        let mut advanced = false;

        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
            advanced = true;
        }

        if chars.peek() == Some(&'/') {
            let mut lookahead = chars.clone();
            lookahead.next();
            if lookahead.peek() == Some(&'*') {
                chars.next(); // /
                chars.next(); // *
                advanced = true;

                loop {
                    match chars.next() {
                        Some('*') if chars.peek() == Some(&'/') => {
                            chars.next();
                            break;
                        }
                        Some(_) => continue,
                        None => break,
                    }
                }
            }
        }

        if !advanced {
            break;
        }
    }
}

fn skip_at_rule(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ';' || c == '{' {
            if c == '{' {
                chars.next();
                skip_block(chars);
            } else {
                chars.next();
            }
            break;
        }
        chars.next();
    }
}

fn skip_block(chars: &mut Peekable<Chars>) {
    let mut depth = 1;
    while let Some(&c) = chars.peek() {
        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                chars.next();
                break;
            }
        }
        chars.next();
    }
}

fn parse_rule(chars: &mut Peekable<Chars>) -> Option<Rule> {
    let selectors = parse_selectors(chars);
    if selectors.is_empty() {
        // Find next '{' or EOF and skip block to recover
        while let Some(&c) = chars.peek() {
            if c == '{' {
                chars.next();
                skip_block(chars);
                break;
            } else if c == '}' {
                break;
            }
            chars.next();
        }
        return None;
    }

    // Expect '{'
    if chars.peek() != Some(&'{') {
        return None; // Invalid syntax
    }
    chars.next();

    let declarations = parse_declarations(chars);

    // Expect '}'
    if chars.peek() == Some(&'}') {
        chars.next();
    }

    Some(Rule {
        selectors,
        declarations,
    })
}

fn parse_selectors(chars: &mut Peekable<Chars>) -> Vec<Selector> {
    let mut selectors = Vec::new();
    loop {
        skip_whitespace_and_comments(chars);
        match chars.peek() {
            Some(&'{') | None => break,
            _ => {}
        }

        if let Some(sel) = parse_selector(chars) {
            selectors.push(sel);
        }

        skip_whitespace_and_comments(chars);
        if chars.peek() == Some(&',') {
            chars.next(); // consume ','
        } else {
            break; // missing comma or end of selectors
        }
    }
    selectors
}

fn parse_selector(chars: &mut Peekable<Chars>) -> Option<Selector> {
    let mut sel = Selector::default();
    let mut parsed_something = false;

    // Check for universal selector
    if chars.peek() == Some(&'*') {
        chars.next();
        return Some(sel);
    }

    // Parse tag name
    let tag = parse_identifier(chars);
    if !tag.is_empty() {
        sel.tag = Some(tag);
        parsed_something = true;
    }

    loop {
        match chars.peek() {
            Some(&'#') => {
                chars.next();
                let id = parse_identifier(chars);
                if !id.is_empty() {
                    sel.id = Some(id);
                    parsed_something = true;
                }
            }
            Some(&'.') => {
                chars.next();
                let class = parse_identifier(chars);
                if !class.is_empty() {
                    sel.classes.push(class);
                    parsed_something = true;
                }
            }
            _ => break,
        }
    }

    if parsed_something {
        Some(sel)
    } else {
        // Skip invalid characters so we don't infinite loop
        if chars.peek().is_some() && chars.peek() != Some(&',') && chars.peek() != Some(&'{') {
            chars.next();
        }
        None
    }
}

fn parse_identifier(chars: &mut Peekable<Chars>) -> String {
    let mut id = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            id.push(c);
            chars.next();
        } else {
            break;
        }
    }
    id
}

fn parse_declarations(chars: &mut Peekable<Chars>) -> Vec<Declaration> {
    let mut decls = Vec::new();
    loop {
        skip_whitespace_and_comments(chars);
        if chars.peek() == Some(&'}') || chars.peek().is_none() {
            break;
        }

        let property = parse_identifier(chars);
        if property.is_empty() {
            // Bad syntax, skip to next ; or }
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next();
                    break;
                } else if c == '}' {
                    break;
                }
                chars.next();
            }
            continue;
        }

        skip_whitespace_and_comments(chars);
        if chars.peek() == Some(&':') {
            chars.next();
            skip_whitespace_and_comments(chars);

            let mut value = String::new();
            while let Some(&c) = chars.peek() {
                if c == ';' || c == '}' {
                    break;
                }
                value.push(c);
                chars.next();
            }

            if chars.peek() == Some(&';') {
                chars.next();
            }

            decls.push(Declaration {
                property,
                value: value.trim().to_string(),
            });
        } else {
            // Missing colon, skip
            while let Some(&c) = chars.peek() {
                if c == ';' {
                    chars.next();
                    break;
                } else if c == '}' {
                    break;
                }
                chars.next();
            }
        }
    }
    decls
}
