import { render, screen } from "@testing-library/react";
import App from "./App";

// xterm needs a real canvas; the terminal is exercised in the running app.
vi.mock("./terminal/Terminal", () => ({ default: () => null }));

test("App renders three regions", () => {
  render(<App />);
  expect(screen.getByRole("navigation", { name: "Projects" })).toBeTruthy();
  expect(screen.getByRole("main", { name: "Terminal" })).toBeTruthy();
  expect(screen.getByRole("complementary", { name: "Contract" })).toBeTruthy();
});
