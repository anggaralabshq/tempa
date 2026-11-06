import { useEffect, useReducer } from "react";
import Terminal from "../terminal/Terminal";
import { leaves, rects } from "./panes";
import { shortcut } from "./shortcuts";
import { empty, reduce } from "./tabs";

let seq = 0;
const nextId = (p: string) => `${p}${++seq}`;
const mac = navigator.platform.toLowerCase().includes("mac");

/** Tabs of split panes; every pane is one terminal in `cwd`. */
export default function Workspace({ cwd, active: shown }: { cwd: string | null; active: boolean }) {
  const [s, dispatch] = useReducer(reduce, empty, (e) => reduce(e, { type: "new", tab: nextId("t"), pane: nextId("p") }));
  const active = s.tabs.find((t) => t.id === s.active);

  const run = (cmd: ReturnType<typeof shortcut>) => {
    if (cmd === "newTab") dispatch({ type: "new", tab: nextId("t"), pane: nextId("p") });
    if (cmd === "splitRight") dispatch({ type: "split", dir: "row", pane: nextId("p") });
    if (cmd === "splitDown") dispatch({ type: "split", dir: "col", pane: nextId("p") });
    if (cmd === "closePane" && active) dispatch({ type: "closePane", pane: active.focus });
  };

  useEffect(() => {
    // Capture phase: runs before xterm's textarea sees the key.
    if (!shown) return; // only the visible project's workspace takes shortcuts
    const onKey = (e: KeyboardEvent) => {
      const cmd = shortcut(e, mac);
      if (!cmd) return;
      e.preventDefault();
      e.stopPropagation();
      run(cmd);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  return (
    <div className="flex h-full flex-col">
      <div role="tablist" className="flex h-8 shrink-0 items-center gap-1 border-b border-white/10 px-1 text-xs">
        {s.tabs.map((t, i) => (
          <div
            key={t.id}
            role="tab"
            aria-selected={t.id === s.active}
            onClick={() => dispatch({ type: "select", tab: t.id })}
            className={`flex cursor-default items-center gap-2 rounded px-2 py-1 ${t.id === s.active ? "bg-white/10" : "text-white/60 hover:bg-white/5"}`}
          >
            Terminal {i + 1}
            <button
              type="button"
              aria-label="Close tab"
              onClick={(e) => {
                e.stopPropagation();
                dispatch({ type: "closeTab", tab: t.id });
              }}
              className="text-white/40 hover:text-white"
            >
              ×
            </button>
          </div>
        ))}
        <button type="button" aria-label="New tab" onClick={() => run("newTab")} className="px-2 text-white/60 hover:text-white">
          +
        </button>
        <div className="ml-auto flex gap-1">
          <button type="button" title="Split right" onClick={() => run("splitRight")} className="px-2 text-white/60 hover:text-white">
            ◫
          </button>
          <button type="button" title="Split down" onClick={() => run("splitDown")} className="px-2 text-white/60 hover:text-white">
            ⊟
          </button>
        </div>
      </div>
      <div className="relative min-h-0 flex-1">
        {s.tabs.length === 0 && (
          <button type="button" onClick={() => run("newTab")} className="absolute inset-0 m-auto h-8 w-40 rounded bg-white/10">
            New terminal
          </button>
        )}
        {/* Flat list keyed by pane id: splitting never remounts an existing terminal. */}
        {s.tabs.flatMap((t) => {
          const r = rects(t.root);
          return leaves(t.root).map((id) => (
            <div
              key={id}
              onMouseDown={() => dispatch({ type: "focus", pane: id })}
              style={{
                left: `${r[id].x}%`,
                top: `${r[id].y}%`,
                width: `${r[id].w}%`,
                height: `${r[id].h}%`,
                display: t.id === s.active ? "block" : "none",
              }}
              className={`absolute border ${t.id === s.active && t.focus === id && leaves(t.root).length > 1 ? "border-sky-500/60" : "border-transparent"}`}
            >
              <Terminal
                cwd={cwd}
                focused={shown && t.id === s.active && t.focus === id}
                onExit={() => dispatch({ type: "closePane", pane: id })}
              />
            </div>
          ));
        })}
      </div>
    </div>
  );
}
