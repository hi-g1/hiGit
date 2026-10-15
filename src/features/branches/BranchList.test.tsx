import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { branches: vi.fn(), checkBranchName: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type BranchListView, type BranchRow } from "../../lib/api";
import { BranchList } from "./BranchList";

const row = (name: string, over: Partial<BranchRow> = {}): BranchRow => ({
  name,
  remote: null,
  isHead: false,
  isDefault: false,
  upstream: null,
  ahead: 0,
  behind: 0,
  gone: false,
  lastAuthor: "민지",
  lastAt: 1_700_000_000,
  parkedCount: 0,
  parkBlocked: false,
  ...over,
});

const view: BranchListView = {
  local: [row("로그인 기능", { isHead: true }), row("main", { isDefault: true, behind: 1 }), row("결제 수정", { parkedCount: 2 })],
  remote: [],
  defaultBranch: "main",
  orphanParked: [],
  remotes: ["origin"],
};

function show(onDone = vi.fn()) {
  vi.mocked(api.branches).mockResolvedValue(view);
  vi.mocked(api.checkBranchName).mockResolvedValue({ ok: true, problem: null });
  vi.mocked(api.executeAction).mockResolvedValue({ kind: "branchCreated", name: "x" });
  render(
    <AppProviders client={createQueryClient()}>
      <BranchList repo="/repo" current="로그인 기능" onDone={onDone}>
        로그인 기능
      </BranchList>
    </AppProviders>,
  );
  return onDone;
}

afterEach(() => vi.clearAllMocks());

describe("브랜치 목록", () => {
  it("단추를 누르면 지금·최근 브랜치와 오른쪽 글자", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: /로그인 기능/ }));
    expect(await screen.findByText("최근에 쓴 브랜치")).toBeInTheDocument();
    expect(screen.getByText("새 버전 1개")).toBeInTheDocument();
    expect(screen.getByText("맡겨 둔 파일 2개")).toBeInTheDocument();
  });

  it("고르면 바로 갈아타고 결과를 알린다", async () => {
    const onDone = show();
    await userEvent.click(screen.getByRole("button", { name: /로그인 기능/ }));
    await userEvent.click(await screen.findByRole("option", { name: /main/ }));
    expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "switch", input: { branch: "main", remote: null } });
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("main 브랜치로 갈아탔어요"));
  });

  it("없는 이름을 쓰고 Enter면 새 브랜치", async () => {
    const onDone = show();
    await userEvent.keyboard("{Control>}b{/Control}");
    const input = await screen.findByLabelText("찾기 또는 새로 만들기");
    await userEvent.type(input, "결제 화면");
    expect(await screen.findByText('+ "결제 화면" 새로 만들기')).toBeInTheDocument();
    // 이름 검사가 끝날 때까지 기다린 뒤 Enter.
    await vi.waitFor(() => expect(api.checkBranchName).toHaveBeenCalledWith("/repo", "결제 화면"));
    await userEvent.keyboard("{ArrowDown}{Enter}");
    await vi.waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("/repo", { id: "branch-create", input: { name: "결제 화면" } }),
    );
    await vi.waitFor(() => expect(onDone).toHaveBeenCalledWith("결제 화면 브랜치를 만들었어요"));
  });

  it("쓸 수 없는 이름이면 Enter가 아무것도 하지 않는다", async () => {
    show();
    vi.mocked(api.checkBranchName).mockResolvedValue({ ok: false, problem: "invalid" });
    await userEvent.click(screen.getByRole("button", { name: /로그인 기능/ }));
    const input = await screen.findByLabelText("찾기 또는 새로 만들기");
    await userEvent.type(input, "a..b");
    expect(await screen.findByText("쓸 수 없는 이름")).toBeInTheDocument();
    await userEvent.keyboard("{Enter}");
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("한글 조합 중 Enter는 무시한다", async () => {
    show();
    await userEvent.click(screen.getByRole("button", { name: /로그인 기능/ }));
    const input = await screen.findByLabelText("찾기 또는 새로 만들기");
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", isComposing: true, bubbles: true }));
    expect(api.executeAction).not.toHaveBeenCalled();
  });
});
