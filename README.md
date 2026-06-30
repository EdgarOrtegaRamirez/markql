# MarkQL: Markdown Query Language

Query Markdown documents with CSS-like selectors. Parse Markdown into an AST, then find headings, code blocks, links, and more with powerful filter expressions.

## Features

- **CSS-like Query Language**: Find nodes by type, attributes, and content
- **Attribute Filters**: `heading[level=2]`, `code_block[lang=python]`
- **Content Filters**: `heading[contains("API")]`
- **Combinators**: Child (`>`), Descendant (space), Sibling (`+`)
- **Multiple Output Formats**: JSON, Markdown, Text, Count, Tree
- **Full AST**: Complete Markdown parser with typed nodes
- **Fast**: Built in Rust for speed

## Installation

```bash
cargo install markql
```

## Quick Start

```bash
# Count all headings in a file
cat README.md | markql query heading --format count

# Find all Python code blocks
cat docs.md | markql query 'code_block[lang=python]' --format json

# Find all links containing "github"
cat notes.md | markql query 'link[url*=github]' --format text

# Show AST tree
cat article.md | markql ast --format tree
```

## Query Language

### Node Types

| Type | Description |
|------|-------------|
| `heading` | Section headings (# through ######) |
| `paragraph` | Block of text |
| `code_block` | Fenced code blocks (``` or ~~~) |
| `inline_code` | Inline code (`code`) |
| `blockquote` | Blockquotes (>) |
| `list` | Ordered or unordered lists |
| `list_item` | Individual list items |
| `link` | Links [text](url) |
| `image` | Images ![alt](url) |
| `emphasis` | Italic (*text*) or bold (**text**) |
| `thematic_break` | Horizontal rules (---, ***, ___) |
| `table` | Markdown tables |
| `text` | Plain text content |

### Attribute Filters

```bash
# Level 2 headings
markql query 'heading[level=2]'

# Python code blocks
markql query 'code_block[lang=python]'

# Links with specific URL
markql query 'link[url=https://example.com]'

# Bold text
markql query 'emphasis[strong=true]'
```

### Content Filters

```bash
# Headings containing "API"
markql query 'heading[contains("API")]'

# Code blocks containing "import"
markql query 'code_block[contains("import")]'
```

### Combinators

```bash
# Direct children: list items inside lists
markql query 'list > list_item'

# Descendants: emphasis inside blockquotes
markql query 'blockquote emphasis'

# Sibling: paragraph after heading
markql query 'heading + paragraph'

# Multiple types
markql query 'heading, code_block, link'
```

## Output Formats

### JSON
```bash
markql query heading --format json
# {"query": "heading", "count": 3, "matches": [...]}
```

### Count
```bash
markql query heading --format count
# 3
```

### Text
```bash
markql query heading --format text
# Introduction
# Getting Started
# Conclusion
```

### Markdown
```bash
markql query heading --format markdown
# # Introduction
# ## Getting Started
# ## Conclusion
```

### Tree
```bash
markql ast --format tree
└── document
    ├── heading: Introduction
    ├── paragraph: Some text here...
    └── code_block: python
```

## Commands

| Command | Description |
|---------|-------------|
| `query <selector>` | Query markdown with a CSS-like selector |
| `ast` | Show the full AST of the markdown |
| `stats` | Show statistics about the document |
| `types` | List all available node types |
| `demo` | Run a series of example queries |

## Architecture

MarkQL uses a pipeline architecture:

1. **Lexer** (`lexer.rs`): Converts markdown text into block and inline nodes
2. **AST** (`ast.rs`): Typed node representation with attributes and text content
3. **Query Lexer** (`query/mod.rs`): Tokenizes CSS-like query selectors
4. **Query Parser** (`query/mod.rs`): Parses tokens into selector AST
5. **Query Executor** (`query/executor.rs`): Matches selectors against markdown AST
6. **Output** (`output.rs`): Formats results in multiple formats

## Library Usage

```rust
use markql::{parse, query_markdown, format_results_str};

let md = "# Hello\n\n## World";
let results = query_markdown(md, "heading").unwrap();
let output = format_results_str(&results, "json", "heading");
println!("{}", output);
```

## License

MIT
