import { describe, expect, it } from "vitest";
import { REMOTE_ACTIONS } from "./useProgress";

describe("원격 작업 목록", () => {
  it("core의 목록을 바인딩으로 받는다(복제·덮어쓰며 올리기 포함, 계획 010 R7)", () => {
    expect([...REMOTE_ACTIONS].sort()).toEqual([
      "clone",
      "fetch",
      "force-push",
      "pull",
      "push",
      "remote-branch-delete",
      "remote-branch-rename",
      "sync",
      "tag-push",
    ]);
  });
});
