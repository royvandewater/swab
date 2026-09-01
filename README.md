# swab

Removes comments introduced by the current branch.

`swab` diffs your working tree against the merge base with the default branch,
finds every comment that lives entirely on added lines, and deletes it. Comments
that existed before the branch are left alone.

## Usage

```
swab [--base <ref>] [--dry-run]
```

- `--base <ref>` — diff against this ref instead of the detected default branch
  (`origin/HEAD`, `origin/main`, `origin/master`, `main`, `master`).
- `--dry-run` — report what would be removed without writing files.

Untracked files are treated as entirely new, so all of their comments are
candidates for removal.

## Behavior

- A comment whose line becomes blank is removed along with its line.
- A trailing comment is removed along with the whitespace before it, leaving the
  code intact.
- A comment that spans both added and pre-existing lines is left alone.
- Comment markers inside string literals are not mistaken for comments.
- Python docstrings and other string literals are strings, not comments, so they
  survive.

## Supported languages

C-family (Rust, TS/JS, Go, Java, C/C++, C#, Swift, Kotlin, Scala, PHP, Dart,
Zig, proto), Python, shell and config formats (sh/bash/zsh/fish, YAML, TOML,
INI, Terraform, Nix, Makefile), Ruby, HTML/XML/Vue/Svelte, CSS/SCSS/Less, and
SQL. Files with unrecognized extensions are skipped.
