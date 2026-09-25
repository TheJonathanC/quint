use std::env;
use std::process;

fn print_usage() {
    println!("USAGE:");
    println!("    quint <URL>");
    println!("    quint --html <STRING>");
    println!("    quint --help");
    println!();
    println!("ARGUMENTS:");
    println!("    <URL>              An HTTP or HTTPS URL to fetch and parse");
    println!();
    println!("OPTIONS:");
    println!("    --html <STRING>    Parse a raw HTML string directly (for debugging)");
    println!("    -h, --help         Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    quint https://example.com");
    println!("    quint --html '<h1>Hello</h1>'");
}

fn eprint_usage() {
    eprintln!("USAGE:");
    eprintln!("    quint <URL>");
    eprintln!("    quint --html <STRING>");
    eprintln!("    quint --help");
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

    if args[0] == "--html" {
        if args.len() < 2 {
            eprintln!("error: --html requires a value");
            process::exit(1);
        }
        let html = &args[1];
        let dom = quint_html::parse(html);
        println!("{:#?}", dom);
        process::exit(0);
    }

    let url = &args[0];
    match quint_net::fetch(url) {
        Ok(response) => {
            if let Some(ct) = &response.content_type
                && !ct.contains("text/html") {
                    eprintln!("warning: Content-Type is '{}', expected text/html", ct);
                }

            let dom = quint_html::parse(&response.body);
            println!("{:#?}", dom);
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
