use crate::ast::Node;

/// Markdown tokenizer that converts raw markdown text into a tree of AST nodes.
/// Uses a line-by-line approach for block-level elements, then inline parsing
/// for inline elements within blocks.
pub struct Parser;

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Parser
    }

    /// Parse markdown text into an AST
    pub fn parse(&self, input: &str) -> Node {
        let lines: Vec<&str> = input.lines().collect();
        let mut nodes = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            // Heading: # ... ######
            if let Some(heading) = parse_heading(trimmed) {
                nodes.push(heading);
                i += 1;
                continue;
            }

            // Thematic break: ---, ***, ___
            if is_thematic_break(trimmed) {
                nodes.push(Node::ThematicBreak);
                i += 1;
                continue;
            }

            // Fenced code block: ``` or ~~~
            if let Some((code_block, lines_consumed)) = parse_code_block(&lines, i) {
                nodes.push(code_block);
                i += lines_consumed;
                continue;
            }

            // Blockquote: >
            if trimmed.starts_with('>') {
                let (blockquote, lines_consumed) = parse_blockquote(&lines, i);
                nodes.push(blockquote);
                i += lines_consumed;
                continue;
            }

            // Table: | ... |
            if trimmed.starts_with('|')
                && i + 1 < lines.len()
                && lines[i + 1].trim().starts_with('|')
            {
                if let Some((table, lines_consumed)) = parse_table(&lines, i) {
                    nodes.push(table);
                    i += lines_consumed;
                    continue;
                }
            }

            // List: - item or 1. item
            if is_list_item(trimmed) {
                let (list, lines_consumed) = parse_list(&lines, i);
                nodes.push(list);
                i += lines_consumed;
                continue;
            }

            // Paragraph: anything else
            let (paragraph, lines_consumed) = parse_paragraph(&lines, i);
            nodes.push(paragraph);
            i += lines_consumed;
        }

        Node::Document(nodes)
    }
}

fn parse_heading(line: &str) -> Option<Node> {
    let level = line.chars().take_while(|&c| c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let text = line[level..].trim();
    if text.is_empty() {
        return None;
    }
    Some(Node::Heading {
        level: level as u8,
        children: parse_inline(text),
    })
}

fn is_thematic_break(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    if chars.len() < 3 {
        return false;
    }
    let c = chars[0];
    if c != '-' && c != '*' && c != '_' {
        return false;
    }
    chars.iter().all(|&ch| ch == c || ch == ' ')
}

fn parse_code_block(lines: &[&str], start: usize) -> Option<(Node, usize)> {
    let line = lines[start].trim();
    let (fence, fence_len) = if line.starts_with("```") {
        ("```", 3)
    } else if line.starts_with("~~~") {
        ("~~~", 3)
    } else {
        return None;
    };

    // Check if it's just the fence (opening with no content)
    let language = if line.len() > fence_len {
        let lang = line[fence_len..].trim();
        if lang.is_empty() {
            None
        } else {
            Some(lang.to_string())
        }
    } else {
        None
    };

    let mut code_lines = Vec::new();
    let mut i = start + 1;
    let mut consumed = 1; // opening fence

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.starts_with(fence) && trimmed.len() >= fence_len {
            // Check if it's a closing fence (no more than the fence chars + optional spaces)
            let after_fence = &trimmed[fence_len..];
            if after_fence.trim().is_empty() {
                consumed += 1; // closing fence
                break;
            }
        }
        code_lines.push(lines[i]);
        consumed += 1;
        i += 1;
    }

    Some((
        Node::CodeBlock {
            language,
            code: code_lines.join("\n"),
        },
        consumed,
    ))
}

fn parse_blockquote(lines: &[&str], start: usize) -> (Node, usize) {
    let mut content_lines = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(stripped) = trimmed.strip_prefix('>') {
            let after = stripped.trim();
            content_lines.push(after);
            i += 1;
        } else if !trimmed.is_empty() && !content_lines.is_empty() {
            // Continuation of blockquote
            content_lines.push(trimmed);
            i += 1;
        } else {
            break;
        }
    }

    let inner_text = content_lines.join("\n");
    let inner_nodes = parse_inline_block(&inner_text);

    (Node::Blockquote(inner_nodes), i - start)
}

fn parse_table(lines: &[&str], start: usize) -> Option<(Node, usize)> {
    let header_line = lines[start].trim();
    if !header_line.starts_with('|') {
        return None;
    }

    // Check for separator line
    if start + 1 >= lines.len() {
        return None;
    }
    let sep_line = lines[start + 1].trim();
    if !sep_line.starts_with('|') || !sep_line.contains("---") {
        return None;
    }

    let header = parse_table_row(header_line);
    let mut rows = Vec::new();
    let mut i = start + 2;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || !line.starts_with('|') {
            break;
        }
        rows.push(parse_table_row(line));
        i += 1;
    }

    Some((Node::Table { header, rows }, i - start))
}

fn parse_table_row(line: &str) -> Vec<crate::ast::Node> {
    let content = line.trim_matches('|');
    content
        .split('|')
        .map(|cell| {
            let cell = cell.trim();
            Node::TableCell(parse_inline(cell))
        })
        .collect()
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim();
    // Unordered: -, *, +
    if (trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ "))
        && !trimmed.starts_with("---")
        && !trimmed.starts_with("***")
        && !trimmed.starts_with("___")
    {
        return true;
    }
    // Ordered: 1. 2. etc
    let bytes = trimmed.as_bytes();
    if !bytes.is_empty() && bytes[0].is_ascii_digit() {
        let dot_pos = trimmed.find(". ");
        if let Some(pos) = dot_pos {
            if pos > 0 && trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    false
}

fn parse_list(lines: &[&str], start: usize) -> (Node, usize) {
    let first_line = lines[start].trim();
    let ordered = first_line
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit());

    let mut items = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }

        let item_text = if ordered {
            let dot_pos = trimmed.find(". ").unwrap_or(0);
            &trimmed[dot_pos + 2..]
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
        {
            &trimmed[2..]
        } else {
            break;
        };

        let children = parse_inline(item_text);
        items.push(Node::ListItem(children));
        i += 1;
    }

    (Node::List { ordered, items }, i - start)
}

fn parse_paragraph(lines: &[&str], start: usize) -> (Node, usize) {
    let mut content_lines = Vec::new();
    let mut i = start;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            break;
        }
        // Stop if we hit a block-level element
        if trimmed.starts_with('#')
            || trimmed.starts_with('>')
            || trimmed.starts_with("```")
            || trimmed.starts_with("~~~")
            || trimmed.starts_with('|')
            || is_thematic_break(trimmed)
            || is_list_item(trimmed)
        {
            break;
        }
        content_lines.push(trimmed);
        i += 1;
    }

    let text = content_lines.join("\n");
    (Node::Paragraph(parse_inline(&text)), i - start)
}

/// Parse inline markdown elements into a list of nodes
pub fn parse_inline(text: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut current_text = String::new();

    while i < len {
        // Inline code: `code`
        if chars[i] == '`' {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            let start = i + 1;
            i += 1;
            while i < len && chars[i] != '`' {
                i += 1;
            }
            if i < len {
                let code: String = chars[start..i].iter().collect();
                nodes.push(Node::InlineCode(code));
                i += 1;
            } else {
                current_text.push('`');
                i = start;
            }
            continue;
        }

        // Image: ![alt](url)
        if chars[i] == '!' && i + 1 < len && chars[i + 1] == '[' {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            let alt_start = i + 2;
            i += 2;
            while i < len && chars[i] != ']' {
                i += 1;
            }
            if i < len && i + 1 < len && chars[i + 1] == '(' {
                let alt: String = chars[alt_start..i].iter().collect();
                i += 2; // skip ](
                let url_start = i;
                while i < len && chars[i] != ')' {
                    i += 1;
                }
                if i < len {
                    let url: String = chars[url_start..i].iter().collect();
                    nodes.push(Node::Image { alt, url });
                    i += 1;
                    continue;
                }
            }
            // Fall through if malformed
            current_text.push_str("!]");
            i = alt_start;
            continue;
        }

        // Link: [text](url)
        if chars[i] == '[' {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            let text_start = i + 1;
            i += 1;
            while i < len && chars[i] != ']' {
                i += 1;
            }
            if i < len && i + 1 < len && chars[i + 1] == '(' {
                let link_text: String = chars[text_start..i].iter().collect();
                i += 2; // skip ](
                let url_start = i;
                while i < len && chars[i] != ')' {
                    i += 1;
                }
                if i < len {
                    let url: String = chars[url_start..i].iter().collect();
                    nodes.push(Node::Link {
                        text: link_text,
                        url,
                    });
                    i += 1;
                    continue;
                }
            }
            // Fall through if malformed
            current_text.push('[');
            i = text_start;
            continue;
        }

        // Strong: **text** or __text__
        if i + 1 < len
            && (chars[i] == '*' && chars[i + 1] == '*' || chars[i] == '_' && chars[i + 1] == '_')
        {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            let delim = chars[i];
            let text_start = i + 2;
            i += 2;
            while i + 1 < len && !(chars[i] == delim && chars[i + 1] == delim) {
                i += 1;
            }
            if i + 1 < len {
                let inner: String = chars[text_start..i].iter().collect();
                nodes.push(Node::Emphasis {
                    strong: true,
                    children: parse_inline(&inner),
                });
                i += 2;
                continue;
            }
            // Fall through
            current_text.push_str(&format!("{}{}", delim, delim));
            i = text_start;
            continue;
        }

        // Emphasis: *text* or _text_
        if chars[i] == '*' || chars[i] == '_' {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            let delim = chars[i];
            let text_start = i + 1;
            i += 1;
            while i < len && chars[i] != delim {
                i += 1;
            }
            if i < len {
                let inner: String = chars[text_start..i].iter().collect();
                nodes.push(Node::Emphasis {
                    strong: false,
                    children: parse_inline(&inner),
                });
                i += 1;
                continue;
            }
            // Fall through
            current_text.push(delim);
            i = text_start;
            continue;
        }

        // Line break: two trailing spaces or backslash
        if chars[i] == '\\' && i + 1 < len && chars[i + 1] == '\n' {
            if !current_text.is_empty() {
                nodes.push(Node::Text(current_text.clone()));
                current_text.clear();
            }
            nodes.push(Node::LineBreak);
            i += 2;
            continue;
        }

        current_text.push(chars[i]);
        i += 1;
    }

    if !current_text.is_empty() {
        nodes.push(Node::Text(current_text));
    }

    nodes
}

/// Parse a block of text that may contain multiple paragraphs separated by blank lines
fn parse_inline_block(text: &str) -> Vec<Node> {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut nodes = Vec::new();

    for para in paragraphs {
        let trimmed = para.trim();
        if !trimmed.is_empty() {
            let inline_nodes = parse_inline(trimmed);
            if inline_nodes.len() == 1 {
                nodes.push(inline_nodes.into_iter().next().unwrap());
            } else {
                nodes.push(Node::Paragraph(inline_nodes));
            }
        }
    }

    if nodes.is_empty() {
        nodes.push(Node::Paragraph(vec![Node::Text(String::new())]));
    }

    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        let parser = Parser::new();
        let doc = parser.parse("# Hello World");
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    Node::Heading { level, children } => {
                        assert_eq!(*level, 1);
                        assert_eq!(children.len(), 1);
                        match &children[0] {
                            Node::Text(t) => assert_eq!(t, "Hello World"),
                            _ => panic!("Expected text node"),
                        }
                    }
                    _ => panic!("Expected heading node"),
                }
            }
            _ => panic!("Expected document node"),
        }
    }

    #[test]
    fn test_parse_code_block() {
        let parser = Parser::new();
        let md = "```python\nprint('hello')\n```";
        let doc = parser.parse(md);
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    Node::CodeBlock { language, code } => {
                        assert_eq!(language.as_deref(), Some("python"));
                        assert_eq!(code, "print('hello')");
                    }
                    _ => panic!("Expected code block"),
                }
            }
            _ => panic!("Expected document"),
        }
    }

    #[test]
    fn test_parse_list() {
        let parser = Parser::new();
        let md = "- Item 1\n- Item 2\n- Item 3";
        let doc = parser.parse(md);
        match doc {
            Node::Document(nodes) => {
                assert_eq!(nodes.len(), 1);
                match &nodes[0] {
                    Node::List { ordered, items } => {
                        assert!(!ordered);
                        assert_eq!(items.len(), 3);
                    }
                    _ => panic!("Expected list"),
                }
            }
            _ => panic!("Expected document"),
        }
    }

    #[test]
    fn test_parse_inline_link() {
        let nodes = parse_inline("Click [here](https://example.com) for more");
        assert_eq!(nodes.len(), 3);
        match &nodes[1] {
            Node::Link { text, url } => {
                assert_eq!(text, "here");
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Expected link"),
        }
    }

    #[test]
    fn test_parse_inline_emphasis() {
        let nodes = parse_inline("This is *italic* and **bold**");
        assert!(nodes
            .iter()
            .any(|n| matches!(n, Node::Emphasis { strong: false, .. })));
        assert!(nodes
            .iter()
            .any(|n| matches!(n, Node::Emphasis { strong: true, .. })));
    }
}
