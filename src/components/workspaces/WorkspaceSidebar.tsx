import { AudioLines, FolderOpen, Pencil, Plus } from "lucide-react";

import type {
  WorkspaceInfo,
  WorkspaceView,
} from "@/src/services/workspaces/workspaces.interfaces";

interface WorkspaceSidebarProps {
  library: WorkspaceView;
  locked: boolean;
  onCreate: () => void;
  onHome: () => void;
  onOpen: (item: WorkspaceInfo) => void;
  onRename: (item: WorkspaceInfo) => void;
}

export function WorkspaceSidebar({
  library,
  locked,
  onHome,
  onCreate,
  onOpen,
  onRename,
}: WorkspaceSidebarProps) {
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
        WORKSPACES{" "}
        <span>{library.items.length.toString().padStart(2, "0")}</span>
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
            className={`workspace-item ${library.active?.id === item.id ? "active-workspace" : ""}`}
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
        <span className="small-dot" /> Saved on this computer
      </div>
    </aside>
  );
}
