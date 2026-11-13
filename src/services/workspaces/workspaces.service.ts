import { invoke } from "@tauri-apps/api/core";

import { isDesktop } from "@/src/config/platform";
import { emptyWorkspaceView } from "@/src/services/workspaces/workspaces.fallback";
import type { WorkspaceView } from "@/src/services/workspaces/workspaces.interfaces";

export function listWorkspaces(): Promise<WorkspaceView> {
  return isDesktop
    ? invoke<WorkspaceView>("workspace_list")
    : Promise.resolve(emptyWorkspaceView);
}

export function createWorkspace(name: string): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("workspace_create", { name });
}

export function openWorkspace(id: string): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("workspace_open", { id });
}

export function renameWorkspace(
  id: string,
  name: string,
): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("workspace_rename", { id, name });
}

export function closeWorkspace(): Promise<WorkspaceView> {
  return invoke<WorkspaceView>("workspace_close");
}
