import { useState } from "react";

import type { WorkspaceDialogState } from "@/src/services/workspaces/workspaces.interfaces";

interface WorkspaceNameDialogProps {
  busy: boolean;
  dialog: WorkspaceDialogState;
  onClose: () => void;
  onSubmit: (name: string) => void;
}

export function WorkspaceNameDialog({
  dialog,
  busy,
  onClose,
  onSubmit,
}: WorkspaceNameDialogProps) {
  const [name, setName] = useState(dialog.name);
  return (
    <div className="modal-backdrop" onClick={() => !busy && onClose()}>
      <form
        className="modal workspace-name-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="workspace-dialog-title"
        onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => {
          e.preventDefault();
          if (name.trim()) onSubmit(name.trim());
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape" && !busy) onClose();
        }}
      >
        <span className="eyebrow">YOUR RECORDING WORKSPACE</span>
        <h2 id="workspace-dialog-title">
          {dialog.mode === "create"
            ? "New workspace"
            : dialog.mode === "recover"
              ? "Name your existing recordings"
              : "Rename workspace"}
        </h2>
        {dialog.mode === "recover" && (
          <p>
            Your tracks and audio files have been preserved. Give this workspace
            a name to continue.
          </p>
        )}
        <label>
          Workspace name
          <input
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            maxLength={80}
            placeholder="e.g. My guitar recordings"
            disabled={busy}
          />
        </label>
        <div className="modal-actions">
          <button
            type="button"
            className="subtle-button"
            disabled={busy}
            onClick={onClose}
          >
            Cancel
          </button>
          <button className="primary-button" disabled={busy || !name.trim()}>
            {busy
              ? "Saving…"
              : dialog.mode === "rename"
                ? "Save name"
                : "Create & open"}
          </button>
        </div>
      </form>
    </div>
  );
}
