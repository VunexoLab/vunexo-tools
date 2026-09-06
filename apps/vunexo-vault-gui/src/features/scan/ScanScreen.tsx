import { useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { isGitRepository, runScan } from "../../lib/tauri/commands";
import type { ScanFinding } from "../../lib/tauri/types";

/**
 * gui-ux.md §5 — Scan. A "Run Scan" button (plus a "Staged only" toggle,
 * enabled only when the open project is a git repository) and a results
 * table: file, line, rule name, redacted preview — the exact same
 * `ScanFinding` shape the CLI prints, never the full matched string. An
 * empty-state "No findings — clean" mirrors the CLI's own colored summary
 * line. Wraps `application::scan::{scan_path,scan_staged}`.
 */
export function ScanScreen() {
  const [isRepo, setIsRepo] = useState<boolean | null>(null);
  const [staged, setStaged] = useState(false);
  const [findings, setFindings] = useState<ScanFinding[] | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    isGitRepository()
      .then(setIsRepo)
      .catch(() => setIsRepo(false));
  }, []);

  const handleScan = async () => {
    setError(null);
    setRunning(true);
    setFindings(null);
    try {
      const results = await runScan(staged && !!isRepo);
      setFindings(results);
    } catch (err) {
      setError(err);
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="page-header">
        <h1 className="text-xl font-semibold">Scan</h1>
        <div className="flex items-center gap-4">
          <label className={`flex items-center gap-2 text-sm ${isRepo ? "text-text-secondary" : "text-text-muted"}`}>
            <input
              type="checkbox"
              checked={staged}
              disabled={!isRepo}
              onChange={(e) => setStaged(e.target.checked)}
            />
            Staged only
          </label>
          <button onClick={() => void handleScan()} disabled={running} className="btn-primary">
            {running ? "Scanning…" : "Run Scan"}
          </button>
        </div>
      </div>

      <ErrorBanner error={error} />

      {findings !== null && findings.length === 0 && (
        <p className="rounded-md border border-success/30 bg-success/10 px-3 py-2 text-sm text-success">
          No findings — clean.
        </p>
      )}

      {findings !== null && findings.length > 0 && (
        <div className="list-card">
          <table className="table-base">
            <thead>
              <tr>
                <th>File</th>
                <th>Line</th>
                <th>Rule</th>
                <th>Preview</th>
              </tr>
            </thead>
            <tbody>
              {findings.map((f, i) => (
                <tr key={`${f.path}:${f.line}:${i}`} className="is-hoverable">
                  <td className="font-mono text-xs">{f.path}</td>
                  <td>{f.line}</td>
                  <td>
                    <span className="badge-warning">{f.rule_name}</span>
                  </td>
                  <td className="font-mono text-text-secondary">{f.redacted_preview}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {findings === null && !running && (
        <p className="text-sm text-text-muted">
          {isRepo
            ? "Scans the current directory (or staged changes) for likely secrets."
            : "Scans the current directory for likely secrets. This folder isn't a git repository, so \"Staged only\" is unavailable."}
        </p>
      )}
    </div>
  );
}
