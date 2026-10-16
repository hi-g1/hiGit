import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ChangedFile, SecretFinding } from "../../lib/api";
import { SecretNotice } from "./SecretNotice";

const file = (path: string, change: ChangedFile["change"] = "untracked"): ChangedFile => ({ path, origPath: null, change, staged: false });

function show(findings: SecretFinding[], files: ChangedFile[]) {
  render(<SecretNotice findings={findings} files={files} onUnstage={vi.fn()} onIgnore={vi.fn()} onSaveAnyway={vi.fn()} busy={false} />);
}

describe("비밀·압축 파일 알림 줄(D-077, D-158)", () => {
  it("압축·실행 파일만이면 그 제목과 줄 문구", () => {
    show(
      [
        { path: "home.zip", rule: "archive", line: null },
        { path: "bin/tool", rule: "executable", line: null },
      ],
      [file("home.zip"), file("bin/tool")],
    );
    expect(screen.getByText("압축·실행 파일 2개")).toBeInTheDocument();
    expect(screen.getByText(/압축 파일/)).toBeInTheDocument();
    expect(screen.getByText(/실행 파일$/)).toBeInTheDocument();
    // 막지 않는다: 그래도 저장, 모두 새 파일이면 제외할 파일에 넣기.
    expect(screen.getByRole("button", { name: "그래도 저장" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /제외할 파일에 추가/ })).toBeInTheDocument();
  });

  it("비밀과 함께 있으면 두 수를 따로 센다", () => {
    show(
      [
        { path: ".env", rule: "envFile", line: null },
        { path: "home.zip", rule: "archive", line: null },
      ],
      [file(".env"), file("home.zip")],
    );
    expect(screen.getByText("키나 비밀번호로 보이는 파일 1개 · 압축·실행 파일 1개")).toBeInTheDocument();
  });
});
