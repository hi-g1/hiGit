import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { openGuide: vi.fn(), openLicenses: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { ApiError, api } from "../../lib/api";
import { useUiStore } from "../../state/uiStore";
import { InfoSection } from "./InfoSection";

function show() {
  render(
    <AppProviders client={createQueryClient()}>
      <InfoSection />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useUiStore.setState({ locale: "ko" }));
});

describe("설정 정보 구역(D-220)", () => {
  it("사용법은 화면 언어로, 오픈 소스 라이선스는 고지 파일을 연다", async () => {
    vi.mocked(api.openGuide).mockResolvedValue(null);
    vi.mocked(api.openLicenses).mockResolvedValue(null);
    act(() => useUiStore.setState({ locale: "ko" }));
    show();
    await userEvent.click(screen.getByRole("button", { name: "사용법" }));
    expect(api.openGuide).toHaveBeenCalledWith("ko");
    await userEvent.click(screen.getByRole("button", { name: "오픈 소스 라이선스" }));
    expect(api.openLicenses).toHaveBeenCalledTimes(1);
  });

  it("English 화면이면 영어 사용법, 못 열면 오류를 보인다", async () => {
    vi.mocked(api.openGuide).mockRejectedValue(new ApiError({ code: "openFailed", detail: "no app" }));
    act(() => useUiStore.setState({ locale: "en" }));
    show();
    await userEvent.click(screen.getByRole("button", { name: "User guide" }));
    expect(api.openGuide).toHaveBeenCalledWith("en");
    await waitFor(() => expect(screen.getByRole("alert")).toBeInTheDocument());
  });
});
