use quint_html::Node;
use quint_style::PropertyMap;

fn main() {
    let html = r#"
        <style>
            .outer { width: 400px; padding-left: 20px; border-left-width: 5px; }
            .inner { margin-left: 10px; width: 100px; }
        </style>
        <div class="outer">
            <div class="inner"></div>
        </div>
    "#;
    let dom = quint_html::parse(html);
    
    // We need the helper extract_styles.
}
