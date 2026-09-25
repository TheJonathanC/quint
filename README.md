# Quint

A toy browser engine built in Rust from scratch.

## Architecture

Quint is organized into a Cargo workspace with the following modular engine components:

- **`quint` (CLI)**: The main binary that coordinates fetching, parsing, styling, layout, rendering, and window management.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests (powered by `ureq` v3). Features URL validation and body size limits.
- **`quint_html`**: A custom, minimal, infallible HTML parser that tokenizes character-by-character and produces a simplified DOM tree.
- **`quint_css`**: A custom CSS parser that generates stylesheets from raw CSS text, supporting simple selectors (tags, classes, IDs) and parsing declarations.
- **`quint_style`**: A style resolution engine that cross-references the DOM tree with CSS rules, applying selector matching, the cascade, specificity sorting, and inheritance.
- **`quint_layout`**: A CSS block layout engine that computes the physical box model (`x`, `y`, `width`, `height`, margins, paddings, and borders) for every element in the document.
- **`quint_render`**: A CPU-based rendering engine using `winit` for cross-platform windowing, `tiny-skia` for 2D graphics (rectangles, color fills), and `font8x8` for basic bitmap text rasterization.
- **`quint_js`**: A JavaScript bridge integrating the `boa` engine. Extracts `<script>` tags, exposes a global `document` API (`createElement`, `getElementById`, `appendChild`), and allows scripts to dynamically mutate the Rust DOM tree.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings. By default, it will open a graphical window and render the page:

```bash
# Fetch, layout, and render a live URL (default 800px viewport)
cargo run -- https://example.com

# Parse a raw HTML string and render it directly (now with JS support!)
cargo run -- --html '<script>document.body.setAttribute("class", "red");</script><body><p>Hello, Quint!</p></body>'
```

### Advanced Diagnostics

You can tap into earlier stages of the browser pipeline to debug how pages are processed. These flags run the engine headlessly and print the intermediate data structures:

```bash
# Output the computed block layout tree (geometry applied)
cargo run -- --layout https://example.com

# Output the styled DOM tree (CSS resolution applied, but no layout geometry)
cargo run -- --styled https://example.com

# Output the raw DOM tree (HTML parsing, plus executed JS modifications)
cargo run -- --dom-only https://example.com
```

## Running Tests

```bash
# Run all unit tests across all engine modules
cargo test --workspace

# Run network-dependent integration tests
cargo test -p quint_net -- --ignored
```
