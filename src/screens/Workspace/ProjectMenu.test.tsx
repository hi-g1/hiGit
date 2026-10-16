import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { recentProjects: vi.fn(), pinProject: vi.fn(), openProject: vi.fn() },
}));
vi.mock("../../lib/dialog", () => ({ pickFolder: vi.fn() }));

import { AppProviders, createQueryClient } from "../../App";
import { api } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { ProjectMenu } from "./ProjectMenu";

const recent = (name: string, pinned = false) => ({
  path: `/home/me/${name}`,
  name,
  lastOpenedAt: 1,
  pinned,
  pinnedBranches: [],
});

function show(list = [recent("지금"), recent("a"), recent("b", true), recent("c")]) {
  vi.mocked(api.recentProjects).mockResolvedValue(list);
  vi.mocked(api.pinProject).mockResolvedValue(null);
  render(
    <AppProviders client={createQueryClient()}>
      <ProjectMenu repo={{ path: "/home/me/지금", name: "지금" }} />
    </AppProviders>,
  );
}

afterEach(() => {
  vi.clearAllMocks();
  act(() => useRepoStore.setState({ repo: null }));
});

describe("프로젝트 메뉴(D-203)", () => {
  it("제목이 단추 안에 들지 않는다", () => {
    show();
    const trigger = screen.getByRole("button", { name: "프로젝트 메뉴: 지금" });
    expect(trigger.querySelector("h1")).toBeNull();
    expect(screen.getByRole("heading", { level: 1 })).toContainElement(trigger);
  });

  it("최근 프로젝트(고정 먼저)·이 프로젝트 고정·폴더 열기·복제·다른 프로젝트 열기", async () => {
    show();
    await vi.waitFor(() => expect(api.recentProjects).toHaveBeenCalled());
    await userEvent.click(screen.getByRole("button", { name: "프로젝트 메뉴: 지금" }));
    const items = (await screen.findAllByRole("menuitem")).map((i) => i.textContent);
    expect(items).toEqual(["b고정", "a", "c", "지금 고정", "폴더 열기Ctrl+O", "복제Ctrl+Shift+O", "다른 프로젝트 열기"]);
    await userEvent.click(screen.getByRole("menuitem", { name: "지금 고정" }));
    expect(api.pinProject).toHaveBeenCalledWith("/home/me/지금", true);
  });

  it("고정한 프로젝트면 고정 풀기", async () => {
    show([recent("지금", true)]);
    await vi.waitFor(() => expect(api.recentProjects).toHaveBeenCalled());
    await userEvent.click(screen.getByRole("button", { name: "프로젝트 메뉴: 지금" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: "고정 풀기" }));
    expect(api.pinProject).toHaveBeenCalledWith("/home/me/지금", false);
  });
});
