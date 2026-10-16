import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { useUiStore } from "../state/uiStore";
import { Term } from "./Term";

describe("Term", () => {
  it("본문에 쉬운 이름을 보여준다", () => {
    render(<Term id="status" />);
    expect(screen.getByText("변경 사항")).toBeInTheDocument();
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("호버하면 원래 이름, 명령, 설명이 툴팁에 뜬다", async () => {
    const user = userEvent.setup();
    render(<Term id="status" />);
    await user.hover(screen.getByText("변경 사항"));

    // 400ms 지연 전에는 뜨지 않는다
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("status");
    expect(tooltip).toHaveTextContent("git status");
    expect(tooltip).toHaveTextContent("마지막 버전 이후 바뀐 파일 목록");
  });

  it("키보드 포커스로도 툴팁이 뜬다", async () => {
    const user = userEvent.setup();
    render(<Term id="log" />);
    await user.tab();

    expect(screen.getByText("버전 기록")).toHaveFocus();
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
    expect(tooltip).toHaveTextContent("버전 저장");
  });

  it("버튼 안에서는 따로 포커스되지 않는다", () => {
    render(<Term id="init" focusable={false} />);
    expect(screen.getByText("버전 관리 시작")).not.toHaveAttribute("tabindex");
  });

  it("글자에는 밑줄·점선·도움말 커서가 없다(D-140)", () => {
    // jsdom에서는 import.meta.url이 파일 주소가 아니라 저장소 루트(vitest가 도는 곳)에서 읽는다.
    const css = readFileSync(join(process.cwd(), "src/ui/Term.module.css"), "utf8");
    const term = css.slice(0, css.indexOf(".tip"));
    expect(term).not.toMatch(/border-bottom|dotted|underline|cursor: help/);
  });

  it("용어 툴팁은 600ms 뒤에 뜬다(400ms엔 아직)", async () => {
    const user = userEvent.setup();
    render(<Term id="status" />);
    await user.hover(screen.getByText("변경 사항"));
    await act(() => new Promise((r) => setTimeout(r, 450)));
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    expect(await screen.findByRole("tooltip")).toHaveTextContent("git status");
  });

  it("용어 설명 보기를 끄면 툴팁도 포커스도 없다", async () => {
    act(() => useUiStore.setState({ termHelp: false }));
    const user = userEvent.setup();
    render(<Term id="log" />);
    expect(screen.getByText("버전 기록")).not.toHaveAttribute("tabindex");
    await user.hover(screen.getByText("버전 기록"));
    await new Promise((r) => setTimeout(r, 700));
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("English에서 이름이 같으면 툴팁 이름 줄을 뺀다", async () => {
    act(() => useUiStore.setState({ locale: "en" }));
    const user = userEvent.setup();
    render(<Term id="branch" />);
    await user.tab();
    const tooltip = await screen.findByRole("tooltip");
    // 명령·설명 두 줄만 남는다.
    expect(tooltip.querySelector("code")?.parentElement?.children).toHaveLength(2);
  });

  it("한국어는 이름 줄이 늘 있다", async () => {
    const user = userEvent.setup();
    render(<Term id="branch" />);
    await user.tab();
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip.querySelector("code")?.parentElement?.children).toHaveLength(3);
  });

  it("영어 용어표를 쓸 수 있다", () => {
    act(() => useUiStore.setState({ locale: "en" }));
    render(<Term id="commit" />);
    expect(screen.getByText("Save version")).toBeInTheDocument();
  });
});
