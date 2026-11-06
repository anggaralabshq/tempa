import { close, leaves, type Pane, split } from "./panes";

export type Tab = { id: string; root: Pane; focus: string };
/** A chat pane: `task` null = Explore in the project root. Panes without one are terminals. */
export type ChatSpec = { agent: string; task: string | null; cwd: string };
export type Tabs = { tabs: Tab[]; active: string | null; chats: Record<string, ChatSpec> };
export type Action =
  | { type: "new"; tab: string; pane: string }
  | { type: "newChat"; tab: string; pane: string; chat: ChatSpec }
  | { type: "select"; tab: string }
  | { type: "closeTab"; tab: string }
  | { type: "split"; dir: "row" | "col"; pane: string }
  | { type: "focus"; pane: string }
  | { type: "closePane"; pane: string };

export const empty: Tabs = { tabs: [], active: null, chats: {} };

/** Forget chat specs of panes that no longer exist. */
function prune(s: Tabs): Tabs {
  const live = new Set(s.tabs.flatMap((t) => leaves(t.root)));
  const chats = Object.fromEntries(Object.entries(s.chats).filter(([p]) => live.has(p)));
  return { ...s, chats };
}

/** Active tab after removing `index`: the previous one, else the next, else none. */
function neighbour(tabs: Tab[], index: number): string | null {
  return (tabs[index - 1] ?? tabs[index + 1])?.id ?? null;
}

function removeTab(s: Tabs, id: string): Tabs {
  const i = s.tabs.findIndex((t) => t.id === id);
  if (i < 0) return s;
  return prune({ ...s, tabs: s.tabs.filter((t) => t.id !== id), active: s.active === id ? neighbour(s.tabs, i) : s.active });
}

/** Pure tab/pane state. Ids come in the action so the reducer stays deterministic. */
export function reduce(s: Tabs, a: Action): Tabs {
  switch (a.type) {
    case "new":
      return { ...s, tabs: [...s.tabs, { id: a.tab, root: { kind: "leaf", id: a.pane }, focus: a.pane }], active: a.tab };
    case "newChat": {
      const t = reduce(s, { type: "new", tab: a.tab, pane: a.pane });
      return { ...t, chats: { ...t.chats, [a.pane]: a.chat } };
    }
    case "select":
      return s.tabs.some((t) => t.id === a.tab) ? { ...s, active: a.tab } : s;
    case "closeTab":
      return removeTab(s, a.tab);
    case "split":
      return {
        ...s,
        tabs: s.tabs.map((t) => (t.id === s.active ? { ...t, root: split(t.root, t.focus, a.dir, a.pane), focus: a.pane } : t)),
      };
    case "focus":
      return { ...s, tabs: s.tabs.map((t) => (leaves(t.root).includes(a.pane) ? { ...t, focus: a.pane } : t)) };
    case "closePane": {
      const tab = s.tabs.find((t) => leaves(t.root).includes(a.pane));
      if (!tab) return s;
      const root = close(tab.root, a.pane);
      if (root === null) return removeTab(s, tab.id);
      const focus = tab.focus === a.pane ? leaves(root)[0] : tab.focus;
      return prune({ ...s, tabs: s.tabs.map((t) => (t.id === tab.id ? { ...t, root, focus } : t)) });
    }
  }
}
