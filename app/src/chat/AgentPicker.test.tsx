import { fireEvent, render, screen } from "@testing-library/react";
import AgentPicker from "./AgentPicker";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => [
    { id: "claude", name: "Claude Code", installed: true, hint: null, lifecycle: true },
    { id: "cursor", name: "Cursor", installed: false, hint: "Cursor is not installed: install the Cursor CLI", lifecycle: false },
  ]),
  Channel: class {},
}));

test("lists agents; a missing one is disabled with its install hint", async () => {
  const pick = vi.fn();
  render(<AgentPicker onPick={pick} />);
  const cursor = (await screen.findByRole("button", { name: /Cursor/ })) as HTMLButtonElement;
  expect(cursor.disabled).toBe(true);
  expect(cursor.title).toContain("install the Cursor CLI");
  expect(screen.getByText("context-only")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: /Claude Code/ }));
  expect(pick).toHaveBeenCalledWith("claude");
});
