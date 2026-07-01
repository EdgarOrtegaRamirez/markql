pub mod executor;

/// Token types for the query language lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Node type name (e.g., "heading", "paragraph")
    Ident(String),
    /// Quoted string value
    Quoted(String),
    /// Equals sign
    Equals,
    /// Child combinator: >
    Child,
    /// Sibling combinator: +
    Sibling,
    /// Opening bracket: [
    LBracket,
    /// Closing bracket: ]
    RBracket,
    /// Pipe for output format
    Pipe,
    /// Comma for multiple selectors
    Comma,
    /// End of input
    Eof,
}

/// Lex a query string into tokens
pub fn lex_query(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        match chars[i] {
            ' ' | '\t' => {
                i += 1;
            }
            '>' => {
                tokens.push(Token::Child);
                i += 1;
            }
            '+' => {
                tokens.push(Token::Sibling);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            '|' => {
                tokens.push(Token::Pipe);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '=' => {
                tokens.push(Token::Equals);
                i += 1;
            }
            '(' | ')' => {
                // Part of contains() syntax, skip
                i += 1;
            }
            '"' | '\'' => {
                let quote = chars[i];
                i += 1;
                let start = i;
                while i < len && chars[i] != quote {
                    i += 1;
                }
                if i < len {
                    let s: String = chars[start..i].iter().collect();
                    i += 1;
                    tokens.push(Token::Quoted(s));
                } else {
                    return Err("Unterminated string in query".to_string());
                }
            }
            _ if chars[i].is_alphabetic() || chars[i] == '_' => {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '-')
                {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(word));
            }
            _ if chars[i].is_ascii_digit() => {
                let start = i;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num: String = chars[start..i].iter().collect();
                tokens.push(Token::Ident(num));
            }
            _ => {
                return Err(format!("Unexpected character: '{}'", chars[i]));
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

/// Represents a query selector
#[derive(Debug, Clone)]
pub enum Selector {
    /// Match a specific node type
    Type(String),
    /// Match with attribute filter
    WithAttr {
        node_type: String,
        key: String,
        value: String,
    },
    /// Match with contains filter
    WithContains { node_type: String, text: String },
    /// Child combinator: parent > child
    Child {
        parent: Box<Selector>,
        child: Box<Selector>,
    },
    /// Descendant combinator: ancestor descendant
    Descendant {
        ancestor: Box<Selector>,
        descendant: Box<Selector>,
    },
    /// Sibling combinator: a + b
    Sibling {
        before: Box<Selector>,
        after: Box<Selector>,
    },
    /// Multiple selectors (comma-separated)
    Multi(Vec<Selector>),
}

impl Selector {
    /// Parse a selector from tokens
    pub fn parse(tokens: &[Token]) -> Result<Self, String> {
        let (sel, _) = parse_selector(tokens, 0)?;
        Ok(sel)
    }
}

fn parse_selector(tokens: &[Token], pos: usize) -> Result<(Selector, usize), String> {
    let (first, pos) = parse_simple_selector(tokens, pos)?;
    parse_combinator(tokens, pos, first)
}

fn parse_combinator(
    tokens: &[Token],
    mut pos: usize,
    mut left: Selector,
) -> Result<(Selector, usize), String> {
    loop {
        if pos >= tokens.len() {
            break;
        }
        match &tokens[pos] {
            Token::Child => {
                pos += 1;
                let (right, new_pos) = parse_simple_selector(tokens, pos)?;
                left = Selector::Child {
                    parent: Box::new(left),
                    child: Box::new(right),
                };
                pos = new_pos;
            }
            Token::Sibling => {
                pos += 1;
                let (right, new_pos) = parse_simple_selector(tokens, pos)?;
                left = Selector::Sibling {
                    before: Box::new(left),
                    after: Box::new(right),
                };
                pos = new_pos;
            }
            Token::Ident(_) => {
                // Descendant combinator (implicit space)
                let (right, new_pos) = parse_simple_selector(tokens, pos)?;
                left = Selector::Descendant {
                    ancestor: Box::new(left),
                    descendant: Box::new(right),
                };
                pos = new_pos;
            }
            Token::Comma => {
                // Multiple selectors
                pos += 1;
                let (right, new_pos) = parse_selector(tokens, pos)?;
                let combined = match (&left, &right) {
                    (Selector::Multi(v), _) => {
                        let mut v = v.clone();
                        v.push(right);
                        Selector::Multi(v)
                    }
                    (_, Selector::Multi(_)) => {
                        let v = vec![left, right];
                        Selector::Multi(v)
                    }
                    _ => Selector::Multi(vec![left, right]),
                };
                return Ok((combined, new_pos));
            }
            _ => break,
        }
    }
    Ok((left, pos))
}

fn parse_simple_selector(tokens: &[Token], pos: usize) -> Result<(Selector, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of query".to_string());
    }

    let node_type = match &tokens[pos] {
        Token::Ident(s) => s.clone(),
        _ => return Err(format!("Expected node type, got {:?}", tokens[pos])),
    };
    let mut pos = pos + 1;

    // Check for attribute filter: [key=value] or [contains("text")]
    if pos < tokens.len() && tokens[pos] == Token::LBracket {
        pos += 1;

        if pos < tokens.len() {
            match &tokens[pos] {
                Token::Ident(key) if key == "contains" => {
                    // contains("text") filter
                    pos += 1;
                    // Expect opening paren (already consumed by lexer)
                    if pos < tokens.len() {
                        if let Token::Quoted(t) = &tokens[pos] {
                            let text = t.clone();
                            pos += 1;
                            // Expect closing paren (already consumed by lexer)
                            if pos < tokens.len() && tokens[pos] == Token::RBracket {
                                pos += 1;
                            }
                            return Ok((Selector::WithContains { node_type, text }, pos));
                        }
                    }
                    return Err("Invalid contains filter syntax".to_string());
                }
                Token::Ident(key) => {
                    let key = key.clone();
                    pos += 1;
                    // Expect =
                    if pos < tokens.len() && tokens[pos] == Token::Equals {
                        pos += 1;
                    } else {
                        return Err(format!("Expected '=', got {:?}", tokens[pos]));
                    }
                    // Expect value (identifier or quoted string)
                    if pos < tokens.len() {
                        let value = match &tokens[pos] {
                            Token::Ident(v) => v.clone(),
                            Token::Quoted(v) => v.clone(),
                            _ => return Err(format!("Expected value, got {:?}", tokens[pos])),
                        };
                        pos += 1;
                        if pos < tokens.len() && tokens[pos] == Token::RBracket {
                            pos += 1;
                        }
                        return Ok((
                            Selector::WithAttr {
                                node_type,
                                key,
                                value,
                            },
                            pos,
                        ));
                    }
                    return Err("Invalid attribute filter syntax".to_string());
                }
                _ => return Err(format!("Expected attribute name, got {:?}", tokens[pos])),
            }
        }
        return Err("Invalid filter syntax".to_string());
    }

    Ok((Selector::Type(node_type), pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple() {
        let tokens = lex_query("heading").unwrap();
        assert_eq!(tokens.len(), 2); // ident + eof
        assert_eq!(tokens[0], Token::Ident("heading".to_string()));
    }

    #[test]
    fn test_lex_child_combinator() {
        let tokens = lex_query("list > heading").unwrap();
        assert!(tokens.contains(&Token::Child));
    }

    #[test]
    fn test_lex_equals() {
        let tokens = lex_query("heading[level=2]").unwrap();
        assert!(tokens.contains(&Token::Equals));
        assert!(tokens.contains(&Token::LBracket));
        assert!(tokens.contains(&Token::RBracket));
    }

    #[test]
    fn test_parse_type_selector() {
        let sel = Selector::parse(&[Token::Ident("heading".to_string()), Token::Eof]).unwrap();
        match sel {
            Selector::Type(s) => assert_eq!(s, "heading"),
            _ => panic!("Expected Type selector"),
        }
    }

    #[test]
    fn test_parse_attr_selector() {
        let sel = Selector::parse(&[
            Token::Ident("heading".to_string()),
            Token::LBracket,
            Token::Ident("level".to_string()),
            Token::Equals,
            Token::Ident("2".to_string()),
            Token::RBracket,
            Token::Eof,
        ])
        .unwrap();
        match sel {
            Selector::WithAttr {
                node_type,
                key,
                value,
            } => {
                assert_eq!(node_type, "heading");
                assert_eq!(key, "level");
                assert_eq!(value, "2");
            }
            _ => panic!("Expected WithAttr selector"),
        }
    }
}
