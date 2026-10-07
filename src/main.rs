use quint_html::Node;
use quint_style::PropertyMap;
use std::env;
use std::process;

fn print_usage() {
    println!("USAGE:");
    println!(
        "    quint <URL>                           # default: render layout tree (800px viewport)"
    );
    println!("    quint --width 1024 <URL>              # render with custom viewport width");
    println!("    quint --layout <URL>                  # print layout tree instead of rendering");
    println!("    quint --styled <URL>                  # print styled tree");
    println!("    quint --dom-only <URL>                # print only the raw DOM");
    println!("    quint --html <STRING>                 # Parse a raw HTML string directly");
    println!("    quint --help                          # Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    quint https://example.com");
    println!("    quint --width 1920 https://example.com");
    println!("    quint --html '<h1>Hello</h1>'");
}

fn eprint_usage() {
    eprintln!("USAGE:");
    eprintln!("    quint [OPTIONS] <URL>");
    eprintln!("    quint --html <STRING>");
    eprintln!("    quint --help");
}

fn extract_styles(nodes: &[Node]) -> String {
    let mut css = String::new();
    for node in nodes {
        if let Node::Element { tag, children, .. } = node {
            if tag == "style" {
                for child in children {
                    if let Node::Text(text) = child {
                        css.push_str(text);
                        css.push('\n');
                    }
                }
            } else {
                css.push_str(&extract_styles(children));
            }
        }
    }
    css
}

fn extract_scripts(nodes: &[Node]) -> String {
    let mut js = String::new();
    for node in nodes {
        if let Node::Element { tag, children, .. } = node {
            if tag == "script" {
                for child in children {
                    if let Node::Text(text) = child {
                        js.push_str(text);
                        js.push('\n');
                    }
                }
            } else {
                js.push_str(&extract_scripts(children));
            }
        }
    }
    js
}

fn extract_linked_stylesheets(nodes: &[Node]) -> Vec<String> {
    let mut hrefs = Vec::new();
    collect_links(nodes, &mut hrefs);
    hrefs
}

fn collect_links(nodes: &[Node], hrefs: &mut Vec<String>) {
    for node in nodes {
        if let Node::Element { tag, children, .. } = node {
            if tag == "link" {
                let is_stylesheet = node
                    .get_attribute("rel")
                    .map(|r| r.eq_ignore_ascii_case("stylesheet"))
                    .unwrap_or(false);
                if is_stylesheet
                    && let Some(href) = node.get_attribute("href")
                {
                    hrefs.push(href.to_string());
                }
            }
            collect_links(children, hrefs);
        }
    }
}

fn extract_script_srcs(nodes: &[Node]) -> Vec<String> {
    let mut srcs = Vec::new();
    collect_script_srcs(nodes, &mut srcs);
    srcs
}

fn collect_script_srcs(nodes: &[Node], srcs: &mut Vec<String>) {
    for node in nodes {
        if let Node::Element { tag, children, .. } = node {
            if tag == "script"
                && let Some(src) = node.get_attribute("src")
            {
                srcs.push(src.to_string());
            }
            collect_script_srcs(children, srcs);
        }
    }
}

fn load_resource(base: &str, target: &str) -> Result<String, String> {
    if base.starts_with("http://") || base.starts_with("https://") {
        let full_url = quint_net::resolve_url(base, target);
        match quint_net::fetch(&full_url) {
            Ok(resp) => Ok(resp.body),
            Err(e) => Err(format!("failed to fetch {}: {}", full_url, e)),
        }
    } else {
        let base_path = if let Some(stripped) = base.strip_prefix("file://") {
            std::path::Path::new(stripped)
        } else {
            std::path::Path::new(base)
        };
        let dir = base_path.parent().unwrap_or(std::path::Path::new("."));
        let target_path = dir.join(target);
        std::fs::read_to_string(&target_path)
            .map_err(|e| format!("failed to read file {:?}: {}", target_path, e))
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprint_usage();
        process::exit(1);
    }

    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_usage();
        process::exit(0);
    }

    let mut is_dom_only = false;
    let mut is_styled = false;
    let mut is_layout = false;
    let mut is_html = false;
    let mut html_str = "";
    let mut url = "";
    let mut width = 800.0;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--dom-only" {
            is_dom_only = true;
        } else if args[i] == "--styled" {
            is_styled = true;
        } else if args[i] == "--layout" {
            is_layout = true;
        } else if args[i] == "--width" {
            if i + 1 < args.len() {
                if let Ok(w) = args[i + 1].parse::<f32>() {
                    width = w.max(0.0);
                } else {
                    eprintln!("error: --width requires a valid number");
                    process::exit(1);
                }
                i += 1;
            } else {
                eprintln!("error: --width requires a value");
                process::exit(1);
            }
        } else if args[i] == "--html" {
            is_html = true;
            if i + 1 < args.len() {
                html_str = &args[i + 1];
                i += 1;
            } else {
                eprintln!("error: --html requires a value");
                process::exit(1);
            }
        } else if url.is_empty() {
            url = &args[i];
        }
        i += 1;
    }

    let (mut dom, base_target) = if is_html {
        (quint_html::parse(html_str), String::new())
    } else {
        if url.is_empty() {
            eprint_usage();
            process::exit(1);
        }

        let is_http = url.starts_with("http://") || url.starts_with("https://");
        let is_file = url.starts_with("file://") || std::path::Path::new(url).exists();

        if is_http {
            match quint_net::fetch(url) {
                Ok(response) => {
                    if let Some(ct) = &response.content_type
                        && !ct.contains("text/html")
                    {
                        eprintln!("warning: Content-Type is '{}', expected text/html", ct);
                    }
                    (quint_html::parse(&response.body), response.final_url)
                }
                Err(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
            }
        } else if is_file {
            let file_path = url.strip_prefix("file://").unwrap_or(url);
            match std::fs::read_to_string(file_path) {
                Ok(content) => (quint_html::parse(&content), url.to_string()),
                Err(e) => {
                    eprintln!("error reading file '{}': {}", file_path, e);
                    process::exit(1);
                }
            }
        } else {
            match quint_net::fetch(url) {
                Ok(response) => (quint_html::parse(&response.body), response.final_url),
                Err(e) => {
                    eprintln!("error: {}", e);
                    process::exit(1);
                }
            }
        }
    };

    // Load external scripts if any
    if !base_target.is_empty() {
        for src in extract_script_srcs(&dom) {
            match load_resource(&base_target, &src) {
                Ok(code) => {
                    if let Err(e) = quint_js::execute_script(&mut dom, &code) {
                        eprintln!("warning: External JS execution failed ({}): {}", src, e);
                    }
                }
                Err(e) => eprintln!("warning: {}", e),
            }
        }
    }

    // Extract and run inline JS
    let js = extract_scripts(&dom);
    if !js.trim().is_empty()
        && let Err(e) = quint_js::execute_script(&mut dom, &js)
    {
        eprintln!("warning: JS execution failed: {}", e);
    }

    if is_dom_only {
        println!("{:#?}", dom);
        process::exit(0);
    }

    let mut css = String::from(quint_style::default_ua_css());
    css.push('\n');

    // Load external stylesheets if any
    if !base_target.is_empty() {
        for href in extract_linked_stylesheets(&dom) {
            match load_resource(&base_target, &href) {
                Ok(sheet_content) => {
                    css.push_str(&sheet_content);
                    css.push('\n');
                }
                Err(e) => eprintln!("warning: {}", e),
            }
        }
    }

    css.push_str(&extract_styles(&dom));
    let stylesheet = quint_css::parse(&css);
    let styled_tree = quint_style::style_tree(&dom, &stylesheet, &PropertyMap::new());

    if is_styled {
        println!("{:#?}", styled_tree);
        process::exit(0);
    }

    let layout_tree = quint_layout::layout_tree(&styled_tree, width);

    if is_layout {
        println!("{:#?}", layout_tree);
        process::exit(0);
    }

    // Default action: render the GUI window
    quint_render::render_window(layout_tree, width);
}
