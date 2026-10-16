import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { executeAction: vi.fn(), identity: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { useToastStore } from "../../ui/toastStore";
import { IdentityForm } from "./IdentityForm";

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <IdentityForm />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  useToastStore.setState({ toast: null });
});

describe("이름과 이메일(D-125)", () => {
  it("지금 값을 보이고, 바꾸기 전에는 저장이 꺼져 있다", async () => {
    vi.mocked(api.identity).mockResolvedValue({ name: "김지원", email: "ji@example.com", complete: true });
    show();
    expect(await screen.findByDisplayValue("김지원")).toBeInTheDocument();
    expect(screen.getByLabelText("이메일")).toHaveValue("ji@example.com");
    expect(screen.getByRole("button", { name: "저장" })).toBeDisabled();
  });

  it("저장하면 전역 설정에 쓰고 다시 읽으며 값은 토스트에 넣지 않는다", async () => {
    vi.mocked(api.identity).mockResolvedValue({ name: null, email: null, complete: false });
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "identitySaved" });
    show();
    await userEvent.type(screen.getByLabelText("이름"), "Jiwon");
    await userEvent.type(screen.getByLabelText("이메일"), "j@example.com{Enter}");
    await waitFor(() =>
      expect(api.executeAction).toHaveBeenCalledWith("", { id: "identity-set", input: { name: "Jiwon", email: "j@example.com" } }),
    );
    await waitFor(() => expect(useToastStore.getState().toast?.text).toBe("저장했어요"));
    expect(useToastStore.getState().toast?.text).not.toContain("Jiwon");
    await waitFor(() => expect(api.identity).toHaveBeenCalledTimes(2));
  });

  it("규칙에 맞지 않으면 오류를 보인다", async () => {
    vi.mocked(api.identity).mockResolvedValue({ name: null, email: null, complete: false });
    vi.mocked(api.executeAction).mockRejectedValue(new ApiError({ code: "identityInvalid", detail: null }));
    show();
    await userEvent.type(screen.getByLabelText("이름"), "-x");
    await userEvent.click(screen.getByRole("button", { name: "저장" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("이름과 이메일을 확인해 주세요");
  });

  it("한글 조합 중 Enter는 보내지 않는다", async () => {
    vi.mocked(api.identity).mockResolvedValue({ name: null, email: null, complete: false });
    show();
    const name = screen.getByLabelText("이름");
    fireEvent.change(name, { target: { value: "김" } });
    const enter = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true, isComposing: true });
    name.dispatchEvent(enter);
    expect(enter.defaultPrevented).toBe(true);
    expect(api.executeAction).not.toHaveBeenCalled();
  });
});
