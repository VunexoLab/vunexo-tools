import { FormEvent, useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { LockIcon } from "../../components/icons";
import { listEnvironments, unlock } from "../../lib/tauri/commands";
import type { EnvironmentInfo } from "../../lib/tauri/types";

/**
 * gui-ux.md §2 — Unlock (gate). Renders instead of the sidebar layout
 * whenever a project is open but not yet unlocked for this session: an
 * environment selector (defaults to the project's `active_environment`) + a
 * masked passphrase field + Unlock. A wrong passphrase shows the same
 * generic "incorrect passphrase" message the CLI gives (via `ErrorBanner`,
 * whose text is always the backend's own `VaultError::WrongPassphrase`
 * wording, never a GUI-authored one) — same oracle-avoidance principle as
 * the CLI.
 */
export function UnlockGate({ projectPath, onUnlocked }: { projectPath: string; onUnlocked: () => void }) {
  const [environments, setEnvironments] = useState<EnvironmentInfo[] | null>(null);
  const [environment, setEnvironment] = useState("");
  const [passphrase, setPassphrase] = useState("");
  const [error, setError] = useState<unknown>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    listEnvironments()
      .then((envs) => {
        setEnvironments(envs);
        setEnvironment(envs.find((e) => e.active)?.name ?? envs[0]?.name ?? "");
      })
      .catch((err) => setError(err));
  }, []);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await unlock(environment, passphrase);
      onUnlocked();
    } catch (err) {
      setError(err);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-8 text-text-primary">
      <div className="card w-full max-w-sm p-8">
        <div className="mb-6 text-center">
          <span className="mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-md bg-accent text-white">
            <LockIcon className="h-5 w-5" />
          </span>
          <h1 className="text-xl font-semibold">Unlock vault</h1>
          <p className="mt-1 break-all text-sm text-text-secondary">{projectPath}</p>
        </div>

        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
          <ErrorBanner error={error} />

          <div>
            <label className="label" htmlFor="unlock-environment">
              Environment
            </label>
            <select
              id="unlock-environment"
              value={environment}
              onChange={(e) => setEnvironment(e.target.value)}
              disabled={!environments || environments.length === 0}
              className="select mt-1"
            >
              {(environments ?? []).map((env) => (
                <option key={env.name} value={env.name}>
                  {env.name}
                  {env.active ? " (active)" : ""}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="label" htmlFor="unlock-passphrase">
              Vault passphrase
            </label>
            <input
              id="unlock-passphrase"
              type="password"
              autoFocus
              required
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
              className="input mt-1"
            />
          </div>

          <button type="submit" disabled={submitting || environment === ""} className="btn-primary w-full">
            {submitting ? "Unlocking…" : "Unlock"}
          </button>
        </form>
      </div>
    </main>
  );
}
