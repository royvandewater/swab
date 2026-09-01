use crate::syntax::Syntax;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub fn comment_spans(source: &str, syntax: &Syntax) -> Vec<Span> {
    let bytes = source.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if let Some((open, close)) = syntax.block.iter().find(|(open, _)| at(bytes, i, open)) {
            let body = i + open.len();
            let end = match find(bytes, body, close) {
                Some(pos) => pos + close.len(),
                None => bytes.len(),
            };
            spans.push(Span { start: i, end });
            i = end;
        } else if let Some(prefix) = syntax.line.iter().find(|prefix| at(bytes, i, prefix)) {
            let end = find(bytes, i + prefix.len(), "\n").unwrap_or(bytes.len());
            spans.push(Span { start: i, end });
            i = end;
        } else if let Some(rule) = syntax.strings.iter().find(|rule| at(bytes, i, rule.open)) {
            i = end_of_string(bytes, i, rule);
        } else {
            i += 1;
        }
    }

    spans
}

fn end_of_string(bytes: &[u8], start: usize, rule: &crate::syntax::StringRule) -> usize {
    let mut i = start + rule.open.len();
    while i < bytes.len() {
        if rule.escape && bytes[i] == b'\\' {
            i += 2;
        } else if at(bytes, i, rule.close) {
            return i + rule.close.len();
        } else if !rule.multiline && bytes[i] == b'\n' {
            return i;
        } else {
            i += 1;
        }
    }
    bytes.len()
}

fn at(bytes: &[u8], index: usize, needle: &str) -> bool {
    bytes[index..].starts_with(needle.as_bytes())
}

fn find(bytes: &[u8], from: usize, needle: &str) -> Option<usize> {
    (from..bytes.len()).find(|&i| at(bytes, i, needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Syntax;

    fn spans(path: &str, source: &str) -> Vec<String> {
        let syntax = Syntax::for_path(path).unwrap();
        comment_spans(source, syntax)
            .into_iter()
            .map(|span| source[span.start..span.end].to_string())
            .collect()
    }

    #[test]
    fn finds_a_line_comment() {
        assert_eq!(spans("a.rs", "let x = 1; // hi\n"), ["// hi"]);
    }

    #[test]
    fn line_comment_stops_at_newline() {
        assert_eq!(
            spans("a.rs", "// one\ncode\n// two\n"),
            ["// one", "// two"]
        );
    }

    #[test]
    fn finds_a_block_comment_spanning_lines() {
        assert_eq!(
            spans("a.rs", "a\n/* one\n   two */\nb\n"),
            ["/* one\n   two */"]
        );
    }

    #[test]
    fn unterminated_block_comment_runs_to_end_of_file() {
        assert_eq!(spans("a.rs", "a\n/* oops\n"), ["/* oops\n"]);
    }

    #[test]
    fn ignores_comment_markers_inside_strings() {
        assert_eq!(
            spans("a.rs", "let s = \"// not a comment\";\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn ignores_comment_markers_inside_block_delimited_strings() {
        assert_eq!(
            spans("a.rs", "let s = \"/* nope */\";\n"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn escaped_quotes_do_not_end_a_string() {
        assert_eq!(
            spans("a.rs", "let s = \"a\\\" // b\"; // real\n"),
            ["// real"]
        );
    }

    #[test]
    fn single_line_strings_do_not_swallow_the_rest_of_the_file() {
        assert_eq!(spans("a.rs", "let c = 'a;\n// real\n"), ["// real"]);
    }

    #[test]
    fn triple_quoted_python_strings_hide_hashes() {
        assert_eq!(
            spans("a.py", "s = \"\"\"# not\n# me\"\"\"\n# yes\n"),
            ["# yes"]
        );
    }

    #[test]
    fn ruby_begin_end_blocks_are_comments() {
        assert_eq!(
            spans("a.rb", "a\n=begin\ndoc\n=end\nb\n"),
            ["=begin\ndoc\n=end"]
        );
    }

    #[test]
    fn html_comments_span_lines() {
        assert_eq!(
            spans("a.html", "<p>x</p>\n<!-- hi\nthere -->\n"),
            ["<!-- hi\nthere -->"]
        );
    }

    #[test]
    fn sql_double_dash_is_a_line_comment() {
        assert_eq!(spans("a.sql", "select 1; -- why\n"), ["-- why"]);
    }
}
