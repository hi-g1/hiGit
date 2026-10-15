import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("앱 이름을 보여준다", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "hiGit" })).toBeInTheDocument();
  });
});
