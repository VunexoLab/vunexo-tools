// The only file in `src/` that imports `@tauri-apps/api` (or a Tauri
// plugin). Everything else calls through `./commands.ts`.
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args);
}

/**
 * OS "open folder" dialog (gui-ux.md §1 — Open Project's "Open Folder"
 * button). Resolves to `null` when the user dismisses it.
 */
export async function chooseProjectFolder(): Promise<string | null> {
  const selected = await open({ multiple: false, directory: true });
  return typeof selected === "string" ? selected : null;
}
