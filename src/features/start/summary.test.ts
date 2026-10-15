import { describe, expect, it } from "vitest";
import type { ProjectSummaryView } from "../../lib/api";
import { branchPrefix, pickLastVersion, pickShort, pickStatus } from "./summary";

const base: ProjectSummaryView = {
  branch: { name: "main", hasCommits: true, upstream: "origin/main", ahead: 0, behind: 0 },
  changedFiles: 0,
  conflictedFiles: 0,
  lastCommit: null,
};
const s = (over: Partial<ProjectSummaryView>, branch: Partial<ProjectSummaryView["branch"]> = {}) => ({
  ...base,
  ...over,
  branch: { ...base.branch, ...branch },
});

describe("상태 고르기", () => {
  it("충돌이 가장 먼저", () => {
    expect(pickStatus(s({ conflictedFiles: 2, changedFiles: 5 }))).toEqual({ kind: "conflict", files: 2 });
    expect(pickShort(s({ conflictedFiles: 2, changedFiles: 5 }))).toEqual({ kind: "conflict", files: 2 });
  });

  it("바뀐 파일, 깨끗함, 특정 버전", () => {
    expect(pickStatus(s({ changedFiles: 3 }, { name: "로그인" }))).toEqual({
      kind: "changed",
      files: 3,
      branch: "로그인",
    });
    expect(pickStatus(s({}))).toEqual({ kind: "clean", branch: "main" });
    expect(pickStatus(s({}, { name: null }))).toEqual({ kind: "detached" });
  });

  it("짧은 상태 우선순위: 바뀐 파일 → 올릴 → 가져올 → 올린 적 없음 → 최신", () => {
    expect(pickShort(s({ changedFiles: 1 }, { ahead: 2 }))).toEqual({ kind: "changed", files: 1 });
    expect(pickShort(s({}, { ahead: 2, behind: 1 }))).toEqual({ kind: "ahead", versions: 2 });
    expect(pickShort(s({}, { behind: 5 }))).toEqual({ kind: "behind", versions: 5 });
    expect(pickShort(s({}, { upstream: null }))).toEqual({ kind: "notUploaded" });
    expect(pickShort(s({}, { upstream: null, hasCommits: false }))).toEqual({ kind: "upToDate" });
    expect(pickShort(s({}))).toEqual({ kind: "upToDate" });
  });

  it("마지막 버전", () => {
    expect(pickLastVersion(s({}))).toEqual({ kind: "none" });
    const commit = {
      oid: "a",
      shortOid: "a",
      parents: [],
      summary: "비밀번호 확인 추가",
      authorName: "지원",
      authorEmail: "j@e",
      authorTime: 1_700_000_000,
    };
    expect(pickLastVersion(s({ lastCommit: commit }))).toEqual({
      kind: "last",
      time: 1_700_000_000,
      summary: "비밀번호 확인 추가",
    });
  });

  it("main·master면 브랜치 이름을 붙이지 않는다", () => {
    expect(branchPrefix(s({}))).toBeNull();
    expect(branchPrefix(s({}, { name: "master" }))).toBeNull();
    expect(branchPrefix(s({}, { name: "디자인 개편" }))).toBe("디자인 개편");
    expect(branchPrefix(s({}, { name: null }))).toBeNull();
  });
});
