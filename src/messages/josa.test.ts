import { describe, expect, it } from "vitest";
import { josa } from "./josa";

describe("josa", () => {
  it("받침에 맞는 조사를 붙인다", () => {
    expect(josa("브랜치", "이/가")).toBe("브랜치가");
    expect(josa("원격 저장소", "이/가")).toBe("원격 저장소가");
    expect(josa("버전 기록", "이/가")).toBe("버전 기록이");
    expect(josa("합치기", "은/는")).toBe("합치기는");
    expect(josa("충돌", "을/를")).toBe("충돌을");
    expect(josa("브랜치", "이에요/예요")).toBe("브랜치예요");
    expect(josa("태그 이름", "이에요/예요")).toBe("태그 이름이에요");
  });

  it("으로/로는 ㄹ 받침 뒤에도 로", () => {
    expect(josa("파일", "으로/로")).toBe("파일로");
    expect(josa("버전 기록", "으로/로")).toBe("버전 기록으로");
    expect(josa("브랜치", "으로/로")).toBe("브랜치로");
  });

  it("한글이 아니면 받침 없음으로 본다", () => {
    expect(josa("main", "이/가")).toBe("main가");
  });
});
