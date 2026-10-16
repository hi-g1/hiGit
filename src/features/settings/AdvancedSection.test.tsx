import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(async () => ({ gitVersion: "2.45.0", gitSecurity: "ok", gitPackageVersion: null, gitPath: "C:\\Program Files\\Git\\cmd\\git.exe" })),
    openLogFolder: vi.fn(async () => null),
  },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { AdvancedSection } from "./AdvancedSection";

describe("설정 고급 구역(D-160)", () => {
  it("Git 위치를 읽기 전용으로 보이고 로그 폴더를 연다", async () => {
    render(
      <AppProviders client={createQueryClient()}>
        <AdvancedSection />
      </AppProviders>,
    );
    expect(screen.getByRole("heading", { name: "고급" })).toBeInTheDocument();
    expect(await screen.findByText("C:\\Program Files\\Git\\cmd\\git.exe")).toBeInTheDocument();
    expect(screen.queryByRole("textbox")).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "로그 폴더 열기" }));
    expect(api.openLogFolder).toHaveBeenCalledTimes(1);
  });
});
