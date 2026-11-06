import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

const projects = [
  { id: "a", name: "api", path: "/src/api" },
  { id: "b", name: "web", path: "/src/web" },
];

const finding = (severity: string) => ({ code: "x", severity, path: null, message: "" });
const status: Record<string, { drift: unknown[]; findings: unknown[] }> = {
  "/src/api": { drift: [finding("error"), finding("error")], findings: [finding("error"), finding("warning")] },
  "/src/web": { drift: [], findings: [] },
};
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, args?: { root?: string }) =>
    cmd === "projects_list" ? projects : cmd === "contract_status" ? status[args?.root ?? ""] : null,
  ),
  Channel: class {},
}));
let changed: (e: { payload: { root: string } }) => void = () => {};
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_: string, cb: typeof changed) => {
    changed = cb;
    return () => {};
  }),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
vi.mock("./contract/ContractPanel", () => ({ default: () => <h2>Contract</h2> }));
// xterm needs a real canvas; terminals are exercised in the running app.
vi.mock("./layout/Workspace", () => ({
  default: ({ cwd, active }: { cwd: string; active: boolean }) => <div data-testid={`ws:${cwd}`} data-active={active} />,
}));

test("App renders three regions", async () => {
  render(<App />);
  expect(screen.getByRole("navigation", { name: "Projects" })).toBeTruthy();
  expect(screen.getByRole("main", { name: "Terminal" })).toBeTruthy();
  expect(screen.getByRole("complementary", { name: "Contract" })).toBeTruthy();
  await screen.findByText("api");
});

test("switching project keeps the other project's terminals mounted", async () => {
  render(<App />);
  await screen.findByText("web");
  expect(screen.getByTestId("ws:/src/api").dataset.active).toBe("true");
  fireEvent.click(screen.getByText("web"));
  await waitFor(() => expect(screen.getByTestId("ws:/src/web").dataset.active).toBe("true"));
  expect(screen.getByTestId("ws:/src/api").dataset.active).toBe("false");
});

test("every project is watched; the badge counts drift plus contract errors", async () => {
  render(<App />);
  expect(await screen.findByLabelText("3 contract problems")).toBeTruthy();
  expect(invoke).toHaveBeenCalledWith("watch_start", { root: "/src/api" });
  expect(invoke).toHaveBeenCalledWith("watch_start", { root: "/src/web" });
});

test("a watcher update refreshes that project's badge", async () => {
  render(<App />);
  await screen.findByLabelText("3 contract problems");
  status["/src/web"] = { drift: [finding("error")], findings: [] };
  changed({ payload: { root: "/src/web" } });
  expect(await screen.findByLabelText("1 contract problem")).toBeTruthy();
});

test("removing a project stops its watcher", async () => {
  render(<App />);
  fireEvent.click(await screen.findByLabelText("Remove api"));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("watch_stop", { root: "/src/api" }));
});
