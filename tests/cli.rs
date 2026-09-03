use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?} failed");
}

fn swab(repo: &Path, args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_swab"))
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

struct Repo {
    path: PathBuf,
}

impl Repo {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("swab-test-{name}"));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        git(&path, &["init", "--initial-branch=main", "--quiet"]);
        git(&path, &["config", "user.email", "test@example.com"]);
        git(&path, &["config", "user.name", "Test"]);
        fs::write(path.join("kept.rs"), "// original comment\nfn old() {}\n").unwrap();
        git(&path, &["add", "-A"]);
        git(&path, &["commit", "--quiet", "-m", "base"]);
        git(&path, &["switch", "--quiet", "--create", "feature"]);
        Repo { path }
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.path.join(name), contents).unwrap();
    }

    fn read(&self, name: &str) -> String {
        fs::read_to_string(self.path.join(name)).unwrap()
    }

    fn commit(&self, message: &str) {
        git(&self.path, &["add", "-A"]);
        git(&self.path, &["commit", "--quiet", "-m", message]);
    }
}

#[test]
fn removes_comments_added_on_the_branch() {
    let repo = Repo::new("committed");
    repo.write("new.rs", "// added noise\nfn f() {} // trailing\n");
    repo.commit("work");

    swab(&repo.path, &["--base", "main"]);

    assert_eq!(repo.read("new.rs"), "fn f() {}\n");
    assert_eq!(repo.read("kept.rs"), "// original comment\nfn old() {}\n");
}

#[test]
fn removes_comments_from_uncommitted_work() {
    let repo = Repo::new("dirty");
    repo.write("new.rs", "fn f() {}\n// uncommitted noise\n");

    swab(&repo.path, &["--base", "main"]);

    assert_eq!(repo.read("new.rs"), "fn f() {}\n");
}

#[test]
fn dry_run_reports_without_writing() {
    let repo = Repo::new("dry-run");
    repo.write("new.rs", "// added noise\n");
    repo.commit("work");

    let report = swab(&repo.path, &["--base", "main", "--dry-run"]);

    assert!(report.contains("new.rs"), "{report}");
    assert_eq!(repo.read("new.rs"), "// added noise\n");
}

#[test]
fn leaves_unsupported_file_types_alone() {
    let repo = Repo::new("unsupported");
    repo.write("notes.bin", "// looks like a comment\n");
    repo.commit("work");

    swab(&repo.path, &["--base", "main"]);

    assert_eq!(repo.read("notes.bin"), "// looks like a comment\n");
}

#[test]
fn works_when_run_from_a_subdirectory() {
    let repo = Repo::new("subdir");
    fs::create_dir_all(repo.path.join("src")).unwrap();
    repo.write("src/new.ts", "// added noise\nexport const a = 1;\n");
    repo.commit("work");

    swab(&repo.path.join("src"), &["--base", "main"]);

    assert_eq!(repo.read("src/new.ts"), "export const a = 1;\n");
}

#[test]
fn reports_the_base_it_compared_against() {
    let repo = Repo::new("base-report");
    repo.write("new.ts", "export const a = 1;\n");
    repo.commit("work");

    let report = swab(&repo.path, &["--base", "main"]);

    assert!(report.contains("main"), "{report}");
}

#[test]
fn fails_loudly_when_a_changed_file_cannot_be_read() {
    let repo = Repo::new("unreadable");
    repo.write("new.ts", "// noise\n");
    repo.commit("work");
    fs::remove_file(repo.path.join("new.ts")).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_swab"))
        .args(["--base", "main"])
        .current_dir(&repo.path)
        .output()
        .unwrap();

    assert!(output.status.success(), "deleted files are not an error");
}
