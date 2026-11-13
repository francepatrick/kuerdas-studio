import { useState } from "react";
import { ArrowRight, FolderOpen } from "lucide-react";

import type {
  WorkspaceInfo,
  WorkspaceView,
} from "@/src/services/workspaces/workspaces.interfaces";

interface WorkspaceStartProps {
  busy: boolean;
  library: WorkspaceView;
  onCreate: (name: string) => void;
  onOpen: (item: WorkspaceInfo) => void;
}

export function WorkspaceStart({
  library,
  busy,
  onCreate,
  onOpen,
}: WorkspaceStartProps) {
  const [name, setName] = useState("");
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
                    ? "Earlier recordings preserved · name this workspace to continue"
                    : new Date(item.createdAt * 1000).toLocaleDateString(
                        undefined,
                        { month: "short", day: "numeric", year: "numeric" },
                      )}
                </small>
              </span>
              <span className="recent-open-label">
                {item.requiresName ? "Name & open" : "Open"}{" "}
                <ArrowRight size={13} />
              </span>
            </button>
          ))}
        </section>
      )}
    </div>
  );
}
