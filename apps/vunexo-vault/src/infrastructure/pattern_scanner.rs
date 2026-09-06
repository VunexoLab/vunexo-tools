//! `Scanner` implementation: the pattern + entropy rules from
//! `docs/vunexo-vault/crypto-and-scanning-engine.md` §5.
//!
//! Rule set:
//! - Five **shape rules** (AWS access key ID, GitHub token, Stripe key,
//!   PEM private-key header, JWT) that flag on regex shape alone,
//!   regardless of entropy — matching the doc's "PEM header, regardless of
//!   what follows" framing.
//! - One **generic assignment** rule (`KEY|SECRET|PASSWORD|TOKEN = value`)
//!   that additionally requires the assigned value to clear the entropy
//!   threshold. This is necessary, not optional: `API_KEY = "your-api-key-here"`
//!   matches the assignment *shape* (it's a `KEY`-named variable assigned a
//!   12+ character value) but is a required true-negative in the doc's own
//!   test vectors, and the doc's own annotation on the true-positive vector
//!   — `AWS_SECRET_ACCESS_KEY = "..."` "(generic assignment + high
//!   entropy)" — confirms both conditions are meant to combine for this
//!   rule specifically.
//! - One **standalone entropy catch-all** for any other quoted or
//!   assigned *unbroken* token (no embedded whitespace) of length >= 20
//!   whose entropy clears the same threshold, even if no named pattern
//!   matched. Restricting candidates to whitespace-free tokens (rather than
//!   e.g. an entire quoted sentence) is what keeps ordinary prose out: an
//!   English sentence's individual words are all far shorter than 20
//!   characters, so a plain-prose line simply never produces a length-20+
//!   unbroken candidate, regardless of its overall entropy including
//!   spaces.
//!
//! The keyword match in the generic-assignment rule (`KEY|SECRET|PASSWORD|
//! TOKEN`) is case-insensitive, a deliberate small widening beyond the
//! doc's literal all-caps regex text (which reads as illustrative, not a
//! case-sensitivity mandate) — this catches common lowercase/mixed-case
//! variable names too, reducing false negatives without affecting any
//! locked test vector either way.
//!
//! Deliberately excluded from all scanning: any path under a `.vunexo`
//! directory component, and any `*.age` file. Those are Vunexo Vault's own
//! ciphertext — ciphertext is inherently high-entropy and would otherwise
//! flood every scan (and every commit, once the pre-commit hook is
//! installed) with findings against the one thing in the whole design that
//! is deliberately safe to commit. This exclusion is an addition beyond the
//! literal text of `crypto-and-scanning-engine.md` §5, made necessary by
//! that document's own storage design (`storage-schema.md`: `.vunexo/` is
//! meant to be committed, not gitignored).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use regex::Regex;

use crate::domain::scan::{redact_preview, ScanFinding};

use crate::application::ports::Scanner;

/// Shannon entropy threshold (bits per character) above which a candidate
/// token is treated as plausibly-random (and thus secret-shaped) rather than
/// natural-language or placeholder text. Chosen empirically against the
/// locked test vectors in `crypto-and-scanning-engine.md` §5 (see the
/// `pattern_scanner` integration tests): comfortably above natural-language
/// prose (~3.9-4.4 bits/char *including spaces*, but prose never survives
/// the whitespace-delimited tokenization above to be scored at all), above a
/// hex-alphabet UUID's theoretical maximum (`log2(16) = 4.0`), and below a
/// realistic random base64/alphanumeric secret's typical entropy (~4.5-6.0
/// bits/char). Per the doc: "tuned during Round 7/8 against the test
/// vectors below, not designed to a specific decimal here."
const ENTROPY_THRESHOLD: f64 = 4.3;

/// Skip scanning files above this size — scanning is a best-effort heuristic
/// safety net, not a guarantee, and this avoids wasting time on huge
/// (typically binary or generated) files.
const MAX_SCAN_FILE_BYTES: u64 = 5 * 1024 * 1024;

fn shannon_entropy(s: &str) -> f64 {
    let len = s.chars().count();
    if len == 0 {
        return 0.0;
    }
    let mut freq: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }
    freq.values()
        .map(|&count| {
            let p = count as f64 / len as f64;
            -p * p.log2()
        })
        .sum()
}

fn overlaps(a: (usize, usize), b: (usize, usize)) -> bool {
    a.0 < b.1 && b.0 < a.1
}

fn is_vunexo_internal_path(path: &Path) -> bool {
    if path.extension().is_some_and(|ext| ext == "age") {
        return true;
    }
    path.components().any(|c| c.as_os_str() == ".vunexo")
}

pub struct PatternScanner {
    aws_access_key_id: Regex,
    github_token: Regex,
    stripe_key: Regex,
    private_key_pem: Regex,
    jwt: Regex,
    generic_assignment: Regex,
    quoted_token: Regex,
    assigned_token: Regex,
}

impl Default for PatternScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternScanner {
    pub fn new() -> Self {
        Self {
            aws_access_key_id: Regex::new(r"AKIA[0-9A-Z]{16}").expect("valid regex"),
            github_token: Regex::new(r"gh[pousr]_[A-Za-z0-9]{36,}").expect("valid regex"),
            stripe_key: Regex::new(r"sk_(?:live|test)_[A-Za-z0-9]{10,}").expect("valid regex"),
            private_key_pem: Regex::new(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----")
                .expect("valid regex"),
            jwt: Regex::new(r"[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}")
                .expect("valid regex"),
            generic_assignment: Regex::new(
                r#"(?i)(?:KEY|SECRET|PASSWORD|TOKEN)\s*=\s*['"]?([A-Za-z0-9+/=_-]{12,})"#,
            )
            .expect("valid regex"),
            quoted_token: Regex::new(r#"["']([A-Za-z0-9+/=_.-]{20,})["']"#).expect("valid regex"),
            assigned_token: Regex::new(r#"=\s*['"]?([A-Za-z0-9+/=_.-]{20,})['"]?"#)
                .expect("valid regex"),
        }
    }

    fn shape_rules(&self) -> [(&Regex, &'static str); 5] {
        [
            (&self.aws_access_key_id, "aws_access_key_id"),
            (&self.github_token, "github_token"),
            (&self.stripe_key, "stripe_key"),
            (&self.private_key_pem, "private_key_pem"),
            (&self.jwt, "jwt"),
        ]
    }

    fn scan_line(&self, path: &Path, line_no: usize, line: &str, findings: &mut Vec<ScanFinding>) {
        let mut flagged_spans: Vec<(usize, usize)> = Vec::new();

        for (regex, rule_name) in self.shape_rules() {
            for m in regex.find_iter(line) {
                findings.push(ScanFinding {
                    path: path.to_path_buf(),
                    line: line_no,
                    rule_name,
                    redacted_preview: redact_preview(m.as_str()),
                });
                flagged_spans.push((m.start(), m.end()));
            }
        }

        for caps in self.generic_assignment.captures_iter(line) {
            let m = caps.get(1).expect("group 1 always present on match");
            let span = (m.start(), m.end());
            if flagged_spans.iter().any(|&s| overlaps(s, span)) {
                continue;
            }
            if shannon_entropy(m.as_str()) >= ENTROPY_THRESHOLD {
                findings.push(ScanFinding {
                    path: path.to_path_buf(),
                    line: line_no,
                    rule_name: "generic_assignment",
                    redacted_preview: redact_preview(m.as_str()),
                });
                flagged_spans.push(span);
            }
        }

        for regex in [&self.quoted_token, &self.assigned_token] {
            for caps in regex.captures_iter(line) {
                let m = caps.get(1).expect("group 1 always present on match");
                let span = (m.start(), m.end());
                if flagged_spans.iter().any(|&s| overlaps(s, span)) {
                    continue;
                }
                if shannon_entropy(m.as_str()) >= ENTROPY_THRESHOLD {
                    findings.push(ScanFinding {
                        path: path.to_path_buf(),
                        line: line_no,
                        rule_name: "high_entropy_string",
                        redacted_preview: redact_preview(m.as_str()),
                    });
                    flagged_spans.push(span);
                }
            }
        }
    }

    fn scan_file(&self, path: &Path, findings: &mut Vec<ScanFinding>) {
        if is_vunexo_internal_path(path) {
            return;
        }
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() > MAX_SCAN_FILE_BYTES {
                return;
            }
        }
        // Best-effort: silently skip anything that isn't readable UTF-8
        // text (e.g. binary files). Scanning is a heuristic safety net, not
        // a guarantee (product-vunexo-vault.md's hard boundary #2).
        let Ok(content) = fs::read_to_string(path) else {
            return;
        };
        for (i, line) in content.lines().enumerate() {
            self.scan_line(path, i + 1, line, findings);
        }
    }

    fn scan_one(&self, path: &Path, findings: &mut Vec<ScanFinding>) {
        if is_vunexo_internal_path(path) {
            return;
        }
        if path.is_dir() {
            let walker = WalkBuilder::new(path).build();
            for entry in walker.flatten() {
                if entry.file_type().is_some_and(|t| t.is_file()) {
                    self.scan_file(entry.path(), findings);
                }
            }
        } else if path.is_file() {
            self.scan_file(path, findings);
        }
        // A nonexistent path (e.g. a staged file that was subsequently
        // deleted before the hook ran) is silently skipped.
    }
}

impl Scanner for PatternScanner {
    fn scan(&self, paths: &[PathBuf]) -> Vec<ScanFinding> {
        let mut findings = Vec::new();
        for path in paths {
            self.scan_one(path, &mut findings);
        }
        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn findings_for_line(line: &str) -> Vec<ScanFinding> {
        let scanner = PatternScanner::new();
        let mut findings = Vec::new();
        scanner.scan_line(Path::new("fixture.env"), 1, line, &mut findings);
        findings
    }

    // --- Round 6 / crypto-and-scanning-engine.md §5 locked test vectors ---
    // Must be flagged (true positives):

    #[test]
    fn flags_aws_access_key_id() {
        let findings = findings_for_line("AKIAIOSFODNN7EXAMPLE");
        assert!(findings.iter().any(|f| f.rule_name == "aws_access_key_id"));
    }

    #[test]
    fn flags_github_pat() {
        let findings = findings_for_line("ghp_16C7e42F292c6912E7710c838347Ae178B4a");
        assert!(findings.iter().any(|f| f.rule_name == "github_token"));
    }

    #[test]
    fn flags_stripe_live_key() {
        let findings = findings_for_line("sk_test_FAKEFAKEFAKEFAKE1234");
        assert!(findings.iter().any(|f| f.rule_name == "stripe_key"));
    }

    #[test]
    fn flags_pem_private_key_header_regardless_of_what_follows() {
        let findings = findings_for_line("-----BEGIN RSA PRIVATE KEY-----");
        assert!(findings.iter().any(|f| f.rule_name == "private_key_pem"));
    }

    #[test]
    fn flags_jwt_shape() {
        let findings =
            findings_for_line("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dQw4w9WgXcQ");
        assert!(findings.iter().any(|f| f.rule_name == "jwt"));
    }

    #[test]
    fn flags_generic_assignment_with_high_entropy() {
        let findings = findings_for_line(
            r#"AWS_SECRET_ACCESS_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY""#,
        );
        assert!(findings.iter().any(|f| f.rule_name == "generic_assignment"));
    }

    // Must NOT be flagged (true negatives):

    #[test]
    fn does_not_flag_placeholder_api_key() {
        let findings = findings_for_line(r#"API_KEY = "your-api-key-here""#);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn does_not_flag_common_placeholder_password() {
        let findings = findings_for_line(r#"password = "changeme""#);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn does_not_flag_normal_prose() {
        let findings =
            findings_for_line("This is just a normal sentence used only for testing purposes.");
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn does_not_flag_uuid_identifier() {
        let findings = findings_for_line(r#"request_id = "550e8400-e29b-41d4-a716-446655440000""#);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    #[test]
    fn does_not_flag_small_base64_image_data_uri() {
        // The `:`/`;`/`,` inside a data URI aren't in the token charset, so
        // this never forms a single unbroken quoted/assigned candidate —
        // see the module doc comment for why that's the intended behavior.
        let findings =
            findings_for_line(r#"logo = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB""#);
        assert!(findings.is_empty(), "unexpected findings: {findings:?}");
    }

    // --- Additional coverage beyond the locked vectors ---

    #[test]
    fn redacted_preview_never_contains_the_full_secret() {
        let findings = findings_for_line("AKIAIOSFODNN7EXAMPLE");
        for f in &findings {
            assert!(!f.redacted_preview.contains("IOSFODNN7EXAMPLE"));
        }
    }

    #[test]
    fn vunexo_vault_files_are_never_scanned() {
        assert!(is_vunexo_internal_path(Path::new(
            ".vunexo/vault/development.age"
        )));
        assert!(is_vunexo_internal_path(Path::new(
            "nested/.vunexo/vault/staging.age"
        )));
        assert!(is_vunexo_internal_path(Path::new("anywhere/secret.age")));
        assert!(!is_vunexo_internal_path(Path::new("src/main.rs")));
    }
}
