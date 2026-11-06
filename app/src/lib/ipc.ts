import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type PtyHandlers = { onData: (bytes: Uint8Array) => void; onExit: (code: number) => void };

export async function ptySpawn(cwd: string | null, cols: number, rows: number, h: PtyHandlers): Promise<number> {
  const onData = new Channel<ArrayBuffer>();
  onData.onmessage = (buf) => h.onData(new Uint8Array(buf));
  const onExit = new Channel<number>();
  onExit.onmessage = h.onExit;
  return invoke<number>("pty_spawn", { cwd, cols, rows, onData, onExit });
}

export const ptyWrite = (id: number, data: string) => invoke("pty_write", { id, data });
export const ptyResize = (id: number, cols: number, rows: number) => invoke("pty_resize", { id, cols, rows });
export const ptyKill = (id: number) => invoke("pty_kill", { id });

export type Project = { id: string; name: string; path: string };
export const projectsList = () => invoke<Project[]>("projects_list");
export const projectsAdd = (path: string) => invoke<Project>("projects_add", { path });
export const projectsRemove = (id: string) => invoke("projects_remove", { id });

export type FindingDto = { code: string; severity: "error" | "warning"; path: string | null; message: string };
export type ContractStatus = {
  has_config: boolean;
  offer: "init" | "import" | null;
  findings: FindingDto[];
  drift: FindingDto[];
  agents: { agent: string; files: number; gaps: number; lifecycle: string }[];
  hooks_ok: boolean;
  cli_version: string | null;
  app_version: string;
  error: string | null;
};
export type ContractAction = "absorb" | "discard" | "init" | "import" | "hooks";
export const contractStatus = (root: string) => invoke<ContractStatus>("contract_status", { root });
export const contractPreview = (root: string, action: "absorb" | "discard") =>
  invoke<string[]>("contract_preview", { root, action });
/** `reviewed`: the file list the user confirmed; the backend refuses if it no longer matches. */
export const contractRun = (root: string, action: ContractAction, reviewed: string[]) =>
  invoke<string[]>("contract_run", { root, action, reviewed });

/** What the watcher reports after a source sync or a change in drift. */
export type ContractUpdate = { root: string; synced: string[]; conflicts: string[]; drift: number; error: string | null };
export const watchStart = (root: string) => invoke("watch_start", { root });
export const watchStop = (root: string) => invoke("watch_stop", { root });
export const onContractChanged = (cb: (u: ContractUpdate) => void) =>
  listen<ContractUpdate>("contract-changed", (e) => cb(e.payload));
/** Sidebar badge: drift plus contract errors. */
export const problemCount = (s: ContractStatus) =>
  s.drift.length + s.findings.filter((f) => f.severity === "error").length;
