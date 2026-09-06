import { useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { getAppInfo, lock } from "../../lib/tauri/commands";
import type { AppInfo } from "../../lib/tauri/types";

/**
 * gui-ux.md §7 — Settings / About. Read-only: vault format version (from
 * `config.toml`), the open project's path, app version, license. One action:
 * "Lock" — clears the in-memory passphrase and returns to the Unlock screen
 * without closing the app or forgetting the open project.
 */
export function SettingsScreen({ onLocked }: { onLocked: () => void }) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [locking, setLocking] = useState(false);

  useEffect(() => {
    getAppInfo()
      .then(setInfo)
      .catch((err) => setError(err));
  }, []);

  const handleLock = async () => {
    setError(null);
    setLocking(true);
    try {
      await lock();
      onLocked();
    } catch (err) {
      setError(err);
      setLocking(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="page-header">
        <h1 className="text-xl font-semibold">Settings</h1>
      </div>

      <ErrorBanner error={error} />

      <div className="list-card">
        <Row label="Project path" value={info?.project_path ?? "—"} mono />
        <Row label="Vault format version" value={info ? String(info.format_version) : "—"} />
        <Row label="App version" value={info?.app_version ?? "—"} />
        <Row label="License" value={info?.license ?? "—"} />
      </div>

      <button onClick={() => void handleLock()} disabled={locking} className="btn-danger">
        {locking ? "Locking…" : "Lock"}
      </button>
    </div>
  );
}

function Row({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex items-center justify-between gap-4 border-t border-border px-4 py-3 text-sm first:border-0">
      <span className="text-text-secondary">{label}</span>
      <span className={`truncate text-right font-medium ${mono ? "font-mono text-xs" : ""}`}>{value}</span>
    </div>
  );
}
