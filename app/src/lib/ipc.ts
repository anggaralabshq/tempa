import { Channel, invoke } from "@tauri-apps/api/core";

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
