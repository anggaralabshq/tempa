import { applyEvent, initial } from "./reducer";

const upd = (update: object) => ({ type: "update", update }) as const;
const chunk = (text: string) => upd({ sessionUpdate: "agent_message_chunk", content: { type: "text", text } });

test("agent chunks join into one message until the user speaks", () => {
  let s = applyEvent(initial, { type: "user", text: "hi" });
  s = applyEvent(applyEvent(s, chunk("Hel")), chunk("lo"));
  expect(s.items).toEqual([
    { kind: "user", text: "hi" },
    { kind: "agent", text: "Hello" },
  ]);
  expect(s.busy).toBe(true);
});

test("tool_call_update merges into its card by id", () => {
  let s = applyEvent(initial, upd({ sessionUpdate: "tool_call", toolCallId: "t1", title: "Read a.ts", kind: "read", status: "pending" }));
  s = applyEvent(s, upd({ sessionUpdate: "tool_call_update", toolCallId: "t1", status: "completed" }));
  expect(s.items).toEqual([{ kind: "tool", id: "t1", title: "Read a.ts", toolKind: "read", status: "completed", locations: [] }]);
});

test("plan update replaces the entries", () => {
  const s = applyEvent(initial, upd({ sessionUpdate: "plan", entries: [{ content: "a", status: "pending", priority: "high" }] }));
  expect(s.plan).toEqual([{ content: "a", status: "pending", priority: "high" }]);
});

test("permission queues and its answer removes it", () => {
  let s = applyEvent(initial, { type: "permission", request: 7, toolCall: { title: "edit" }, options: [] });
  expect(s.pending.map((p) => p.request)).toEqual([7]);
  s = applyEvent(s, { type: "answered", request: 7 });
  expect(s.pending).toEqual([]);
});

test("turn end clears busy; exit ends the chat and drops pending requests", () => {
  let s = applyEvent(applyEvent(initial, { type: "user", text: "x" }), { type: "turn_end", stopReason: "end_turn" });
  expect(s.busy).toBe(false);
  s = applyEvent(applyEvent(s, { type: "permission", request: 1, toolCall: {}, options: [] }), { type: "exit", stderr: "boom" });
  expect(s.ended).toBe(true);
  expect(s.pending).toEqual([]);
  expect(s.items.at(-1)).toEqual({ kind: "note", tone: "error", text: "The agent exited.\nboom" });
});
