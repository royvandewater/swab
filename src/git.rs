use std::collections::{HashMap, HashSet};

pub type AddedLines = HashMap<String, HashSet<usize>>;

pub fn added_lines(diff: &str) -> AddedLines {
    let mut files: AddedLines = HashMap::new();
    let mut current: Option<String> = None;

    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("+++ ") {
            current = target_path(rest);
        } else if let Some(rest) = line.strip_prefix("@@ ") {
            let Some(path) = current.clone() else { continue };
            let (start, count) = added_range(rest);
            if count > 0 {
                files.entry(path).or_default().extend(start..start + count);
            }
        }
    }

    files
}

fn target_path(rest: &str) -> Option<String> {
    let path = rest.split('\t').next()?;
    if path == "/dev/null" {
        return None;
    }
    Some(path.strip_prefix("b/").unwrap_or(path).to_string())
}

fn added_range(rest: &str) -> (usize, usize) {
    let Some(added) = rest.split_whitespace().find_map(|part| part.strip_prefix('+')) else {
        return (0, 0);
    };
    let (start, count) = match added.split_once(',') {
        Some((start, count)) => (start, count),
        None => (added, "1"),
    };
    match (start.parse(), count.parse()) {
        (Ok(start), Ok(count)) => (start, count),
        _ => (0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(diff: &str, path: &str) -> Vec<usize> {
        let mut found: Vec<usize> = added_lines(diff).remove(path).unwrap_or_default().into_iter().collect();
        found.sort();
        found
    }

    const ONE_FILE: &str = "diff --git a/src/main.rs b/src/main.rs\n\
index 0000000..1111111 100644\n\
--- a/src/main.rs\n\
+++ b/src/main.rs\n\
@@ -1,0 +2,3 @@ fn main() {\n\
+one\n\
+two\n\
+three\n";

    #[test]
    fn collects_added_line_numbers_from_a_hunk() {
        assert_eq!(lines(ONE_FILE, "src/main.rs"), [2, 3, 4]);
    }

    #[test]
    fn a_hunk_without_a_count_covers_one_line() {
        let diff = "--- a/a.rs\n+++ b/a.rs\n@@ -3 +7 @@\n+x\n";
        assert_eq!(lines(diff, "a.rs"), [7]);
    }

    #[test]
    fn merges_multiple_hunks_in_one_file() {
        let diff = "--- a/a.rs\n+++ b/a.rs\n@@ -1,0 +1,1 @@\n+x\n@@ -9,0 +11,2 @@\n+y\n+z\n";
        assert_eq!(lines(diff, "a.rs"), [1, 11, 12]);
    }

    #[test]
    fn separates_files() {
        let diff = "--- a/a.rs\n+++ b/a.rs\n@@ -0,0 +1,1 @@\n+x\n--- a/b.rs\n+++ b/b.rs\n@@ -0,0 +5,1 @@\n+y\n";
        assert_eq!(lines(diff, "a.rs"), [1]);
        assert_eq!(lines(diff, "b.rs"), [5]);
    }

    #[test]
    fn ignores_deleted_files() {
        let diff = "--- a/gone.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-x\n-y\n";
        assert!(added_lines(diff).is_empty());
    }

    #[test]
    fn ignores_hunks_that_only_remove_lines() {
        let diff = "--- a/a.rs\n+++ b/a.rs\n@@ -1,2 +0,0 @@\n-x\n-y\n";
        assert_eq!(lines(diff, "a.rs"), Vec::<usize>::new());
    }
}
