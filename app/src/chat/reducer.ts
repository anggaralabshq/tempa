import type { ChatEvent } from "../lib/ipc";

export type Item =
  | { kind: "user" | "agent"; text: string }
  | { kind: "tool"; id: string; title: string; toolKind: string; status: string; locations: { path: string }[] }
  | { kind: "note"; tone: "error" | "info"; text: string };
export type PlanEntry = { content: string; status: string; priority: string };
export type Option = { optionId: string; name: string; kind: string };
export type Pending = { request: number; toolCall: { title?: string; kind?: string }; options: Option[] };
export type State = { items: Item[]; plan: PlanEntry[]; pending: Pending[]; busy: boolean; ended: boolean };

export const initial: State = { items: [], plan: [], pending: [], busy: false, ended: false };

const note = (s: State, tone: "error" | "info", text: string): State => ({ ...s, items: [...s.items, { kind: "note", tone, text }] });

/** ACP `session/update` payloads, see agentclientprotocol.com/protocol/prompt-turn. */
function update(s: State, u: ChatEvent): State {
  switch (u.sessionUpdate) {
    case "agent_message_chunk": {
      const text = u.content?.type === "text" ? String(u.content.text) : "";
      const last = s.items.at(-1);
      if (last?.kind === "agent") return { ...s, items: [...s.items.slice(0, -1), { ...last, text: last.text + text }] };
      return { ...s, items: [...s.items, { kind: "agent", text }] };
    }
    case "tool_call":
      return {
        ...s,
        items: [
          ...s.items,
          { kind: "tool", id: u.toolCallId, title: u.title ?? "", toolKind: u.kind ?? "other", status: u.status ?? "pending", locations: u.locations ?? [] },
        ],
      };
    case "tool_call_update":
      return {
        ...s,
        items: s.items.map((i) =>
          i.kind === "tool" && i.id === u.toolCallId
            ? { ...i, title: u.title ?? i.title, status: u.status ?? i.status, locations: u.locations ?? i.locations }
            : i,
        ),
      };
    case "plan":
      return { ...s, plan: u.entries ?? [] };
    default:
      return s; // thoughts, commands, mode changes: not shown in v1
  }
}

export function applyEvent(s: State, e: ChatEvent): State {
  switch (e.type) {
    case "user":
      return { ...s, busy: true, items: [...s.items, { kind: "user", text: e.text }] };
    case "update":
      return update(s, e.update);
    case "permission":
      return { ...s, pending: [...s.pending, { request: e.request, toolCall: e.toolCall ?? {}, options: e.options ?? [] }] };
    case "answered":
      return { ...s, pending: s.pending.filter((p) => p.request !== e.request) };
    case "turn_end":
      return { ...s, busy: false, pending: [] };
    case "error":
      return note({ ...s, busy: false }, "error", e.message);
    case "exit":
      return note({ ...s, busy: false, ended: true, pending: [] }, "error", `The agent exited.${e.stderr ? `\n${e.stderr}` : ""}`);
    default:
      return s;
  }
}
