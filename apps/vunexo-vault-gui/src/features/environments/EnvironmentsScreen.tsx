import { FormEvent, useEffect, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { listEnvironments, setActiveEnvironment } from "../../lib/tauri/commands";
import type { EnvironmentInfo } from "../../lib/tauri/types";

/**
 * gui-ux.md §4 — Environments. A simple list (same active-environment `*`
 * treatment as the CLI's `env list`) with a "New Environment" button (name
 * input, created empty — matching the CLI's own deferred-vault-file-creation
 * behavior) and a "Switch to" action per row. Both actions call the same
 * `use_environment` Tauri command (wrapping
 * `application::environments::use_environment`, which already treats a new
 * name and an existing one identically) via `setActiveEnvironment` here.
 */
export function EnvironmentsScreen() {
  const [environments, setEnvironments] = useState<EnvironmentInfo[] | null>(null);
  const [error, setError] = useState<unknown>(null);
  const [rowError, setRowError] = useState<unknown>(null);
  const [newName, setNewName] = useState("");
  const [creating, setCreating] = useState(false);
  const [switching, setSwitching] = useState<string | null>(null);

  const refresh = () => listEnvironments().then(setEnvironments).catch(setError);

  useEffect(() => {
    void refresh();
  }, []);

  const handleCreate = async (e: FormEvent) => {
    e.preventDefault();
    setRowError(null);
    setCreating(true);
    try {
      await setActiveEnvironment(newName);
      setNewName("");
      await refresh();
    } catch (err) {
      setRowError(err);
    } finally {
      setCreating(false);
    }
  };

  const handleSwitch = async (name: string) => {
    setRowError(null);
    setSwitching(name);
    try {
      await setActiveEnvironment(name);
      await refresh();
    } catch (err) {
      setRowError(err);
    } finally {
      setSwitching(null);
    }
  };

  return (
    <div className="space-y-4">
      <div className="page-header">
        <h1 className="text-xl font-semibold">Environments</h1>
      </div>

      <ErrorBanner error={error} />
      <ErrorBanner error={rowError} />

      <form onSubmit={(e) => void handleCreate(e)} className="flex gap-2">
        <input
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="e.g. staging"
          className="input"
        />
        <button type="submit" disabled={creating || newName.trim() === ""} className="btn-primary whitespace-nowrap">
          {creating ? "Creating…" : "New Environment"}
        </button>
      </form>

      <div className="list-card">
        <table className="table-base">
          <thead>
            <tr>
              <th>Name</th>
              <th>Created</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {(environments ?? []).map((env) => (
              <tr key={env.name} className="is-hoverable">
                <td className="font-medium">
                  <span className="inline-flex items-center gap-2">
                    {env.active && <span className="badge-success">active</span>}
                    {env.name}
                  </span>
                </td>
                <td className="text-text-secondary">{env.created_at}</td>
                <td className="text-right">
                  {!env.active && (
                    <button
                      onClick={() => void handleSwitch(env.name)}
                      disabled={switching === env.name}
                      className="row-action-accent"
                    >
                      {switching === env.name ? "Switching…" : "Switch to"}
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
