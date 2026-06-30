use crate::ast::Node;
use crate::query::{lex_query, Selector};

/// Execute a query against a Markdown AST and return matching nodes
pub fn execute_query(ast: &Node, query: &str) -> Result<Vec<MatchResult>, String> {
    let tokens = lex_query(query)?;
    let selector = Selector::parse(&tokens)?;
    let mut results = Vec::new();
    execute_selector(ast, &selector, &mut results, vec![]);
    Ok(results)
}

/// A result from query execution
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub node: Node,
    pub path: Vec<usize>,
    pub depth: usize,
}

fn execute_selector(
    node: &Node,
    selector: &Selector,
    results: &mut Vec<MatchResult>,
    path: Vec<usize>,
) {
    match selector {
        Selector::Type(type_name) => {
            if node.type_name() == type_name.as_str() {
                let depth = path.len();
                results.push(MatchResult {
                    node: node.clone(),
                    path: path.clone(),
                    depth,
                });
            }
            for (i, child) in node.children().iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(i);
                execute_selector(child, selector, results, new_path);
            }
        }
        Selector::WithAttr {
            node_type,
            key,
            value,
        } => {
            if node.type_name() == node_type.as_str() && node.matches_attribute(key, value) {
                let depth = path.len();
                results.push(MatchResult {
                    node: node.clone(),
                    path: path.clone(),
                    depth,
                });
            }
            for (i, child) in node.children().iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(i);
                execute_selector(child, selector, results, new_path);
            }
        }
        Selector::WithContains { node_type, text } => {
            if node.type_name() == node_type.as_str() {
                let content = node.text_content();
                if content.contains(text.as_str()) {
                    let depth = path.len();
                    results.push(MatchResult {
                        node: node.clone(),
                        path: path.clone(),
                        depth,
                    });
                }
            }
            for (i, child) in node.children().iter().enumerate() {
                let mut new_path = path.clone();
                new_path.push(i);
                execute_selector(child, selector, results, new_path);
            }
        }
        Selector::Child { parent, child } => {
            let mut parent_results = Vec::new();
            execute_selector(node, parent, &mut parent_results, path.clone());
            for parent_match in &parent_results {
                let parent_node = &parent_match.node;
                for (i, child_node) in parent_node.children().iter().enumerate() {
                    if node_matches(child_node, child) {
                        let mut new_path = parent_match.path.clone();
                        new_path.push(i);
                        let depth = new_path.len();
                        results.push(MatchResult {
                            node: (**child_node).clone(),
                            path: new_path,
                            depth,
                        });
                    }
                }
            }
        }
        Selector::Descendant {
            ancestor,
            descendant,
        } => {
            let mut ancestor_results = Vec::new();
            execute_selector(node, ancestor, &mut ancestor_results, path.clone());
            for ancestor_match in &ancestor_results {
                let ancestor_node = &ancestor_match.node;
                collect_descendants(ancestor_node, descendant, results, &ancestor_match.path);
            }
        }
        Selector::Sibling { before, after } => {
            let children = node.children();
            let mut prev_matched = false;
            for (i, child) in children.iter().enumerate() {
                if prev_matched {
                    if node_matches(child, after) {
                        let mut new_path = path.clone();
                        new_path.push(i);
                        let depth = new_path.len();
                        results.push(MatchResult {
                            node: (*child).clone(),
                            path: new_path,
                            depth,
                        });
                    }
                    prev_matched = false;
                }
                if node_matches(child, before) {
                    prev_matched = true;
                }
            }
        }
        Selector::Multi(selectors) => {
            for sel in selectors {
                execute_selector(node, sel, results, path.clone());
            }
        }
    }
}

fn collect_descendants(
    node: &Node,
    selector: &Selector,
    results: &mut Vec<MatchResult>,
    base_path: &[usize],
) {
    for (i, child) in node.children().iter().enumerate() {
        if node_matches(child, selector) {
            let mut new_path = base_path.to_vec();
            new_path.push(i);
            let depth = new_path.len();
            results.push(MatchResult {
                node: (*child).clone(),
                path: new_path,
                depth,
            });
        }
        let mut p = base_path.to_vec();
        p.push(i);
        collect_descendants(child, selector, results, &p);
    }
}

fn node_matches(node: &Node, selector: &Selector) -> bool {
    match selector {
        Selector::Type(type_name) => node.type_name() == type_name.as_str(),
        Selector::WithAttr {
            node_type,
            key,
            value,
        } => node.type_name() == node_type.as_str() && node.matches_attribute(key, value),
        Selector::WithContains { node_type, text } => {
            if node.type_name() != node_type.as_str() {
                return false;
            }
            node.text_content().contains(text.as_str())
        }
        Selector::Multi(selectors) => selectors.iter().any(|s| node_matches(node, s)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Parser;

    fn parse_and_query(md: &str, query: &str) -> Vec<MatchResult> {
        let parser = Parser::new();
        let ast = parser.parse(md);
        execute_query(&ast, query).unwrap()
    }

    #[test]
    fn test_query_headings() {
        let results = parse_and_query("# Title\n\n## Subtitle\n\nText", "heading");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_heading_level() {
        let results = parse_and_query("# H1\n\n## H2\n\n### H3", "heading[level=2]");
        assert_eq!(results.len(), 1);
        match &results[0].node {
            Node::Heading { level, .. } => assert_eq!(*level, 2),
            _ => panic!("Expected heading"),
        }
    }

    #[test]
    fn test_query_code_blocks() {
        let results = parse_and_query(
            "```python\nprint('hi')\n```\n\n```rust\nfn main() {}\n```",
            "code_block[lang=python]",
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_links() {
        let results = parse_and_query("Click [here](https://example.com) for info", "link");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_child_combinator() {
        let results = parse_and_query("- **bold item**\n- normal", "list_item > emphasis");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_descendant_combinator() {
        let md = "> This is a **bold** word in a blockquote";
        let results = parse_and_query(md, "blockquote emphasis");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_contains() {
        let results = parse_and_query(
            "# Hello World\n\n## Goodbye World",
            r#"heading[contains("Hello")]"#,
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_multi_type() {
        let results = parse_and_query(
            "# Heading\n\nParagraph\n\n```code```",
            "heading, code_block",
        );
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_no_matches() {
        let results = parse_and_query("# Title\n\nText", "table");
        assert!(results.is_empty());
    }
}
