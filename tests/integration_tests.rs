use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

fn run_markql(args: &[&str], input: &str) -> String {
    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .current_dir("/root/workspace/markql")
        .spawn()
        .expect("Failed to run markql");

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(input.as_bytes()).ok();
    }

    let output = child.wait_with_output().expect("Failed to wait for output");
    String::from_utf8(output.stdout).unwrap_or_default()
}

#[test]
fn test_query_headings_from_stdin() {
    let input = "# Title\n\n## Subtitle\n\nSome text";
    let output = run_markql(&["query", "heading", "--format", "count"], input);
    assert_eq!(output.trim(), "2");
}

#[test]
fn test_query_code_blocks_from_stdin() {
    let input = "```python\nprint('hello')\n```\n\n```rust\nfn main() {}\n```";
    let output = run_markql(&["query", "code_block[lang=python]", "--format", "json"], input);
    assert!(output.contains("\"python\""));
}

#[test]
fn test_query_links_from_stdin() {
    let input = "Click [here](https://example.com) for more info";
    let output = run_markql(&["query", "link", "--format", "text"], input);
    assert!(output.contains("here"));
}

#[test]
fn test_query_heading_level() {
    let input = "# H1\n\n## H2\n\n### H3";
    let output = run_markql(&["query", "heading[level=2]", "--format", "count"], input);
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_query_contains() {
    let input = "# Hello World\n\n## Goodbye World";
    let output = run_markql(&["query", "heading[contains(\"Hello\")]", "--format", "count"], input);
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_query_multi_type() {
    let input = "# Heading\n\nParagraph\n\n```code```";
    let output = run_markql(&["query", "heading, code_block", "--format", "count"], input);
    assert_eq!(output.trim(), "2");
}

#[test]
fn test_stats_output() {
    let input = "# Title\n\nParagraph text\n\n```code```";
    let output = run_markql(&["stats"], input);
    assert!(output.contains("heading"));
    assert!(output.contains("paragraph"));
    assert!(output.contains("code_block"));
}

#[test]
fn test_types_output() {
    let output = run_markql(&["types"], "");
    assert!(output.contains("heading"));
    assert!(output.contains("paragraph"));
    assert!(output.contains("code_block"));
    assert!(output.contains("link"));
    assert!(output.contains("emphasis"));
}

#[test]
fn test_file_input() {
    let mut tmpfile = NamedTempFile::new().unwrap();
    writeln!(tmpfile, "# Title\n\n## Subtitle").unwrap();
    
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("query")
        .arg("heading")
        .arg("--file")
        .arg(tmpfile.path())
        .arg("--format")
        .arg("count")
        .current_dir("/root/workspace/markql")
        .output()
        .expect("Failed to run markql");
    
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "2");
}

#[test]
fn test_json_output_is_valid_json() {
    let input = "# Title\n\nSome text";
    let output = run_markql(&["query", "heading", "--format", "json"], input);
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("Invalid JSON output");
    assert_eq!(parsed["query"], "heading");
    assert_eq!(parsed["count"], 1);
}

#[test]
fn test_child_combinator() {
    let input = "- **bold item**\n- normal";
    let output = run_markql(&["query", "list_item > emphasis", "--format", "count"], input);
    assert_eq!(output.trim(), "1");
}

#[test]
fn test_descendant_combinator() {
    let input = "> This is **bold** text";
    let output = run_markql(&["query", "blockquote emphasis", "--format", "count"], input);
    assert_eq!(output.trim(), "1");
}
