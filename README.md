# swab

Arr! Swabs the decks o' comments yer current branch dragged aboard.

`swab` diffs yer working tree against the merge base with the default branch,
hunts down every comment that lives entirely on added lines, and sends it to
Davy Jones' locker. Comments that were aboard before ye set sail are left be.

## Usage

```
swab [--base <ref>] [--dry-run]
```

- `--base <ref>` — diff against this ref instead o' the detected default branch
  (`origin/HEAD`, `origin/main`, `origin/master`, `main`, `master`).
- `--dry-run` — report what would walk the plank without touchin' a single file.

Untracked files be treated as fresh cargo, so every comment in 'em is fit fer
the deep.

## Behavior

- A comment whose line ends up blank is thrown overboard, line and all.
- A trailin' comment is scuttled along with the whitespace afore it, leavin' the
  code shipshape.
- A comment that spans both added and pre-existin' lines be spared.
- Comment markers hidin' inside string literals ain't mistaken fer comments.
- Python docstrings and other string literals be strings, not comments, so they
  live to sail another day.

## Supported languages

C-family (Rust, TS/JS, Go, Java, C/C++, C#, Swift, Kotlin, Scala, PHP, Dart,
Zig, proto), Python, shell and config formats (sh/bash/zsh/fish, YAML, TOML,
INI, Terraform, Nix, Makefile), Ruby, HTML/XML/Vue/Svelte, CSS/SCSS/Less, and
SQL. Files with unrecognized extensions be left ashore.

## License

MIT, ye scallywag.
