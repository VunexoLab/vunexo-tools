//! Secret-scanning domain types: [`ScanFinding`] and the rule identifiers
//! used by `infrastructure::pattern_scanner`.

use std::fmt;
use std::path::PathBuf;

/// One likely-secret finding from a scan.
///
/// Per `crypto-and-scanning-engine.md` §5, this never carries the full
/// matched string — only a redacted preview — so a scan's own output can't
/// itself become a leak vector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanFinding {
    pub path: PathBuf,
    pub line: usize,
    pub rule_name: &'static str,
    pub redacted_preview: String,
}

impl fmt::Display for ScanFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: [{}] {}",
            self.path.display(),
            self.line,
            self.rule_name,
            self.redacted_preview
        )
    }
}

/// Redact a matched string to its first 4 and last 4 characters, with the
/// middle replaced by `...`. Strings of 8 characters or fewer are redacted
/// entirely, since a partial preview of a short string reveals most or all
/// of it.
pub fn redact_preview(matched: &str) -> String {
    let chars: Vec<char> = matched.chars().collect();
    if chars.len() <= 8 {
        return "...".to_string();
    }
    let first: String = chars[..4].iter().collect();
    let last: String = chars[chars.len() - 4..].iter().collect();
    format!("{first}...{last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_long_strings_to_first_and_last_four() {
        assert_eq!(
            redact_preview("AKIAIOSFODNN7EXAMPLE"),
            "AKIA...MPLE".to_string()
        );
    }

    #[test]
    fn redacts_short_strings_entirely() {
        assert_eq!(redact_preview("changeme"), "...".to_string());
        assert_eq!(redact_preview("short"), "...".to_string());
    }

    #[test]
    fn display_matches_cli_ux_format() {
        let finding = ScanFinding {
            path: PathBuf::from("src/main.rs"),
            line: 12,
            rule_name: "aws_access_key_id",
            redacted_preview: "AKIA...MPLE".to_string(),
        };
        assert_eq!(
            finding.to_string(),
            "src/main.rs:12: [aws_access_key_id] AKIA...MPLE"
        );
    }
}
