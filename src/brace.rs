//! Bash-style brace expansion: `{a,b,c}`, nested groups, and ranges like
//! `{1..5}`, `{01..10..2}`, `{a..e}`.
//!
//! This lets `mkdir2 project/{src,tests,docs}` create three directories,
//! `mkdir2 file{01..05}` create five zero-padded directories, and groups
//! can be nested and combined freely, e.g. `a{1,2}b{x,y}`.

/// Expand all brace patterns in `input`, returning every resulting string.
/// If `input` contains no expandable braces, returns a single-element
/// vector containing `input` unchanged.
pub fn expand(input: &str) -> Vec<String> {
    expand_inner(input)
}

fn expand_inner(s: &str) -> Vec<String> {
    match find_outer_brace(s) {
        None => vec![s.to_string()],
        Some((open, close)) => {
            let prefix = &s[..open];
            let body = &s[open + 1..close];
            let suffix = &s[close + 1..];

            let items: Vec<String> = if let Some(range_items) = try_parse_range(body) {
                range_items
            } else {
                let parts = split_top_level(body, ',');
                if parts.len() > 1 {
                    let mut expanded = Vec::new();
                    for part in parts {
                        expanded.extend(expand_inner(&part));
                    }
                    expanded
                } else {
                    // No comma, no range: bash treats this as a literal
                    // `{...}` rather than an expansion. Keep it as text
                    // and continue scanning the rest of the string.
                    let mut out = Vec::new();
                    for suf in expand_inner(suffix) {
                        out.push(format!("{prefix}{{{body}}}{suf}"));
                    }
                    return out;
                }
            };

            let mut results = Vec::new();
            for item in items {
                for suf in expand_inner(suffix) {
                    results.push(format!("{prefix}{item}{suf}"));
                }
            }
            results
        }
    }
}

/// Find the first top-level (unescaped, properly nested) `{...}` pair.
/// Returns byte offsets of the opening and closing brace.
fn find_outer_brace(s: &str) -> Option<(usize, usize)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'{' => {
                let mut depth: usize = 1;
                let mut j = i + 1;
                while j < bytes.len() {
                    match bytes[j] {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                return Some((i, j));
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                // Unmatched '{': treat as a literal character and keep
                // scanning the remainder of the string.
                i += 1;
            }
            _ => i += 1,
        }
    }
    None
}

/// Split `s` on `sep`, but only when at brace-nesting depth 0.
fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut result = Vec::new();
    let mut depth: i32 = 0;
    let mut current = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            current.push(c);
            if let Some(nc) = chars.next() {
                current.push(nc);
            }
            continue;
        }
        match c {
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            c if c == sep && depth == 0 => {
                result.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    result.push(current);
    result
}

/// Try to parse `body` as a range expression: `start..end` or
/// `start..end..step`. Supports signed integers (with zero-padding
/// preserved) and single ASCII letters (e.g. `a..e`).
fn try_parse_range(body: &str) -> Option<Vec<String>> {
    if body.contains(',') {
        return None;
    }
    let parts: Vec<&str> = body.split("..").collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }

    // Numeric range.
    if let (Ok(start), Ok(end)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
        let step: i64 = if parts.len() == 3 {
            match parts[2].parse::<i64>() {
                Ok(v) if v != 0 => v.abs(),
                _ => 1,
            }
        } else {
            1
        };

        let pad_width = {
            let digits = parts[0].trim_start_matches('-');
            if digits.len() > 1 && digits.starts_with('0') {
                digits.len()
            } else {
                0
            }
        };

        let mut out = Vec::new();
        if start <= end {
            let mut v = start;
            while v <= end {
                out.push(format_padded(v, pad_width));
                v += step;
            }
        } else {
            let mut v = start;
            while v >= end {
                out.push(format_padded(v, pad_width));
                v -= step;
            }
        }
        return Some(out);
    }

    // Single-letter alphabetic range (no step support, mirroring the most
    // common bash use case).
    if parts.len() == 2 {
        let a: Vec<char> = parts[0].chars().collect();
        let b: Vec<char> = parts[1].chars().collect();
        if a.len() == 1 && b.len() == 1 && a[0].is_ascii_alphabetic() && b[0].is_ascii_alphabetic()
        {
            let (start, end) = (a[0] as u8, b[0] as u8);
            let mut out = Vec::new();
            if start <= end {
                for c in start..=end {
                    out.push((c as char).to_string());
                }
            } else {
                for c in (end..=start).rev() {
                    out.push((c as char).to_string());
                }
            }
            return Some(out);
        }
    }

    None
}

fn format_padded(v: i64, width: usize) -> String {
    if width == 0 {
        return v.to_string();
    }
    if v < 0 {
        format!("-{:0width$}", -v, width = width)
    } else {
        format!("{v:0width$}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_comma_list() {
        let mut got = expand("dir{1,2,3}");
        got.sort();
        assert_eq!(got, vec!["dir1", "dir2", "dir3"]);
    }

    #[test]
    fn nested_groups() {
        let mut got = expand("project/{src/{main,lib},tests}");
        got.sort();
        let mut expected = vec!["project/src/main", "project/src/lib", "project/tests"];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn deeply_nested_groups() {
        let mut got = expand("a/{b/{c,d}/e,f}");
        got.sort();
        let mut expected = vec!["a/b/c/e", "a/b/d/e", "a/f"];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn numeric_range() {
        let got = expand("file{1..3}");
        assert_eq!(got, vec!["file1", "file2", "file3"]);
    }

    #[test]
    fn numeric_range_padded() {
        let got = expand("file{01..03}");
        assert_eq!(got, vec!["file01", "file02", "file03"]);
    }

    #[test]
    fn numeric_range_padded_negative() {
        let got = expand("neg{-05..-03}");
        assert_eq!(got, vec!["neg-05", "neg-04", "neg-03"]);
    }

    #[test]
    fn numeric_range_with_step() {
        let got = expand("v{0..10..5}");
        assert_eq!(got, vec!["v0", "v5", "v10"]);
    }

    #[test]
    fn numeric_range_descending() {
        let got = expand("v{3..1}");
        assert_eq!(got, vec!["v3", "v2", "v1"]);
    }

    #[test]
    fn alpha_range() {
        let got = expand("{a..e}");
        assert_eq!(got, vec!["a", "b", "c", "d", "e"]);
    }

    #[test]
    fn no_braces_passthrough() {
        let got = expand("plain/path");
        assert_eq!(got, vec!["plain/path"]);
    }

    #[test]
    fn single_item_brace_is_literal() {
        // Bash treats a brace with no comma/range as literal text.
        let got = expand("{onlyone}");
        assert_eq!(got, vec!["{onlyone}"]);
    }

    #[test]
    fn multiple_groups_cartesian_product() {
        let mut got = expand("a{1,2}b{x,y}");
        got.sort();
        let mut expected = vec!["a1bx", "a1by", "a2bx", "a2by"];
        expected.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn empty_input_returns_empty_string() {
        assert_eq!(expand(""), vec![""]);
    }
}
