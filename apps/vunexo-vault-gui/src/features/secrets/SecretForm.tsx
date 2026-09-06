import { FormEvent, useState } from "react";
import { ErrorBanner } from "../../components/ErrorBanner";
import { EyeIcon, EyeOffIcon } from "../../components/icons";
import { Modal } from "../../components/Modal";

/**
 * Add/overwrite-value form for the Secrets screen (gui-ux.md §3). Wraps
 * `application::secrets::set_secret` exactly — no new validation rules
 * beyond what that use case already enforces. Editing an existing key never
 * pre-fills its current value (reveal is always a separate, explicit
 * action) — this form always asks for a fresh value to write, same as the
 * CLI's own `secrets set KEY` always overwriting rather than editing in
 * place.
 */
export function SecretForm({
  existingKey,
  onCancel,
  onSubmit,
}: {
  existingKey?: string;
  onCancel: () => void;
  onSubmit: (key: string, value: string) => Promise<unknown>;
}) {
  const [key, setKey] = useState(existingKey ?? "");
  const [value, setValue] = useState("");
  const [showValue, setShowValue] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await onSubmit(key, value);
    } catch (err) {
      setError(err);
      setSubmitting(false);
    }
  };

  return (
    <Modal onClose={onCancel}>
      <div className="card space-y-4 p-6">
        <h2 className="text-base font-semibold">{existingKey ? `Set new value for ${existingKey}` : "Add secret"}</h2>

        <form onSubmit={(e) => void handleSubmit(e)} className="space-y-4">
          <ErrorBanner error={error} />

          <div>
            <label className="label" htmlFor="secret-key">
              Key
            </label>
            <input
              id="secret-key"
              required
              disabled={!!existingKey}
              value={key}
              onChange={(e) => setKey(e.target.value)}
              placeholder="API_KEY"
              className="input mt-1 disabled:opacity-70"
            />
          </div>

          <div>
            <label className="label" htmlFor="secret-value">
              Value
            </label>
            <div className="relative mt-1">
              <input
                id="secret-value"
                required
                type={showValue ? "text" : "password"}
                value={value}
                onChange={(e) => setValue(e.target.value)}
                className="input pr-10"
              />
              <button
                type="button"
                onClick={() => setShowValue((v) => !v)}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary"
                aria-label={showValue ? "Hide value" : "Show value"}
              >
                {showValue ? <EyeOffIcon className="h-4 w-4" /> : <EyeIcon className="h-4 w-4" />}
              </button>
            </div>
          </div>

          <div className="flex gap-2 pt-2">
            <button type="submit" disabled={submitting || key === "" || value === ""} className="btn-primary flex-1">
              {submitting ? "Saving…" : "Save"}
            </button>
            <button type="button" onClick={onCancel} disabled={submitting} className="btn-secondary">
              Cancel
            </button>
          </div>
        </form>
      </div>
    </Modal>
  );
}
