export interface WorkspaceInfo {
  createdAt: number;
  id: string;
  legacy: boolean;
  name: string;
  requiresName: boolean;
}

export interface WorkspaceView {
  active: WorkspaceInfo | null;
  items: WorkspaceInfo[];
}

export type WorkspaceDialogMode = "create" | "rename" | "recover";

export interface WorkspaceDialogState {
  id?: string;
  mode: WorkspaceDialogMode;
  name: string;
}
