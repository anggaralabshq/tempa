import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import Chat from "./Chat";

let channel: { onmessage: (e: unknown) => void } | null = null;
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => (cmd === "chat_start" ? { id: 3, current_model: null, models: [] } : null)),
  Channel: class {
    onmessage: (e: unknown) => void = () => {};
    constructor() {
      channel = this;
    }
  },
}));

const open = async () => {
  render(<Chat root="/r" cwd="/r" agent="claude" task={null} focused />);
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("chat_start", expect.objectContaining({ agent: "claude", cwd: "/r" })));
};

test("Stop cancels the running turn", async () => {
  await open();
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "fix it" } });
  fireEvent.click(screen.getByRole("button", { name: "Send" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("chat_prompt", { id: 3, text: "fix it" }));
  fireEvent.click(await screen.findByRole("button", { name: "Stop" }));
  expect(invoke).toHaveBeenCalledWith("chat_cancel", { id: 3 });
});

test("a permission request offers Allow once and Reject, never always", async () => {
  await open();
  channel?.onmessage({
    type: "permission",
    request: 9,
    toolCall: { title: "bun test", kind: "execute" },
    options: [
      { optionId: "a1", name: "Allow", kind: "allow_once" },
      { optionId: "a2", name: "Always allow", kind: "allow_always" },
      { optionId: "r1", name: "Reject", kind: "reject_once" },
    ],
  });
  expect(await screen.findByText("bun test")).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Always allow" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Allow once" }));
  expect(invoke).toHaveBeenCalledWith("chat_permission", { id: 3, request: 9, option: "a1" });
});

test("closing the chat stops the agent", async () => {
  const { unmount } = render(<Chat root="/r" cwd="/r" agent="claude" task={null} focused />);
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("chat_start", expect.anything()));
  unmount();
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("chat_stop", { id: 3 }));
});
