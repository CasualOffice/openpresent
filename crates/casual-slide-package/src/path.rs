//! Entry-path safety. Rejects traversal, absolute, and drive-qualified paths
//! before any part is read. See `docs/21-PARSER-LIMITS.md`.

/// Returns `true` if `name` is a safe relative package part path.
///
/// Rejects: empty names, absolute paths (`/…` or `\…`), Windows drive-qualified
/// paths (`C:…`), and any path containing a `..` component. Both `/` and `\` are
/// treated as separators so a `\`-escaped path cannot slip through.
pub fn is_safe_part_path(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.starts_with('/') || name.starts_with('\\') {
        return false;
    }
    let bytes = name.as_bytes();
    // Windows drive letter, e.g. "C:whatever".
    if bytes.len() >= 2 && bytes[1] == b':' {
        return false;
    }
    if name.chars().any(char::is_control) {
        return false;
    }
    // Percent-encoded dots and separators decode into a different path for the
    // next reader. OPC forbids them; other escapes such as `%20` are ordinary.
    let lower = name.to_ascii_lowercase();
    if ["%2e", "%2f", "%5c"].iter().any(|e| lower.contains(e)) {
        return false;
    }
    let body = name.strip_suffix('/').unwrap_or(name);
    for component in body.split(['/', '\\']) {
        // OPC forbids empty segments and segments ending in a dot, which covers
        // `.` and `..`.
        if component.is_empty() || component.ends_with('.') {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::is_safe_part_path;

    #[test]
    fn accepts_normal_part_paths() {
        assert!(is_safe_part_path("xl/workbook.xml"));
        assert!(is_safe_part_path("[Content_Types].xml"));
        assert!(is_safe_part_path("xl/worksheets/sheet1.xml"));
    }

    #[test]
    fn rejects_traversal_and_absolute() {
        assert!(!is_safe_part_path(""));
        assert!(!is_safe_part_path("../evil.xml"));
        assert!(!is_safe_part_path("xl/../../etc/passwd"));
        assert!(!is_safe_part_path("/etc/passwd"));
        assert!(!is_safe_part_path("\\windows\\system32"));
        assert!(!is_safe_part_path("C:/Windows/win.ini"));
        assert!(!is_safe_part_path("xl\\..\\secret"));
    }
}
