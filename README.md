# Quint

A toy browser engine built in Rust from scratch. Now featuring TrueType fonts, HTML5 parsing, Taffy Flexbox, and GPU Acceleration via Vello/WGPU!

## Architecture

Quint is organized into a Cargo workspace with the following modular engine components:

- **`quint` (CLI)**: The main binary that coordinates fetching, parsing, styling, layout, rendering, and window management.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests (powered by `ureq` v3). Features URL validation and body size limits.
- **`quint_html`**: Uses `scraper` (and Mozilla's `html5ever` internally) to provide completely spec-compliant HTML5 parsing, dynamically mapped into a simple infallible `Node` tree.
- **`quint_css`**: A custom CSS parser that generates stylesheets from raw CSS text, supporting simple selectors (tags, classes, IDs) and parsing declarations.
- **`quint_style`**: A style resolution engine that cross-references the DOM tree with CSS rules, applying selector matching, the cascade, specificity sorting, and inheritance.
- **`quint_layout`**: A CSS layout engine leveraging `taffy` for robust Flexbox/Grid computation, enhanced with a custom `ab_glyph` measure function for exact text geometry wrapping.
- **`quint_render`**: A 100% GPU-accelerated 2D renderer using `wgpu` and `vello`. Replaces `tiny-skia` by dynamically translating TrueType font outlines directly into `vello::kurbo::BezPath` structures mapped onto a GPU Command Buffer.
- **`quint_js`**: A JavaScript bridge integrating the `boa` engine. Extracts `<script>` tags, exposes a global `document` API (`createElement`, `getElementById`, `appendChild`), and allows scripts to dynamically mutate the Rust DOM tree.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings. By default, it will open a graphical window and render the page on your GPU:

```bash
# Fetch, layout, and render a live URL (default 800px viewport)
cargo run -- https://example.com

# Parse a raw HTML string and render it directly (now with JS support!)
cargo run -- --html '<script>document.body.setAttribute("class", "red");</script><body><p>Hello, Quint!</p></body>'
```
