use crate::ast::Node;
use crate::query::executor::MatchResult;
use serde_json::{json, Value};

/// Output format for query results
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Markdown,
    Text,
    Count,
    Tree,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "markdown" | "md" => OutputFormat::Markdown,
            "text" | "txt" => OutputFormat::Text,
            "count" => OutputFormat::Count,
            "tree" => OutputFormat::Tree,
            _ => OutputFormat::Json,
        }
    }
}

/// Format match results according to the specified format
pub fn format_results(results: &[MatchResult], format: &OutputFormat, query: &str) -> String {
    match format {
        OutputFormat::Json => format_json(results, query),
        OutputFormat::Markdown => format_markdown(results),
        OutputFormat::Text => format_text(results),
        OutputFormat::Count => format!("{}", results.len()),
        OutputFormat::Tree => format_tree(results),
    }
}

fn format_json(results: &[MatchResult], query: &str) -> String {
    let items: Vec<Value> = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            json!({
                "index": i,
                "type": r.node.type_name(),
                "path": r.path,
                "depth": r.depth,
                "text": r.node.text_content(),
                "node": node_to_json(&r.node),
            })
        })
        .collect();

    let output = json!({
        "query": query,
        "count": results.len(),
        "matches": items,
    });

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

fn node_to_json(node: &Node) -> Value {
    match node {
        Node::Document(children) => json!({
            "type": "document",
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::Heading { level, children } => json!({
            "type": "heading",
            "level": level,
            "text": node.text_content(),
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::Paragraph(children) => json!({
            "type": "paragraph",
            "text": node.text_content(),
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::CodeBlock { language, code } => json!({
            "type": "code_block",
            "language": language,
            "code": code,
        }),
        Node::Blockquote(children) => json!({
            "type": "blockquote",
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::List { ordered, items } => json!({
            "type": "list",
            "ordered": ordered,
            "items": items.iter().map(|item| {
                item.children().iter().map(|n| node_to_json(n)).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
        }),
        Node::ListItem(children) => json!({
            "type": "list_item",
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::Link { text, url } => json!({
            "type": "link",
            "text": text,
            "url": url,
        }),
        Node::Image { alt, url } => json!({
            "type": "image",
            "alt": alt,
            "url": url,
        }),
        Node::Emphasis { strong, children } => json!({
            "type": "emphasis",
            "strong": strong,
            "text": node.text_content(),
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::InlineCode(code) => json!({
            "type": "inline_code",
            "code": code,
        }),
        Node::Text(text) => json!({
            "type": "text",
            "text": text,
        }),
        Node::ThematicBreak => json!({
            "type": "thematic_break",
        }),
        Node::LineBreak => json!({
            "type": "line_break",
        }),
        Node::Table { header, rows } => json!({
            "type": "table",
            "header": header.iter().map(node_to_json).collect::<Vec<_>>(),
            "rows": rows.iter().map(|row| {
                row.iter().map(node_to_json).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
        }),
        Node::TableRow(cells) => json!({
            "type": "table_row",
            "cells": cells.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
        Node::TableCell(children) => json!({
            "type": "table_cell",
            "children": children.iter().map(node_to_json).collect::<Vec<_>>(),
        }),
    }
}

fn format_markdown(results: &[MatchResult]) -> String {
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }
        output.push_str(&node_to_markdown(&result.node));
    }
    output
}

fn node_to_markdown(node: &Node) -> String {
    match node {
        Node::Heading { level, children } => {
            let prefix = "#".repeat(*level as usize);
            let text = render_inline_markdown(children);
            format!("{} {}", prefix, text)
        }
        Node::Paragraph(children) => render_inline_markdown(children),
        Node::CodeBlock { language, code } => {
            let lang = language.as_deref().unwrap_or("");
            format!("```{}\n{}\n```", lang, code)
        }
        Node::Blockquote(children) => {
            let inner: Vec<String> = children.iter().map(node_to_markdown).collect();
            inner.join("\n").lines().map(|l| format!("> {}", l)).collect::<Vec<_>>().join("\n")
        }
        Node::List { ordered, items } => items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let text = item.text_content();
                if *ordered {
                    format!("{}. {}", i + 1, text)
                } else {
                    format!("- {}", text)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Node::Link { text, url } => format!("[{}]({})", text, url),
        Node::Image { alt, url } => format!("![{}]({})", alt, url),
        Node::Emphasis { strong, children } => {
            let inner = render_inline_markdown(children);
            if *strong {
                format!("**{}**", inner)
            } else {
                format!("*{}*", inner)
            }
        }
        Node::InlineCode(code) => format!("`{}`", code),
        Node::Text(text) => text.clone(),
        Node::ThematicBreak => "---".to_string(),
        Node::LineBreak => "  ".to_string(),
        _ => node.text_content(),
    }
}

fn render_inline_markdown(nodes: &[Node]) -> String {
    nodes.iter().map(node_to_markdown).collect::<Vec<_>>().join("")
}

fn format_text(results: &[MatchResult]) -> String {
    results.iter().map(|r| r.node.text_content()).collect::<Vec<_>>().join("\n")
}

fn format_tree(results: &[MatchResult]) -> String {
    let mut output = String::new();
    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&format_tree_node(&result.node, "", i == results.len() - 1));
    }
    output
}

fn format_tree_node(node: &Node, prefix: &str, is_last: bool) -> String {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last { "    " } else { "│   " };

    let mut output = format!("{}{}{}\n", prefix, connector, node.type_name());

    // Add attributes
    match node {
        Node::Heading { level, .. } => {
            output.push_str(&format!("{}    level: {}\n", prefix, level));
        }
        Node::CodeBlock { language, .. } => {
            if let Some(lang) = language {
                output.push_str(&format!("{}    lang: {}\n", prefix, lang));
            }
        }
        Node::Link { url, .. } => {
            output.push_str(&format!("{}    url: {}\n", prefix, url));
        }
        Node::Image { url, .. } => {
            output.push_str(&format!("{}    url: {}\n", prefix, url));
        }
        Node::List { ordered, .. } => {
            output.push_str(&format!("{}    ordered: {}\n", prefix, ordered));
        }
        _ => {}
    }

    let children = node.children();
    for (i, child) in children.iter().enumerate() {
        let new_prefix = format!("{}{}", prefix, child_prefix);
        output.push_str(&format_tree_node(child, &new_prefix, i == children.len() - 1));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Parser;

    fn parse_and_format(md: &str, format: &str) -> String {
        let parser = Parser::new();
        let ast = parser.parse(md);
        let results = crate::query::executor::execute_query(&ast, "heading").unwrap();
        let fmt = OutputFormat::from_str(format);
        format_results(&results, &fmt, "heading")
    }

    #[test]
    fn test_json_output() {
        let output = parse_and_format("# Hello\n\nWorld", "json");
        assert!(output.contains("\"query\": \"heading\""));
        assert!(output.contains("\"Hello\""));
    }

    #[test]
    fn test_count_output() {
        let output = parse_and_format("# H1\n\n## H2\n\n### H3", "count");
        assert_eq!(output.trim(), "3");
    }

    #[test]
    fn test_text_output() {
        let output = parse_and_format("# Hello\n\n## World", "text");
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
    }

    #[test]
    fn test_markdown_output() {
        let output = parse_and_format("# Hello", "markdown");
        assert_eq!(output.trim(), "# Hello");
    }
}
