use std::collections::HashSet;

use crate::lexer::{Span, comment_spans};
use crate::syntax::Syntax;

pub struct Stripped {
    pub text: String,
    pub removed: usize,
}

pub fn strip(source: &str, syntax: &Syntax, added: &HashSet<usize>) -> Stripped {
    let lines = LineIndex::new(source);
    let mut cuts = Vec::new();

    for span in comment_spans(source, syntax) {
        let first = lines.line_of(span.start);
        let last = lines.line_of(span.end.saturating_sub(1));
        if !(first..=last).all(|line| added.contains(&line)) {
            continue;
        }
        cuts.push(cut(source, &lines, span, first, last));
    }

    let mut text = source.to_string();
    for range in cuts.iter().rev() {
        text.replace_range(range.clone(), "");
    }

    Stripped {
        text,
        removed: cuts.len(),
    }
}

fn cut(
    source: &str,
    lines: &LineIndex,
    span: Span,
    first: usize,
    last: usize,
) -> std::ops::Range<usize> {
    let line_start = lines.start_of(first);
    let line_end = lines.end_of(last);
    let prefix_blank = source[line_start..span.start].trim().is_empty();
    let suffix_blank = source[span.end..line_end].trim().is_empty();

    match (prefix_blank, suffix_blank) {
        (true, true) => line_start..source.len().min(line_end + 1),
        (true, false) => line_start..span.end + leading_blanks(&source[span.end..line_end]),
        _ => span.start - trailing_blanks(&source[line_start..span.start])..span.end,
    }
}

fn leading_blanks(text: &str) -> usize {
    text.len() - text.trim_start_matches([' ', '\t']).len()
}

fn trailing_blanks(text: &str) -> usize {
    text.len() - text.trim_end_matches([' ', '\t']).len()
}

struct LineIndex {
    starts: Vec<usize>,
    length: usize,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(source.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex {
            starts,
            length: source.len(),
        }
    }

    fn line_of(&self, offset: usize) -> usize {
        self.starts.partition_point(|&start| start <= offset)
    }

    fn start_of(&self, line: usize) -> usize {
        self.starts[line - 1]
    }

    fn end_of(&self, line: usize) -> usize {
        self.starts.get(line).map_or(self.length, |&next| next - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Syntax;

    fn swab(path: &str, source: &str, added: &[usize]) -> String {
        strip(
            source,
            Syntax::for_path(path).unwrap(),
            &added.iter().copied().collect(),
        )
        .text
    }

    #[test]
    fn removes_a_whole_line_comment_on_an_added_line() {
        assert_eq!(
            swab("a.rs", "let x = 1;\n// noise\nlet y = 2;\n", &[2]),
            "let x = 1;\nlet y = 2;\n"
        );
    }

    #[test]
    fn keeps_indentation_free_of_leftover_blank_lines() {
        assert_eq!(
            swab("a.rs", "fn f() {\n    // noise\n}\n", &[2]),
            "fn f() {\n}\n"
        );
    }

    #[test]
    fn strips_a_trailing_comment_but_keeps_the_code() {
        assert_eq!(swab("a.rs", "let x = 1; // noise\n", &[1]), "let x = 1;\n");
    }

    #[test]
    fn leaves_comments_on_unchanged_lines_alone() {
        assert_eq!(
            swab("a.rs", "// old\nlet x = 1;\n", &[2]),
            "// old\nlet x = 1;\n"
        );
    }

    #[test]
    fn removes_a_block_comment_whose_lines_are_all_added() {
        assert_eq!(
            swab("a.rs", "/* one\n   two */\nlet x = 1;\n", &[1, 2]),
            "let x = 1;\n"
        );
    }

    #[test]
    fn keeps_a_block_comment_that_is_only_partly_added() {
        let source = "/* one\n   two */\nlet x = 1;\n";
        assert_eq!(swab("a.rs", source, &[2]), source);
    }

    #[test]
    fn keeps_code_that_follows_a_block_comment_on_the_same_line() {
        assert_eq!(
            swab("a.rs", "/* noise */ let x = 1;\n", &[1]),
            "let x = 1;\n"
        );
    }

    #[test]
    fn handles_a_file_without_a_trailing_newline() {
        assert_eq!(swab("a.rs", "let x = 1;\n// noise", &[2]), "let x = 1;\n");
    }

    #[test]
    fn counts_the_comments_it_removed() {
        let source = "// a\n// b\nlet x = 1; // c\n";
        let added = [1, 2, 3].iter().copied().collect();
        assert_eq!(
            strip(source, Syntax::for_path("a.rs").unwrap(), &added).removed,
            3
        );
    }

    #[test]
    fn leaves_a_file_with_no_added_comments_untouched() {
        let source = "let x = 1;\n";
        assert_eq!(
            strip(
                source,
                Syntax::for_path("a.rs").unwrap(),
                &[1].into_iter().collect()
            )
            .removed,
            0
        );
    }
}
