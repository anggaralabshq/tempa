import { render, screen } from "@testing-library/react";
import App from "./App";

test("App renders three regions", () => {
  render(<App />);
  expect(screen.getByRole("navigation", { name: "Projects" })).toBeTruthy();
  expect(screen.getByRole("main", { name: "Terminal" })).toBeTruthy();
  expect(screen.getByRole("complementary", { name: "Contract" })).toBeTruthy();
});
