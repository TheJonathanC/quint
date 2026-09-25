use quint_html::Node;
use quint_style::PropertyMap;
use std::env;
use std::process;

fn print_usage() {
    println!("USAGE:");
    println!(
        "    quint <URL>                           # default: print layout tree (800px viewport)"
    );
    println!("    quint --width 1024 <URL>              # layout with custom viewport width");
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

    let dom = if is_html {
        quint_html::parse(html_str)
    } else {
        if url.is_empty() {
            eprint_usage();
            process::exit(1);
        }
        match quint_net::fetch(url) {
            Ok(response) => {
                if let Some(ct) = &response.content_type
                    && !ct.contains("text/html")
                {
                    eprintln!("warning: Content-Type is '{}', expected text/html", ct);
                }
                quint_html::parse(&response.body)
            }
            Err(e) => {
                eprintln!("error: {}", e);
                process::exit(1);
            }
        }
    };

    if is_dom_only {
        println!("{:#?}", dom);
        process::exit(0);
    }

    let css = extract_styles(&dom);
    let stylesheet = quint_css::parse(&css);
    let styled_tree = quint_style::style_tree(&dom, &stylesheet, &PropertyMap::new());

    if is_styled {
        println!("{:#?}", styled_tree);
        process::exit(0);
    }

    let layout_tree = quint_layout::layout_tree(&styled_tree, width);
    println!("{:#?}", layout_tree);
}
