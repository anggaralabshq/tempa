import { close, leaves, type Pane, rects, split } from "./panes";

const leaf = (id: string): Pane => ({ kind: "leaf", id });

test("split replaces the leaf with a split", () => {
  expect(split(leaf("a"), "a", "row", "b")).toEqual({ kind: "split", dir: "row", a: leaf("a"), b: leaf("b") });
});

test("split targets the right leaf in a tree", () => {
  const t = split(split(leaf("a"), "a", "row", "b"), "b", "col", "c");
  expect(t).toEqual({ kind: "split", dir: "row", a: leaf("a"), b: { kind: "split", dir: "col", a: leaf("b"), b: leaf("c") } });
});

test("close promotes the sibling", () => {
  const t = split(split(leaf("a"), "a", "row", "b"), "b", "col", "c");
  expect(close(t, "b")).toEqual({ kind: "split", dir: "row", a: leaf("a"), b: leaf("c") });
  expect(close(t, "a")).toEqual({ kind: "split", dir: "col", a: leaf("b"), b: leaf("c") });
});

test("close of the last leaf returns null", () => {
  expect(close(leaf("a"), "a")).toBeNull();
});

test("close of an unknown leaf changes nothing", () => {
  const t = split(leaf("a"), "a", "row", "b");
  expect(close(t, "zz")).toEqual(t);
});

test("leaves lists in visual order", () => {
  const t = split(split(leaf("a"), "a", "col", "b"), "a", "row", "c");
  expect(leaves(t)).toEqual(["a", "c", "b"]);
});

test("rects split the space in halves, as percentages", () => {
  const t = split(split(leaf("a"), "a", "row", "b"), "b", "col", "c");
  expect(rects(t)).toEqual({
    a: { x: 0, y: 0, w: 50, h: 100 },
    b: { x: 50, y: 0, w: 50, h: 50 },
    c: { x: 50, y: 50, w: 50, h: 50 },
  });
});
