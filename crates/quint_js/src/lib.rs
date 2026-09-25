use boa_engine::{Context, Source};
use quint_html::Node;

pub fn execute_script(dom: &mut Vec<Node>, script: &str) -> Result<(), String> {
    let mut context = Context::default();

    // 1. Serialize DOM to JSON
    let dom_json = serde_json::to_string(dom).map_err(|e| e.to_string())?;

    // 2. Inject JSON string as a global variable
    let injected_json = format!(
        "const __RUST_DOM_JSON__ = {};",
        serde_json::to_string(&dom_json).unwrap()
    );
    context
        .eval(Source::from_bytes(injected_json.as_bytes()))
        .map_err(|e| e.to_string())?;

    // 3. Document Polyfill
    let polyfill = r#"
        class Element {
            constructor(tag) {
                this.Element = {
                    tag: tag,
                    attributes: [],
                    children: []
                };
            }
            setAttribute(name, value) {
                let attr = this.Element.attributes.find(a => a[0] === name);
                if (attr) { attr[1] = value; }
                else { this.Element.attributes.push([name, value]); }
            }
            appendChild(child) {
                if (child.Element) {
                    this.Element.children.push(child.Element);
                } else if (child.Text !== undefined) {
                    this.Element.children.push(child);
                } else {
                    this.Element.children.push(child);
                }
            }
        }
        
        function wrapNode(rustNode) {
            if (!rustNode || !rustNode.Element) return rustNode;
            
            let wrapper = {
                Element: rustNode.Element,
                setAttribute(name, value) {
                    let attr = this.Element.attributes.find(a => a[0] === name);
                    if (attr) { attr[1] = value; }
                    else { this.Element.attributes.push([name, value]); }
                },
                appendChild(child) {
                    if (child.Element) {
                        this.Element.children.push(child.Element);
                    } else if (child.Text !== undefined) {
                        this.Element.children.push(child);
                    } else {
                        this.Element.children.push(child);
                    }
                }
            };
            
            return wrapper;
        }

        const document = {
            _tree: JSON.parse(__RUST_DOM_JSON__),
            createElement: function(tag) { return new Element(tag); },
            createTextNode: function(text) { return { Text: text }; },
            getElementById: function(id) {
                function search(nodes) {
                    for (let node of nodes) {
                        if (node.Element) {
                            let attr = node.Element.attributes.find(a => a[0] === 'id');
                            if (attr && attr[1] === id) return wrapNode(node);
                            let found = search(node.Element.children);
                            if (found) return found;
                        }
                    }
                    return null;
                }
                return search(this._tree);
            },
            body: null
        };
        
        for (let node of document._tree) {
            if (node.Element && node.Element.tag === 'body') {
                document.body = wrapNode(node);
                break;
            }
            if (node.Element && node.Element.tag === 'html') {
                for (let child of node.Element.children) {
                    if (child.Element && child.Element.tag === 'body') {
                        document.body = wrapNode(child);
                        break;
                    }
                }
            }
        }
        if (!document.body && document._tree.length > 0) {
            document.body = wrapNode(document._tree[0]);
        }
    "#;

    context
        .eval(Source::from_bytes(polyfill))
        .map_err(|e| e.to_string())?;

    // 4. Run User Script
    context
        .eval(Source::from_bytes(script))
        .map_err(|e| e.to_string())?;

    // 5. Serialize back
    let new_json_val = context
        .eval(Source::from_bytes("JSON.stringify(document._tree)"))
        .map_err(|e| e.to_string())?;

    if let Some(js_str) = new_json_val.as_string() {
        let new_json = js_str.to_std_string_escaped();
        let new_dom: Vec<Node> = serde_json::from_str(&new_json).map_err(|e| e.to_string())?;
        *dom = new_dom;
        Ok(())
    } else {
        Err("Failed to serialize document._tree".to_string())
    }
}
