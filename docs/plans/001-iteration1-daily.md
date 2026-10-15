# 001 반복 1: 매일 쓰는 기능
- 관련 결정: D-002, D-003, D-006, D-007, D-010, D-011, D-013, D-014, D-015, D-019 (확정). D-020, D-021, D-022, D-023 (확정, 2026-10-15). D-017(제안)은 다루지 않는다(저장 기록은 계속 단순 목록).
- 관련 명세: spec.md "원칙" 3(지금 상태가 항상 보인다), "화면 구성"의 작업 공간(왼쪽 갈래 목록, 가운데 바뀐 파일과 비교, 오른쪽 저장 상자)과 저장 기록 탭, 용어표의 `stage`·`commit`·`diff`·`branch`·`switch`·`fetch`·`pull`·`push`, "오류 메시지 원칙". roadmap.md "반복 1".

## 목표
Windows와 Ubuntu에서 저장소를 열면 상단에 현재 갈래·바뀐 파일 수·올릴/받을 저장 수가 늘 보이고, 파일을 골라 저장에 넣고, 비교를 보고, 메시지를 써서 저장하고, 갈래를 만들고 바꾸고, 새 소식 확인·최신으로 받기·올리기를 할 수 있다. 파일을 바꾸면 화면이 저절로 새로 고쳐진다.
그 앞에 기반 세 가지를 갖춘다: 타입 있는 액션 입력, 저장소에 포함된 커밋 전 검사 hook, CI의 Windows 실행 파일 artifact.

## 현재 상태 (main, 반복 0 완료)
반복 0 계획(000)과 다른 점과 이 계획이 기대는 사실만 적는다.
- `crates/git`: `GitBackend`는 `version`, `discover`, `status`, `log`, `run`만 있다. `GitOp`는 `Init` 하나. `Runner::run`은 stdin을 `Stdio::null()`로 막고 env `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, `GIT_OPTIONAL_LOCKS=0`, `-c core.quotepath=off -c color.ui=never`를 준다. `GitError { Spawn, Failed { args, code, stderr }, Parse }`.
- `crates/core`: `Action` 트레이트가 `&serde_json::Value`를 받는다. `ActionMeta { id, glossary_id, risk, input_schema: Vec<InputField> }`. `Executor`는 caution 이상만 `Ticket { action_id, input: Value, folder, issued_at }`을 보관해 토큰을 비교하고 실행은 `NotYetSupported`로 거부(D-011). `Core`(bootstrap.rs)가 `preview/execute(folder, id, &ActionInput, token)`에서 `ActionInput = BTreeMap<String,String>`을 `to_value`로 `Value`로 바꾼다. `Queries`는 `status`, `log`만. `view.rs`에 `StatusView.branch`(name, has_commits, upstream, ahead, behind)가 이미 있다.
- `src-tauri/src`: `commands.rs`(app_status, repo_open, action_list, action_preview, action_execute, query_status, query_log), `bindings.rs`(builder + `export_all`이 `bindings.ts`, `action-meta.json`을 씀). 이벤트는 아직 없다.
- 프론트: `ActionButton`(actionId, folder로 `api.executeAction(folder, id)` 호출, caution 이상은 숨김), `Home`(관리 시작), `Workspace`(바뀐 내용·저장 기록 두 패널, "새로 고침" 버튼), `messages/ko.ts`·`en.ts`의 오류 문구는 `Record<ErrorCode, string>`으로 빠짐없이 강제.
- `crates/watcher`, `crates/hosting`은 빈 크레이트. core는 watcher에 아직 의존하지 않는다.
- CI(`.github/workflows/ci.yml`): `ubuntu-22.04`, `windows-latest`에서 lint → typecheck → test → fmt → clippy → cargo test → 바인딩 최신 검사 → `pnpm build` → `pnpm tauri build --no-bundle`. 결과물은 올리지 않는다.
- 커밋 전 검사는 수동뿐(hook 없음).

## 범위
들어가는 것:
- 기반: ① 타입 있는 액션 입력(D-020), ② 저장소 hook(D-022), ③ CI Windows artifact.
- 액션(전부 `safe`/`normal`, D-011 범위 안): `stage`(저장에 넣기), `unstage`(저장에서 빼기, 새 용어), `commit`(저장하기), `branch-create`(작업 갈래 만들기, 용어 `branch`), `switch`(갈래 바꾸기), `fetch`(새 소식 확인), `pull`(최신으로 받기, fast-forward만), `push`(올리기).
- 조회: `diff`(비교 보기), `branches`(갈래 목록), 갈래 이름 검사.
- 파일 변경 감시(`crates/watcher`) → `repo-changed` 이벤트 → 화면 자동 새로 고침.
- 상단 상태 표시, 작업 공간 3단 배치, 저장 기록 탭 분리.

들어가지 않는 것: 합치기·충돌 화면·잠시 치워두기·변경 취소(반복 2), 진행률 이벤트 `op-progress`(반복 1은 진행 중 표시만), 원격 추가·관리(`remote`), 이름·이메일 설정 화면, 자동 새 소식 확인(H-12에서 정함), 설치 파일(H-6에서 정함), 노선도.

### 범위 나누기 제안 (H-7)
한 반복에 22단계(11단계는 커밋 2개)라 크다. 임의로 빼지 않고 나누는 안을 제시한다.
- **1a (기반 + 내 컴퓨터 안의 작업)**: 1~3단계(기반), 4·5·7단계(git), 10단계의 diff 부분, 11단계, 14단계(감시), 15~18·21단계(화면), 22단계.
- **1b (갈래와 원격)**: 6·8·9단계(git), 10단계의 갈래 조회 부분, 12·13단계(core), 19·20단계(화면).
- 아래 단계 번호는 나누든 안 나누든 그대로 쓴다. 나누면 1a 끝에 22단계를 한 번, 1b 끝에 한 번 더 한다. 1a의 15단계 왼쪽 패널은 1b 전까지 현재 갈래 이름만 보여준다.

## 완료 기준
- 1~3단계: 아래 각 단계의 완료 기준.
- Ubuntu 22.04와 Windows 11에서 `pnpm tauri dev`로 저장소를 열면 상단에 현재 갈래 이름, 바뀐 파일 수, 올릴 저장 수, 받을 저장 수(연결된 갈래가 없으면 "아직 올린 적 없음")가 보인다.
- 바뀐 파일의 체크 상자로 저장에 넣기/빼기를 하면 목록 상태가 바로 바뀐다(한글·공백 파일명, 파일 1,000개 한 번에 넣기 포함).
- 파일을 고르면 가운데에 마지막 저장과의 줄 단위 비교가 보인다(추가/삭제 줄 색 토큰, 새 파일, 삭제 파일, 이진 파일 안내, 큰 파일 잘림 안내).
- 메시지를 쓰고 "저장하기"를 누르면(또는 Ctrl+Enter, 한글 조합 중 Enter는 무시) 저장 기록 탭에 새 저장이 맨 위에 보인다. 넣은 파일이 없거나 메시지가 비면 버튼이 꺼져 있다.
- 왼쪽 갈래 목록에서 새 갈래를 만들고 바꿀 수 있다. 잘못된 이름은 입력 중에 안내가 뜬다. 바꿀 때 덮어쓸 변경이 있으면 쉬운 안내와 "자세히 보기"가 뜬다.
- 로컬 원격(임시 bare 저장소)과 GitHub 저장소 각각에서 새 소식 확인 → 받을 저장 수 갱신, 최신으로 받기(fast-forward), 올리기(연결된 갈래 없을 때 포함)가 된다. 올리기가 거부되면 "올리기 전에 최신 내용을 먼저 받아야 해요." 안내와 [최신으로 받기] 버튼이 뜬다.
- 앱 밖(편집기)에서 파일을 저장하면 1초 안에 바뀐 파일 목록과 상단 숫자가 바뀐다. 터미널에서 `git commit`해도 바뀐다.
- `pnpm test`가 레지스트리의 모든 액션 `glossaryId`·위험도를 spec.md 표와 대조해 통과하고, spec.md 표·`ko.ts`·`en.ts`가 새 용어 `unstage`까지 일치한다.
- 두 OS CI 초록, Windows artifact를 받아 깨끗한 Windows 11에서 실행된다.

## 단계
각 단계 = 커밋 하나. 단계마다 push해서 CI 초록을 확인한다. 2단계 이후에는 hook이 커밋·push마다 검사한다. 바인딩이 바뀌는 단계는 같은 커밋에서 `cargo run -p app --bin gen-bindings`로 `src/lib/bindings.ts`·`action-meta.json`을 다시 만든다.

### A. 기반 (사용자 지시 순서)

1. ① 액션 입력을 액션별 구조체 + `ActionInput` enum으로 (`core: type action inputs per action`)
   - 전제: D-020 확정(H-1, H-2, H-3). 확정되면 같은 커밋 또는 바로 앞 `docs:` 커밋에서 `docs/architecture.md`를 고친다(Action 트레이트 시그니처, `ActionMeta`에서 `input_schema` 삭제, "프론트와 백엔드 계약"의 `ActionInput` 문단, "새 액션 추가 절차" 4번에 "`core::view::ActionInput`에 변형 추가").
   - 변경 파일:
     - `crates/core/src/view.rs` → `view/mod.rs` + `view/input.rs`로 나눈다(둘 다 `core::view` 안이라 D-019 범위 그대로). `input.rs`:
       ```rust
       #[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Type)]
       #[serde(tag = "id", content = "input", rename_all = "kebab-case")]
       pub enum ActionInput { Init }   // 이후 단계에서 Stage(StageInput) 등 추가
       impl ActionInput { pub fn action_id(&self) -> &'static str { match self { Self::Init => "init" } } }
       ```
       `Serialize`는 테스트에서 태그 문자열과 `action_id()`가 같은지 확인하려고 붙인다. 기존 `type ActionInput = BTreeMap<…>`, `InputFieldView`, `InputKindView`, `ActionMetaView.input_schema` 삭제.
     - `crates/core/src/action.rs`: `preview/execute(&self, ctx, input: &ActionInput)`. `InputField`, `InputKind`, `ActionMeta.input_schema` 삭제(H-1). 입력 변형이 맞지 않을 때의 `CoreError::InputMismatch`(→ `ErrorCode::Internal`, 화면 버그이므로 문구 따로 없음) 추가.
     - `crates/core/src/executor.rs`: `Ticket.input: ActionInput`, 비교는 `PartialEq`. `preview/execute(ctx, id, &ActionInput, token)`의 시그니처는 `id`를 유지한다(가짜 액션 테스트가 임의 id를 쓰기 위해). 
     - `crates/core/src/bootstrap.rs`: `Core::preview(folder, &ActionInput)`, `Core::execute(folder, &ActionInput, token)`이 `input.action_id()`로 실행기를 부른다. `to_value` 삭제.
     - `crates/core/src/actions/init.rs`: `input_schema` 줄 삭제, 시그니처 변경. 변형 검사는 하지 않는다(입력 없음).
     - `crates/core/src/lib.rs`: `InputField`, `InputKind` re-export 삭제.
     - `crates/core/Cargo.toml`: `serde_json`을 `[dev-dependencies]`로 옮길 수 있으면 옮긴다(테스트에서만 씀).
     - `crates/core/tests/{common/mod.rs, executor.rs, init.rs}`, `executor.rs` 안 단위 테스트: `json!({...})` → `ActionInput`. 이 단계에는 변형이 `Init` 하나뿐이라 토큰 테스트의 "입력이 다름" 경우를 만들 수 없다. 그 경우만 빼고 11b단계(`Commit`이 생긴 뒤)에서 되살린다고 테스트 주석에 적는다. 대안: 1단계에서 `Commit(CommitInput)` 변형을 미리 넣는다(H-2 참고).
     - `src-tauri/src/commands.rs`: `action_preview(folder, input: ActionInput)`, `action_execute(folder, input: ActionInput, preview_token)`. `id` 인자 삭제(H-3).
     - `src-tauri/src/bindings.rs`: 바뀌지 않거나 import 정리만. `export_all`은 그대로.
     - 생성물 `src/lib/bindings.ts`, `src/lib/action-meta.json`.
     - `src/lib/api.ts`: `previewAction(folder, input: ActionInput)`, `executeAction(folder, input: ActionInput, previewToken?)`, `ActionInput` 타입 re-export.
     - `src/features/actions/ActionButton.tsx`: props를 `input: ActionInput`으로(메타는 `input.id`로 찾음). `src/screens/Home/Home.tsx`: `input={{ id: "init" }}`.
     - `src/App.test.tsx` 등 `executeAction` 모킹 테스트.
   - 테스트:
     - core: 기존 실행기·init 테스트가 새 타입으로 통과. 새 테스트 `action_input_tags_match_registry`: 각 변형 샘플을 `serde_json::to_value`해 `["id"]`가 `action_id()`와 같고, `all_metas()`의 id 집합과 변형 id 집합이 같다(새 액션을 등록하고 변형을 빠뜨리거나 그 반대면 실패).
     - 프론트: `ActionButton`이 `{ id: "init" }`로 `executeAction`을 부르는지.
   - 완료 기준: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm test` 통과. `gen-bindings`를 두 번 돌려도 `git diff`가 비어 있다. `bindings.ts`에 `export type ActionInput = { id: "init" }`가 있다. `rg "serde_json::Value|BTreeMap" src-tauri/crates/core/src`가 비어 있다. `rg "input_schema|inputSchema" src src-tauri`가 비어 있다. 두 OS에서 "관리 시작"이 그대로 동작(수동).
   - Windows/Ubuntu: 차이 없음. 생성 파일은 LF(반복 0과 같음).
   - 위험: specta rc.25가 adjacently tagged enum(`tag`+`content`)과 unit 변형을 섞어 내보내는 모양을 먼저 확인한다. 기대 출력은 `{ id: "init" } | { id: "commit"; input: CommitInput }`. 이상하면 internally tagged(`tag = "id"`, 변형을 `Commit { message }` 구조체 변형으로)로 바꾸고 D-020 문구를 고친다. 이 확인은 코드 작성 전에 10분짜리 실험으로 한다(커밋하지 않음).

2. ② 커밋 전 검사 hook (`chore: enforce checks with repository git hooks`)
   - 결정: D-022(확정). pre-commit은 바뀐 쪽 빠른 검사, pre-push는 전체 검사. 설치는 `prepare`로 자동(H-5).
   - 변경 파일:
     - `scripts/check.mjs`: `node scripts/check.mjs fast|full`. 명령을 차례로 실행하고 처음 실패에서 멈춰 어떤 검사가 실패했는지 한국어로 출력, 종료 코드 1. `child_process.spawnSync(cmd, args, { stdio: "inherit", shell: process.platform === "win32" })`(Windows의 `pnpm.cmd` 실행). Rust 명령은 `cwd: "src-tauri"`.
       - `fast`(pre-commit): `git diff --cached --name-only -z`로 스테이지된 파일을 보고 나눈다.
         - 프론트 쪽(`src/`, `package.json`, `pnpm-lock.yaml`, `*.config.*`, `tsconfig.json`, `index.html`, `docs/spec.md`(용어표 테스트), `scripts/`)이 있으면: `pnpm lint` → `pnpm typecheck` → `pnpm test`.
         - Rust 쪽(`src-tauri/`)이 있으면: `cargo fmt --all --check` → `cargo clippy --workspace --all-targets -- -D warnings`.
         - 둘 다 없으면(문서만 바뀐 커밋) 아무것도 하지 않는다.
         - `fast --all` 옵션: 스테이지와 상관없이 위 두 묶음을 모두 실행(수동용).
       - `full`(pre-push): 프론트 묶음 전부 → `cargo fmt --all --check` → clippy → `cargo test --workspace` → `cargo run -p app --bin gen-bindings` 후 `git diff --exit-code -- src/lib`.
     - `.githooks/pre-commit`, `.githooks/pre-push`: `#!/bin/sh` + `cd "$(git rev-parse --show-toplevel)" && exec node scripts/check.mjs fast`(push는 `full`). 실행 비트는 `git add --chmod=+x .githooks/*`.
     - `scripts/install-hooks.mjs`: `.git`이 있는 작업 폴더에서만 `git config core.hooksPath .githooks`. 없으면 아무것도 안 하고 0. 이미 다른 값이면 덮어쓰기 전에 알림.
     - `package.json` scripts: `"prepare": "node scripts/install-hooks.mjs"`, `"check:fast": "node scripts/check.mjs fast --all"`, `"check:full": "node scripts/check.mjs full"`.
     - `eslint.config.js`: `scripts/**/*.mjs`를 Node 전역으로 린트.
     - `scripts/check.test.mjs`(vitest, node 환경): 스테이지 파일 목록 → 실행할 묶음 고르기 함수 단위 테스트(프론트만, Rust만, 둘 다, 문서만).
     - `CLAUDE.md` "명령": `pnpm check:fast`, `pnpm check:full`, hook 설명. "커밋": "커밋 때 pre-commit이 바뀐 쪽을 빠르게, push 때 pre-push가 전체 테스트를 검사한다. `--no-verify`로 건너뛰지 않는다. Rust 코드를 고친 커밋은 커밋 전에 `pnpm check:full`을 돌린다(D-022)."
     - `README.md` "커밋 전 검사" 절: 자동 설치, 확인(`git config core.hooksPath` → `.githooks`), Windows Git Bash가 `node`, `pnpm`, `cargo`를 PATH에서 찾아야 한다는 점, 수동 실행 명령.
   - 테스트: 새로 clone 후 `pnpm install` → `core.hooksPath`가 `.githooks`. `color: #fff`를 넣고 커밋 → 거부. 문서만 바꾼 커밋 → 검사 없이 통과. Rust 테스트를 일부러 실패시키고 push → 거부(push 전에 되돌림). Windows: cmd·PowerShell·VS Code 소스 제어에서 hook 실행.
   - 완료 기준: 위 확인을 두 OS에서 1회씩. `pnpm test`에 `check.test.mjs` 포함 통과.
   - Windows/Ubuntu: hook 파일은 `.gitattributes`의 `eol=lf`(CRLF면 `/bin/sh^M` 오류). Ubuntu는 실행 비트 필요.
   - 위험: hook은 작업 폴더 상태를 검사하므로 스테이지하지 않은 수정이 결과에 섞인다(README에 적음). GUI Git 도구(hiGit 포함)에서 커밋해도 hook이 돈다.

3. ③ CI에서 Windows 실행 파일 artifact 올리기 (`ci: upload windows build artifact`)
   - 변경 파일: `.github/workflows/ci.yml`, `src-tauri/.cargo/config.toml`(신규), `packaging/windows/README.txt`(받은 사람용 실행 안내, ASCII 파일명), `README.md`("Windows 시험판 받기" 짧은 절).
   - Windows crt-static(사용자 지시): `src-tauri/.cargo/config.toml`에
     ```toml
     # Windows 실행 파일이 VC++ 재배포 패키지 없이 뜨도록 C 런타임을 정적으로 링크한다.
     [target.x86_64-pc-windows-msvc]
     rustflags = ["-C", "target-feature=+crt-static"]
     ```
     Rust 코드가 아니라 빌드 설정이므로 "OS별 코드는 platform" 규칙의 대상이 아니다. CI Windows 작업에 확인 단계를 넣는다: `dumpbin /dependents src-tauri/target/release/higit.exe` 출력에 `VCRUNTIME140.dll`, `MSVCP140.dll`이 없어야 통과(없으면 실패). tauri-build의 기본 정적 링크와 겹쳐도 문제없다.
   - `ci.yml` 변경:
     - `on`에 `workflow_dispatch:` 추가(사람이 아무 커밋에서나 수동으로 받기).
     - `pnpm tauri build --no-bundle` 다음에 Windows에서만:
       ```yaml
       - name: Windows 결과물 모으기
         if: runner.os == 'Windows'
         shell: pwsh
         run: |
           New-Item -ItemType Directory -Force artifact | Out-Null
           Copy-Item src-tauri/target/release/higit.exe artifact/
           Copy-Item THIRD_PARTY_NOTICES.md artifact/
           Copy-Item packaging/windows/README.txt artifact/
       - name: Windows 결과물 올리기
         if: runner.os == 'Windows' && github.event_name != 'pull_request'   # H-6
         uses: actions/upload-artifact@v7
         with:
           name: higit-windows-x64-${{ github.sha }}
           path: artifact/
           retention-days: 14          # H-6
           if-no-files-found: error
       ```
     - `THIRD_PARTY_NOTICES.md`를 함께 넣는 이유: 글꼴 OFL은 배포 시 라이선스 원문 동봉이 조건이다(D-014). 실행 파일만 올리면 조건을 어긴다.
   - `README.txt` 내용(한국어, 짧게): 실행 조건(아래), SmartScreen 경고 시 "추가 정보 → 실행", 시험판이라 서명 없음, 문제 보고 방법.
   - 실행 조건(README.txt와 README.md에 같은 내용):
     - Windows 10(1809 이상)/11 x64. ARM64는 시험하지 않음.
     - WebView2 런타임: Windows 11은 기본 포함. Windows 10은 대부분 Edge 업데이트로 깔려 있지만 없으면 창이 뜨지 않는다 → Microsoft "Evergreen Bootstrapper"로 설치. `--no-bundle` 실행 파일은 설치 프로그램과 달리 WebView2를 자동으로 깔지 않는다.
     - Git for Windows 2.30 이상(앱이 시작할 때 확인하고 없으면 안내).
     - Visual C++ 런타임: crt-static으로 필요 없다(CI의 `dumpbin` 검사로 고정). 깨끗한 Windows에서 한 번 더 실행해 확인한다.
     - 화면 자원(HTML/JS/글꼴)은 exe 안에 들어 있어 exe 하나로 실행된다.
   - 테스트: main에 push → Actions 실행 화면의 Artifacts에 `higit-windows-x64-<sha>`가 있다 → 내려받아 압축을 풀면 `higit.exe`, `THIRD_PARTY_NOTICES.md`, `README.txt` 세 개 → Windows 11 실기기에서 더블클릭 → 홈 화면이 뜨고 폴더 열기 가능.
   - 완료 기준: 위 확인 1회. `workflow_dispatch`로 수동 실행해도 artifact가 생긴다.
   - Windows/Ubuntu: Windows 작업에만 해당. Ubuntu 결과물(실행 파일, AppImage)은 범위 밖(필요하면 H-6에 추가).
   - 위험: 서명 없는 exe라 SmartScreen이 막을 수 있다. 결과물 이름에 전체 sha를 쓰면 길지만 어느 커밋인지 확실하다. artifact 저장 용량은 비공개 저장소 요금제 한도를 쓴다(exe 수십 MB × 14일 보존).

### B. git 크레이트

4. git: stdin 입력, 경로 목록 전달, 실패 분류 (`git: add stdin input and failure classification`)
   - 전제: D-021 확정(H-17).
   - 변경 파일: `crates/git/src/cli/runner.rs`(`run_with_stdin(cwd, args, input: &[u8])` 추가, 기존 `run`은 stdin null 유지), `crates/git/src/failure.rs`(신규), `error.rs`(`GitError::Failed`에 `kind: FailureKind` 추가 또는 `GitError::kind()` 메서드), `lib.rs`.
   - 경로 목록은 명령줄 대신 `--pathspec-from-file=- --pathspec-file-nul`로 stdin에 NUL 구분으로 넘긴다(Windows 명령줄 길이 32K 제한, 파일 1,000개 대응). 경로 명령에는 `--literal-pathspecs` 전역 옵션을 붙여 `*`, `[` 같은 글자를 패턴으로 해석하지 않는다.
   - `FailureKind`: `NothingToCommit`, `IdentityMissing`, `InvalidRefName`, `BranchExists`, `WouldOverwrite`, `NotFastForward`(pull), `PushRejected`, `NoUpstream`, `SshHostUnverified`(`Host key verification failed`), `SshAuthFailed`(`Permission denied (publickey…)`, 키 비밀번호가 필요한데 ssh-agent에 없을 때 포함), `HttpAuthFailed`(`Authentication failed`), `RemoteUnreachable`, `Other`. 문구 대응표는 `failure.rs` 한 곳에(D-021, D-023).
   - SSH 키 비밀번호 구분(사용자 지시): BatchMode에서는 "비밀번호가 걸린 키"와 "키 없음/등록 안 됨"이 둘 다 `Permission denied (publickey)`로 끝날 수 있다. 이 단계에서 실험으로 확인한다: 비밀번호 걸린 키 + agent 없음으로 `ssh -o BatchMode=yes`를 실행해 stderr에 구분 가능한 문구가 있는지(OpenSSH 버전별). 있으면 `SshKeyLocked`를 따로 두고, 없으면 `SshAuthFailed` 안내에 "키에 비밀번호가 있으면 ssh-agent에 올려 주세요"를 함께 쓴다. 실험 결과를 계획의 위험 절에 적는다.
   - 로케일(사용자 지시): `Runner`는 `LC_ALL=C`에 더해 `LANGUAGE`를 지운다(`env_remove("LANGUAGE")`, LANGUAGE가 LC_ALL보다 우선하는 gettext 경우 대비). 테스트: 부모 환경에 `LANG=ko_KR.UTF-8`, `LANGUAGE=ko`, `LC_MESSAGES=ko_KR.UTF-8`를 넣고 실패 명령(예: 저장소 아닌 곳의 `rev-parse`)의 stderr가 영어(`not a git repository`)인지 확인. 대조군으로 같은 환경에서 `LC_ALL` 없이 git을 실행해 한국어가 나오면 "번역이 설치된 환경에서 검증됨"으로, 영어가 나오면(번역 미설치) 테스트는 통과하되 그 사실을 출력한다. CI Ubuntu에는 `language-pack-ko` 설치 단계를 추가해 실제로 검증한다.
   - 테스트: `run_with_stdin`로 한글·공백 경로 3개를 NUL로 넘겨 받는지(다음 단계 `add`에서 실제 사용), 분류 함수 단위 테스트(실제 git stderr 표본). 각 `FailureKind`의 실제 재현 테스트는 해당 작업 단계(7~9)에서 추가한다.
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: stdin 쓰기와 출력 읽기를 동시에 하지 않으면 큰 입력에서 막힐 수 있다 → 입력은 별도 스레드에서 쓰거나 `Command::output` 전에 `spawn` 후 `stdin.write_all` → `wait_with_output`. Windows 파이프 버퍼가 작아 이 차이가 먼저 드러난다.
   - 위험: stderr 문구가 git 버전마다 다를 수 있다 → CI의 두 OS git 버전으로 재현 테스트, 모르면 `Other`.

5. git: diff (`git: add diff with unified patch parser`)
   - 변경 파일: `backend.rs`(`fn diff(&self, repo, target: &DiffTarget) -> Result<FileDiff>`), `cli/diff.rs`(신규), `types.rs`(`DiffTarget { path, orig_path, kind: WorktreeVsHead | Untracked | Index }`, `FileDiff { path, binary, truncated, hunks: Vec<Hunk> }`, `Hunk { old_start, old_len, new_start, new_len, lines: Vec<DiffLine> }`, `DiffLine { kind: Context|Added|Removed, text, no_newline_at_end }`), `cli/mod.rs`.
   - 명령:
     - 기본(H-9 추천안): `git diff --no-color --no-ext-diff --no-textconv -U3 -M HEAD -- <path>`(저장된 쪽과 안 된 쪽을 합쳐 "마지막 저장과 비교"). 저장이 없는 저장소는 `--cached`와 작업 폴더를 합칠 수 없으므로 빈 트리 대비: `git diff --no-color ... 4b825dc642cb6eb9a060e54bf8d69288fbee4904 -- <path>`(빈 트리 해시, SHA-256 저장소는 `git hash-object -t tree --stdin </dev/null`로 구함).
     - 새 파일(추적 안 함): `git diff --no-index --no-color -- /dev/null <path>`(종료 코드 1이 "차이 있음"이므로 실패로 보지 않는다). Git for Windows도 `/dev/null`을 특별 처리한다 → Windows CI 테스트로 확인.
     - 출력은 unified diff 형식(기계용으로 안정된 형식)만 파싱한다. 이진은 `Binary files … differ` 줄로 판별.
   - 큰 파일: 줄 5,000개 또는 1MB를 넘으면 자르고 `truncated: true`.
   - 테스트: 수정·추가·삭제 줄, 여러 hunk, 끝 줄바꿈 없음(`\ No newline at end of file`), 이름 바꿈, 새 파일, 삭제 파일, 이진 파일, 한글 내용과 한글 파일명, CRLF 파일(줄 끝 `\r` 보존 여부), 저장 없는 저장소, 잘림.
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: 테스트 저장소는 `core.autocrlf=false`로 만들어 CRLF 내용을 그대로 비교한다. 사용자의 `autocrlf=true`(Windows 기본)에서는 diff에 줄 끝 차이가 안 보이는 게 정상. 텍스트는 바이트로 받아 lossy UTF-8(EUC-KR 파일은 깨져 보임 → 위험에 기록).

6. git: 갈래 목록, 원격 목록, 갈래 이름 검사 (`git: add branches, remotes and ref name check`) — 1b
   - 변경 파일: `backend.rs`(`branches`, `remotes`, `check_branch_name`), `cli/branches.rs`(신규), `types.rs`(`Branch { name, kind: Local | Remote, is_head, upstream, ahead, behind, gone }`, `Remote { name }`).
   - 명령: `git for-each-ref --format=%(refname)%00%(objectname)%00%(HEAD)%00%(upstream)%00%(upstream:track,nobracket) refs/heads refs/remotes`(레코드는 줄, 필드는 NUL. 갈래 이름에 줄바꿈은 들어갈 수 없다). `refs/remotes/*/HEAD`는 뺀다. 원격은 `git remote`(줄당 이름 하나). 이름 검사는 `git check-ref-format --branch <name>`(종료 코드만 본다).
   - 테스트: 로컬 갈래 여러 개, 현재 갈래 표시, 한글 갈래 이름, upstream ahead/behind/gone, 원격 갈래, 저장 없는 저장소(갈래 0개), 잘못된 이름(`a..b`, `-x`, 공백, `@{`)과 올바른 한글 이름.
   - 완료 기준: `cargo test -p git` 두 OS 통과.

7. git: 저장에 넣기·빼기·저장하기 (`git: add stage, unstage and commit ops`)
   - 변경 파일: `types.rs`(`GitOp::Stage { paths }`, `Unstage { paths }`, `Commit { message }`), `cli/mod.rs`(매핑), `cli/run.rs`로 `run` 매핑을 옮겨도 됨.
   - 매핑:
     - Stage: `git --literal-pathspecs add -A --pathspec-from-file=- --pathspec-file-nul`(`-A`로 지운 파일도 넣음).
     - Unstage: 저장이 있으면 `git --literal-pathspecs restore --staged --pathspec-from-file=- --pathspec-file-nul`. 저장이 없는 저장소(HEAD 없음)는 `restore --staged`가 실패하므로 `git --literal-pathspecs rm --cached -r -q --pathspec-from-file=- --pathspec-file-nul`.
     - Commit: `git commit -F -`(메시지를 stdin으로, 한글·여러 줄 안전). 사용자의 commit hook과 서명 설정은 그대로 따른다(D-003).
   - 테스트(실제 git): 한글·공백·`*`가 든 파일명 넣기/빼기, 지운 파일 넣기, 1,000개 넣기, 저장 없는 저장소에서 빼기, 저장 후 `log` 맨 위 요약이 메시지 첫 줄, 넣은 것 없이 저장 → `NothingToCommit`, 이름·이메일 없는 환경(`GIT_CONFIG_GLOBAL=<빈 파일>`, `GIT_CONFIG_NOSYSTEM=1`, `user.useConfigOnly=true`) → `IdentityMissing`.
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: Windows에서 `-F -`로 넘긴 UTF-8 메시지가 그대로 저장되는지(`i18n.commitEncoding` 기본 UTF-8) 테스트로 확인. 테스트 환경에서 사용자 전역 설정이 섞이지 않게 `GIT_CONFIG_GLOBAL`을 임시 파일로.

8. git: 갈래 만들기·바꾸기 (`git: add branch create and switch ops`) — 1b
   - 변경 파일: `types.rs`(`GitOp::CreateBranch { name, start: Option<String> }`, `Switch { branch, track: Option<String> }`), `cli/mod.rs`.
   - 매핑: CreateBranch `git branch <name> [<start>]`. `-`로 시작하는 이름이 옵션으로 읽히지 않도록 core가 6단계의 이름 검사를 먼저 통과시킨 이름만 넘긴다. Switch: 로컬 갈래 `git switch <branch>`, 원격 갈래 `git switch -c <local> --track <remote>/<branch>`. `--no-guess`로 이름 추측을 끈다(동작을 명시적으로).
   - 테스트: 만들기 후 `branches`에 보임, 같은 이름 → `BranchExists`, 바꾸기 후 `status.branch.head`, 덮어쓸 변경이 있는 바꾸기 → `WouldOverwrite`, 겹치지 않는 변경은 따라옴(H-15), 원격 갈래로 바꾸기(로컬 bare 원격).
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: 대소문자만 다른 갈래 이름(`Feature`/`feature`)은 Windows 파일 시스템에서 충돌한다 → 테스트로 결과를 기록하고 core의 이름 검사에서 기존 갈래와 대소문자 무시 비교로 막을지 H로(H-18).

9. git: 새 소식 확인·최신으로 받기·올리기 (`git: add fetch, pull and push ops`) — 1b
   - 변경 파일: `types.rs`(`GitOp::Fetch`, `Pull`, `Push { set_upstream: Option<(String, String)> }`), `cli/mod.rs`, `cli/runner.rs`(네트워크 작업용 env).
   - 매핑(H-10~H-13 추천안 기준): Fetch `git fetch --all`(prune 안 함), Pull `git pull --ff-only`, Push `git push` 또는 `git push -u <remote> HEAD:refs/heads/<branch>`.
   - 네트워크 작업에는 추가로 `GIT_ASKPASS`를 비워 두고(`GIT_TERMINAL_PROMPT=0`은 이미 있음), SSH 질문으로 멈추지 않게 하는 방법을 H-13 결정대로 넣는다. 자격 증명 도우미(Windows Git Credential Manager 창)는 막지 않는다.
   - 테스트(오프라인, 임시 bare 저장소를 `file://` 원격으로): fetch 후 behind 증가, ff 가능한 pull, 갈라진 상태 pull → `NotFastForward`, push 성공, 원격이 앞서 있을 때 push → `PushRejected`, upstream 없는 push에 `set_upstream`, 원격 없는 저장소 → 분류. 존재하지 않는 원격 주소 → `RemoteUnreachable`. 인증 실패는 CI에서 재현이 어려우므로 분류 단위 테스트만.
   - 완료 기준: `cargo test -p git` 두 OS 통과.
   - Windows/Ubuntu: `file://` 경로는 Windows에서 `file:///C:/...` 형식. 테스트 유틸에 OS 분기 없이 `url::Url::from_file_path` 대신 git이 받는 일반 경로(`C:\...`)를 그대로 원격 주소로 쓴다(git은 로컬 경로를 원격으로 받음). OS 분기 코드가 필요하면 platform에 둔다.
   - 위험: 실제 GitHub 인증은 수동 확인만 가능.

### C. core, app

10. core: diff·갈래 조회와 오류 코드 (`core: add diff and branch queries`)
    - 변경 파일: `core/src/query.rs`(`diff(repo, DiffRequest)`, `branches(repo)`, `check_branch_name(repo, name)`), `core/src/view/{mod.rs, diff.rs, branch.rs}`(`DiffRequest { path, origPath, kind }`, `FileDiffView`, `HunkView`, `DiffLineView`, `BranchListView { local, remote }`, `BranchRow`, `BranchNameCheck { ok, reason }`), `core/src/error.rs`(`CoreError::GitFailure(FailureKind, GitError)` 또는 `Git` 변형에서 분류를 꺼내는 방식), `view` `ErrorCode`에 `nothingToCommit`, `identityMissing`, `invalidBranchName`, `branchExists`, `wouldOverwrite`, `notFastForward`, `pushRejected`, `noUpstream`, `noRemote`, `authFailed`, `remoteUnreachable` 추가, `src-tauri/src/commands.rs`(`query_diff`, `query_branches`, `query_branch_name`), `bindings.rs`(명령 목록), 생성물, `src/lib/api.ts`, `src/messages/{ko,en}.ts`(새 오류 문구. `Record<ErrorCode,…>`라 빠지면 typecheck 실패).
    - 1a로 나누면 이 단계는 diff만, 갈래 조회는 1b의 12단계 앞에 별도 커밋으로.
    - 테스트: 가짜 백엔드로 view 변환, 분류된 git 오류 → `ErrorCode` 대응 표 테스트.
    - 완료 기준: `cargo test --workspace`, `pnpm typecheck`, `pnpm test`, 바인딩 최신.
    - 문구 예: `pushRejected` "올리기 전에 최신 내용을 먼저 받아야 해요.", `wouldOverwrite` "저장하지 않은 변경이 덮어써져요. 먼저 저장해 주세요.", `identityMissing` "저장에 쓸 이름과 이메일이 설정되어 있지 않아요."(H-16), `authFailed` "원격 저장소에 로그인하지 못했어요."

11. 용어 + core 액션: 저장에 넣기·빼기·저장하기 (`glossary: add unstage term` → `core: add stage, unstage and commit actions`) — 두 커밋
    - 11a 용어(H-8): `docs/spec.md` 표에서 `stage` 바로 아래에 행 추가 `| unstage | 저장에서 빼기 | unstage · \`git restore --staged\` | 다음 저장에 넣었던 파일을 다시 빼기 | safe | 매일 |`, `src/glossary/types.ts`의 `TERM_IDS`에 같은 순서로, `ko.ts`, `en.ts`("Unstage", "Take a file back out of the next commit"). 완료 기준: `pnpm test`(spec-sync)가 통과하고, 표 순서와 `TERM_IDS` 순서를 일부러 어긋나게 하면 실패.
    - 11b 액션(새 액션 추가 절차 3~4, 6):
      - `view/input.rs`: `Stage(StageInput { paths: Vec<String> })`, `Unstage(UnstageInput { paths })`, `Commit(CommitInput { message: String })`.
      - `core/src/actions/{stage.rs, unstage.rs, commit.rs}`, `actions/mod.rs` 등록. 메타: `stage`/`stage`/Safe, `unstage`/`unstage`/Safe, `commit`/`commit`/Safe.
      - `available`: 셋 다 `repo`가 있을 때만(`NotARepo`).
      - 입력 검사(core에서, git 전에): `paths` 비면 `InvalidInput`, 경로에 `..` 구성요소나 절대 경로가 있으면 `InvalidInput`(저장소 밖 접근 방지), Windows 구분자 `\`는 `/`로 바꾸지 않고 거부(화면은 status가 준 `/` 경로만 보냄). 메시지가 공백뿐이면 `InvalidInput`.
      - `Outcome`: `Staged`, `Unstaged`, `Committed { short_oid, summary }`(저장 후 `log` 1개로 얻음). `view::ActionResult`에 같은 변형.
      - 저장소별 작업 잠금: `Core`에 `Mutex<HashSet<PathBuf>>` 또는 저장소별 `Mutex`를 두어 같은 저장소의 액션을 한 번에 하나만 실행(`index.lock` 충돌 방지). 다른 액션이 실행 중이면 `Busy` 오류 코드("다른 작업이 끝날 때까지 기다려 주세요.").
      - 1단계에서 미뤘던 실행기 "입력이 다름" 토큰 테스트를 `Commit` 두 값으로 되살린다.
      - `registry-glossary.test.ts`는 손대지 않아도 새 액션을 검사한다.
    - 테스트: 가짜 백엔드로 각 액션이 올바른 `GitOp`를 부르는지, 입력 검사, 저장소 아님, 잠금(두 스레드). 실제 git 통합 테스트 1개: 파일 만들기 → stage → commit → log 맨 위.
    - 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신, `action-meta.json`에 네 액션.
    - Windows/Ubuntu: 경로 검사에서 `C:` 드라이브 접두사와 `\\` UNC도 거부되는지 두 OS 단위 테스트(문자열 검사라 OS 분기 불필요).

12. core 액션: 갈래 만들기·바꾸기 (`core: add branch-create and switch actions`) — 1b
    - `view/input.rs`: `BranchCreate(BranchCreateInput { name, start: Option<String> })`, `Switch(SwitchInput { branch: String, remote: Option<String> })`.
    - `actions/{branch_create.rs, switch.rs}`. 메타: `branch-create`/`branch`/Safe, `switch`/`switch`/Normal. "만들고 바로 바꾸기"는 화면이 두 액션을 차례로 부른다(한 액션으로 묶으면 위험도가 `branch`의 safe와 어긋남).
    - `available`: 저장소 + 저장이 하나 이상(`has_commits`, 저장 없는 저장소에서 갈래를 만들 수 없음 → 새 `UnavailableReason::NoCommits`).
    - 테스트: 가짜 백엔드 호출, 이름 검사 실패 시 git 작업을 부르지 않음, 저장 없음.
    - 완료 기준: 11b와 같음.

13. core 액션: 새 소식 확인·최신으로 받기·올리기 (`core: add fetch, pull and push actions`) — 1b
    - `view/input.rs`: `Fetch`, `Pull`, `Push`(셋 다 입력 없음 unit 변형).
    - `actions/{fetch.rs, pull.rs, push.rs}`. 메타: `fetch`/Safe, `pull`/Normal, `push`/Normal.
    - `available`: 원격이 하나 이상(`NoRemote`). pull은 upstream이 있어야(`NoUpstream`). push는 갈래 위에 있어야(detached면 `Detached`).
    - push의 upstream 정하기(H-11 추천안): upstream이 있으면 그대로. 없으면 원격이 하나면 그것, 여럿이면 `origin`, 없으면 `NoRemote`. 같은 이름의 원격 갈래로 `-u`.
    - 네트워크 작업은 저장소 잠금을 잡는 동안 오래 걸릴 수 있다 → 화면은 진행 중 표시. 취소는 범위 밖(위험에 기록).
    - 테스트: 가짜 백엔드로 upstream 정하기 경우 전부, `available` 조합.
    - 완료 기준: 11b와 같음.

14. watcher + core + app: 파일 변경 감시와 `repo-changed` 이벤트 (`watcher: watch repository and emit repo-changed`)
    - 변경 파일:
      - `crates/watcher/Cargo.toml`(`notify`, `notify-debouncer-full`; workspace 의존성에 버전 추가), `crates/watcher/src/lib.rs`: `RepoWatcher::start(root: &Path, on_change: Box<dyn Fn(ChangeSet) + Send>) -> Result<RepoWatcher>`, `Drop`이면 감시 중지. `ChangeSet { worktree: bool, git: bool }`. 300ms 디바운스(architecture.md "성능 기준"). `.git/` 안에서는 `HEAD`, `index`, `refs/**`, `packed-refs`, `FETCH_HEAD`만 `git`으로 보고 `*.lock`, `objects/`, `logs/`는 버린다. watcher는 다른 내부 크레이트를 모른다.
      - `crates/core/Cargo.toml`(watcher 의존 추가), `core/src/watch.rs`: `Core::watch(repo, sink: Arc<dyn Fn(view::RepoChanged) + Send + Sync>)`(이전 감시는 멈추고 하나만 유지), `Core::unwatch()`. 감시 시작 실패(예: inotify 한도)는 오류로 돌려주되 화면은 조용히 "창 포커스 때 새로 고침"으로 떨어진다(코드 `watchUnavailable`).
      - `core/src/view/mod.rs`: `RepoChanged { repo: String, kinds: Vec<RepoChangeKind> }`, `RepoChangeKind { Worktree, Git }`.
      - `src-tauri/src/commands.rs`: `repo_watch(repo)`, `repo_unwatch()`. sink는 `AppHandle::emit("repo-changed", payload)`.
      - `src-tauri/src/bindings.rs`: 이벤트 이름 상수 `REPO_CHANGED = "repo-changed"`와 `RepoChanged` 타입 내보내기(`builder.typ::<RepoChanged>()`). core가 tauri·tauri-specta에 의존하지 않도록 typed event derive(`tauri_specta::Event`)는 쓰지 않는다(D-019).
      - `src/lib/events.ts`(신규): `onRepoChanged(handler)` — `@tauri-apps/api/event`의 `listen`을 여기서만 쓴다. `eslint.config.js`의 `no-restricted-imports`에 `@tauri-apps/api/event`도 `src/lib/` 밖 금지로 추가.
    - 테스트: watcher 단위 테스트(임시 폴더에서 파일 쓰기 → 1초 안에 `worktree`, `.git/index` 건드리기 → `git`, `.git/objects/xx` → 이벤트 없음, 300ms 안의 여러 변경 → 이벤트 1회). core: 두 번째 `watch`가 첫 감시를 멈춤. 이벤트 이름이 Rust 상수와 `events.ts`에서 같은지 확인하는 테스트(생성된 바인딩에 상수를 내보내거나 `action-meta.json`처럼 작은 json으로 내보내 비교).
    - 완료 기준: `cargo test --workspace` 두 OS 통과(파일 이벤트 테스트는 타이밍 여유를 넉넉히, 최대 5초 대기). 수동: 편집기에서 파일 저장 → 1초 안에 목록 갱신.
    - Windows/Ubuntu: Ubuntu는 inotify가 폴더마다 감시를 하나씩 쓰므로 `node_modules`, `target`이 큰 저장소에서 `max_user_watches` 한도에 걸릴 수 있다(22.04 기본값은 메모리에 따라 정해짐) → 실패 시 위 대체 동작. Windows는 `ReadDirectoryChangesW`가 루트 폴더 핸들을 잡아서 앱이 열고 있는 동안 저장소 폴더 이름 바꾸기·지우기가 막힐 수 있다 → 작업 공간을 닫으면 `repo_unwatch`. 이 앱의 조회는 `GIT_OPTIONAL_LOCKS=0`이라 status가 index를 쓰지 않아 이벤트 되먹임 고리가 생기지 않는다(유지해야 함, 테스트로 고정: status 호출 후 `git` 이벤트 없음).
    - .gitignore 대상 제외(사용자 지시): watcher가 `ignore` 크레이트의 `gitignore::GitignoreBuilder`로 저장소의 `.gitignore`(하위 폴더 포함)와 `.git/info/exclude`를 읽어 작업 폴더 이벤트를 거른다. `.gitignore` 파일 자체가 바뀌면 규칙을 다시 읽고 `worktree` 이벤트를 보낸다. 무시된 폴더(`node_modules`, `src-tauri/target` 등)는 가능하면 감시 등록 자체를 하지 않는다(notify의 비재귀 감시 + 하위 폴더 선별 등록. 어려우면 이벤트만 거르고 inotify 한도 위험은 남긴다). 전역 `core.excludesFile`은 watcher가 git 설정을 읽을 수 없으므로 core가 경로를 알아내 넘긴다(`git config --path core.excludesFile`, Query 하나 추가). watcher는 여전히 다른 내부 크레이트를 모른다.
    - 감시 실패 대체(사용자 지시): 시작 실패(inotify 한도 등)뿐 아니라 실행 중 오류(notify 오류 이벤트, 감시 폴더 삭제)도 core가 `RepoChanged`와 별도로 `watch-status { repo, active: false }` 이벤트로 알린다. 화면은 감시가 꺼져 있을 때 TanStack Query의 창 포커스 새로 고침을 켜고, 감시가 켜져 있을 때는 끈다(이중 갱신 방지).
    - 테스트 추가: 무시된 파일(`node_modules/x.js`, `target/a`) 쓰기 → 이벤트 없음, `.gitignore` 수정 → 규칙 갱신 후 이전에 무시되던 파일 이벤트 발생, 감시 중 폴더 삭제 → `active: false`.
    - 위험: 빌드 산출물 폭주는 위 제외로 대부분 사라진다. 남는 것은 무시되지 않은 큰 생성물 → 디바운스 + 화면 1초 throttle.

### D. 화면
app 명령과 바인딩은 따로 단계를 두지 않고 10~14단계의 core 변경과 같은 커밋에 넣는다(바인딩 최신 검사가 단계마다 통과해야 하므로).

15. 작업 공간 배치와 상단 상태 표시 (`screens: workspace layout and status bar`)
    - 변경 파일: `src/screens/Workspace/{Workspace.tsx, Workspace.module.css}`, `src/screens/Workspace/StatusBar.tsx` + CSS(신규), `src/screens/History/History.tsx`(저장 기록 탭, 기존 `HistoryList` 사용), `src/state/repoStore.ts`(현재 탭, 고른 파일), `src/messages/{ko,en}.ts`, 테스트.
    - 배치(spec.md "화면 구성"): 상단 막대(저장소 이름 · 상태 표시 · 탭 "작업 공간 | <Term id="log" />" · 새로 고침 · 다른 폴더 열기). 작업 공간 탭: 왼쪽 갈래 목록 자리(1b 전에는 현재 갈래 이름만), 가운데 바뀐 파일 + 비교, 오른쪽 저장 상자. 최소 창 너비 800px에서 왼쪽 패널은 접힌다.
    - 상태 표시(spec.md 원칙 3, `StatusView.branch` 사용): `<Term id="branch" />` 이름(detached면 "특정 저장을 보는 중"), `<Term id="status" />` N개, upstream이 있으면 "올릴 저장 N"(`<Term id="push" />`와 함께), "받을 저장 N"(`<Term id="pull" />`), 없으면 "아직 올린 적 없음". 숫자는 tokens.css 색만 쓴다.
    - 테스트: 상태 표시 문구(upstream 있음/없음/detached/저장 없음), 탭 전환.
    - 완료 기준: `pnpm test`, `pnpm lint` 통과. 두 OS 수동: 800px 너비, 라이트/다크, 툴팁.
    - Windows/Ubuntu: WebKitGTK에서 CSS grid `minmax`·`container queries` 지원 여부 확인(D-001). 쓰기 전에 Ubuntu 22.04의 WebKitGTK 2.4x에서 동작하는지 본다.

16. 저장에 넣기·빼기 화면 (`features: stage and unstage from changed files`)
    - 변경 파일: `src/screens/Workspace/ChangedFiles.tsx`(체크 상자 열, 행 클릭은 비교 선택), `src/features/stage/useStageMutation.ts`, "모두 넣기/모두 빼기", 테스트.
    - 동작: 체크 → `executeAction(folder, { id: "stage", input: { paths: [path] } })`, 해제 → `unstage`. 이름 바꿈은 `origPath`도 같이 보낸다. 성공 후 `status` 쿼리 무효화(감시 이벤트를 기다리지 않음). 낙관적 갱신은 하지 않는다(실패 시 되돌리기 복잡).
    - 체크 상자 이름표는 `<Term id="stage" />`·`<Term id="unstage" />`를 aria/툴팁으로.
    - 테스트: api 모킹으로 체크·해제 호출 인자, 실패 시 `ErrorNotice`, 모두 넣기.
    - 완료 기준: `pnpm test` 통과, 수동: 한글 파일명, 1,000개 모두 넣기가 2초 안.

17. 비교 보기 화면 (`features: diff viewer`)
    - 변경 파일: `src/features/diff/{DiffView.tsx, DiffView.module.css, model.ts, model.test.ts}`, `src/lib/queries.ts`(`useDiff(repo, request)`), 테스트.
    - 동작: 고른 파일의 비교. 줄 번호 두 열, 추가 줄 `--diff-add-bg`/`--added`, 삭제 줄 `--diff-del-bg`/`--deleted`, 코드 글꼴. 이진·잘림·빈 차이 안내. 제목 `<Term id="diff" />`. 긴 줄은 가로 스크롤.
    - `model.ts`: `FileDiffView` → 그리기용 행(순수 함수, 단위 테스트).
    - 테스트: 모델 변환, 이진·잘림 문구.
    - 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 한글 내용, CRLF 파일, 5,000줄 파일 스크롤.
    - Windows/Ubuntu: 줄 끝 `\r`은 화면에서 지우고 "이 파일은 CRLF 줄바꿈" 작은 안내(Windows 사용자가 흔히 만남).

18. 저장 상자 (`features: commit box`)
    - 변경 파일: `src/features/commit/{CommitBox.tsx, CommitBox.module.css, CommitBox.test.tsx}`.
    - 동작: 여러 줄 입력, 주 버튼 `<Term id="commit" />`(화면의 유일한 주 버튼, design.md). 넣은 파일 0개 또는 메시지가 공백뿐이면 꺼짐(H-14). Ctrl+Enter로 저장, `event.nativeEvent.isComposing`이면 무시(design.md 한글 입력). 성공 시 입력을 비우고 `status`, `log` 무효화, "저장했어요 · <짧은 해시>" 한 줄. `identityMissing`이면 안내 + "자세히 보기"(H-16).
    - 테스트: 버튼 활성 조건, Ctrl+Enter, 조합 중 Enter 무시, 성공 후 비움, 오류 문구.
    - 완료 기준: `pnpm test`. 두 OS 수동: 한글 IME(Windows MS 입력기, Ubuntu IBus/fcitx)로 메시지 입력 중 Enter가 저장을 실행하지 않음.

19. 갈래 목록·만들기·바꾸기 화면 (`features: branch list, create and switch`) — 1b
    - 변경 파일: `src/features/branches/{BranchList.tsx, CreateBranchDialog.tsx, *.module.css, *.test.tsx}`, `src/lib/queries.ts`(`useBranches`, `useBranchNameCheck`), 대화상자에 Radix Dialog(D-004, 새 의존성 `@radix-ui/react-dialog`).
    - 동작: 왼쪽 목록에 로컬 갈래(현재 갈래는 왼쪽 2px `--accent` 막대), 접힌 "원격 갈래". 행 더블클릭 또는 메뉴 "바꾸기" → `switch`. "새 <Term id="branch" />" → 대화상자(이름 입력, 300ms 디바운스로 이름 검사, "만들고 바로 바꾸기" 체크 기본 켬) → `branch-create` 후 체크 시 `switch`. 만들기는 됐는데 바꾸기가 실패하면 그 사실을 문구로.
    - 테스트: 목록, 이름 검사 안내, 두 액션 순서 호출, 바꾸기 실패 문구.
    - 완료 기준: `pnpm test`, 수동 두 OS.

20. 받기·올리기·새 소식 확인 화면 (`features: fetch, pull and push in status bar`) — 1b
    - 변경 파일: `src/screens/Workspace/StatusBar.tsx`, `src/features/sync/{SyncButtons.tsx, *.test.tsx}`.
    - 동작: 상태 표시 옆 `<Term id="fetch" />`, `<Term id="pull" />`(받을 저장이 있을 때 강조), `<Term id="push" />`(올릴 저장이 있거나 upstream 없을 때). 실행 중에는 버튼 끄고 진행 표시. `pushRejected` 오류에는 [최신으로 받기] 버튼(spec.md 예시). `notFastForward`는 "내 저장과 원격 저장이 갈라져 있어요. 합치기는 다음 버전에서 지원해요."(H-10). 성공 후 `status`, `log`, `branches` 무효화.
    - 테스트: 버튼 표시 조건, 오류별 해결 버튼, 실행 중 꺼짐.
    - 완료 기준: `pnpm test`. 수동: GitHub 비공개 저장소에서 Windows(Credential Manager)·Ubuntu(HTTPS 토큰 또는 SSH 키) 각각 push/pull.

21. 변경 감시를 화면에 연결 (`screens: refresh on repo-changed`)
    - 변경 파일: `src/lib/useRepoWatch.ts`(Workspace가 열릴 때 `repo_watch`, 닫힐 때 `repo_unwatch`, `onRepoChanged` → `worktree`면 `status`·`diff`, `git`이면 `status`·`log`·`branches` 무효화, 1초 throttle), `Workspace.tsx`, 테스트.
    - 감시를 못 쓰거나 실행 중 꺼지면(`watchUnavailable` 또는 `watch-status`의 `active: false`) 창 포커스 새로 고침을 켠다. 감시가 켜져 있으면 끈다. 사용자에게 따로 표시하지 않는다.
    - 테스트: 가짜 이벤트로 무효화되는 쿼리 키, throttle, 닫을 때 unwatch.
    - 완료 기준: `pnpm test`. 두 OS 수동: 앱 밖 편집기 저장·터미널 커밋 후 1초 안에 갱신.

### E. 마무리

22. 문서 정리와 점검 (`docs: wrap up iteration 1`)
    - 변경 파일: `docs/architecture.md`(GitBackend 주석의 "반복 1에서 구현" 표시 갱신, `remotes`·`check_branch_name` 추가, watcher 절, 이벤트 절의 `op-progress`는 아직 없음 표시), `docs/roadmap.md`(반복 1 완료 표시, 1a/1b로 나눴다면 반영), `CLAUDE.md` "명령" 절 최종 확인, `docs/spec.md` 열린 질문 갱신.
    - `code-analyst`로 점검: 화면에서 `invoke`/`listen` 직접 호출 없음, `rg "cfg\((windows|target_os)" src-tauri --glob '!crates/platform/**'` 비어 있음, `cargo tree -p watcher -e normal`에 내부 크레이트 없음, 화면의 Git 용어가 모두 `<Term>`, tokens.css 밖 색 값 없음.
    - 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full` 통과.

## 용어 변경
- spec.md 용어표에 추가(11a, H-8): `| unstage | 저장에서 빼기 | unstage · \`git restore --staged\` | 다음 저장에 넣었던 파일을 다시 빼기 | safe | 매일 |` (`stage` 행 바로 아래).
- `src/glossary/types.ts`(`TERM_IDS`), `ko.ts`, `en.ts`에 같은 항목(D-015).
- 기존 용어의 화면 사용 추가: `diff`, `stage`, `commit`, `branch`, `switch`, `fetch`, `pull`, `push`, `log`(탭).
- 새 액션 id `branch-create`는 용어 `branch`를 쓴다(새 용어 아님). 화면 문구 "새 …"·"만들기"는 `messages`에 둔다.

## 위험과 열린 질문
- specta rc.25의 adjacently tagged enum 출력 모양(1단계 실험으로 먼저 확인).
- 성공 출력 외에 stderr를 분류하는 것은 D-003 보완이 필요하다(D-021 제안).
- (4단계 실험 결과, 2026-10-15) 비밀번호 걸린 SSH 키를 BatchMode에서 "키 없음"과 구분할 수 있는지는 확인하지 못했다. 시험하려면 그 키를 실제 GitHub 계정에 등록해야 하고(바깥 계정 변경이라 하지 않음), 사용자 `~/.ssh/config` 없이는 이 PC에서 GitHub SSH 접속이 시간 초과됐다. 그래서 `SshKeyLocked`는 두지 않고 `SshAuthFailed` 안내에 "키에 비밀번호가 있으면 ssh-agent에 올려 주세요"를 함께 쓴다. 호스트 확인 실패(`SshHostUnverified`)는 문구가 분명해 따로 둔다.
- SSH 원격에서 호스트 키 확인·암호 질문이 터미널(tty)로 나와 `pnpm tauri dev`처럼 터미널에서 띄운 앱의 git이 멈출 수 있다(H-13). 데스크톱에서 띄운 앱은 tty가 없어 실패로 끝난다.
- 네트워크 작업 취소·시간 제한이 없다. 멈춘 push는 앱을 닫아야 끝난다(반복 2 이후 `op-progress`와 함께).
- 비교가 lossy UTF-8이라 EUC-KR 등 다른 인코딩 파일은 깨져 보인다.
- 사용자의 commit hook(예: 이 저장소의 pre-commit)이 hiGit의 "저장하기"에서도 돈다. 오래 걸리면 화면이 진행 중으로 오래 남는다. hook 실패 원문은 "자세히 보기"로 보인다.
- Ubuntu inotify 한도, Windows 폴더 핸들 잠금(14단계).
- (14단계 구현 결과) `.gitignore` 대상은 **이벤트 단계에서 거른다**. 감시 등록은 저장소 루트 재귀 하나라서, Ubuntu에서는 무시된 폴더(`node_modules`, `target`)에도 inotify 감시가 걸린다. 폴더별 선별 등록은 Windows에서 폴더마다 핸들을 잡아 잠금 문제를 키우므로 OS별 전략이 필요하다(OS별 코드는 platform으로). 한도에 걸리면 감시 시작 실패 → 창 포커스 새로 고침으로 대체된다. 필요해지면 별도 단계로 다룬다.
- (14단계 구현 결과) 전역 무시 규칙은 `core.excludesFile`이 설정된 경우만 읽는다. 설정이 없을 때 git이 쓰는 기본 위치(`$XDG_CONFIG_HOME/git/ignore`, `~/.config/git/ignore`)는 홈 폴더 찾기가 OS별이라 아직 반영하지 않았다. 그 파일에만 있는 패턴의 파일이 바뀌면 화면이 한 번 더 새로 고쳐질 뿐 결과는 같다.
- Windows artifact는 서명이 없고 VC++ 런타임 정적 링크 여부를 실기기로 확인해야 한다(3단계).
- 작업 공간 3단 배치가 800px 최소 너비에서 좁다.

## 결정 결과 (2026-10-15)
- D-020, D-021, D-022 확정, D-023(SSH 질문 끄기) 추가 확정.
- H-1 `input_schema` 없앤다. H-2 트레이트가 `view::ActionInput`을 그대로 받는다, `Commit` 변형은 미리 넣지 않는다(11b에서 토큰 테스트 되살림). H-3 명령에서 `id` 인자를 없앤다.
- H-4 사용자 지시로 변경: pre-commit은 바뀐 쪽 빠른 검사, pre-push는 전체 검사(2단계). H-5 `prepare`로 자동 설치.
- H-6 main push와 수동 실행에서만 올림, 14일 보존, 설치 파일은 반복 5, Ubuntu 결과물은 올리지 않음.
- H-7 1a / 1b로 나눈다. **1a**: 1, 2, 3, 4, 5, 7, 10(diff 부분), 11a, 11b, 14, 15, 16, 17, 18, 21, 22. **1b**: 6, 8, 9, 10(갈래 조회), 12, 13, 19, 20, 22.
- H-8 `unstage` "저장에서 빼기" · `git restore --staged` · safe 그대로.
- H-9 마지막 저장과 합쳐 비교. H-10 `--ff-only`. H-11 자동 `-u`. H-12 `--all`, prune 안 함, 열 때 1회 + 수동.
- H-13 → D-023. H-14 버튼 끄기. H-15 git 기본 동작. H-16 안내만. H-17 → D-021 확정. H-18 두 OS 모두 막는다.
- 사용자 추가 지시: SSH 실패 종류 구분(4단계), 한국어 로케일에서도 영어 오류 테스트(4단계), Windows crt-static(3단계), 감시 실패 시 창 포커스 새로 고침과 .gitignore 대상 감시 제외(14·21단계).
