import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";

const projects = [
  { id: "a", name: "api", path: "/src/api" },
  { id: "b", name: "web", path: "/src/web" },
];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => (cmd === "projects_list" ? projects : null)),
  Channel: class {},
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));
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
