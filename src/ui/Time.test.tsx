import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { formatExact } from "../lib/time";
import { Time } from "./Time";

describe("시각 부품", () => {
  it("글자는 규칙대로, 호버에는 정확한 시각", () => {
    const now = new Date(2026, 9, 16, 15, 30).getTime();
    const at = Math.floor(now / 1000) - 5 * 60;
    render(<Time at={at} now={now} />);
    const time = screen.getByText("5분 전");
    expect(time.tagName).toBe("TIME");
    expect(time).toHaveAttribute("title", formatExact(at, "ko"));
    expect(time).toHaveAttribute("dateTime", new Date(at * 1000).toISOString());
  });
});
