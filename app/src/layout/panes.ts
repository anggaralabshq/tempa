/** Split-pane layout as a binary tree. Pure functions; React state holds the tree. */
export type Pane = { kind: "leaf"; id: string } | { kind: "split"; dir: "row" | "col"; a: Pane; b: Pane };
export type Rect = { x: number; y: number; w: number; h: number };

/** Replace leaf `id` with a split of it and a new leaf `newId` (row = side by side). */
export function split(t: Pane, id: string, dir: "row" | "col", newId: string): Pane {
  if (t.kind === "leaf") {
    return t.id === id ? { kind: "split", dir, a: t, b: { kind: "leaf", id: newId } } : t;
  }
  return { ...t, a: split(t.a, id, dir, newId), b: split(t.b, id, dir, newId) };
}

/** Remove leaf `id`; its sibling takes the parent's place. `null` when nothing is left. */
export function close(t: Pane, id: string): Pane | null {
  if (t.kind === "leaf") return t.id === id ? null : t;
  const a = close(t.a, id);
  const b = close(t.b, id);
  if (a === null) return b;
  if (b === null) return a;
  return { ...t, a, b };
}

export function leaves(t: Pane): string[] {
  return t.kind === "leaf" ? [t.id] : [...leaves(t.a), ...leaves(t.b)];
}

/**
 * Position of every leaf, in percent of the container. Leaves are rendered flat
 * with these rects so a split never remounts (and kills) an existing terminal.
 * ponytail: equal halves; drag-to-resize stores a ratio per split when needed.
 */
export function rects(t: Pane, r: Rect = { x: 0, y: 0, w: 100, h: 100 }): Record<string, Rect> {
  if (t.kind === "leaf") return { [t.id]: r };
  const first: Rect = t.dir === "row" ? { ...r, w: r.w / 2 } : { ...r, h: r.h / 2 };
  const second: Rect = t.dir === "row" ? { ...r, x: r.x + r.w / 2, w: r.w / 2 } : { ...r, y: r.y + r.h / 2, h: r.h / 2 };
  return { ...rects(t.a, first), ...rects(t.b, second) };
}
