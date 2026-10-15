import { describe, expect, it } from "vitest";
import type { ConflictHunkView } from "../../lib/api";
import {
  canFinish,
  customStart,
  hunkPick,
  nextUnresolved,
  pickHunk,
  progress,
  savedPreview,
  step,
  toResolutions,
  type ConflictFile,
  type Picks,
} from "./model";

const files: ConflictFile[] = [
  { path: "a.txt", kind: "text", hunks: 2 },
  { path: "그림.png", kind: "whole", hunks: 1 },
];

const hunk: ConflictHunkView = {
  index: 0,
  line: 14,
  ours: ["내 줄"],
  base: ["원래 줄"],
  theirs: ["남 줄 1", "남 줄 2"],
  before: ["앞"],
  after: ["뒤"],
};

describe("충돌 모델", () => {
  it("남은 곳을 센다", () => {
    let picks: Picks = {};
    expect(progress(files, picks)).toEqual({ total: 3, remaining: 3, byFile: { "a.txt": 2, "그림.png": 1 } });
    picks = pickHunk(picks, files[0]!, 1, { kind: "theirs" });
    expect(progress(files, picks).remaining).toBe(2);
    expect(hunkPick(picks, "a.txt", 0)).toBeNull();
    expect(hunkPick(picks, "a.txt", 1)).toEqual({ kind: "theirs" });
    picks = pickHunk(picks, files[0]!, 0, { kind: "both" });
    picks = { ...picks, "그림.png": { kind: "theirs" } };
    expect(progress(files, picks).remaining).toBe(0);
    expect(canFinish(files, picks)).toBe(true);
    expect(canFinish([], {})).toBe(true);
  });

  it("저장될 모습은 맥락과 고른 쪽의 줄", () => {
    expect(savedPreview(hunk, null)).toEqual([
      { text: "앞", from: "context" },
      { text: "뒤", from: "context" },
    ]);
    expect(savedPreview(hunk, { kind: "both" }).map((l) => l.from)).toEqual([
      "context",
      "ours",
      "theirs",
      "theirs",
      "context",
    ]);
    expect(savedPreview(hunk, { kind: "custom", text: "직접\n고침" }).map((l) => l.text)).toEqual([
      "앞",
      "직접",
      "고침",
      "뒤",
    ]);
    expect(customStart(hunk, null)).toBe("내 줄");
    expect(customStart(hunk, { kind: "theirs" })).toBe("남 줄 1\n남 줄 2");
  });

  it("다음 안 고른 곳과 이동", () => {
    let picks: Picks = pickHunk({}, files[0]!, 0, { kind: "ours" });
    expect(nextUnresolved(files, picks, null)).toEqual({ path: "a.txt", hunk: 1 });
    expect(nextUnresolved(files, picks, { path: "a.txt", hunk: 1 })).toEqual({ path: "그림.png", hunk: 0 });
    picks = pickHunk(picks, files[0]!, 1, { kind: "ours" });
    picks = { ...picks, "그림.png": { kind: "ours" } };
    expect(nextUnresolved(files, picks, null)).toBeNull();
    expect(step(files, { path: "a.txt", hunk: 1 }, 1)).toEqual({ path: "그림.png", hunk: 0 });
    expect(step(files, { path: "a.txt", hunk: 0 }, -1)).toEqual({ path: "a.txt", hunk: 0 });
  });

  it("core 입력으로 바꾼다", () => {
    let picks: Picks = pickHunk({}, files[0]!, 0, { kind: "custom", text: "글" });
    picks = pickHunk(picks, files[0]!, 1, { kind: "both" });
    picks = { ...picks, "그림.png": { kind: "theirs" } };
    expect(toResolutions(files, picks)).toEqual([
      {
        path: "a.txt",
        choice: { kind: "hunks", choices: [{ kind: "custom", text: "글" }, { kind: "both" }] },
      },
      { path: "그림.png", choice: { kind: "theirs" } },
    ]);
  });
});
