use serde::{Deserialize, Serialize};

/// Type alias for table cells (vector of Node)
pub type TableCell = Node;

/// Represents a Markdown AST node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Node {
    Document(Vec<Node>),
    Heading {
        level: u8,
        children: Vec<Node>,
    },
    Paragraph(Vec<Node>),
    CodeBlock {
        language: Option<String>,
        code: String,
    },
    Blockquote(Vec<Node>),
    List {
        ordered: bool,
        items: Vec<Node>,
    },
    ListItem(Vec<Node>),
    Table {
        header: Vec<TableCell>,
        rows: Vec<Vec<TableCell>>,
    },
    TableRow(Vec<TableCell>),
    TableCell(Vec<Node>),
    ThematicBreak,
    Link {
        text: String,
        url: String,
    },
    Image {
        alt: String,
        url: String,
    },
    Emphasis {
        strong: bool,
        children: Vec<Node>,
    },
    InlineCode(String),
    Text(String),
    LineBreak,
}

impl Node {
    /// Get the node type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Node::Document(_) => "document",
            Node::Heading { .. } => "heading",
            Node::Paragraph(_) => "paragraph",
            Node::CodeBlock { .. } => "code_block",
            Node::Blockquote(_) => "blockquote",
            Node::List { .. } => "list",
            Node::ListItem(_) => "list_item",
            Node::Table { .. } => "table",
            Node::TableRow(_) => "table_row",
            Node::TableCell(_) => "table_cell",
            Node::ThematicBreak => "thematic_break",
            Node::Link { .. } => "link",
            Node::Image { .. } => "image",
            Node::Emphasis { .. } => "emphasis",
            Node::InlineCode(_) => "inline_code",
            Node::Text(_) => "text",
            Node::LineBreak => "line_break",
        }
    }

    /// Get children of this node
    pub fn children(&self) -> Vec<&Node> {
        match self {
            Node::Document(nodes) | Node::Paragraph(nodes) | Node::Blockquote(nodes) => {
                nodes.iter().collect()
            }
            Node::Heading { children, .. } | Node::Emphasis { children, .. } => {
                children.iter().collect()
            }
            Node::List { items, .. } => items.iter().collect(),
            Node::ListItem(children) => children.iter().collect(),
            Node::Table { header, rows } => {
                let mut result: Vec<&Node> = header.iter().collect();
                for row in rows {
                    result.extend(row.iter());
                }
                result
            }
            Node::TableRow(cells) => cells.iter().collect(),
            Node::TableCell(nodes) => nodes.iter().collect(),
            _ => vec![],
        }
    }

    /// Get text content of this node (recursively)
    pub fn text_content(&self) -> String {
        match self {
            Node::Text(s) => s.clone(),
            Node::InlineCode(s) => s.clone(),
            Node::CodeBlock { code, .. } => code.clone(),
            Node::Link { text, .. } => text.clone(),
            Node::Image { alt, .. } => alt.clone(),
            Node::ThematicBreak | Node::LineBreak => String::new(),
            _ => {
                let mut result = String::new();
                for child in self.children() {
                    let t = child.text_content();
                    if !result.is_empty() && !t.is_empty() {
                        result.push(' ');
                    }
                    result.push_str(&t);
                }
                result
            }
        }
    }

    /// Get attribute value by name
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        match (name, self) {
            ("level", Node::Heading { level, .. }) => Some(level.to_string()),
            ("lang", Node::CodeBlock { language, .. }) => language.clone(),
            ("url", Node::Link { url, .. }) => Some(url.clone()),
            ("url", Node::Image { url, .. }) => Some(url.clone()),
            ("alt", Node::Image { alt, .. }) => Some(alt.clone()),
            ("text", Node::Link { text, .. }) => Some(text.clone()),
            ("text", Node::Image { alt, .. }) => Some(alt.clone()),
            ("strong", Node::Emphasis { strong, .. }) => Some(strong.to_string()),
            ("ordered", Node::List { ordered, .. }) => Some(ordered.to_string()),
            ("text", Node::Text(s)) => Some(s.clone()),
            ("code", Node::InlineCode(s)) => Some(s.clone()),
            ("code", Node::CodeBlock { code, .. }) => Some(code.clone()),
            _ => None,
        }
    }

    /// Check if this node matches a simple attribute condition
    pub fn matches_attribute(&self, name: &str, value: &str) -> bool {
        if let Some(attr_val) = self.get_attribute(name) {
            attr_val == value
        } else {
            false
        }
    }
}
