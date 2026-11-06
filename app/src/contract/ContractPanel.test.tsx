import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import ContractPanel from "./ContractPanel";

const status = {
  has_config: true,
  offer: null,
  findings: [{ code: "contract.empty_heading", severity: "error", path: "HARNESS.md", message: 'heading "Gate" is empty' }],
  drift: [{ code: "drift.modified", severity: "error", path: "CLAUDE.md", message: "edited directly" }],
  agents: [{ agent: "claude", files: 4, gaps: 0, lifecycle: "mcp" }],
  hooks_ok: true,
  cli_version: "0.0.9",
  app_version: "0.1.0",
  error: null,
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, args: { action?: string }) => {
    if (cmd === "contract_status") return status;
    if (cmd === "contract_preview") return args.action === "discard" ? ["CLAUDE.md"] : ["AGENTS.md"];
    return [];
  }),
  Channel: class {},
}));

test("shows findings, drift, agents and a version warning", async () => {
  render(<ContractPanel root="/r" />);
  await screen.findByText("CLAUDE.md");
  expect(screen.getByText('heading "Gate" is empty')).toBeTruthy();
  expect(screen.getByText("claude")).toBeTruthy();
  expect(screen.getByText(/0\.0\.9/)).toBeTruthy();
});

test("discard lists the affected files and runs only after confirmation", async () => {
  render(<ContractPanel root="/r" />);
  fireEvent.click(await screen.findByRole("button", { name: "Discard" }));
  await screen.findByText(/overwrite these files/i);
  expect(vi.mocked(invoke)).not.toHaveBeenCalledWith("contract_run", expect.anything());
  fireEvent.click(screen.getByRole("button", { name: "Confirm" }));
  await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith("contract_run", { root: "/r", action: "discard" }));
});
