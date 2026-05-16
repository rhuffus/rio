//! Managed block markers and operations.
//!
//! A rio-managed file contains a delimited block whose contents are owned by
//! the plugin. Everything outside the block belongs to the user.

use anyhow::{Result, anyhow};

pub const BLOCK_OPEN: &str = "# >>> RhuffusIO Managed Block >>>";
pub const BLOCK_CLOSE: &str = "# <<< RhuffusIO Managed Block <<<";

/// Location of the rio-managed block within a file's text.
#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    /// 0-indexed line of the opening marker.
    pub start_line: usize,
    /// 0-indexed line of the closing marker.
    pub end_line: usize,
    /// Inner content between the markers (each line followed by `\n`).
    pub content: String,
}

/// Locate the managed block. Returns `Ok(None)` if no markers are present.
pub fn find(text: &str) -> Result<Option<Block>> {
    let mut open: Option<usize> = None;
    let mut close: Option<usize> = None;

    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed == BLOCK_OPEN {
            if open.is_some() {
                return Err(anyhow!(
                    "malformed block: duplicate open marker at line {}",
                    i + 1
                ));
            }
            open = Some(i);
        } else if trimmed == BLOCK_CLOSE {
            if open.is_none() {
                return Err(anyhow!(
                    "malformed block: close marker without open at line {}",
                    i + 1
                ));
            }
            if close.is_some() {
                return Err(anyhow!(
                    "malformed block: duplicate close marker at line {}",
                    i + 1
                ));
            }
            close = Some(i);
        }
    }

    match (open, close) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(anyhow!("malformed block: open without matching close")),
        (Some(s), Some(e)) => {
            let content: String = text
                .lines()
                .skip(s + 1)
                .take(e.saturating_sub(s + 1))
                .map(|l| format!("{l}\n"))
                .collect();
            Ok(Some(Block {
                start_line: s,
                end_line: e,
                content,
            }))
        }
        _ => unreachable!("close before open is caught earlier"),
    }
}

/// Insert or replace the managed block in `text`. When `text` has no block,
/// the block is appended at the end (separated by a blank line if `text` is
/// non-empty and doesn't already end with one).
pub fn upsert(text: &str, content: &str) -> Result<String> {
    let rendered = render_block(content);
    match find(text)? {
        Some(block) => Ok(replace(text, &block, &rendered)),
        None => Ok(append(text, &rendered)),
    }
}

/// Remove the managed block. Returns `text` verbatim when no block is present.
pub fn remove(text: &str) -> Result<String> {
    let Some(block) = find(text)? else {
        return Ok(text.to_string());
    };
    Ok(replace(text, &block, ""))
}

fn render_block(content: &str) -> String {
    let mut s = String::with_capacity(content.len() + BLOCK_OPEN.len() + BLOCK_CLOSE.len() + 4);
    s.push_str(BLOCK_OPEN);
    s.push('\n');
    s.push_str(content);
    if !content.is_empty() && !content.ends_with('\n') {
        s.push('\n');
    }
    s.push_str(BLOCK_CLOSE);
    s.push('\n');
    s
}

fn replace(text: &str, block: &Block, replacement: &str) -> String {
    let mut out = String::new();
    let mut wrote_replacement = false;
    for (i, line) in text.split_inclusive('\n').enumerate() {
        if i < block.start_line {
            out.push_str(line);
        } else if i == block.start_line {
            out.push_str(replacement);
            wrote_replacement = true;
        } else if i > block.end_line {
            out.push_str(line);
        }
    }
    if !wrote_replacement {
        out.push_str(replacement);
    }
    out
}

fn append(text: &str, rendered: &str) -> String {
    if text.is_empty() {
        return rendered.to_string();
    }
    let mut out = String::with_capacity(text.len() + rendered.len() + 2);
    out.push_str(text);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(rendered);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn join(parts: &[&str]) -> String {
        let mut s = String::new();
        for p in parts {
            s.push_str(p);
            s.push('\n');
        }
        s
    }

    #[test]
    fn find_returns_none_on_empty() {
        assert_eq!(find("").unwrap(), None);
    }

    #[test]
    fn find_returns_none_when_no_block() {
        assert_eq!(find("hello\nworld\n").unwrap(), None);
    }

    #[test]
    fn find_locates_block_with_content() {
        let text = join(&["before", BLOCK_OPEN, "export FOO=bar", BLOCK_CLOSE, "after"]);
        let block = find(&text).unwrap().unwrap();
        assert_eq!(block.start_line, 1);
        assert_eq!(block.end_line, 3);
        assert_eq!(block.content, "export FOO=bar\n");
    }

    #[test]
    fn find_locates_empty_block() {
        let text = join(&[BLOCK_OPEN, BLOCK_CLOSE]);
        let block = find(&text).unwrap().unwrap();
        assert_eq!(block.content, "");
    }

    #[test]
    fn find_rejects_open_without_close() {
        let text = join(&[BLOCK_OPEN, "hello"]);
        assert!(find(&text).is_err());
    }

    #[test]
    fn find_rejects_close_without_open() {
        let text = join(&[BLOCK_CLOSE]);
        assert!(find(&text).is_err());
    }

    #[test]
    fn find_rejects_duplicate_open() {
        let text = join(&[BLOCK_OPEN, BLOCK_OPEN, BLOCK_CLOSE]);
        assert!(find(&text).is_err());
    }

    #[test]
    fn upsert_appends_to_empty_file() {
        let result = upsert("", "alias gs='git status'").unwrap();
        let block = find(&result).unwrap().unwrap();
        assert_eq!(block.content, "alias gs='git status'\n");
    }

    #[test]
    fn upsert_appends_to_file_without_block() {
        let result = upsert("user line\n", "managed").unwrap();
        let block = find(&result).unwrap().unwrap();
        assert_eq!(block.content, "managed\n");
        assert!(result.starts_with("user line\n"));
    }

    #[test]
    fn upsert_replaces_existing_block() {
        let initial = join(&["first", BLOCK_OPEN, "old", BLOCK_CLOSE, "last"]);
        let result = upsert(&initial, "new").unwrap();
        let block = find(&result).unwrap().unwrap();
        assert_eq!(block.content, "new\n");
        assert!(result.starts_with("first\n"));
        assert!(result.ends_with("last\n"));
    }

    #[test]
    fn remove_strips_block_keeping_surroundings() {
        let initial = join(&["first", BLOCK_OPEN, "inner", BLOCK_CLOSE, "last"]);
        let result = remove(&initial).unwrap();
        assert!(!result.contains(BLOCK_OPEN));
        assert!(!result.contains(BLOCK_CLOSE));
        assert_eq!(result, "first\nlast\n");
    }

    #[test]
    fn remove_is_noop_when_no_block() {
        let initial = "hello\nworld\n";
        assert_eq!(remove(initial).unwrap(), initial);
    }
}
