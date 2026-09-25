# Quint

A toy browser engine built in Rust from scratch.

## Architecture

Quint is organized into a Cargo workspace with the following modular engine components:

- **`quint` (CLI)**: The main binary that coordinates fetching, parsing, styling, layout, and outputting the result.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests (powered by `ureq` v3). Features URL validation and body size limits.
- **`quint_html`**: A custom, minimal, infallible HTML parser that tokenizes character-by-character and produces a simplified DOM tree.
- **`quint_css`**: A custom CSS parser that generates stylesheets from raw CSS text, supporting simple selectors (tags, classes, IDs) and parsing declarations.
- **`quint_style`**: A style resolution engine that cross-references the DOM tree with CSS rules, applying selector matching, the cascade, specificity sorting, and inheritance.
- **`quint_layout`**: A CSS block layout engine that computes the physical box model (`x`, `y`, `width`, `height`, margins, paddings, and borders) for every element in the document.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings for debugging. By default, it will output the fully computed block layout tree:

```bash
# Fetch and compute layout for a live URL (default 800px viewport)
cargo run -- https://example.com

# Compute layout with a custom viewport width
cargo run -- --width 1024 https://example.com

# Parse a raw HTML string and layout directly
cargo run -- --html '<style>p { color: red; }</style><p>Hello, Quint!</p>'
```

### Advanced Diagnostics

You can tap into earlier stages of the browser pipeline to debug how pages are processed:

```bash
# Output the styled DOM tree (CSS resolution applied, but no layout geometry)
cargo run -- --styled https://example.com

# Output the raw DOM tree (HTML parsing only, no CSS or layout)
cargo run -- --dom-only https://example.com
```

## Running Tests

```bash
# Run all unit tests across all engine modules
cargo test --workspace

# Run network-dependent integration tests
cargo test -p quint_net -- --ignored
```
