import { shortcut } from "./shortcuts";

const k = (key: string, mods: Partial<{ meta: boolean; ctrl: boolean; shift: boolean }> = {}) => ({
  key,
  metaKey: !!mods.meta,
  ctrlKey: !!mods.ctrl,
  shiftKey: !!mods.shift,
});

test("macOS uses Cmd", () => {
  expect(shortcut(k("t", { meta: true }), true)).toBe("newTab");
  expect(shortcut(k("d", { meta: true }), true)).toBe("splitRight");
  expect(shortcut(k("D", { meta: true, shift: true }), true)).toBe("splitDown");
  expect(shortcut(k("w", { meta: true }), true)).toBe("closePane");
  expect(shortcut(k("d", { ctrl: true }), true)).toBeNull();
});

test("Windows/Linux leave Ctrl+D and Ctrl+W to the shell", () => {
  expect(shortcut(k("d", { ctrl: true }), false)).toBeNull();
  expect(shortcut(k("w", { ctrl: true }), false)).toBeNull();
  expect(shortcut(k("T", { ctrl: true, shift: true }), false)).toBe("newTab");
  expect(shortcut(k("E", { ctrl: true, shift: true }), false)).toBe("splitRight");
  expect(shortcut(k("O", { ctrl: true, shift: true }), false)).toBe("splitDown");
  expect(shortcut(k("W", { ctrl: true, shift: true }), false)).toBe("closePane");
});
