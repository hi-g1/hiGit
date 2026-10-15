import { describe, expect, it, vi } from "vitest";

vi.mock("./bindings", () => ({
  commands: {
    projectOpen: vi.fn(),
  },
}));

import { ApiError, api, toApiError } from "./api";
import { commands } from "./bindings";

describe("api", () => {
  it("성공한 결과를 그대로 돌려준다", async () => {
    vi.mocked(commands.projectOpen).mockResolvedValue({ kind: "notARepo", folder: "/p" });
    await expect(api.openProject("/p", "picked")).resolves.toEqual({ kind: "notARepo", folder: "/p" });
    expect(commands.projectOpen).toHaveBeenCalledWith("/p", "picked");
  });

  it("백엔드 오류를 ApiError로 바꾼다", async () => {
    vi.mocked(commands.projectOpen).mockRejectedValue({ code: "folderNotFound", detail: "원문" });
    const error = await api.openProject("/없음", "recent").catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect(error).toMatchObject({ code: "folderNotFound", detail: "원문" });
  });

  it("모르는 오류는 internal로 바꾼다", () => {
    expect(toApiError("문자열 오류")).toMatchObject({ code: "internal", detail: "문자열 오류" });
  });
});
