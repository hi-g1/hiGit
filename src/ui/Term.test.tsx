import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it } from "vitest";
import { useUiStore } from "../state/uiStore";
import { Term } from "./Term";

afterEach(() => {
  act(() => useUiStore.setState({ termDisplay: "friendly", locale: "ko" }));
});

describe("Term", () => {
  it("본문에 쉬운 이름을 보여준다", () => {
    render(<Term id="status" />);
    expect(screen.getByText("바뀐 내용")).toBeInTheDocument();
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("호버하면 원래 이름, 명령, 설명이 툴팁에 뜬다", async () => {
    const user = userEvent.setup();
    render(<Term id="status" />);
    await user.hover(screen.getByText("바뀐 내용"));

    // 400ms 지연 전에는 뜨지 않는다
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("status");
    expect(tooltip).toHaveTextContent("git status");
    expect(tooltip).toHaveTextContent("마지막 저장 이후 바뀐 파일 목록");
  });

  it("키보드 포커스로도 툴팁이 뜬다", async () => {
    const user = userEvent.setup();
    render(<Term id="log" />);
    await user.tab();

    expect(screen.getByText("저장 기록")).toHaveFocus();
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("git log");
  });

  it("명령이 없는 용어는 명령 줄을 생략한다", async () => {
    const user = userEvent.setup();
    render(<Term id="conflict" />);
    await user.tab();

    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("merge conflict");
    expect(tooltip.querySelector("code")).toBeNull();
  });

  it("원래 이름 보기에서는 본문과 툴팁 첫 줄이 바뀐다", async () => {
    act(() => useUiStore.setState({ termDisplay: "git" }));
    const user = userEvent.setup();
    render(<Term id="commit" />);
    expect(screen.getByText("commit")).toBeInTheDocument();

    await user.tab();
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("저장하기");
  });

  it("영어 용어표를 쓸 수 있다", () => {
    act(() => useUiStore.setState({ locale: "en" }));
    render(<Term id="commit" />);
    expect(screen.getByText("Save")).toBeInTheDocument();
  });
});
