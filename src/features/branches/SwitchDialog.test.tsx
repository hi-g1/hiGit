import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { switchPlan: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type SwitchPlanView } from "../../lib/api";
import { SwitchDialog } from "./SwitchDialog";

function open(plan: SwitchPlanView | Promise<SwitchPlanView>, onChoose = vi.fn()) {
  vi.mocked(api.switchPlan).mockReturnValue(Promise.resolve(plan));
  render(
    <AppProviders client={createQueryClient()}>
      <SwitchDialog
        repo="/repo"
        target={{ branch: "디자인-수정", remote: null }}
        from="main"
        files={3}
        pending={null}
        onChoose={onChoose}
        onClose={vi.fn()}
      />
    </AppProviders>,
  );
  return onChoose;
}

const ok: SwitchPlanView = { dirtyFiles: 3, canCarry: true, reason: null, blocked: [], blockedTotal: 0 };

afterEach(() => vi.clearAllMocks());

describe("브랜치 바꾸기 대화상자(D-195)", () => {
  it("두 선택과 설명, 조회 중에는 둘 다 진행 표시", async () => {
    let resolve: (v: SwitchPlanView) => void = () => {};
    open(new Promise<SwitchPlanView>((r) => (resolve = r)));
    expect(screen.getByRole("dialog", { name: "고친 파일 3개가 있어요" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "새 브랜치로 옮기기" })).toHaveAttribute("aria-busy", "true");
    expect(screen.getByRole("button", { name: "여기 두기" })).toHaveAttribute("aria-busy", "true");
    expect(screen.getByText("고친 파일이 디자인-수정 브랜치로 따라가요")).toBeInTheDocument();
    expect(screen.getByText("main 브랜치에 두고, 돌아오면 다시 꺼내요")).toBeInTheDocument();
    resolve(ok);
    await waitFor(() => expect(screen.getByRole("button", { name: "새 브랜치로 옮기기" })).toHaveFocus());
  });

  it("겹치면 새 브랜치로 옮기기를 끄고 까닭, 포커스는 여기 두기", async () => {
    const onChoose = open({ dirtyFiles: 3, canCarry: false, reason: "overlap", blocked: ["a.txt", "b.txt", "c.txt"], blockedTotal: 3 });
    const leave = screen.getByRole("button", { name: "여기 두기" });
    await waitFor(() => expect(leave).toHaveFocus());
    const carry = screen.getByRole("button", { name: "새 브랜치로 옮기기" });
    expect(carry).toHaveAttribute("aria-disabled", "true");
    expect(carry).toHaveAccessibleDescription("겹치는 파일이 있어요 · a.txt 외 2개");
    await userEvent.keyboard("{Enter}");
    expect(onChoose).toHaveBeenCalledWith("park");
    await userEvent.click(carry);
    expect(onChoose).toHaveBeenCalledTimes(1);
  });

  it("두고 간 파일·너무 많음의 까닭", async () => {
    open({ dirtyFiles: 3, canCarry: false, reason: "parkedThere", blocked: ["x.txt"], blockedTotal: 1 });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "새 브랜치로 옮기기" })).toHaveAccessibleDescription(
        "디자인-수정 브랜치에 두고 간 파일이 있어요",
      ),
    );
  });

  it("그사이 고친 파일이 없어졌으면 묻지 않고 바꾼다", async () => {
    const onChoose = open({ dirtyFiles: 0, canCarry: true, reason: null, blocked: [], blockedTotal: 0 });
    await waitFor(() => expect(onChoose).toHaveBeenCalledWith("park"));
  });
});
