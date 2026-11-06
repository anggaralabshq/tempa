export type Command = "newTab" | "splitRight" | "splitDown" | "closePane";
type Key = { key: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean };

/**
 * macOS: Cmd+T, Cmd+D, Cmd+Shift+D, Cmd+W. Elsewhere Ctrl+Shift+T/E/O/W (Terminator),
 * because plain Ctrl+D (EOF) and Ctrl+W (delete word) belong to the shell.
 */
export function shortcut(e: Key, mac: boolean): Command | null {
  const key = e.key.toLowerCase();
  if (mac) {
    if (!e.metaKey || e.ctrlKey) return null;
    if (key === "t" && !e.shiftKey) return "newTab";
    if (key === "d") return e.shiftKey ? "splitDown" : "splitRight";
    if (key === "w" && !e.shiftKey) return "closePane";
    return null;
  }
  if (!e.ctrlKey || !e.shiftKey || e.metaKey) return null;
  return ({ t: "newTab", e: "splitRight", o: "splitDown", w: "closePane" } as const)[key as "t"] ?? null;
}
