//! Cross-platform path handling.
//!
//! `mkdir2` accepts paths using either `/` or `\` as a separator on *any*
//! platform, so a pattern written on Linux works unchanged on Windows and
//! vice versa.

use std::path::PathBuf;

/// Normalize a user-supplied path string so that both `/` and `\` are
/// treated as path separators regardless of the host platform, and
/// reassemble it using the platform's native separator.
///
/// Examples:
/// - `a/b\c`        -> `a/b/c` (or `a\b\c` on Windows)
/// - `C:\foo/bar`    -> `C:\foo\bar` on Windows
/// - `/usr/local/bin` stays absolute on Unix
pub fn normalize_path<S: AsRef<str>>(input: S) -> PathBuf {
    let input = input.as_ref();
    let unified = input.replace('\\', "/");

    let mut components = unified.split('/').filter(|p| !p.is_empty()).peekable();
    let mut result = PathBuf::new();

    // Preserve POSIX-style absolute paths.
    if unified.starts_with('/') {
        result.push(std::path::MAIN_SEPARATOR.to_string());
    }

    // Preserve Windows-style drive prefixes (`C:`) so `C:foo` doesn't
    // collapse into a drive-relative path.
    if let Some(first) = components.peek() {
        if is_windows_drive(first) {
            let drive = components.next().unwrap();
            result.push(format!("{drive}{}", std::path::MAIN_SEPARATOR));
        }
    }

    for part in components {
        result.push(part);
    }

    result
}

fn is_windows_drive(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Returns true if `input` looks absolute under either Unix or Windows
/// conventions, independent of the host platform.
#[allow(dead_code)]
pub fn looks_absolute(input: &str) -> bool {
    let unified = input.replace('\\', "/");
    if unified.starts_with('/') {
        return true;
    }
    let bytes = unified.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_separators_become_uniform() {
        let p = normalize_path("a/b\\c");
        let s = p.to_string_lossy().replace('\\', "/");
        assert_eq!(s, "a/b/c");
    }

    #[test]
    fn backslash_only_path() {
        let p = normalize_path("foo\\bar\\baz");
        let s = p.to_string_lossy().replace('\\', "/");
        assert_eq!(s, "foo/bar/baz");
    }

    #[test]
    fn forward_slash_absolute_preserved() {
        let p = normalize_path("/usr/local/bin");
        let s = p.to_string_lossy();
        assert!(p.is_absolute() || s.starts_with('/'));
    }

    #[test]
    fn windows_drive_is_detected() {
        assert!(looks_absolute("C:\\Users\\test"));
        assert!(looks_absolute("C:/Users/test"));
    }

    #[test]
    fn relative_path_is_not_absolute() {
        assert!(!looks_absolute("foo/bar"));
        assert!(!looks_absolute("foo\\bar"));
    }

    #[test]
    fn trailing_and_duplicate_separators_collapse() {
        let p = normalize_path("a//b///c/");
        let s = p.to_string_lossy().replace('\\', "/");
        assert_eq!(s, "a/b/c");
    }
}
