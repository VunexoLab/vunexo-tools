import { useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { hookStatus, installHook, uninstallHook } from "../../lib/tauri/commands";

/**
 * gui-ux.md §6 — Hooks. A single toggle-shaped control: "Pre-commit
 * protection: Installed / Not installed," with an Install/Uninstall button
 * that wraps the same `HookInstaller` port the CLI's `hooks install`/
 * `uninstall` use. If a foreign (non-Vunexo) hook is already present,
 * `install` refuses with the same `HookAlreadyExists` message as the CLI
 * (surfaced verbatim by `ErrorBanner`) rather than silently doing nothing.
 */
export function HooksScreen() {
  const [installed, setInstalled] = useState<boolean | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  const refresh = () => hookStatus().then(setInstalled).catch((err) => setError(err));

  useEffect(() => {
    void refresh();
  }, []);

  const handleToggle = async () => {
    setError(null);
    setBusy(true);
    try {
      if (installed) {
        await uninstallHook();
      } else {
        await installHook();
      }
      await refresh();
    } catch (err) {
      setError(err);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="page-header">
        <h1 className="text-xl font-semibold">Hooks</h1>
      </div>

      <ErrorBanner error={error} />

      <div className="card flex items-center justify-between p-6">
        <div>
          <p className="text-sm font-medium">Pre-commit protection</p>
          <p className="mt-0.5 text-sm text-text-secondary">
            {installed === null ? "Checking…" : installed ? "Installed" : "Not installed"}
          </p>
          <p className="mt-2 max-w-md text-xs text-text-muted">
            Runs <code>vunexo secrets scan --staged</code> before every commit and blocks it if a likely secret is
            found.
          </p>
        </div>
        <button
          onClick={() => void handleToggle()}
          disabled={busy || installed === null}
          className={installed ? "btn-danger" : "btn-primary"}
        >
          {busy ? "Working…" : installed ? "Uninstall" : "Install"}
        </button>
      </div>
    </div>
  );
}
