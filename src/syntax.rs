pub struct StringRule {
    pub open: &'static str,
    pub close: &'static str,
    pub escape: bool,
    pub multiline: bool,
}

pub struct Syntax {
    pub line: &'static [&'static str],
    pub block: &'static [(&'static str, &'static str)],
    pub strings: &'static [StringRule],
}

const fn quoted(open: &'static str, close: &'static str, multiline: bool) -> StringRule {
    StringRule { open, close, escape: true, multiline }
}

const C_STRINGS: &[StringRule] = &[
    quoted("\"", "\"", false),
    quoted("'", "'", false),
    quoted("`", "`", true),
];

const C_FAMILY: Syntax = Syntax {
    line: &["//"],
    block: &[("/*", "*/")],
    strings: C_STRINGS,
};

const PYTHON: Syntax = Syntax {
    line: &["#"],
    block: &[],
    strings: &[
        quoted("\"\"\"", "\"\"\"", true),
        quoted("'''", "'''", true),
        quoted("\"", "\"", false),
        quoted("'", "'", false),
    ],
};

const SHELL: Syntax = Syntax {
    line: &["#"],
    block: &[],
    strings: &[quoted("\"", "\"", true), quoted("'", "'", true)],
};

const RUBY: Syntax = Syntax {
    line: &["#"],
    block: &[("=begin", "=end")],
    strings: &[quoted("\"", "\"", false), quoted("'", "'", false)],
};

const HTML: Syntax = Syntax {
    line: &[],
    block: &[("<!--", "-->")],
    strings: &[],
};

const CSS: Syntax = Syntax {
    line: &[],
    block: &[("/*", "*/")],
    strings: &[quoted("\"", "\"", false), quoted("'", "'", false)],
};

const SQL: Syntax = Syntax {
    line: &["--"],
    block: &[("/*", "*/")],
    strings: &[quoted("'", "'", false), quoted("\"", "\"", false)],
};

impl Syntax {
    pub fn for_path(path: &str) -> Option<&'static Syntax> {
        match extension(path)?.to_ascii_lowercase().as_str() {
            "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "go" | "java" | "c" | "h"
            | "cc" | "cpp" | "hpp" | "cs" | "swift" | "kt" | "kts" | "scala" | "php" | "dart"
            | "proto" | "zig" | "json5" | "jsonc" => Some(&C_FAMILY),
            "py" | "pyi" => Some(&PYTHON),
            "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml" | "toml" | "ini" | "cfg" | "conf"
            | "dockerfile" | "tf" | "tfvars" | "nix" | "pl" | "r" | "ex" | "exs" | "jl"
            | "makefile" | "mk" | "gitignore" | "env" => Some(&SHELL),
            "rb" | "rake" | "gemspec" => Some(&RUBY),
            "html" | "htm" | "xml" | "vue" | "svelte" | "svg" | "xhtml" => Some(&HTML),
            "css" | "scss" | "sass" | "less" => Some(&CSS),
            "sql" => Some(&SQL),
            _ => None,
        }
    }
}

fn extension(path: &str) -> Option<&str> {
    let name = path.rsplit('/').next()?;
    name.rsplit_once('.').map(|(_, ext)| ext)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_files_use_c_family_syntax() {
        assert_eq!(Syntax::for_path("src/main.rs").unwrap().line, ["//"]);
    }

    #[test]
    fn c_family_has_block_comments() {
        assert_eq!(Syntax::for_path("a.ts").unwrap().block, [("/*", "*/")]);
    }

    #[test]
    fn c_family_strings_escape_with_backslash() {
        let quotes: Vec<&str> = Syntax::for_path("a.go")
            .unwrap()
            .strings
            .iter()
            .map(|s| s.open)
            .collect();
        assert_eq!(quotes, ["\"", "'", "`"]);
    }

    #[test]
    fn python_uses_hash_and_triple_quoted_strings() {
        let syntax = Syntax::for_path("a.py").unwrap();
        assert_eq!(syntax.line, ["#"]);
        assert_eq!(syntax.strings[0].open, "\"\"\"");
    }

    #[test]
    fn html_uses_angle_bracket_block_comments() {
        assert_eq!(Syntax::for_path("a.html").unwrap().block, [("<!--", "-->")]);
    }

    #[test]
    fn sql_uses_double_dash_line_comments() {
        let syntax = Syntax::for_path("a.sql").unwrap();
        assert_eq!(syntax.line, ["--"]);
        assert_eq!(syntax.block, [("/*", "*/")]);
    }

    #[test]
    fn ruby_has_begin_end_block_comments() {
        assert_eq!(Syntax::for_path("a.rb").unwrap().block, [("=begin", "=end")]);
    }

    #[test]
    fn unknown_extensions_have_no_syntax() {
        assert!(Syntax::for_path("a.bin").is_none());
        assert!(Syntax::for_path("LICENSE").is_none());
    }
}
