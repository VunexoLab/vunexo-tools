import { useEffect, useState } from "react";
import { ConfirmDialog } from "../../components/ConfirmDialog";
import { ErrorBanner } from "../../components/ErrorBanner";
import { CopyIcon, EyeIcon, EyeOffIcon } from "../../components/icons";
import { getSecret, listSecrets, removeSecret, setSecret } from "../../lib/tauri/commands";
import { SecretForm } from "./SecretForm";

/**
 * gui-ux.md §3 — Secrets (default section once unlocked). A `.card`-wrapped
 * (here: `.list-card`) table: key name + a masked value column (dots, with a
 * per-row reveal-eye toggle and a copy button — copying doesn't require
 * revealing first), plus row actions (Edit, Remove) as pill buttons. Wraps
 * `application::secrets::{set_secret,get_secret,list_secrets,remove_secret}`
 * exactly.
 */
export function SecretsScreen() {
  const [keys, setKeys] = useState<string[] | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [rowError, setRowError] = useState<unknown>(null);
  const [revealed, setRevealed] = useState<Record<string, string>>({});
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [formTarget, setFormTarget] = useState<"new" | string | null>(null);
  const [removeTarget, setRemoveTarget] = useState<string | null>(null);

  const refresh = () => {
    setError(null);
    return listSecrets()
      .then(setKeys)
      .catch((err) => setError(err));
  };

  useEffect(() => {
    void refresh();
  }, []);

  const toggleReveal = async (key: string) => {
    setRowError(null);
    if (key in revealed) {
      setRevealed((r) => {
        const next = { ...r };
        delete next[key];
        return next;
      });
      return;
    }
    try {
      const value = await getSecret(key);
      setRevealed((r) => ({ ...r, [key]: value }));
    } catch (err) {
      setRowError(err);
    }
  };

  const copyValue = async (key: string) => {
    setRowError(null);
    try {
      const value = revealed[key] ?? (await getSecret(key));
      await navigator.clipboard.writeText(value);
      setCopiedKey(key);
      window.setTimeout(() => setCopiedKey((k) => (k === key ? null : k)), 1500);
    } catch (err) {
      setRowError(err);
    }
  };

  return (
    <div className="space-y-4">
      <div className="page-header">
        <h1 className="text-xl font-semibold">Secrets</h1>
        <button onClick={() => setFormTarget("new")} className="btn-primary btn-sm">
          + Add Secret
        </button>
      </div>

      <ErrorBanner error={error} />
      <ErrorBanner error={rowError} />

      {formTarget !== null && (
        <SecretForm
          existingKey={formTarget === "new" ? undefined : formTarget}
          onCancel={() => setFormTarget(null)}
          onSubmit={async (key, value) => {
            await setSecret(key, value);
            setFormTarget(null);
            setRevealed((r) => {
              const next = { ...r };
              delete next[key];
              return next;
            });
            await refresh();
          }}
        />
      )}

      <div className="list-card">
        <table className="table-base">
          <thead>
            <tr>
              <th>Key</th>
              <th>Value</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {(keys ?? []).map((key) => (
              <tr key={key} className="is-hoverable">
                <td className="font-medium">{key}</td>
                <td>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => void toggleReveal(key)}
                      className="text-text-muted hover:text-text-primary"
                      aria-label={key in revealed ? "Hide value" : "Reveal value"}
                    >
                      {key in revealed ? <EyeOffIcon className="h-4 w-4" /> : <EyeIcon className="h-4 w-4" />}
                    </button>
                    <span className="font-mono text-text-secondary">
                      {key in revealed ? revealed[key] : "••••••••"}
                    </span>
                    <button
                      onClick={() => void copyValue(key)}
                      className="text-text-muted hover:text-text-primary"
                      aria-label="Copy value"
                    >
                      <CopyIcon className="h-4 w-4" />
                    </button>
                    {copiedKey === key && <span className="text-xs text-success">Copied</span>}
                  </div>
                </td>
                <td className="text-right">
                  <div className="flex justify-end gap-1">
                    <button onClick={() => setFormTarget(key)} className="row-action-accent">
                      Edit
                    </button>
                    <button onClick={() => setRemoveTarget(key)} className="row-action-danger">
                      Remove
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {keys !== null && keys.length === 0 && (
        <p className="text-sm text-text-muted">No secrets yet in this environment — click "+ Add Secret" to add one.</p>
      )}

      {removeTarget && (
        <ConfirmDialog
          title="Remove this secret?"
          message={`"${removeTarget}" will be permanently removed from this environment's vault. This can't be undone.`}
          confirmLabel="Remove"
          danger
          onCancel={() => setRemoveTarget(null)}
          onConfirm={async () => {
            await removeSecret(removeTarget);
            setRemoveTarget(null);
            setRevealed((r) => {
              const next = { ...r };
              delete next[removeTarget];
              return next;
            });
            await refresh();
          }}
        />
      )}
    </div>
  );
}
