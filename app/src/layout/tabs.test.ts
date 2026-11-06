import { leaves } from "./panes";
import { empty, reduce, type Tabs } from "./tabs";

const two = (): Tabs => reduce(reduce(empty, { type: "new", tab: "t1", pane: "p1" }), { type: "new", tab: "t2", pane: "p2" });

test("new tab becomes active with one focused pane", () => {
  const s = two();
  expect(s.active).toBe("t2");
  expect(s.tabs.map((t) => t.id)).toEqual(["t1", "t2"]);
  expect(s.tabs[1].focus).toBe("p2");
});

test("split acts on the focused pane of the active tab and focuses the new pane", () => {
  const s = reduce(two(), { type: "split", dir: "row", pane: "p3" });
  const t2 = s.tabs[1];
  expect(leaves(t2.root)).toEqual(["p2", "p3"]);
  expect(t2.focus).toBe("p3");
  expect(leaves(s.tabs[0].root)).toEqual(["p1"]);
});

test("closing a pane focuses a remaining pane of that tab", () => {
  let s = reduce(two(), { type: "split", dir: "col", pane: "p3" });
  s = reduce(s, { type: "closePane", pane: "p3" });
  expect(s.tabs[1].focus).toBe("p2");
});

test("closing the last pane closes the tab and activates its neighbour", () => {
  const s = reduce(two(), { type: "closePane", pane: "p2" });
  expect(s.tabs.map((t) => t.id)).toEqual(["t1"]);
  expect(s.active).toBe("t1");
});

test("a pane exiting in a background tab keeps the active tab", () => {
  const s = reduce(two(), { type: "closePane", pane: "p1" });
  expect(s.active).toBe("t2");
  expect(s.tabs.map((t) => t.id)).toEqual(["t2"]);
});

test("closing the active tab activates the previous one; last tab leaves none", () => {
  let s = reduce(two(), { type: "closeTab", tab: "t2" });
  expect(s.active).toBe("t1");
  s = reduce(s, { type: "closeTab", tab: "t1" });
  expect(s).toEqual(empty);
});

test("newChat opens a chat tab; closing its pane forgets the chat", () => {
  const chat = { agent: "claude", task: null, cwd: "/r" };
  let s = reduce(empty, { type: "newChat", tab: "t1", pane: "p1", chat });
  expect(s.active).toBe("t1");
  expect(s.chats.p1).toEqual(chat);
  s = reduce(s, { type: "split", dir: "row", pane: "p2" });
  expect(s.chats.p2).toBeUndefined(); // a split opens a terminal
  s = reduce(s, { type: "closePane", pane: "p1" });
  expect(s.chats.p1).toBeUndefined();
});
