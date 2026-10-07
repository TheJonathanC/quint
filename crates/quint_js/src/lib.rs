use boa_engine::{Context, Source};
use quint_html::Node;

pub fn execute_script(dom: &mut Vec<Node>, script: &str) -> Result<(), String> {
    let dom_clone = dom.clone();
    let script_owned = script.to_string();

    let handle = std::thread::Builder::new()
        .name("js-runtime".to_string())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || -> Result<Vec<Node>, String> {
            let mut dom_clone = dom_clone;
            execute_script_inner(&mut dom_clone, &script_owned)?;
            Ok(dom_clone)
        })
        .map_err(|e| e.to_string())?;

    match handle.join() {
        Ok(result) => {
            let new_dom = result?;
            *dom = new_dom;
            Ok(())
        }
        Err(e) => {
            Err(format!("JS thread terminated: {:?}", e))
        }
    }
}

fn execute_script_inner(dom: &mut Vec<Node>, script: &str) -> Result<(), String> {
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

    // 3. Document and DOM Polyfill
    let polyfill = r#"
        // Setup console
        const console = {
            log: function(...args) { },
            warn: function(...args) { },
            error: function(...args) { },
            info: function(...args) { },
            debug: function(...args) { }
        };

        function toRustNode(child) {
            if (!child) return null;
            if (child.Element) return { Element: child.Element };
            if (child.Text !== undefined) return { Text: child.Text };
            return child;
        }

        function createStyleProxy(element) {
            return new Proxy({}, {
                get(target, prop) {
                    let styleAttr = element.getAttribute('style') || '';
                    let decls = styleAttr.split(';').map(d => d.trim()).filter(Boolean);
                    let kebab = String(prop).replace(/([A-Z])/g, '-$1').toLowerCase();
                    for (let decl of decls) {
                        let [k, v] = decl.split(':').map(s => s.trim());
                        if (k === kebab || k === prop) return v;
                    }
                    return '';
                },
                set(target, prop, val) {
                    let kebab = String(prop).replace(/([A-Z])/g, '-$1').toLowerCase();
                    let styleAttr = element.getAttribute('style') || '';
                    let decls = styleAttr.split(';').map(d => d.trim()).filter(Boolean);
                    let updated = false;
                    let newDecls = decls.map(d => {
                        let [k] = d.split(':').map(s => s.trim());
                        if (k === kebab || k === prop) {
                            updated = true;
                            return kebab + ': ' + val;
                        }
                        return d;
                    });
                    if (!updated) {
                        newDecls.push(kebab + ': ' + val);
                    }
                    element.setAttribute('style', newDecls.join('; '));
                    return true;
                }
            });
        }

        class ElementWrapper {
            constructor(rawNode) {
                this.Element = rawNode.Element;
                this.style = createStyleProxy(this);
            }

            get tagName() {
                return (this.Element.tag || '').toUpperCase();
            }

            get id() {
                return this.getAttribute('id') || '';
            }
            set id(val) {
                this.setAttribute('id', val);
            }

            get className() {
                return this.getAttribute('class') || '';
            }
            set className(val) {
                this.setAttribute('class', val);
            }

            get classList() {
                const self = this;
                return {
                    add(...tokens) {
                        let classes = (self.className || '').split(/\s+/).filter(Boolean);
                        for (let tok of tokens) {
                            if (!classes.includes(tok)) classes.push(tok);
                        }
                        self.className = classes.join(' ');
                    },
                    remove(...tokens) {
                        let classes = (self.className || '').split(/\s+/).filter(Boolean);
                        classes = classes.filter(c => !tokens.includes(c));
                        self.className = classes.join(' ');
                    },
                    contains(token) {
                        let classes = (self.className || '').split(/\s+/).filter(Boolean);
                        return classes.includes(token);
                    },
                    toggle(token) {
                        if (this.contains(token)) {
                            this.remove(token);
                            return false;
                        } else {
                            this.add(token);
                            return true;
                        }
                    }
                };
            }

            getAttribute(name) {
                let attr = this.Element.attributes.find(a => a[0].toLowerCase() === name.toLowerCase());
                return attr ? attr[1] : null;
            }

            setAttribute(name, value) {
                let attr = this.Element.attributes.find(a => a[0].toLowerCase() === name.toLowerCase());
                if (attr) {
                    attr[1] = String(value);
                } else {
                    this.Element.attributes.push([name, String(value)]);
                }
            }

            hasAttribute(name) {
                return this.getAttribute(name) !== null;
            }

            removeAttribute(name) {
                this.Element.attributes = this.Element.attributes.filter(
                    a => a[0].toLowerCase() !== name.toLowerCase()
                );
            }

            appendChild(child) {
                let rust = toRustNode(child);
                if (rust) {
                    this.Element.children.push(rust);
                }
                return child;
            }

            removeChild(child) {
                let target = child.Element || child;
                let idx = this.Element.children.findIndex(c => {
                    if (c.Element && target.Element) return c.Element === target.Element;
                    return c === target;
                });
                if (idx !== -1) {
                    this.Element.children.splice(idx, 1);
                }
                return child;
            }

            get children() {
                let res = [];
                for (let c of this.Element.children) {
                    if (c.Element) res.push(wrapNode(c));
                }
                return res;
            }

            get textContent() {
                function collect(node) {
                    if (!node) return '';
                    if (node.Text !== undefined) return node.Text;
                    if (node.Element) {
                        return node.Element.children.map(collect).join('');
                    }
                    return '';
                }
                return collect(this);
            }

            set textContent(val) {
                this.Element.children = [{ Text: String(val) }];
            }

            get innerHTML() {
                function serialize(node) {
                    if (!node) return '';
                    if (node.Text !== undefined) return node.Text;
                    if (node.Element) {
                        let tag = node.Element.tag;
                        let attrs = node.Element.attributes
                            .map(([k, v]) => ` ${k}="${v}"`)
                            .join('');
                        let inner = node.Element.children.map(serialize).join('');
                        return `<${tag}${attrs}>${inner}</${tag}>`;
                    }
                    return '';
                }
                return this.Element.children.map(serialize).join('');
            }

            set innerHTML(html) {
                // Simple parser for standard HTML tags or text
                this.Element.children = [];
                let s = String(html).trim();
                let regex = /<([a-zA-Z0-9]+)([^>]*)>([\s\S]*?)<\/\1>|<([a-zA-Z0-9]+)([^>]*)\/?>|([^<]+)/g;
                let match;
                while ((match = regex.exec(s)) !== null) {
                    if (match[1]) {
                        // Pair tag
                        let tag = match[1];
                        let innerText = match[3];
                        let el = new ElementWrapper({
                            Element: { tag: tag, attributes: [], children: innerText ? [{ Text: innerText }] : [] }
                        });
                        this.appendChild(el);
                    } else if (match[4]) {
                        // Self-closing
                        let el = new ElementWrapper({
                            Element: { tag: match[4], attributes: [], children: [] }
                        });
                        this.appendChild(el);
                    } else if (match[6] && match[6].trim()) {
                        this.appendChild({ Text: match[6] });
                    }
                }
            }

            querySelector(sel) {
                let all = this.querySelectorAll(sel);
                return all.length > 0 ? all[0] : null;
            }

            querySelectorAll(sel) {
                let results = [];
                let s = sel.trim();
                let isClass = s.startsWith('.');
                let isId = s.startsWith('#');
                let target = isClass || isId ? s.slice(1) : s.toLowerCase();

                function walk(node) {
                    if (!node || !node.Element) return;
                    let matches = false;
                    if (isId) {
                        let id = (node.getAttribute && node.getAttribute('id')) ||
                            (node.Element.attributes.find(a => a[0] === 'id') || [])[1];
                        if (id === target) matches = true;
                    } else if (isClass) {
                        let cls = (node.getAttribute && node.getAttribute('class')) ||
                            (node.Element.attributes.find(a => a[0] === 'class') || [])[1] || '';
                        if (cls.split(/\s+/).includes(target)) matches = true;
                    } else {
                        if ((node.Element.tag || '').toLowerCase() === target) matches = true;
                    }

                    if (matches) results.push(wrapNode(node));

                    for (let child of node.Element.children) {
                        if (child.Element) walk(wrapNode(child));
                    }
                }

                for (let child of this.Element.children) {
                    if (child.Element) walk(wrapNode(child));
                }

                return results;
            }
        }

        function wrapNode(rustNode) {
            if (!rustNode || !rustNode.Element) return rustNode;
            if (rustNode instanceof ElementWrapper) return rustNode;
            return new ElementWrapper(rustNode);
        }

        const document = {
            _tree: JSON.parse(__RUST_DOM_JSON__),
            createElement: function(tag) {
                return new ElementWrapper({
                    Element: { tag: tag, attributes: [], children: [] }
                });
            },
            createTextNode: function(text) {
                return { Text: String(text) };
            },
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
            body: null,
            head: null,
            documentElement: null,
            title: '',
            compatMode: "CSS1Compat",
            addEventListener: function() {},
            removeEventListener: function() {},
            getElementsByTagName: function(tag) {
                let results = [];
                let t = tag.toLowerCase();
                function search(nodes) {
                    for (let node of nodes) {
                        if (node.Element) {
                            if (t === '*' || (node.Element.tag || '').toLowerCase() === t) {
                                results.push(wrapNode(node));
                            }
                            search(node.Element.children);
                        }
                    }
                }
                search(this._tree);
                return results;
            },
            getElementsByClassName: function(cls) {
                let results = [];
                function search(nodes) {
                    for (let node of nodes) {
                        if (node.Element) {
                            let attr = node.Element.attributes.find(a => a[0] === 'class');
                            if (attr && attr[1].split(/\s+/).includes(cls)) {
                                results.push(wrapNode(node));
                            }
                            search(node.Element.children);
                        }
                    }
                }
                search(this._tree);
                return results;
            },
            querySelector: function(sel) {
                let all = this.querySelectorAll(sel);
                return all.length > 0 ? all[0] : null;
            },
            querySelectorAll: function(sel) {
                let results = [];
                let s = sel.trim();
                let isClass = s.startsWith('.');
                let isId = s.startsWith('#');
                let target = isClass || isId ? s.slice(1) : s.toLowerCase();

                function walk(nodes) {
                    for (let node of nodes) {
                        if (node.Element) {
                            let matches = false;
                            if (isId) {
                                let id = (node.Element.attributes.find(a => a[0] === 'id') || [])[1];
                                if (id === target) matches = true;
                            } else if (isClass) {
                                let cls = (node.Element.attributes.find(a => a[0] === 'class') || [])[1] || '';
                                if (cls.split(/\s+/).includes(target)) matches = true;
                            } else {
                                if (s === '*' || (node.Element.tag || '').toLowerCase() === target) matches = true;
                            }

                            if (matches) results.push(wrapNode(node));
                            walk(node.Element.children);
                        }
                    }
                }
                walk(this._tree);
                return results;
            }
        };

        // Find body, head, and documentElement
        for (let node of document._tree) {
            if (node.Element && node.Element.tag === 'body') {
                document.body = wrapNode(node);
            } else if (node.Element && node.Element.tag === 'head') {
                document.head = wrapNode(node);
            } else if (node.Element && node.Element.tag === 'html') {
                document.documentElement = wrapNode(node);
                for (let child of node.Element.children) {
                    if (child.Element && child.Element.tag === 'body') {
                        document.body = wrapNode(child);
                    } else if (child.Element && child.Element.tag === 'head') {
                        document.head = wrapNode(child);
                    }
                }
            }
        }
        if (!document.body && document._tree.length > 0) {
            document.body = wrapNode(document._tree[0]);
        }
        if (!document.documentElement && document._tree.length > 0) {
            document.documentElement = wrapNode(document._tree[0]);
        }

        const window = globalThis;
        const self = globalThis;
        window.window = window;
        window.self = window;
        window.document = document;
        window.console = console;
        window.location = {
            href: "https://google.com",
            protocol: "https:",
            host: "google.com",
            hostname: "google.com",
            pathname: "/",
            search: "",
            hash: ""
        };
        window.navigator = {
            userAgent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Quint/1.0",
            language: "en-US",
            languages: ["en-US", "en"]
        };
        window.innerWidth = 800;
        window.innerHeight = 600;
        window.addEventListener = function() {};
        window.removeEventListener = function() {};
        window.setTimeout = function(fn) { return 0; };
        window.clearTimeout = function() {};
        window.requestAnimationFrame = function(fn) { return 0; };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_script_dom_mutation() {
        let mut dom = vec![Node::Element {
            tag: "body".to_string(),
            attributes: vec![],
            children: vec![
                Node::Element {
                    tag: "p".to_string(),
                    attributes: vec![("id".to_string(), "target".to_string())],
                    children: vec![Node::Text("Hello".to_string())],
                },
            ],
        }];

        let script = r#"
            console.log("Starting test script");
            let p = document.getElementById("target");
            p.textContent = "Modified by JS";
            p.classList.add("highlight", "bold");
            p.style.color = "red";

            let btn = document.createElement("button");
            btn.textContent = "Click Me";
            document.body.appendChild(btn);
        "#;

        execute_script(&mut dom, script).unwrap();

        // Check paragraph
        let p = dom[0].find_by_id("target").unwrap();
        assert_eq!(p.text_content(), "Modified by JS");
        assert!(p.has_class("highlight"));
        assert!(p.has_class("bold"));
        assert_eq!(p.get_attribute("style"), Some("color: red"));

        // Check appended button
        let buttons = dom[0].find_all_by_tag("button");
        assert_eq!(buttons.len(), 1);
        assert_eq!(buttons[0].text_content(), "Click Me");
    }

    #[test]
    fn test_query_selector_and_inner_html() {
        let mut dom = vec![Node::Element {
            tag: "body".to_string(),
            attributes: vec![],
            children: vec![
                Node::Element {
                    tag: "div".to_string(),
                    attributes: vec![("class".to_string(), "container".to_string())],
                    children: vec![],
                },
            ],
        }];

        let script = r#"
            let container = document.querySelector(".container");
            container.innerHTML = "<span>Inner Span</span>";
        "#;

        execute_script(&mut dom, script).unwrap();

        let spans = dom[0].find_all_by_tag("span");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text_content(), "Inner Span");
    }
}
