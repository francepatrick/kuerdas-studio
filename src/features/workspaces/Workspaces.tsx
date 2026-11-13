import { useState } from 'react';
import { AudioLines, FolderOpen, Plus, Pencil, ArrowRight } from 'lucide-react';
import type { WorkspaceInfo, WorkspaceView } from '../../lib/audio';
export type WorkspaceDialogState = {
  mode: 'create' | 'rename' | 'recover';
  id?: string;
  name: string;
};
export function WorkspaceSidebar({
  library,
  locked,
  onHome,
  onCreate,
  onOpen,
  onRename,
}: {
  library: WorkspaceView;
  locked: boolean;
  onHome: () => void;
  onCreate: () => void;
  onOpen: (item: WorkspaceInfo) => void;
  onRename: (item: WorkspaceInfo) => void;
}) {
  return (
    <aside className="sidebar workspace-sidebar">
      <button
        className="brand"
        onClick={onHome}
        disabled={locked}
        aria-label="Workspace home"
      >
        <span className="brand-mark">
          <AudioLines size={25} />
        </span>
        <span>
          kuerdas<span className="brand-sub">STUDIO</span>
        </span>
      </button>
      <div className="workspace-label">
        WORKSPACES{' '}
        <span>{library.items.length.toString().padStart(2, '0')}</span>
      </div>
      <button
        className="workspace-create-button"
        disabled={locked}
        onClick={onCreate}
      >
        <Plus size={16} /> New workspace
      </button>
      <div className="workspace-list">
        {library.items.map((item) => (
          <div
            key={item.id}
            className={`workspace-item ${library.active?.id === item.id ? 'active-workspace' : ''}`}
          >
            <button
              className="workspace-open-button"
              onClick={() => onOpen(item)}
              disabled={locked}
              title={item.name}
            >
              <FolderOpen size={17} />
              <span>{item.name}</span>
              {library.active?.id === item.id && <span className="small-dot" />}
            </button>
            <button
              className="workspace-rename-button"
              aria-label={`Rename workspace ${item.name}`}
              title="Rename workspace"
              disabled={locked}
              onClick={() => onRename(item)}
            >
              <Pencil size={12} />
            </button>
          </div>
        ))}
      </div>
      <div className="workspace-local-note">
        <span className="small-dot" /> Saved on this Mac
      </div>
    </aside>
  );
}
export function WorkspaceStart({
  library,
  busy,
  onCreate,
  onOpen,
}: {
  library: WorkspaceView;
  busy: boolean;
  onCreate: (name: string) => void;
  onOpen: (item: WorkspaceInfo) => void;
}) {
  const [name, setName] = useState('');
  return (
    <div className="workspace-start">
      <div className="start-heading">
        <span className="eyebrow">KUERDAS RECORDING STUDIO</span>
        <h1>A workspace for your next recording.</h1>
        <p>
          Give your session a name. Keep its tracks, takes and edits together.
        </p>
      </div>
      <form
        className="workspace-start-form"
        onSubmit={(e) => {
          e.preventDefault();
          if (name.trim()) onCreate(name.trim());
        }}
      >
        <label htmlFor="workspace-name">Workspace name</label>
        <div>
          <input
            id="workspace-name"
            autoFocus
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="e.g. Midnight guitar session"
            maxLength={80}
            disabled={busy}
          />
          <button className="primary-button" disabled={busy || !name.trim()}>
            Create workspace <ArrowRight size={15} />
          </button>
        </div>
      </form>
      {library.items.length > 0 && (
        <section className="recent-workspaces">
          <h2>Your workspaces</h2>
          {library.items.map((item) => (
            <button
              key={item.id}
              disabled={busy}
              onClick={() => onOpen(item)}
              className="recent-workspace"
            >
              <span className="recent-workspace-icon">
                <FolderOpen size={22} />
              </span>
              <span>
                <strong>{item.name}</strong>
                <small>
                  {item.requiresName
                    ? 'Earlier recordings preserved · name this workspace to continue'
                    : new Date(item.createdAt * 1000).toLocaleDateString(
                        undefined,
                        { month: 'short', day: 'numeric', year: 'numeric' },
                      )}
                </small>
              </span>
              <span className="recent-open-label">
                {item.requiresName ? 'Name & open' : 'Open'}{' '}
                <ArrowRight size={13} />
              </span>
            </button>
          ))}
        </section>
      )}
    </div>
  );
}
export function WorkspaceNameDialog({
  dialog,
  busy,
  onClose,
  onSubmit,
}: {
  dialog: WorkspaceDialogState;
  busy: boolean;
  onClose: () => void;
  onSubmit: (name: string) => void;
}) {
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
          if (e.key === 'Escape' && !busy) onClose();
        }}
      >
        <span className="eyebrow">YOUR RECORDING WORKSPACE</span>
        <h2 id="workspace-dialog-title">
          {dialog.mode === 'create'
            ? 'New workspace'
            : dialog.mode === 'recover'
              ? 'Name your existing recordings'
              : 'Rename workspace'}
        </h2>
        {dialog.mode === 'recover' && (
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
              ? 'Saving…'
              : dialog.mode === 'rename'
                ? 'Save name'
                : 'Create & open'}
          </button>
        </div>
      </form>
    </div>
  );
}
