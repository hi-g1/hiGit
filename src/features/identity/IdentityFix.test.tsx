import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { AppProviders, createQueryClient } from "../../App";
import { ApiError } from "../../lib/api";
import { ErrorNotice } from "../../ui/ErrorNotice";
import { useSettingsStore } from "../settings/settingsStore";

const missing = new ApiError({ code: "identityMissing", detail: null });

function show(ui: React.ReactElement) {
  render(<AppProviders client={createQueryClient()}>{ui}</AppProviders>);
}

afterEach(() => act(() => useSettingsStore.setState({ open: false, connect: false })));

describe("이름·이메일이 없어 실패한 오류 줄(D-126)", () => {
  it("어느 오류 줄이든 '이름과 이메일 설정'이 설정을 연다", async () => {
    show(<ErrorNotice error={missing} />);
    expect(screen.getByRole("alert")).toHaveTextContent("버전에 남길 이름과 이메일이 설정되어 있지 않아요.");
    await userEvent.click(screen.getByRole("button", { name: "이름과 이메일 설정" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: false });
  });

  it("그 자리에서 고치게 하는 곳은 단추를 끌 수 있고, 다른 오류에는 없다", () => {
    show(
      <>
        <ErrorNotice error={missing} codeAction={false} />
        <ErrorNotice error={new ApiError({ code: "pushRejected", detail: null })} />
      </>,
    );
    expect(screen.queryByRole("button", { name: "이름과 이메일 설정" })).toBeNull();
  });
});
