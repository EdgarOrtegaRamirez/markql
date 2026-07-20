# AGENTS.md — Notes for AI Agents

## Project Overview
MarkQL is a Rust CLI tool and library for querying Markdown documents with CSS-like selectors.

## Architecture
- `src/ast.rs` — AST node types (Node enum with 17 variants)
- `src/lexer.rs` — Markdown parser (line-by-line block parsing + inline parser)
- `src/query/mod.rs` — Query lexer and parser
- `src/query/executor.rs` — Query execution engine
- `src/output.rs` — Output formatters (JSON, Markdown, Text, Count, Tree)
- `src/main.rs` — CLI entry point with clap

## Key Algorithms
1. **Markdown Parsing**: Line-by-line block state machine + inline content parser
2. **Query Parsing**: Token-based lexer → recursive descent parser → selector AST
3. **Query Execution**: Tree traversal with depth tracking for child/descendant combinators
4. **Node Matching**: Attribute filters with `=` (exact), `*=` (contains), `!=` (not equal)

## Build & Test
```bash
cargo build
cargo test
cargo test -- --nocapture  # verbose output
```

## Query Syntax
- `heading` — find all headings
- `heading[level=2]` — level 2 headings
- `code_block[lang=python]` — Python code blocks
- `heading[contains("API")]` — headings containing "API"
- `list > list_item` — direct children
- `blockquote emphasis` — descendants
- `heading + paragraph` — next sibling
- `heading, code_block` — multiple types
- `list_item:first_child` — first child of parent

## Node Types
`heading`, `paragraph`, `code_block`, `inline_code`, `blockquote`, `list`, `list_item`, `link`, `image`, `emphasis`, `thematic_break`, `table`, `text`, `document`

## Dependencies
- `serde` / `serde_json` — JSON output
- `clap` — CLI argument parsing
- `anyhow` — Error handling
- `colored` — Terminal colors
- `tempfile` — Integration test fixtures

## Common Pitfalls
- Rust lifetime issues with `Vec<&Node>` vs `&[Node]` — use `text_content()` method
- Clap conflict with short flags (`-f`) — use long flags only
- The `Query::Text` variant handles text content, not node type "text"

## Next Steps
- Add more attribute operators (`^=`, `$=`, `~=`)
- Support regex patterns in contains()
- Add `:last-child` pseudo-selector
- Improve error messages for invalid queries
