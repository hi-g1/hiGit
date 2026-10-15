// pnpm install 때 저장소의 .githooks를 쓰도록 설정한다(D-022).
// git 작업 폴더가 아니면(배포 압축본 등) 아무것도 하지 않는다.
import { spawnSync } from "node:child_process";

const git = (args) => spawnSync("git", args, { encoding: "utf8" });

if (git(["rev-parse", "--is-inside-work-tree"]).stdout?.trim() !== "true") {
  process.exit(0);
}

const current = git(["config", "--local", "--get", "core.hooksPath"]).stdout?.trim();
if (current === ".githooks") process.exit(0);
if (current) {
  console.log(`core.hooksPath가 "${current}"였습니다. 저장소 hook(.githooks)으로 바꿉니다.`);
}
const result = git(["config", "--local", "core.hooksPath", ".githooks"]);
if (result.status !== 0) {
  console.error("git hook을 설정하지 못했습니다:", result.stderr);
  process.exit(1);
}
console.log("git hook을 설정했습니다: 커밋 전 빠른 검사, push 전 전체 검사");
