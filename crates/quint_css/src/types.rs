#[derive(Debug, Clone, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub relation: Option<(Combinator, Box<Selector>)>,
}

impl Selector {
    pub fn specificity(&self) -> (usize, usize, usize) {
        let mut id = if self.id.is_some() { 1 } else { 0 };
        let mut classes = self.classes.len();
        let mut tag = if self.tag.is_some() { 1 } else { 0 };
        if let Some((_, prev)) = &self.relation {
            let (p_id, p_cls, p_tag) = prev.specificity();
            id += p_id;
            classes += p_cls;
            tag += p_tag;
        }
        (id, classes, tag)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}
