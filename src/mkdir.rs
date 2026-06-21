//! Core directory-creation engine: expands targets, applies force-recreate
//! semantics, sets permissions, and produces a structured report.

use crate::brace;
use crate::color::Theme;
use crate::pathnorm::normalize_path;

use serde::Serialize;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Created,
    AlreadyExists,
    Failed,
    DryRunWouldCreate,
    Skipped,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActionResult {
    pub path: String,
    pub action: Action,
    pub message: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub created: usize,
    pub already_existed: usize,
    pub skipped: usize,
    pub failed: usize,
    pub results: Vec<ActionResult>,
}

pub struct Options {
    pub create_parents: bool,
    pub force: bool,
    pub recursive_remove: bool,
    pub interactive: bool,
    pub mode: Option<u32>,
    pub gitkeep: bool,
    pub dry_run: bool,
    pub strict: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub json: bool,
}

/// Gather every target path: CLI-provided patterns plus optional patterns
/// read from a `--from` file, all run through brace expansion.
pub fn collect_targets(
    raw_paths: &[String],
    from_file: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    let mut patterns: Vec<String> = raw_paths.to_vec();

    if let Some(path) = from_file {
        let content = if path == "-" {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        } else {
            fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("could not read --from file '{path}': {e}"))?
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            patterns.push(trimmed.to_string());
        }
    }

    let mut expanded = Vec::new();
    for pattern in &patterns {
        expanded.extend(brace::expand(pattern));
    }
    Ok(expanded)
}

/// Process every target and return an aggregated summary.
pub fn run(targets: &[String], opts: &Options, theme: &Theme) -> Summary {
    let mut summary = Summary::default();
    for raw in targets {
        let path = normalize_path(raw);
        let result = process_one(&path, opts, theme);
        match result.action {
            Action::Created | Action::DryRunWouldCreate => summary.created += 1,
            Action::AlreadyExists => summary.already_existed += 1,
            Action::Skipped => summary.skipped += 1,
            Action::Failed => summary.failed += 1,
        }
        summary.results.push(result);
    }
    summary
}

fn process_one(path: &Path, opts: &Options, theme: &Theme) -> ActionResult {
    let display = path.display().to_string();

    if path.exists() {
        if opts.force {
            if opts.interactive && !confirm_removal(&display) {
                log_warn(theme, opts, &format!("skipped (kept existing): {display}"));
                return ActionResult {
                    path: display,
                    action: Action::Skipped,
                    message: Some("kept by user".into()),
                };
            }
            if !opts.dry_run {
                if let Err(e) = remove_existing(path, opts) {
                    return fail(
                        &display,
                        theme,
                        opts,
                        &format!("could not remove existing path: {e}"),
                    );
                }
            }
            log_warn(theme, opts, &format!("removed existing: {display}"));
        } else if path.is_dir() {
            if opts.strict {
                return fail(
                    &display,
                    theme,
                    opts,
                    "already exists (pass --force to recreate)",
                );
            }
            log_info(theme, opts, &format!("already exists: {display}"));
            return ActionResult {
                path: display,
                action: Action::AlreadyExists,
                message: None,
            };
        } else {
            return fail(
                &display,
                theme,
                opts,
                "a non-directory file already exists at this path",
            );
        }
    }

    if opts.dry_run {
        log_info(theme, opts, &format!("would create: {display}"));
        return ActionResult {
            path: display,
            action: Action::DryRunWouldCreate,
            message: None,
        };
    }

    let create_result = if opts.create_parents {
        fs::create_dir_all(path)
    } else {
        fs::create_dir(path)
    };

    if let Err(e) = create_result {
        return fail(&display, theme, opts, &format!("{e}"));
    }

    if let Some(mode) = opts.mode {
        if let Err(e) = set_mode(path, mode) {
            log_warn(
                theme,
                opts,
                &format!("created but could not set mode on {display}: {e}"),
            );
        }
    }

    if opts.gitkeep {
        let _ = fs::write(path.join(".gitkeep"), b"");
    }

    log_success(theme, opts, &format!("created: {display}"));
    ActionResult {
        path: display,
        action: Action::Created,
        message: None,
    }
}

fn remove_existing(path: &Path, opts: &Options) -> io::Result<()> {
    if path.is_dir() {
        if opts.recursive_remove {
            fs::remove_dir_all(path)
        } else {
            fs::remove_dir(path)
        }
    } else {
        fs::remove_file(path)
    }
}

fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        Ok(())
    }
}

fn confirm_removal(display: &str) -> bool {
    print!("Remove existing '{display}' before recreating? [y/N] ");
    let _ = io::stdout().flush();
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn fail(display: &str, theme: &Theme, opts: &Options, reason: &str) -> ActionResult {
    if !opts.json {
        eprintln!("{}", theme.error(&format!("failed: {display}: {reason}")));
    }
    ActionResult {
        path: display.to_string(),
        action: Action::Failed,
        message: Some(reason.to_string()),
    }
}

fn log_warn(theme: &Theme, opts: &Options, message: &str) {
    if !opts.quiet && !opts.json {
        eprintln!("{}", theme.warn(message));
    }
}

fn log_info(theme: &Theme, opts: &Options, message: &str) {
    if opts.verbose && !opts.quiet && !opts.json {
        println!("{}", theme.info(message));
    }
}

fn log_success(theme: &Theme, opts: &Options, message: &str) {
    if opts.verbose && !opts.quiet && !opts.json {
        println!("{}", theme.success(message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn quiet_opts() -> Options {
        Options {
            create_parents: true,
            force: false,
            recursive_remove: true,
            interactive: false,
            mode: None,
            gitkeep: false,
            dry_run: false,
            strict: false,
            verbose: false,
            quiet: true,
            json: true,
        }
    }

    #[test]
    fn creates_nested_directory() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("a/b/c");
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &quiet_opts(), &theme);
        assert_eq!(result.action, Action::Created);
        assert!(target.is_dir());
    }

    #[test]
    fn already_existing_directory_is_noop_by_default() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("exists");
        fs::create_dir(&target).unwrap();
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &quiet_opts(), &theme);
        assert_eq!(result.action, Action::AlreadyExists);
    }

    #[test]
    fn strict_mode_fails_on_existing_directory() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("exists");
        fs::create_dir(&target).unwrap();
        let mut opts = quiet_opts();
        opts.strict = true;
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &opts, &theme);
        assert_eq!(result.action, Action::Failed);
    }

    #[test]
    fn force_recreates_directory_with_contents() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("recreate_me");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("leftover.txt"), b"old").unwrap();

        let mut opts = quiet_opts();
        opts.force = true;
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &opts, &theme);
        assert_eq!(result.action, Action::Created);
        assert!(target.is_dir());
        assert!(!target.join("leftover.txt").exists());
    }

    #[test]
    fn dry_run_does_not_touch_filesystem() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("ghost");
        let mut opts = quiet_opts();
        opts.dry_run = true;
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &opts, &theme);
        assert_eq!(result.action, Action::DryRunWouldCreate);
        assert!(!target.exists());
    }

    #[test]
    fn gitkeep_file_is_created() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("with_keep");
        let mut opts = quiet_opts();
        opts.gitkeep = true;
        let theme = Theme {
            enabled: false,
            emoji: false,
            ..Theme::default()
        };
        let result = process_one(&target, &opts, &theme);
        assert_eq!(result.action, Action::Created);
        assert!(target.join(".gitkeep").is_file());
    }

    #[test]
    fn collect_targets_expands_braces() {
        let raw = vec!["dir{1,2,3}".to_string()];
        let targets = collect_targets(&raw, None).unwrap();
        let mut sorted = targets.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["dir1", "dir2", "dir3"]);
    }
}
