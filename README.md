# Quint

A toy browser engine built in Rust from scratch.

## Architecture

Quint is organized into a Cargo workspace with the following structure:

- **`quint` (CLI)**: The main binary that accepts URLs, coordinates fetching and parsing, and outputs the resulting DOM tree.
- **`quint_net`**: Handles networking and HTTP/HTTPS requests (powered by `ureq` v3). Features URL validation and body size limits.
- **`quint_html`**: A custom, minimal, infallible HTML parser that tokenizes character-by-character and produces a simplified DOM tree.

## Usage

You can use the CLI to fetch real web pages or parse raw HTML strings for debugging:

```bash
# Fetch and parse a live URL
cargo run -- https://example.com

# Parse a raw HTML string directly
cargo run -- --html '<div><h1>Hello, Quint!</h1></div>'

# Show help
cargo run -- --help
```

## Running Tests

```bash
# Run unit tests
cargo test --workspace

# Run network-dependent integration tests
cargo test -p quint_net -- --ignored
```
