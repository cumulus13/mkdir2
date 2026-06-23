//! Black-box integration tests that invoke the compiled `mkdir2` binary
//! exactly the way a user would, via `std::process::Command`.

use std::process::Command;
use tempfile::tempdir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_mkdir2")
}

#[test]
fn creates_simple_directory() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg("hello")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("hello").is_dir());
}

#[test]
fn brace_expansion_creates_multiple_directories() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg("project/{src,tests,docs}")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("project/src").is_dir());
    assert!(dir.path().join("project/tests").is_dir());
    assert!(dir.path().join("project/docs").is_dir());
}

#[test]
fn numeric_range_with_padding_creates_zero_padded_dirs() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg("day-{01..03}")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("day-01").is_dir());
    assert!(dir.path().join("day-02").is_dir());
    assert!(dir.path().join("day-03").is_dir());
}

#[test]
fn backslash_separator_works_like_forward_slash() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg(r"data\2024\reports")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("data/2024/reports").is_dir());
}

#[test]
fn force_recreates_and_clears_existing_contents() {
    let dir = tempdir().unwrap();
    let target = dir.path().join("recreate_me");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("old.txt"), b"stale").unwrap();

    let status = Command::new(bin())
        .current_dir(dir.path())
        .args(["-f", "recreate_me"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(target.is_dir());
    assert!(!target.join("old.txt").exists());
}

#[test]
fn dry_run_does_not_create_anything() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .args(["-n", "ghost/{a,b}"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(!dir.path().join("ghost").exists());
}

#[test]
fn strict_mode_fails_on_existing_directory() {
    let dir = tempdir().unwrap();
    std::fs::create_dir(dir.path().join("already_here")).unwrap();

    let status = Command::new(bin())
        .current_dir(dir.path())
        .args(["--strict", "already_here"])
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn gitkeep_flag_creates_marker_file() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .args(["--gitkeep", "keepme"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("keepme/.gitkeep").is_file());
}

#[test]
fn json_output_is_valid_and_reports_correct_created_count() {
    let dir = tempdir().unwrap();
    let output = Command::new(bin())
        .current_dir(dir.path())
        .args(["--json", "batch/{a,b,c}"])
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json output must be valid, parseable JSON");
    assert_eq!(parsed["created"], 3);
    assert_eq!(parsed["failed"], 0);
    assert_eq!(parsed["results"].as_array().unwrap().len(), 3);
}

#[test]
fn from_file_batch_creates_directories_with_comments_and_blank_lines() {
    let dir = tempdir().unwrap();
    let patterns_file = dir.path().join("patterns.txt");
    std::fs::write(&patterns_file, "# a comment\nbatchdir/{x,y}\n\nbatchdir2\n").unwrap();

    let status = Command::new(bin())
        .current_dir(dir.path())
        .args(["--from", patterns_file.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("batchdir/x").is_dir());
    assert!(dir.path().join("batchdir/y").is_dir());
    assert!(dir.path().join("batchdir2").is_dir());
}

#[test]
fn no_paths_given_is_an_error() {
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn existing_file_at_target_path_fails_instead_of_overwriting() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("not_a_dir"), b"i am a file").unwrap();

    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg("not_a_dir")
        .status()
        .unwrap();
    assert!(!status.success());
}

#[test]
fn spaces_around_commas_in_braces_are_trimmed() {
    // "dir1/{dir2, dir3}" and "dir1/{dir2,dir3}" must produce identical results.
    // No leading/trailing space must survive into the created directory names.
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg("dir1/{dir2, dir3}")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("dir1/dir2").is_dir());
    assert!(dir.path().join("dir1/dir3").is_dir());
    // Confirm the space-prefixed variant does NOT exist.
    assert!(!dir.path().join("dir1/ dir3").exists());
}

#[test]
fn backslash_before_brace_expands_same_as_forward_slash() {
    // dir1\{dir2,dir3} must create the same dirs as dir1/{dir2,dir3}.
    let dir = tempdir().unwrap();
    let status = Command::new(bin())
        .current_dir(dir.path())
        .arg(r"dir1\{dir2, dir3}")
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.path().join("dir1/dir2").is_dir());
    assert!(dir.path().join("dir1/dir3").is_dir());
    assert!(!dir.path().join("dir1/ dir3").exists());
}
