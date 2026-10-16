import { act, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError } from "../lib/api";
import { TOAST_DURATION_MS, ToastHost } from "./Toast";
import { useToastStore } from "./toastStore";

afterEach(() => {
  vi.useRealTimers();
  act(() => useToastStore.getState().clear());
});

describe("오류 토스트(D-166)", () => {
  it("성공과 다른 모양, role=alert, 닫을 때까지 남는다", () => {
    vi.useFakeTimers();
    render(<ToastHost />);
    act(() => useToastStore.getState().showError(new ApiError({ code: "branchProtected", detail: "error: refusing" })));
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("지금 브랜치");
    act(() => vi.advanceTimersByTime(TOAST_DURATION_MS * 3));
    expect(screen.getByRole("alert")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "닫기" }));
    expect(useToastStore.getState().toast).toBeNull();
  });

  it("자세히 보기에 Git 원문, 해결 버튼은 하나", () => {
    const onClick = vi.fn();
    render(<ToastHost />);
    act(() =>
      useToastStore
        .getState()
        .showError(new ApiError({ code: "undoStale", detail: "fatal: bad object" }), { label: "작업 내역", onClick }),
    );
    expect(screen.getByText("자세히 보기")).toBeInTheDocument();
    expect(screen.getByText("fatal: bad object")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "작업 내역" }));
    expect(onClick).toHaveBeenCalledTimes(1);
    expect(useToastStore.getState().toast).toBeNull();
  });

  it("성공 토스트는 지금처럼 잠깐 뜨고 사라진다", () => {
    vi.useFakeTimers();
    render(<ToastHost />);
    act(() => useToastStore.getState().show("버전을 저장했어요"));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByText("버전을 저장했어요")).toBeInTheDocument();
    act(() => vi.advanceTimersByTime(TOAST_DURATION_MS + 500));
    expect(useToastStore.getState().toast).toBeNull();
  });
});
