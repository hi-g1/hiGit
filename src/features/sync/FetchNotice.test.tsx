import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { AppProviders, createQueryClient } from "../../App";
import { FetchNotice } from "./FetchNotice";
import { useSyncStore } from "./syncStore";

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <FetchNotice />
    </AppProviders>,
  );
}

afterEach(() => {
  act(() => useSyncStore.setState({ fetchFailedAt: null, fetchFailedCode: null, fetchNoticeDismissed: false }));
});

describe("원격 확인 실패 알림 줄(D-197)", () => {
  it("실패하면 줄과 해결 버튼 하나, 닫으면 다음 실패까지 숨긴다", async () => {
    act(() => useSyncStore.setState({ fetchFailedAt: Date.now() - 3 * 60_000, fetchFailedCode: "githubExpired" }));
    show();
    expect(screen.getByRole("status")).toHaveTextContent("원격 확인 못 함 · 3분 전");
    expect(screen.getByRole("button", { name: "다시 연결" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "닫기" }));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    act(() => useSyncStore.setState({ fetchFailedAt: Date.now(), fetchNoticeDismissed: false }));
    expect(screen.getByRole("status")).toBeInTheDocument();
  });

  it("해결 버튼이 없는 실패는 닫기만, 성공하면 사라진다", () => {
    act(() => useSyncStore.setState({ fetchFailedAt: Date.now(), fetchFailedCode: "remoteUnreachable" }));
    show();
    expect(screen.getAllByRole("button").map((b) => b.textContent)).toEqual(["닫기"]);
    act(() => useSyncStore.setState({ fetchFailedAt: null, fetchFailedCode: null }));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });
});
