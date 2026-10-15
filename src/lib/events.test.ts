import { describe, expect, it, vi } from "vitest";

const listeners = new Map<string, (e: { payload: unknown }) => void>();
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, cb: (e: { payload: unknown }) => void) => {
    listeners.set(name, cb);
    return () => listeners.delete(name);
  }),
}));

import { onRepoChanged, onWatchStatus } from "./events";

describe("events", () => {
  it("Rust가 내보낸 이벤트 이름으로 구독하고 페이로드를 넘긴다", async () => {
    const changed = vi.fn();
    const status = vi.fn();
    await onRepoChanged(changed);
    await onWatchStatus(status);

    listeners.get("repo-changed")?.({ payload: { repo: "/r", worktree: true, git: false } });
    listeners.get("watch-status")?.({ payload: { repo: "/r", active: false, reason: "한도" } });

    expect(changed).toHaveBeenCalledWith({ repo: "/r", worktree: true, git: false });
    expect(status).toHaveBeenCalledWith({ repo: "/r", active: false, reason: "한도" });
  });
});
