import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  api: { rewritePlan: vi.fn(), executeAction: vi.fn() },
}));

import { AppProviders, createQueryClient } from "../../App";
import { api, type RewritePlanView } from "../../lib/api";
import { ko } from "../../messages/ko";
import { useRepoStore } from "../../state/repoStore";
import { Rewrite } from "./Rewrite";

const id = (c: string) => c.repeat(40);
/** 새 것부터: 셋 둘 하나. */
const plan = (over: Partial<RewritePlanView> = {}): RewritePlanView => ({
  cards: ["c", "b", "a"].map((c, i) => ({
    oid: id(c),
    shortOid: c.repeat(7),
    summary: ["셋", "둘", "하나"][i]!,
    message: `${["셋", "둘", "하나"][i]}\n`,
    author: "지원",
    time: 1_700_000_000,
  })),
  blocked: null,
  staleLabels: 0,
  ...over,
});

function show(onDone = vi.fn()) {
  const client = createQueryClient();
  render(
    <AppProviders client={client}>
      <Rewrite repo="/r" onDone={onDone} />
    </AppProviders>,
  );
  return { client, onDone };
}

const cardsOf = () => within(screen.getByRole("list", { name: ko.cardList })).getAllByRole("listitem");
const names = () => cardsOf().map((c) => c.querySelector("bdi")?.textContent);

beforeEach(() => {
  act(() => useRepoStore.setState({ view: "rewrite", rewriteOids: [id("c"), id("b"), id("a")] }));
});
afterEach(() => vi.clearAllMocks());

describe("기록 정리 화면", () => {
  it("계획을 카드로, 바뀐 것이 없으면 정리하기가 꺼져 있다", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan());
    show();
    await screen.findByText("셋");
    expect(api.rewritePlan).toHaveBeenCalledWith("/r", [id("c"), id("b"), id("a")]);
    expect(names()).toEqual(["셋", "둘", "하나"]);
    expect(screen.getByRole("button", { name: ko.rewriteRun })).toBeDisabled();
    expect(screen.getByText(ko.rewriteSummary(3, 0, 0))).toBeInTheDocument();
  });

  it("옮기기·묶기·빼기·메시지 고치기 뒤 정리하기가 core 입력을 보낸다", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan());
    vi.mocked(api.executeAction).mockResolvedValue({ kind: "rewritten", kept: 1, dropped: 1, joined: 1 });
    const { onDone } = show();
    await screen.findByText("셋");
    // 셋을 아래로(둘 아래): 둘 셋 하나
    await userEvent.click(within(cardsOf()[0]!).getByRole("button", { name: ko.cardDown }));
    expect(names()).toEqual(["둘", "셋", "하나"]);
    expect(within(cardsOf()[1]!).getByText(ko.cardStatus.moved)).toBeInTheDocument();
    // 셋을 하나에 묶고 하나의 메시지를 고친다.
    await userEvent.click(within(cardsOf()[1]!).getByRole("button", { name: ko.cardJoin }));
    expect(within(cardsOf()[2]!).getByText(ko.cardGroup(2))).toBeInTheDocument();
    await userEvent.click(within(cardsOf()[2]!).getByRole("button", { name: ko.cardEdit }));
    const editor = screen.getByRole("textbox", { name: ko.cardEdit });
    await userEvent.clear(editor);
    await userEvent.type(editor, "하나와 셋");
    fireEvent.keyDown(editor, { key: "Escape" });
    expect(useRepoStore.getState().view).toBe("rewrite");
    // 둘을 뺀다.
    await userEvent.click(within(cardsOf()[0]!).getByRole("button", { name: ko.cardDrop }));
    expect(screen.getByText(ko.rewriteSummary(1, 1, 1))).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: ko.rewriteRun }));
    expect(api.executeAction).toHaveBeenCalledWith("/r", {
      id: "rebase-i",
      input: {
        steps: [
          { oid: id("a"), kind: "pick", message: "하나와 셋" },
          { oid: id("c"), kind: "squash", message: null },
          { oid: id("b"), kind: "drop", message: null },
        ],
      },
    });
    await waitFor(() => expect(onDone).toHaveBeenCalledWith(ko.rewritten));
    expect(useRepoStore.getState().view).toBe("history");
  });

  it("카드 키: Alt+↑↓ 옮기기, Delete 빼기, Enter 메시지 고치기, 한글 조합 중 무시", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan());
    show();
    await screen.findByText("셋");
    const first = cardsOf()[0]!;
    first.focus();
    fireEvent.keyDown(first, { key: "ArrowDown", altKey: true });
    expect(names()).toEqual(["둘", "셋", "하나"]);
    await waitFor(() => expect(cardsOf()[1]).toHaveFocus());
    expect(screen.getByText(ko.cardMovedTo("셋", 2))).toBeInTheDocument();
    fireEvent.keyDown(cardsOf()[1]!, { key: "Delete", isComposing: true });
    expect(screen.getByText(ko.rewriteSummary(3, 0, 0))).toBeInTheDocument();
    fireEvent.keyDown(cardsOf()[1]!, { key: "Delete" });
    expect(screen.getByText(ko.rewriteSummary(2, 1, 0))).toBeInTheDocument();
    fireEvent.keyDown(cardsOf()[1]!, { key: "ArrowDown" });
    await waitFor(() => expect(cardsOf()[2]).toHaveFocus());
    fireEvent.keyDown(cardsOf()[2]!, { key: "Enter" });
    expect(screen.getByRole("textbox", { name: ko.cardEdit })).toHaveValue("하나");
  });

  it("막힌 까닭이 있으면 알리고 카드는 읽기만", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan({ blocked: "published", staleLabels: 2 }));
    show();
    expect(await screen.findByRole("alert")).toHaveTextContent(ko.rewriteBlocked.published);
    expect(screen.getByText(ko.staleLabels(2))).toBeInTheDocument();
    for (const b of within(cardsOf()[0]!).getAllByRole("button")) expect(b).toBeDisabled();
    expect(screen.getByRole("button", { name: ko.rewriteRun })).toBeDisabled();
  });

  it("계획을 다시 읽어도 고친 카드는 남고 막힌 까닭만 바뀐다", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan());
    const { client } = show();
    await screen.findByText("셋");
    await userEvent.click(within(cardsOf()[0]!).getByRole("button", { name: ko.cardDown }));
    vi.mocked(api.rewritePlan).mockResolvedValue(plan({ blocked: "hasChanges" }));
    await act(() => client.invalidateQueries({ queryKey: ["rewritePlan", "/r"] }));
    expect(await screen.findByRole("alert")).toHaveTextContent(ko.rewriteBlocked.hasChanges);
    expect(names()).toEqual(["둘", "셋", "하나"]);
  });

  it("Esc와 ← 버전 기록은 버전 기록으로", async () => {
    vi.mocked(api.rewritePlan).mockResolvedValue(plan());
    show();
    await screen.findByText("셋");
    await userEvent.keyboard("{Escape}");
    expect(useRepoStore.getState().view).toBe("history");
    act(() => useRepoStore.setState({ view: "rewrite" }));
    await userEvent.click(screen.getByRole("button", { name: ko.backToLog }));
    expect(useRepoStore.getState().view).toBe("history");
  });
});
