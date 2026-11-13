import { isTauri } from "@tauri-apps/api/core";

/** True inside the Tauri desktop shell; false in the browser preview. */
export const isDesktop = isTauri();
