import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: {
    appStatus: vi.fn(),
    recentProjects: vi.fn(),
    projectSummary: vi.fn(),
    listActions: vi.fn(async () => []),
    routePreview: vi.fn(),
    identity: vi.fn(),
    githubStatus: vi.fn(),
    executeAction: vi.fn(),
  },
}));
vi.mock("../../lib/dialog", () => ({ pickFolder: vi.fn() }));

import { AppProviders, createQueryClient } from "../../App";
import { useSettingsStore } from "../../features/settings/settingsStore";
import { api, type IdentityView } from "../../lib/api";
import { useRepoStore } from "../../state/repoStore";
import { Start } from "./Start";

const empty: IdentityView = { name: null, email: null, complete: false, noreplyEmail: null };
const github = (over: Partial<Awaited<ReturnType<typeof api.githubStatus>>> = {}) => ({
  configured: true,
  connected: false,
  login: null,
  remembered: false,
  ...over,
});

function renderStart() {
  return render(
    <AppProviders client={createQueryClient()}>
      <Start />
    </AppProviders>,
  );
}

const title = "버전에 남길 이름부터 정해요";

beforeEach(() => {
  vi.mocked(api.appStatus).mockResolvedValue({ gitVersion: "2.43.0", gitSecurity: "ok", gitPackageVersion: null, gitPath: "/usr/bin/git" });
  vi.mocked(api.recentProjects).mockResolvedValue([]);
  vi.mocked(api.identity).mockResolvedValue(empty);
  vi.mocked(api.githubStatus).mockResolvedValue(github());
});
afterEach(() => {
  vi.clearAllMocks();
  act(() => {
    useRepoStore.setState({ repo: null, pendingInit: null, startNotice: null });
    useSettingsStore.setState({ open: false, connect: false });
  });
});

describe("처음 실행 패널(D-126, start-first.png)", () => {
  it("최근 프로젝트가 없고 이름·이메일이 비었으면 그림의 세 줄", async () => {
    renderStart();
    expect(await screen.findByRole("heading", { name: title })).toBeInTheDocument();
    expect(screen.getByText("처음 한 번만 하면 돼요")).toBeInTheDocument();
    expect(screen.getByText("Git을 찾았어요 · 버전 2.43.0")).toBeInTheDocument();
    expect(screen.getByRole("form", { name: "이름과 이메일" })).toBeInTheDocument();
    expect(screen.getByText("저장한 버전마다 이 이름이 남아요")).toBeInTheDocument();
    expect(screen.getByText("올리기와 검토 요청에 필요해요. 나중에 설정에서 해도 돼요")).toBeInTheDocument();
    expect(screen.getByText("○")).toBeInTheDocument();
    expect(screen.getByText("아직 연 프로젝트가 없어요. 오른쪽 위의 폴더 열기나 복제로 시작해요.")).toBeInTheDocument();
  });

  it("이름만 있어도(이메일 없음) 보인다", async () => {
    vi.mocked(api.identity).mockResolvedValue({ ...empty, name: "김준호" });
    renderStart();
    expect(await screen.findByRole("heading", { name: title })).toBeInTheDocument();
    expect(screen.getByLabelText("이름")).toHaveValue("김준호");
  });

  it("이름·이메일이 다 있으면 Git 한 줄만", async () => {
    vi.mocked(api.identity).mockResolvedValue({ name: "김준호", email: "j@example.com", complete: true, noreplyEmail: null });
    renderStart();
    expect(await screen.findByText("✓ Git을 찾았어요 · 버전 2.43.0")).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: title })).toBeNull();
  });

  it("최근 프로젝트가 있으면 보이지 않는다", async () => {
    vi.mocked(api.recentProjects).mockResolvedValue([{ path: "/home/me/blog", name: "blog", lastOpenedAt: 1_700_000_000 }]);
    vi.mocked(api.projectSummary).mockResolvedValue({
      branch: { name: "main", hasCommits: false, upstream: null, ahead: 0, behind: 0 },
      changedFiles: 0,
      conflictedFiles: 0,
      lastCommit: null,
    });
    renderStart();
    expect(await screen.findByRole("heading", { name: "blog" })).toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: title })).toBeNull();
  });

  it("나중에는 이번 실행 동안 숨기고 저장하지 않는다", async () => {
    const user = userEvent.setup();
    renderStart();
    await user.click(await screen.findByRole("button", { name: "나중에" }));
    expect(screen.queryByRole("heading", { name: title })).toBeNull();
    expect(screen.getByText("✓ Git을 찾았어요 · 버전 2.43.0")).toBeInTheDocument();
    expect(api.executeAction).not.toHaveBeenCalled();
  });

  it("저장하면 다시 읽어 패널이 사라진다", async () => {
    vi.mocked(api.executeAction).mockImplementation(async () => {
      vi.mocked(api.identity).mockResolvedValue({ name: "김준호", email: "j@example.com", complete: true, noreplyEmail: null });
      return { kind: "identitySaved" };
    });
    const user = userEvent.setup();
    renderStart();
    await user.type(await screen.findByLabelText("이름"), "김준호");
    await user.type(screen.getByLabelText("이메일"), "j@example.com{Enter}");
    await waitFor(() => expect(screen.queryByRole("heading", { name: title })).toBeNull());
    expect(api.executeAction).toHaveBeenCalledWith("", { id: "identity-set", input: { name: "김준호", email: "j@example.com" } });
    expect(screen.getByText("✓ Git을 찾았어요 · 버전 2.43.0")).toBeInTheDocument();
  });

  it("GitHub 연결 단추는 설정의 연결 패널을 연다", async () => {
    const user = userEvent.setup();
    renderStart();
    await user.click(await screen.findByRole("button", { name: "연결" }));
    expect(useSettingsStore.getState()).toMatchObject({ open: true, connect: true });
  });

  it("연결돼 있으면 ✓와 로그인 이름, 단추 없음", async () => {
    vi.mocked(api.githubStatus).mockResolvedValue(github({ connected: true, login: "junho", remembered: true }));
    renderStart();
    expect(await screen.findByText("@junho")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "연결" })).toBeNull();
    expect(screen.queryByText("○")).toBeNull();
  });

  it("이 앱에 GitHub 연결이 준비되지 않았으면 연결 단추가 없다", async () => {
    vi.mocked(api.githubStatus).mockResolvedValue(github({ configured: false }));
    renderStart();
    await screen.findByRole("heading", { name: title });
    await waitFor(() => expect(api.githubStatus).toHaveBeenCalled());
    expect(screen.queryByRole("button", { name: "연결" })).toBeNull();
  });
});
