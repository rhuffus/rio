//! tree-sitter-bash wrapper.
//!
//! Exposes a parser and a helper that walks the AST to extract top-level
//! "stanzas" (variable assignments, function definitions, and other top-level
//! statements). Stanzas are the unit of comparison the reconciler uses to
//! detect drift between the managed block and the rest of the file.

use anyhow::{Result, anyhow};
use std::ops::Range;
use tree_sitter::{Node, Parser, Tree};

pub fn parse(source: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    let language = tree_sitter_bash::language();
    parser
        .set_language(&language)
        .map_err(|e| anyhow!("setting bash language: {e}"))?;
    parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("tree-sitter failed to parse input"))
}

#[derive(Debug, PartialEq, Eq)]
pub struct Stanza {
    pub kind: StanzaKind,
    pub text: String,
    pub byte_range: Range<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StanzaKind {
    Assignment { name: String },
    Function { name: String },
    Other,
}

/// Walk top-level children of the parsed program and classify each.
pub fn top_level_stanzas(source: &str) -> Result<Vec<Stanza>> {
    let tree = parse(source)?;
    let root = tree.root_node();
    let mut out = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        let range = child.byte_range();
        let text = source[range.clone()].to_string();
        let kind = classify(child, source);
        out.push(Stanza {
            kind,
            text,
            byte_range: range,
        });
    }
    Ok(out)
}

fn classify(node: Node, source: &str) -> StanzaKind {
    match node.kind() {
        "variable_assignment" => node
            .child_by_field_name("name")
            .map(|n| StanzaKind::Assignment {
                name: source[n.byte_range()].to_string(),
            })
            .unwrap_or(StanzaKind::Other),
        "function_definition" => node
            .child_by_field_name("name")
            .map(|n| StanzaKind::Function {
                name: source[n.byte_range()].to_string(),
            })
            .unwrap_or(StanzaKind::Other),
        _ => StanzaKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_empty_input() {
        let tree = parse("").unwrap();
        assert_eq!(tree.root_node().kind(), "program");
    }

    #[test]
    fn parses_simple_assignment() {
        let stanzas = top_level_stanzas("FOO=bar\n").unwrap();
        assert_eq!(stanzas.len(), 1);
        assert_eq!(
            stanzas[0].kind,
            StanzaKind::Assignment {
                name: "FOO".to_string()
            }
        );
    }

    #[test]
    fn parses_function_definition() {
        let stanzas = top_level_stanzas("myfn() { echo hi; }\n").unwrap();
        assert_eq!(stanzas.len(), 1);
        assert_eq!(
            stanzas[0].kind,
            StanzaKind::Function {
                name: "myfn".to_string()
            }
        );
    }

    #[test]
    fn parses_multiple_top_level_statements() {
        let source = "FOO=bar\nalias gs='git status'\nBAR=baz\n";
        let stanzas = top_level_stanzas(source).unwrap();
        assert_eq!(stanzas.len(), 3);
        assert!(matches!(
            stanzas[0].kind,
            StanzaKind::Assignment { ref name } if name == "FOO"
        ));
        assert!(matches!(stanzas[1].kind, StanzaKind::Other));
        assert!(matches!(
            stanzas[2].kind,
            StanzaKind::Assignment { ref name } if name == "BAR"
        ));
    }

    #[test]
    fn assignment_text_is_preserved() {
        let source = "  FOO=bar  \n";
        let stanzas = top_level_stanzas(source).unwrap();
        assert_eq!(stanzas.len(), 1);
        // Text reflects the byte range of the node, which excludes leading
        // whitespace tokens but includes the assignment itself.
        assert!(stanzas[0].text.contains("FOO=bar"));
    }
}
