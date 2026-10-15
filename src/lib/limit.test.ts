import { describe, expect, it } from "vitest";
import { limiter } from "./limit";

describe("동시 실행 묶기", () => {
  it("동시에 max개까지만 돈다", async () => {
    const run = limiter(3);
    let now = 0;
    let peak = 0;
    const task = () =>
      run(async () => {
        now += 1;
        peak = Math.max(peak, now);
        await new Promise((r) => setTimeout(r, 5));
        now -= 1;
        return 1;
      });
    const results = await Promise.all(Array.from({ length: 20 }, task));
    expect(results).toHaveLength(20);
    expect(peak).toBe(3);
  });
});
