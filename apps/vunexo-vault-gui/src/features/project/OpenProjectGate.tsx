import { FormEvent, useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { FolderIcon, LockIcon } from "../../components/icons";
import { chooseProjectFolder } from "../../lib/tauri/client";
import { initVault, listRecentProjects, openProject } from "../../lib/tauri/commands";
import type { RecentProject } from "../../lib/tauri/types";

/**
 * gui-ux.md §1 — Open Project (gate). Renders instead of the sidebar layout
 * whenever no project is currently open: an "Open Folder" picker plus a
 * "Recent Projects" list (GUI-only preference state, not part of the
 * portable `.vunexo/` folder). Selecting a folder with no `.vunexo/` yet
 * offers "Initialize a vault here" inline, rather than failing.
 */
export function OpenProjectGate({
  onProjectOpened,
}: {
  onProjectOpened: (path: string, options: { hasVault: boolean; unlocked: boolean }) => void;
}) {
  const [recent, setRecent] = useState<RecentProject[]>([]);
  const [error, setError] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [pendingInit, setPendingInit] = useState<string | null>(null);

  useEffect(() => {
    listRecentProjects().then(setRecent).catch(() => setRecent([]));
  }, []);

  const openPath = async (path: string) => {
    setError(null);
    setBusy(true);
    try {
      const result = await openProject(path);
      if (result.has_vault) {
        onProjectOpened(result.path, { hasVault: true, unlocked: false });
      } else {
        setPendingInit(result.path);
      }
    } catch (err) {
      setError(err);
    } finally {
      setBusy(false);
    }
  };

  const handleOpenFolder = async () => {
    setError(null);
    const path = await chooseProjectFolder();
    if (path) await openPath(path);
  };

  if (pendingInit) {
    return (
      <InitVaultForm
        path={pendingInit}
        onBack={() => setPendingInit(null)}
        onInitialized={() => onProjectOpened(pendingInit, { hasVault: true, unlocked: true })}
      />
    );
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-8 text-text-primary">
      <div className="card w-full max-w-lg p-8">
        <div className="mb-6 text-center">
          <span className="mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-md bg-accent text-white">
            <LockIcon className="h-5 w-5" />
          </span>
          <h1 className="text-xl font-semibold">Vunexo Vault</h1>
          <p className="mt-1 text-sm text-text-secondary">
            Open a project folder that has (or should have) a <code>.vunexo/</code> vault.
          </p>
        </div>

        <ErrorBanner error={error} />

        <button onClick={() => void handleOpenFolder()} disabled={busy} className="btn-primary mt-4 w-full">
          <FolderIcon className="h-4 w-4" />
          {busy ? "Opening…" : "Open Folder"}
        </button>

        {recent.length > 0 && (
          <div className="mt-6">
            <h2 className="mb-2 text-xs font-medium uppercase tracking-wide text-text-muted">Recent Projects</h2>
            <div className="list-card">
              <ul>
                {recent.map((p) => (
                  <li key={p.path} className="border-t border-border first:border-0">
                    <button
                      onClick={() => void openPath(p.path)}
                      disabled={busy}
                      className="flex w-full items-center justify-between gap-3 px-4 py-2.5 text-left text-sm hover:bg-surface-hover disabled:pointer-events-none disabled:opacity-50"
                    >
                      <span className="truncate">{p.path}</span>
                      <span className="shrink-0 text-xs text-text-muted">{p.last_opened.slice(0, 10)}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </div>
    </main>
  );
}

/** The CLI's exact `PassphraseMismatch` wording ("passphrases did not
 * match") — reused here even though this particular check runs client-side
 * against the confirm field, never reaching the backend, since it is the
 * same "two masked entries didn't match" condition the CLI's own
 * `read_new_passphrase` reports under that exact message. */
const PASSPHRASE_MISMATCH_MESSAGE = "passphrases did not match";

function InitVaultForm({
  path,
  onBack,
  onInitialized,
}: {
  path: string;
  onBack: () => void;
  onInitialized: () => void;
}) {
  const [passphrase, setPassphrase] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState<unknown>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    if (passphrase !== confirm) {
      setError({ kind: "general", message: PASSPHRASE_MISMATCH_MESSAGE });
      return;
    }
    setSubmitting(true);
    try {
      await initVault(passphrase);
      onInitialized();
    } catch (err) {
      setError(err);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-8 text-text-primary">
      <div className="card w-full max-w-md p-8">
        <div className="mb-6 text-center">
          <span className="mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-md bg-accent text-white">
            <LockIcon className="h-5 w-5" />
          </span>
          <h1 className="text-xl font-semibold">Initialize a vault here</h1>
          <p className="mt-1 break-all text-sm text-text-secondary">
            No vault found in <code>{path}</code>. Choose a passphrase to create one.
          </p>
        </div>

        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
          <ErrorBanner error={error} />

          <div>
            <label className="label" htmlFor="new-passphrase">
              New passphrase
            </label>
            <input
              id="new-passphrase"
              type="password"
              required
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="input mt-1"
            />
          </div>

          <div>
            <label className="label" htmlFor="confirm-passphrase">
              Confirm passphrase
            </label>
            <input
              id="confirm-passphrase"
              type="password"
              required
              value={confirm}
              onChange={(e) => setConfirm(e.target.value)}
              className="input mt-1"
            />
          </div>

          <div className="flex gap-2 pt-2">
            <button type="submit" disabled={submitting || passphrase === ""} className="btn-primary flex-1">
              {submitting ? "Initializing…" : "Initialize vault"}
            </button>
            <button type="button" onClick={onBack} disabled={submitting} className="btn-secondary">
              Back
            </button>
          </div>
        </form>
      </div>
    </main>
  );
}
