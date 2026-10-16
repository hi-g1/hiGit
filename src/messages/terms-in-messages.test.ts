// @vitest-environment node
// 오류 문장 안의 용어는 glossary에서 꺼낸다(D-029). 용어표가 바뀌면 문장도 바뀌어야 한다.
import { afterEach, describe, expect, it, vi } from "vitest";
import { ko as glossary } from "../glossary/ko";

afterEach(() => {
  vi.doUnmock("../glossary/ko");
  vi.resetModules();
});

describe("문장 속 용어", () => {
  it("지금 용어표의 쉬운 이름이 들어 있다", async () => {
    const { ko } = await import("./ko");
    expect(ko.errors.branchExists).toContain(glossary.branch.friendly);
    expect(ko.errors.pushRejected).toContain(glossary.push.friendly);
    expect(ko.errors.pushRejected).toContain(glossary.pull.friendly);
    expect(ko.errors.remoteUnreachable).toContain(glossary.remote.friendly);
    expect(ko.changeKind.conflicted).toBe(glossary.conflict.friendly);
    // 검토 요청 화면(반복 4)
    expect(ko.pr.currentBranch).toContain(glossary.branch.friendly);
    expect(ko.pr.base).toContain(glossary.branch.friendly);
    expect(ko.pr.connectToSee).toContain(glossary.pr.friendly);
    expect(ko.clone.run).toBe(glossary.clone.friendly);
  });

  it("용어표 이름을 바꾸면 문장과 조사가 함께 바뀐다", async () => {
    vi.doMock("../glossary/ko", async (importOriginal) => {
      const real = await importOriginal<typeof import("../glossary/ko")>();
      return {
        ko: {
          ...real.ko,
          branch: { ...real.ko.branch, friendly: "작업 줄기" },
          merge: { ...real.ko.merge, friendly: "병합" },
        },
      };
    });
    const { ko } = await import("./ko");
    expect(ko.errors.branchExists).toBe("같은 이름의 작업 줄기가 이미 있어요.");
    expect(ko.errors.noUpstream).toContain("작업 줄기예요.");
    expect(ko.errors.notFastForward).toContain("먼저 병합을 해 주세요.");
    expect(ko.pr.currentBranch).toBe("지금 작업 줄기");
    expect(ko.pr.base).toBe("대상 작업 줄기");
  });
});
