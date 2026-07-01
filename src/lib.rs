pub mod ast;
pub mod lexer;
pub mod output;
pub mod query;

use ast::Node;
use output::{format_results, OutputFormat};
use query::executor::{execute_query, MatchResult};

/// Parse markdown text into an AST
pub fn parse(markdown: &str) -> Node {
    let parser = lexer::Parser::new();
    parser.parse(markdown)
}

/// Query a markdown AST with a CSS-like selector
pub fn query(ast: &Node, query_str: &str) -> Result<Vec<MatchResult>, String> {
    execute_query(ast, query_str)
}

/// Parse and query markdown in one step
pub fn query_markdown(markdown: &str, query_str: &str) -> Result<Vec<MatchResult>, String> {
    let ast = parse(markdown);
    query(&ast, query_str)
}

/// Format query results into the specified output format
pub fn format_results_str(results: &[MatchResult], format: &str, query_str: &str) -> String {
    let fmt = OutputFormat::parse(format);
    format_results(results, &fmt, query_str)
}

/// Parse, query, and format in one step
pub fn run(markdown: &str, query_str: &str, fmt: &str) -> Result<String, String> {
    let results = query_markdown(markdown, query_str)?;
    Ok(format_results_str(&results, fmt, query_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_query() {
        let md = "# Title\n\n## Subtitle\n\nSome text\n\n```rust\nfn main() {}\n```";
        let results = query_markdown(md, "heading").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_run_json() {
        let md = "# Hello\n\nWorld";
        let output = run(md, "heading", "json").unwrap();
        assert!(output.contains("\"Hello\""));
    }

    #[test]
    fn test_run_count() {
        let md = "# H1\n\n## H2\n\n### H3";
        let output = run(md, "heading", "count").unwrap();
        assert_eq!(output.trim(), "3");
    }

    #[test]
    fn test_run_text() {
        let md = "# Hello\n\n## World";
        let output = run(md, "heading", "text").unwrap();
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn test_query_code_blocks() {
        let md = "```python\nprint('hi')\n```\n\n```rust\nfn main() {}\n```";
        let results = query_markdown(md, "code_block[lang=python]").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_links() {
        let md = "Click [here](https://example.com) for info";
        let results = query_markdown(md, "link").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_error_handling() {
        let result = query_markdown("# Title", "invalid[[[query");
        assert!(result.is_err());
    }
}
