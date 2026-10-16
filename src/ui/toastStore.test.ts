import { beforeEach, describe, expect, it, vi } from "vitest";
import { MAX_ERROR_TOASTS, MAX_TOASTS, useToastStore, type ToastAction } from "./toastStore";

const undo = (onClick = vi.fn()): ToastAction => ({
  label: "되돌리기",
  kbd: "Ctrl+Z",
  onClick,
  kind: "undo",
  stale: { label: "작업 내역", onClick: vi.fn() },
});
const texts = () => useToastStore.getState().toasts.map((t) => t.text || "오류");

beforeEach(() => useToastStore.getState().clear());

describe("토스트 쌓기(D-192)", () => {
  it("되돌리기 토스트 뒤에 평범한 토스트가 와도 둘 다 남는다", () => {
    const { show } = useToastStore.getState();
    show("저장했어요", undo());
    show("복사했어요");
    expect(texts()).toEqual(["저장했어요", "복사했어요"]);
    expect(useToastStore.getState().toasts[0]!.action?.label).toBe("되돌리기");
    expect(useToastStore.getState().toast?.text).toBe("복사했어요");
  });

  it("새 되돌리기 토스트가 오면 앞 토스트의 되돌리기는 작업 내역으로 바뀐다", () => {
    const { show } = useToastStore.getState();
    show("저장했어요", undo());
    show("브랜치를 바꿨어요", undo());
    const [first, second] = useToastStore.getState().toasts;
    expect(first!.action?.label).toBe("작업 내역");
    expect(second!.action?.label).toBe("되돌리기");
  });

  it(`셋(${MAX_TOASTS})을 넘으면 가장 오래된 것부터 빠진다`, () => {
    const { show } = useToastStore.getState();
    for (const t of ["하나", "둘", "셋", "넷"]) show(t);
    expect(texts()).toEqual(["둘", "셋", "넷"]);
  });

  it(`오류는 닫을 때까지 남고 둘(${MAX_ERROR_TOASTS})까지, 셋째는 가장 오래된 오류를 민다`, () => {
    const { show, showError, clear } = useToastStore.getState();
    showError(new Error("1"));
    for (const t of ["하나", "둘", "셋", "넷"]) show(t);
    expect(useToastStore.getState().toasts.filter((t) => t.error)).toHaveLength(1);
    showError(new Error("2"));
    showError(new Error("3"));
    const errors = useToastStore.getState().toasts.filter((t) => t.error);
    expect(errors.map((t) => (t.error!.value as Error).message)).toEqual(["2", "3"]);
    clear(errors[0]!.id);
    expect(useToastStore.getState().toasts.filter((t) => t.error)).toHaveLength(1);
  });
});
