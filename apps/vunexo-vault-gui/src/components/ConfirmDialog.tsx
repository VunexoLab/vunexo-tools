import { useState } from "react";
import { Modal } from "./Modal";
import { ErrorBanner } from "./ErrorBanner";

/** The shared confirmation dialog for destructive actions (remove secret, uninstall hook). Mirrors the sibling apps' `ConfirmDialog`. */
export function ConfirmDialog({
  title,
  message,
  confirmLabel = "Confirm",
  danger = false,
  onConfirm,
  onCancel,
  children,
}: {
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
  onConfirm: () => Promise<unknown>;
  onCancel: () => void;
  children?: React.ReactNode;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);

  const handleConfirm = async () => {
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
    } catch (err) {
      setError(err);
      setBusy(false);
    }
  };

  return (
    <Modal onClose={onCancel}>
      <div className="card space-y-3 p-4">
        <h2 className="text-base font-semibold">{title}</h2>
        <p className="text-sm text-text-secondary">{message}</p>
        <ErrorBanner error={error} />
        {children}
        <div className="flex gap-2 pt-2">
          <button onClick={() => void handleConfirm()} disabled={busy} className={danger ? "btn-danger" : "btn-primary"}>
            {busy ? "Working…" : confirmLabel}
          </button>
          <button onClick={onCancel} disabled={busy} className="btn-secondary">
            Cancel
          </button>
        </div>
      </div>
    </Modal>
  );
}
