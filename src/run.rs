use std::path::PathBuf;
use std::process::Command;

use crate::git::added_lines;
use crate::strip::strip;
use crate::syntax::Syntax;

pub struct Options {
    pub base: Option<String>,
    pub dry_run: bool,
}

pub struct Swabbed {
    pub path: String,
    pub removed: usize,
}

pub struct Report {
    pub base: String,
    pub files: Vec<Swabbed>,
}

pub fn run(options: &Options) -> Result<Report, String> {
    let base = match &options.base {
        Some(base) => base.clone(),
        None => default_base()?,
    };
    let root = PathBuf::from(capture(&["rev-parse", "--show-toplevel"])?.trim());
    let merge_base = capture(&["merge-base", "HEAD", &base])?.trim().to_string();
    let diff = capture(&[
        "diff",
        "--unified=0",
        "--no-color",
        "--no-ext-diff",
        &merge_base,
    ])?;

    let mut files = added_lines(&diff);
    for path in untracked()? {
        let Ok(source) = std::fs::read_to_string(root.join(&path)) else {
            continue;
        };
        files.insert(path, (1..=source.lines().count()).collect());
    }

    let mut swabbed = Vec::new();
    for (path, added) in files {
        let Some(syntax) = Syntax::for_path(&path) else {
            continue;
        };
        let full_path = root.join(&path);
        let Ok(source) = std::fs::read_to_string(&full_path) else {
            continue;
        };
        let result = strip(&source, syntax, &added);
        if result.removed == 0 {
            continue;
        }
        if !options.dry_run {
            std::fs::write(&full_path, &result.text).map_err(|e| format!("{path}: {e}"))?;
        }
        swabbed.push(Swabbed {
            path,
            removed: result.removed,
        });
    }

    swabbed.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Report {
        base,
        files: swabbed,
    })
}

fn untracked() -> Result<Vec<String>, String> {
    let listing = capture(&["ls-files", "--others", "--exclude-standard"])?;
    Ok(listing.lines().map(str::to_string).collect())
}

fn default_base() -> Result<String, String> {
    let candidates = [
        "origin/HEAD",
        "origin/main",
        "origin/master",
        "main",
        "master",
    ];
    candidates
        .into_iter()
        .find(|candidate| capture(&["rev-parse", "--verify", "--quiet", candidate]).is_ok())
        .map(str::to_string)
        .ok_or_else(|| "could not find a default branch; pass --base <ref>".to_string())
}

fn capture(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| e.to_string())
}
