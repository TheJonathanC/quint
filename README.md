# Quint Engine

A modular, modern browser engine built from scratch in Rust. Quint evolved from a toy layout engine into a fully-featured, hardware-accelerated browser stack that supports TrueType fonts, Flexbox, HTML5 standards, and JavaScript execution.

## Architecture & Crate Ecosystem

Quint is organized into a Cargo workspace with distinct, decoupled crates mapping to the standard browser pipeline:

- **`quint` (CLI)**: The main executable that coordinates fetching, parsing, styling, layout, rendering, and window management via **`winit`**.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests powered by **`ureq`**. Features URL validation and body size limits.
- **`quint_html`**: Uses **`scraper`** (and Mozilla's **`html5ever`**) for spec-compliant HTML5 parsing, dynamically mapped into a simple, infallible `Node` tree.
- **`quint_css`**: Parses stylesheets from raw CSS text, supporting simple selectors (tags, classes, IDs), standard declarations, and `vw`/`vh`/`em`/`px`/`%` unit resolution.
- **`quint_style`**: A style resolution engine that cross-references the DOM tree with CSS rules, applying selector matching, the cascade, specificity sorting, and inheritance.
- **`quint_layout`**: A robust block layout engine utilizing **`taffy`** for full Flexbox and CSS Grid capabilities. It hooks into font metrics for precise text-wrapping and container sizing.
- **`quint_render`**: A 100% GPU-accelerated 2D renderer using **`wgpu`** and Linebender's **`vello`**. It leverages **`ab_glyph`** to parse TrueType fonts, dynamically translates their quadratic/cubic vector curves into `kurbo::BezPath` structures, and submits them to the GPU for hardware rasterization.
- **`quint_js`**: A JavaScript bridge integrating the **`boa`** engine. Extracts `<script>` tags, exposes a global `document` API (`createElement`, `getElementById`, `appendChild`), and allows scripts to dynamically mutate the Rust DOM tree.

## Features Supported
* **True Typography**: Vector text rendering with kerning, ascender/descender metrics, and sub-pixel accuracy.
* **Modern Layout**: `display: flex` and `display: grid` support out of the box.
* **Standards Compliant DOM**: Survives malformed HTML just like a real browser.
* **JavaScript Interop**: Scripts can mutate the DOM, which triggers layout recalculations and repaints.
* **Hardware Acceleration**: High-performance rendering pipeline using compute shaders.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings. By default, it opens a graphical window and renders the page using the GPU:

```bash
# Fetch, layout, and render a live URL (default 800px viewport)
cargo run -- https://example.com

# Parse a raw HTML string and render it directly (now with JS support!)
cargo run -- --html '<style>body { display: flex; }</style><script>document.body.setAttribute("class", "red");</script><body><p>Hello, Quint!</p></body>'
```
