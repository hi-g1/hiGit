// 커밋·push 전 검사(D-022). 사용법: node scripts/check.mjs fast [--all] | full
// fast: 스테이지된 파일 기준으로 바뀐 쪽만 빠르게(pre-commit). --all이면 두 쪽 모두.
// full: 전체 테스트와 바인딩 최신 검사(pre-push).
import { spawnSync } from "node:child_process";
import { selectGroups } from "./check-groups.mjs";

const [mode = "fast", ...flags] = process.argv.slice(2);
// Windows에서 pnpm은 pnpm.cmd라서 셸을 거쳐야 실행된다.
const shell = process.platform === "win32";

function run(label, cmd, args, cwd = ".") {
  console.log(`\n▶ ${label}`);
  const result = spawnSync(cmd, args, { stdio: "inherit", cwd, shell });
  if (result.status !== 0) {
    console.error(`\n✖ 검사 실패: ${label}`);
    console.error("  고친 뒤 다시 시도해 주세요. (--no-verify로 건너뛰지 않습니다)");
    process.exit(1);
  }
}

function stagedFiles() {
  const out = spawnSync("git", ["diff", "--cached", "--name-only", "-z"], { encoding: "utf8" });
  if (out.status !== 0) {
    console.error("스테이지된 파일을 읽지 못했습니다.");
    process.exit(1);
  }
  return out.stdout.split("\0").filter(Boolean);
}

function frontend() {
  run("프론트 린트", "pnpm", ["-s", "lint"]);
  run("프론트 타입 검사", "pnpm", ["-s", "typecheck"]);
  run("프론트 테스트", "pnpm", ["-s", "test"]);
}

function rustQuick() {
  run("Rust 형식", "cargo", ["fmt", "--all", "--check"], "src-tauri");
  run("Rust 린트", "cargo", ["clippy", "-q", "--workspace", "--all-targets", "--", "-D", "warnings"], "src-tauri");
}

if (mode === "fast") {
  const groups = flags.includes("--all") ? { frontend: true, rust: true } : selectGroups(stagedFiles());
  if (!groups.frontend && !groups.rust) {
    console.log("검사할 코드 변경이 없습니다.");
  }
  if (groups.frontend) frontend();
  if (groups.rust) rustQuick();
} else if (mode === "full") {
  frontend();
  rustQuick();
  run("Rust 테스트", "cargo", ["test", "-q", "--workspace"], "src-tauri");
  run("바인딩 생성", "cargo", ["run", "-q", "-p", "app", "--bin", "gen-bindings"], "src-tauri");
  run("바인딩이 최신인지", "git", ["diff", "--exit-code", "--", "src/lib"]);
} else {
  console.error(`알 수 없는 검사 종류: ${mode} (fast 또는 full)`);
  process.exit(1);
}
console.log("\n✔ 검사 통과");
