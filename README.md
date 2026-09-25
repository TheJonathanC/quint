# Quint

A toy browser engine built in Rust from scratch. Now featuring TrueType fonts, HTML5 parsing, and JavaScript execution!

## Architecture

Quint is organized into a Cargo workspace with the following modular engine components:

- **`quint` (CLI)**: The main binary that coordinates fetching, parsing, styling, layout, rendering, and window management.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests (powered by `ureq` v3). Features URL validation and body size limits.
- **`quint_html`**: Uses `scraper` (and Mozilla's `html5ever` internally) to provide completely spec-compliant HTML5 parsing, dynamically mapped into a simple infallible `Node` tree.
- **`quint_css`**: A custom CSS parser that generates stylesheets from raw CSS text, supporting simple selectors (tags, classes, IDs) and parsing declarations.
- **`quint_style`**: A style resolution engine that cross-references the DOM tree with CSS rules, applying selector matching, the cascade, specificity sorting, and inheritance.
- **`quint_layout`**: A CSS block layout engine that computes the physical box model (`x`, `y`, `width`, `height`, margins, paddings, borders). Features true text metrics calculation using `ab_glyph`.
- **`quint_render`**: A CPU-based rendering engine using `winit` for cross-platform windowing, `tiny-skia` for graphics, and `ab_glyph` for beautiful TrueType font rasterization.
- **`quint_js`**: A JavaScript bridge integrating the `boa` engine. Extracts `<script>` tags, exposes a global `document` API (`createElement`, `getElementById`, `appendChild`), and allows scripts to dynamically mutate the Rust DOM tree.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings. By default, it will open a graphical window and render the page:

```bash
# Fetch, layout, and render a live URL (default 800px viewport)
cargo run -- https://example.com

# Parse a raw HTML string and render it directly (now with JS support!)
cargo run -- --html '<script>document.body.setAttribute("class", "red");</script><body><p>Hello, Quint!</p></body>'
```
