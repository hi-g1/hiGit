# 000 반복 0: 뼈대
- 관련 결정: D-001~D-004, D-006~D-015, D-018, D-019 (확정). D-005는 D-010으로 대체됨. D-017(제안)은 구현하지 않고 구조만 대비한다.
- 관련 명세: spec.md "원칙" 1·3, "화면 구성"의 홈·작업 공간(일부), "기능 목록과 용어표" 전체(glossary 데이터), "오류 메시지 원칙", roadmap.md "반복 0"

## 목표
Windows와 Ubuntu에서 앱을 실행해 폴더를 열면 바뀐 파일 목록과 저장 기록이 보이고, 화면의 Git 용어에 마우스를 올리면(또는 키보드로 포커스하면) 원래 이름·명령·설명이 뜬다. 이후 반복에서 기능을 붙일 구조(크레이트, GitBackend, Action 레지스트리와 실행기, Query, glossary, 토큰, CI)가 갖춰진다.

## 현재 상태
- 저장소에 코드가 없다. 있는 것: `CLAUDE.md`, `docs/`(research 포함), `.claude/agents/*.md`.
- 아직 git 저장소가 아니다. 1단계가 `git init`이다. 비공개 GitHub 저장소는 사람이 만들고, 4단계(CI)부터 push해 확인한다.

## 범위
들어가는 것(roadmap 반복 0):
- Tauri 2 + React 18 + TS(strict) + Vite. Rust 워크스페이스: `src-tauri/`가 app 크레이트 겸 루트, 나머지는 `src-tauri/crates/`(D-018).
- `GitBackend` 트레이트와 `CliBackend`의 `version`, `discover`, `status`, `log`, `run`(`GitOp::Init`만)(architecture.md에 "반복 0" 표시된 것만).
- `Action` 레지스트리와 실행기(`Executor`). 첫 실제 액션은 `init`(관리 시작, risk `safe`). 조회는 `core::query`(D-010).
- `Term` 부품, glossary(`ko.ts`, `en.ts`), 디자인 토큰과 글꼴.
- Windows·Ubuntu CI. 3단계 직후에 만들고 이후 단계마다 검사를 더한다.
- 완료 기준에 필요한 최소 부속: 폴더 선택 대화상자, `repo_open`, git 찾기와 2.30 확인(D-003), 오류 "자세히 보기".

들어가지 않는 것:
- init을 뺀 상태 변경 액션 전부.
- 노선도 보기(D-017 제안). 저장 기록은 단순 목록. 그리기 계층은 폴더와 화면 모델만 나눠 둔다.
- 파일 변경 감시(`watcher`는 빈 크레이트), 상단 상태 표시, 백업 참조, UndoJournal, 최근 프로젝트, 설정 화면, 언어 전환 UI, 다크 모드 전환 UI, 우클릭 메뉴, 단일 인스턴스, 명령줄 인자, 설치 파일. `hosting`은 빈 크레이트.

## 완료 기준
- Ubuntu 22.04와 Windows 11에서 `pnpm tauri dev`로 앱이 뜨고, 홈의 "폴더 열기"로 Git 저장소를 고르면 작업 공간으로 넘어간다.
- 작업 공간에 바뀐 파일 목록(새 파일·수정·삭제·이름 바꿈·충돌 구분, 한글 파일명 포함)과 저장 기록 목록(최근 200개, 요약·작성자·시각·짧은 해시)이 보인다. 저장이 없는 저장소는 빈 상태 문구.
- "바뀐 내용", "저장 기록" 제목은 `<Term>`이고, 호버 약 400ms 뒤(또는 Tab 포커스) 위쪽에 원래 이름·명령·설명 툴팁이 뜬다.
- 저장소가 아닌 폴더를 고르면 안내 문구 옆에 `<Term id="init" />` 버튼("관리 시작")이 보이고, 누르면 그 폴더가 저장소가 되어 작업 공간으로 넘어간다.
- git 없음, git 2.30 미만, init 실패에서 쉬운 안내와 "자세히 보기"(원문)가 보인다.
- 레지스트리의 액션(`init`) `glossary_id`가 glossary에 있고 위험도가 spec.md 표와 같음을 `pnpm test`가 확인한다.
- `ko.ts`가 spec.md 용어표 29행과 일치하고, `en.ts`가 같은 id 집합을 가짐을 `pnpm test`가 확인한다.
- GitHub Actions에서 `ubuntu-22.04`, `windows-latest` 모두 lint, 테스트, `tauri build --no-bundle` 통과.
- `CLAUDE.md` "명령" 절이 실제로 동작하는 명령으로 채워져 있다.

## 반복 0에서 확정할 명령
마지막 단계(17)에서 `CLAUDE.md`에 기록한다(H-9 승인: `--all-targets`, `typecheck`, `fmt --check` 포함).

| 용도 | 명령 | 어디서 | 검증 방법 |
| --- | --- | --- | --- |
| 개발 실행 | `pnpm tauri dev` | 루트 | 창이 뜨고 홈 화면이 보임(두 OS 수동) |
| 프론트 테스트 | `pnpm test` (`vitest run`) | 루트 | CI 통과 |
| 프론트 타입 검사 | `pnpm typecheck` (`tsc --noEmit`) | 루트 | CI 통과 |
| 프론트 린트 | `pnpm lint` (ESLint + Stylelint) | 루트 | CI 통과. 일부러 `#fff`를 넣으면 실패하는지 1회 확인 |
| Rust 테스트 | `cargo test --workspace` | `src-tauri/` | CI 통과 |
| Rust 린트 | `cargo clippy --workspace --all-targets -- -D warnings` | `src-tauri/` | CI 통과 |
| Rust 형식 | `cargo fmt --all --check` | `src-tauri/` | CI 통과 |
| 바인딩 생성 | `cargo run -p app --bin gen-bindings` | `src-tauri/` | `src/lib/bindings.ts`, `src/lib/action-meta.json` 갱신. CI에서 생성 후 `git diff --exit-code` |
| 빌드 확인 | `pnpm tauri build --no-bundle` | 루트 | CI 통과(설치 파일은 반복 5) |

## 단계
각 단계 = 커밋 하나. 4단계 이후 모든 단계는 push해서 CI 초록을 확인하고, 새 검사 명령이 생기면 같은 커밋에서 CI에 추가한다.

### A. 저장소, 뼈대, CI

1. 저장소 초기화 (`chore: init repository`)
   - 변경 파일: `.gitignore`(node_modules, dist, src-tauri/target, *.log), `.gitattributes`(`* text=auto eol=lf`, `*.woff2 *.png *.ico *.icns binary`), `.editorconfig`, `rust-toolchain.toml`(stable 특정 버전 + rustfmt, clippy), `.nvmrc`(`22`). 기존 `docs/`, `CLAUDE.md`, `.claude/` 포함(D-013).
   - 완료 기준: 첫 커밋 후 `git status`가 깨끗하다. Windows에서 clone 후 `git ls-files --eol docs/spec.md`가 `w/lf`.

2. Tauri 2 + React 18 + Vite 뼈대 (`app: scaffold tauri 2 and react`)
   - 변경 파일: `package.json`(React 18 고정, `@tauri-apps/api`·`@tauri-apps/cli` 2.x, `packageManager` pnpm 정확한 버전, `engines.node` `>=22 <23`, scripts `dev`/`build`/`tauri`), `pnpm-lock.yaml`, `tsconfig.json`(strict, `noUncheckedIndexedAccess`), `vite.config.ts`(고정 포트, `clearScreen: false`), `index.html`, `src/main.tsx`, `src/App.tsx`(앱 이름만).
   - `src-tauri/`(D-018): `Cargo.toml`(`[package] name = "app"` + `[workspace] members = [".", "crates/*"]`, `[workspace.dependencies]`로 버전 한곳 관리), `build.rs`, `tauri.conf.json`(`productName` `hiGit`, `identifier` `dev.higit.desktop`, D-009), `capabilities/default.json`, `icons/`, `src/main.rs`, `src/lib.rs`.
   - `create-tauri-app` 템플릿은 React 19를 넣으므로 18로 내린다(D-004).
   - 완료 기준: `pnpm install && pnpm tauri dev`로 두 OS에서 창이 뜬다. `pnpm build`, `cd src-tauri && cargo build` 성공.
   - Ubuntu 필요 패키지: `libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev build-essential libxdo-dev`. Windows: WebView2 런타임, MSVC 빌드 도구.

3. 하위 크레이트 뼈대 (`chore: add workspace crates`)
   - 변경 파일: `src-tauri/crates/{core,git,hosting,platform,watcher}/{Cargo.toml, src/lib.rs}`, `src-tauri/Cargo.toml`(app이 `higit-core`만 의존).
   - 패키지 이름: `crates/core`는 `higit-core`(D-012), 나머지는 폴더 이름 그대로.
   - 의존: `app → higit-core → git, platform`. `git`, `hosting`, `platform`, `watcher`는 내부 크레이트를 의존하지 않는다.
   - 완료 기준: `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` 통과. `cargo tree -p git -e normal`(그리고 hosting, platform, watcher)에 다른 내부 크레이트가 없다.

4. CI (`ci: build and test on windows and ubuntu`)
   - 앞당긴 이유: Windows 빌드 문제를 초반에 잡는다(이 개발 PC는 Ubuntu).
   - 변경 파일: `.github/workflows/ci.yml`, `README.md`(개발 환경 준비: 두 OS 필수 패키지, Node/pnpm/Rust 버전).
   - 매트릭스: `ubuntu-22.04`, `windows-latest`. 공통 준비: checkout → `pnpm/action-setup`(`packageManager`) → `actions/setup-node`(`.nvmrc`) → Rust(`rust-toolchain.toml`) → 캐시(`Swatinem/rust-cache`, pnpm store) → `pnpm install --frozen-lockfile`. Ubuntu만 apt 패키지 설치. `git --version`을 로그에 남긴다.
   - 이 단계의 검사: `cargo fmt --all --check` → `cargo clippy --workspace --all-targets -- -D warnings` → `cargo test --workspace` → `pnpm build` → `pnpm tauri build --no-bundle`.
   - 이후 단계가 더할 검사: `pnpm lint`·`pnpm typecheck`·`pnpm test`(5단계), gen-bindings 최신 여부(15단계).
   - 완료 기준: 비공개 원격에 push해 두 작업 모두 초록.
   - 위험: Windows 경로 길이(260자) 제한에 걸리면 `CARGO_TARGET_DIR`을 짧게.

5. 프론트 린트·테스트 도구 (`chore: add frontend lint and test tooling`)
   - 변경 파일: `eslint.config.js`(flat, typescript-eslint, react-hooks), `.stylelintrc.json`, `vitest.config.ts`(jsdom), `src/test/setup.ts`, `package.json` scripts(`lint`, `typecheck`, `test`), `ci.yml`에 세 검사 추가.
   - 규칙을 도구로 강제:
     - ESLint `no-restricted-imports`: `@tauri-apps/api/core`(`invoke`)는 `src/lib/` 밖에서 금지. 화면이 core를 우회하지 못한다(D-010).
     - Stylelint: hex·이름 색·`rgb(` 등 금지, `src/styles/tokens.css`만 예외. 그림자 값도 토큰으로.
   - 완료 기준: 세 명령 통과(샘플 테스트 1개). 임시로 `color: #fff`를 넣으면 `pnpm lint` 실패 확인 후 되돌림. CI 초록.

### B. 디자인 토큰, glossary, Term

6. 디자인 토큰과 글꼴 (`ui: add design tokens and fonts`)
   - 변경 파일: `src/styles/tokens.css`(design.md 색 라이트/다크, 간격, 모서리, 글자 크기, 굵기, 줄 간격, 그림자, 움직임, `--graph-1`~`--graph-6`), `src/styles/base.css`(리셋, 글꼴, 얇은 스크롤바, `prefers-reduced-motion`), `src/styles/fonts.css`, `THIRD_PARTY_NOTICES.md`.
   - 글꼴(D-014):
     - Pretendard: `pretendard` 패키지의 가변 글꼴 woff2 하나만. `fonts.css`에서 `@font-face { font-family: "Pretendard"; src: url("pretendard/dist/web/variable/woff2/PretendardVariable.woff2") format("woff2-variations"); font-weight: 45 920; }`. 패키지의 전체 CSS(`pretendard.css`)는 import하지 않는다.
     - JetBrains Mono: `@fontsource/jetbrains-mono/latin-400.css`만 import.
     - `THIRD_PARTY_NOTICES.md`에 두 글꼴의 저작권 표시와 OFL 1.1 원문.
   - 다크 모드: `prefers-color-scheme` + `:root[data-theme]` 선택자 자리만.
   - 완료 기준: `pnpm lint` 통과. `pnpm build` 후 `dist/assets`에 글꼴 파일이 Pretendard 1개, JetBrains Mono 1개(latin 400)뿐인지 확인. 두 OS에서 한글·영문·모노 글꼴과 얇은 스크롤바를 눈으로 비교.

7. glossary 데이터 (`glossary: add ko and en terms`)
   - 변경 파일: `src/glossary/types.ts`(`TermId` 유니온, `TermEntry { friendly, gitName, command: string | null, description }`), `ko.ts`(spec.md 표 29행 그대로), `en.ts`(같은 id의 영어 문구, D-015), `index.ts`(조회 함수, 언어 인자는 받되 기본 `ko`), `spec-sync.test.ts`.
   - 테스트: spec.md "기능 목록과 용어표"를 파싱해 `ko.ts`와 id·쉬운 이름·원래 이름·명령·설명이 완전히 일치. `en.ts`는 id 집합이 같고, `gitName`·`command`가 ko와 같으며 빈 문자열이 없음. 파서는 `\r?\n`으로 나눈다.
   - 위험도는 glossary가 아니라 `ActionMeta.risk`가 갖는다.
   - 완료 기준: `pnpm test` 통과. spec.md 설명 한 글자를 바꾸면 실패함을 1회 확인하고 되돌림.

8. `Term` 부품과 툴팁 (`ui: add Term and Tooltip`)
   - 변경 파일: `src/ui/Tooltip.tsx` + CSS(Radix Tooltip, 지연 400ms, `side="top"`), `src/ui/Term.tsx` + CSS, `src/state/uiStore.ts`(Zustand: `termDisplay: "friendly" | "git"`), `src/ui/Term.test.tsx`.
   - 동작: 본문에 쉬운 이름, 툴팁 세 줄(원래 이름 600 / 명령 모노 / 설명 muted). `git` 모드면 서로 바뀜. 명령이 `null`이면 둘째 줄 생략. `tabIndex=0`으로 키보드 포커스에도 뜸. `id`는 `TermId` 타입.
   - 테스트: 호버(가짜 타이머 400ms)·포커스로 툴팁 표시, `git` 모드 전환, 명령 없는 용어(`conflict`)는 두 줄.
   - 완료 기준: `pnpm test`, `pnpm lint` 통과.

### C. 백엔드

9. platform: git 찾기, 경로 정규화, 자식 프로세스 설정 (`platform: find git and normalize paths`)
   - 변경 파일: `src-tauri/crates/platform/src/{lib.rs, git_exe.rs, path.rs, process.rs}`(`which`, `dunce`).
   - `find_git()`: PATH, Windows는 실패 시 `C:\Program Files\Git\cmd\git.exe`. `normalize_repo_path()`: Windows `\\?\` 제거(`dunce`). `configure_child(&mut Command)`: Windows에서 `CREATE_NO_WINDOW`.
   - `git`은 `platform`을 모르므로 core가 경로와 `configure_child`를 받아 `CliBackend`에 넘긴다.
   - 테스트: 두 OS에서 `find_git()`이 `Some`, Windows에서 `\\?\` 없음, 한글 경로 왕복.
   - 완료 기준: `cargo test -p platform` 통과. `rg "cfg\((windows|target_os)" src-tauri --glob '!crates/platform/**'`가 비어 있다.

10. git: `GitBackend` 트레이트와 CLI 실행기 (`git: add GitBackend trait and cli runner`)
    - 변경 파일: `src-tauri/crates/git/src/{lib.rs, backend.rs, cli/mod.rs, cli/runner.rs, types.rs, error.rs}`(`thiserror`, dev: `tempfile`). serde·specta는 의존하지 않는다(D-019).
    - 트레이트: architecture.md에서 "반복 0"으로 표시된 `version`, `discover`, `status`, `log`, `run`만 선언한다(`run`은 12a단계에서 구현). 나머지는 표시된 반복에서 추가한다(H-6).
    - 모든 호출에 `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, `-c core.quotepath=off`, `-c color.ui=never`. 출력은 바이트로 받아 UTF-8 디코딩.
    - 오류: `GitError { kind: NotFound | NotARepo | Failed { code, stderr } }`. 원문 stderr 보존.
    - 테스트: `version()` ≥ 2.30, `discover()`가 하위 폴더에서 최상위 찾음, 저장소 아닌 폴더에서 `None`. 테스트 저장소는 `-c user.name -c user.email -c core.autocrlf=false -c init.defaultBranch=main`으로 만든다.
    - 완료 기준: `cargo test -p git` 두 OS(CI) 통과.

11. git: status 파서 (`git: parse status porcelain v2`)
    - 변경 파일: `cli/status.rs`, `types.rs`(`Status { branch, entries }`, `StatusEntry { path, orig_path, index, worktree, kind }`).
    - 명령: `git status --porcelain=v2 -z --branch --untracked-files=all`. 레코드 `1`, `2`, `u`, `?`, `#` 헤더.
    - 테스트: 새 파일, 수정, 삭제, stage된 수정, 이름 바꿈, 충돌, 한글·공백·특수문자 파일명, 하위 폴더, 저장 없는 저장소.
    - 완료 기준: `cargo test -p git` 통과.

12. git: log 파서 (`git: parse log`)
    - 변경 파일: `cli/log.rs`, `types.rs`(`Commit { oid, short_oid, parents, author_name, author_email, author_time, summary }`, `LogQuery { skip, limit, rev }`).
    - 명령: `git log -z --format=%H%x00%h%x00%P%x00%an%x00%ae%x00%at%x00%s --skip --max-count`. 빈 저장소는 `rev-parse --verify -q HEAD`로 먼저 확인.
    - `LogQuery`는 나중에 확대·축소 단계 필드를 더할 수 있게 구조체로 둔다(D-017 대비, 지금은 추가하지 않음).
    - 테스트: 커밋 3개, 병합 커밋, 한글 메시지·작성자, 빈 저장소, skip/limit 경계.
    - 완료 기준: `cargo test -p git` 통과.

12a. git: 쓰기 실행과 init (`git: add run with init op`)
    - 변경 파일: `git/src/{backend.rs, op.rs, cli/run.rs}`. `GitOp { Init }`, `OpOutput { stdout, stderr }`.
    - `run(target, GitOp::Init)` → `git init` (대상 폴더에서). 기본 갈래 이름은 사용자 git 설정(`init.defaultBranch`)을 따르고 앱이 정하지 않는다.
    - 이미 저장소인 폴더는 core가 막는다(`available`). git 쪽은 그대로 실행만.
    - 테스트: 빈 임시 폴더에서 init 후 `discover()`가 그 폴더를 돌려줌, 한글 이름 폴더에서 init, 쓰기 권한 없는 폴더에서 `Failed`(Ubuntu만, Windows는 건너뜀).
    - 완료 기준: `cargo test -p git` 두 OS(CI) 통과.

13. core: Action, 레지스트리, 실행기, init 액션 (`core: add action registry, executor and init action`)
    - 변경 파일: `src-tauri/crates/core/src/{lib.rs, action.rs, registry.rs, executor.rs, state.rs, error.rs, actions/mod.rs, actions/init.rs}`, `tests/executor.rs`, `tests/init.rs`.
    - `Action` 트레이트와 `ActionMeta { id, glossary_id, risk, input_schema }`(architecture.md). `Risk`는 `Safe < Normal < Caution < Danger`. `Registry`는 id 중복 금지.
    - `Executor::preview` → `Preview` + 1회용 미리보기 토큰. `Executor::execute`: `Safe`/`Normal`은 바로 실행, `Caution` 이상은 토큰 검사 후 `NotYetSupported`로 거부(D-011).
    - `RepoState { folder, repo: Option<RepoPath> }`.
    - `init` 액션: `id = "init"`, `glossary_id = "init"`, `risk = Safe`, 입력 없음. `available`: `repo`가 `None`일 때만, 아니면 "이미 Git으로 관리하는 폴더예요" 코드(`AlreadyRepo`). `execute`: `GitBackend::run(folder, GitOp::Init)` 후 `discover`로 새 `RepoPath`를 결과에 담는다. 이 액션이 반복 0의 유일한 등록 액션이다.
    - 테스트(가짜 액션·가짜 백엔드): id 중복 오류, Safe 즉시 실행, Caution·Danger 거부(토큰 유무 모두), 만료·불일치 토큰 거부, init의 `available`(저장소 아님 → 가능, 저장소 → 불가), init 실행 시 `GitOp::Init` 호출과 결과. 실제 git으로 임시 폴더 init 통합 테스트 1개.
    - 완료 기준: `cargo test -p higit-core` 통과.

14. core: Query, view 타입, 시작 처리 (`core: add queries and view types`)
    - 변경 파일: `core/src/{query.rs, view.rs, bootstrap.rs}`, `tests/query.rs`(가짜 `GitBackend`), `Cargo.toml`(`serde`, `specta` 정확한 RC 버전).
    - `core::view`: 화면에 보내는 타입만 둔다(`Status`, `StatusEntry`, `LogQuery`, `LogPage`, `CommitRow`, `RepoInfo`, `AppStatus`, `ErrorDto { code, detail }`). serde·specta derive는 이 모듈에만(D-019). git 타입에서 옮기는 `From` 구현도 여기.
    - `core::query::Queries`: `status(repo)`, `log(repo, LogQuery)`. 실행기·레지스트리를 거치지 않는다.
    - `bootstrap::init() -> Result<Core>`: `platform::find_git()` → `CliBackend::new(path, platform::configure_child)` → `version()` ≥ 2.30 → 레지스트리에 `init` 등록 → `Core { registry, executor, queries }`. 오류 `GitMissing`, `GitTooOld { found }`. `Core::open_folder(path) -> RepoState`: 정규화 → `discover`. 저장소가 아니어도 오류가 아니라 `repo: None`으로 돌려준다(화면이 "관리 시작"을 보여 주도록).
    - view 타입에 `FolderInfo { folder, repo: Option<RepoInfo> }`, `ActionResult`(init은 새 `RepoInfo`)를 더한다.
    - 오류는 코드 + 원문 `detail`만 내보내고, 한국어 문구는 프론트가 코드로 고른다.
    - 테스트: 가짜 백엔드로 status·log 변환, git 2.29에서 `GitTooOld`, 저장소 아닌 경로에서 `repo: None`.
    - 완료 기준: `cargo test -p higit-core`, clippy 통과. `rg "GitBackend|git::" src-tauri/src`가 비어 있다(app은 core만 씀).

15. app: Tauri 명령과 타입 바인딩 (`app: add commands and specta bindings`)
    - 변경 파일: `src-tauri/src/{lib.rs, commands.rs, bindings.rs, bin/gen-bindings.rs}`, `src-tauri/Cargo.toml`(`tauri-specta`, `specta-typescript` 정확한 RC 버전 `=`, `tauri-plugin-dialog`), `capabilities/default.json`(`dialog:allow-open`), 생성물 `src/lib/bindings.ts`, `src/lib/action-meta.json`, 손으로 쓰는 `src/lib/api.ts`, `src/lib/registry-glossary.test.ts`, `ci.yml`에 gen-bindings 최신 여부 검사 추가.
    - 명령(`<영역>_<동사>`): `app_status`, `repo_open(path) -> FolderInfo`, `action_list`, `action_preview`, `action_execute(folder, id, input, preview_token?) -> ActionResult`, `query_status(repo) -> Status`, `query_log(repo, LogQuery) -> LogPage`. 명령은 얇게: 입력 검사 → core 호출.
    - `bindings.rs`(D-019): 명령 목록, specta builder, TS 내보내기, `action-meta.json` 내보내기를 이 파일 하나에 모은다. `lib.rs`의 앱 실행과 `gen-bindings`가 이 파일의 함수만 부른다. ts-rs로 바꿀 때 고칠 곳은 이 파일과 `core::view`뿐이어야 한다.
    - `registry-glossary.test.ts`: `action-meta.json`의 모든 `glossary_id`가 `ko.ts`에 있고 위험도가 spec.md 표와 같은지(반복 0은 `init` 1개, `safe`).
    - 완료 기준: `gen-bindings` 두 번 연속 실행해도 `git diff`가 비어 있다. `pnpm typecheck`, `pnpm test`, `cargo test --workspace`, CI 초록.
    - 생성 파일은 LF로 쓴다.

### D. 화면

16. 홈과 작업 공간 화면 (`screens: add home and workspace`)
    - 변경 파일: `src/App.tsx`(홈 ↔ 작업 공간, Zustand), `src/state/repoStore.ts`, `src/lib/queries.ts`(TanStack Query: `useStatus`, `useLog` — 내부는 `api.queryStatus`, `api.queryLog`), `src/lib/dialog.ts`, `src/screens/Home/*`, `src/screens/Workspace/{Workspace.tsx, ChangedFiles.tsx, *.module.css}`, `src/features/history/{model.ts, HistoryList.tsx, HistoryList.module.css}`, `src/ui/{Button, ListRow, EmptyState, ErrorNotice}.tsx` + CSS, `src/messages/{ko.ts, en.ts}`(오류 코드 → 문구), 테스트.
    - 홈: 제목 + "폴더 열기" 버튼. `open({ directory: true })` → `repo_open`.
    - 작업 공간: `<Term id="status" />` 목록(경로 모노, 상태 색 토큰, 행 32px)과 `<Term id="log" />` 목록(요약, 작성자, 상대 시각, 짧은 해시). 창 포커스 시 다시 불러오기 + "새로 고침".
    - 저장 기록 그리기 대비(architecture.md "저장 기록 그리기 계층"): `features/history/model.ts`가 `LogPage`를 `HistoryRow[]`로 바꾸고, `HistoryList`는 `HistoryRow[]`만 받는다. `layout/`, `render/`는 만들지 않는다(노선도 보기 때 추가).
    - 저장소가 아닌 폴더: 안내 "이 폴더는 아직 Git으로 관리하지 않는 폴더예요." 옆에 주 버튼 `<Term id="init" />`("관리 시작"). 버튼은 `action_list`의 `init` 메타로 만들고 `action_execute(folder, "init")`를 부른다(safe라 미리보기 없음). 성공하면 결과의 `RepoInfo`로 작업 공간에 들어가 빈 상태를 보여 준다. 실패하면 `ErrorNotice`.
    - 빈 상태와 오류: "마지막 저장 이후 바뀐 파일이 없어요.", "아직 저장한 기록이 없어요.", GitMissing, GitTooOld, init 실패. 원문은 "자세히 보기".
    - 테스트: api 모킹으로 목록·빈 상태·오류 문구·"자세히 보기" 펼침, 저장소 아닌 폴더에서 "관리 시작" 표시와 클릭 시 `action_execute` 호출·작업 공간 전환, `model.ts` 변환 단위 테스트.
    - 완료 기준: `pnpm test`, `pnpm lint` 통과. 두 OS 수동 확인: 이 저장소 열기, 한글 파일명, 빈 저장소, 일반 폴더에서 "관리 시작" → 작업 공간, 툴팁 호버·Tab, 라이트/다크.
    - 커지면 16a(홈 + 열기 + 오류 + 관리 시작), 16b(바뀐 파일), 16c(저장 기록)로 나눈다.

### E. 마무리

17. `CLAUDE.md` 명령 절 확정 (`docs: confirm commands for iteration 0`)
    - 변경 파일: `CLAUDE.md` "명령" 절(위 표로 교체, "(반복 0에서 확정 후 채운다)" 삭제), `.claude/agents/code-analyst.md`의 `cargo tree` 표기 점검.
    - 완료 기준: 새로 clone한 두 OS에서 명령을 차례로 실행해 모두 성공. `code-analyst` 규칙 점검 위반 0건.

## 용어 변경
- spec.md 용어표: 변경 없음.
- glossary: `ko.ts`, `en.ts` 신규(29개 항목).
- 화면에서 `<Term>`으로 쓰는 id: `status`, `log`, `init`(빈 상태 문구에서 `commit`도 가능).

## 위험과 열린 질문
- tauri-specta 2가 RC다. 정확한 버전 고정, 결정적 출력 확인, 교체 범위를 `bindings.rs`·`core::view`로 한정(D-019).
- 실제 액션은 `init`(safe) 하나라 실행기의 caution 이상 경로는 가짜 액션 테스트로만 검증된다.
- Pretendard 가변 글꼴은 한글 전체를 담아 woff2 하나가 수 MB다. 앱 크기 목표(D-001의 5~15MB) 안인지 6단계 빌드에서 확인한다.
- 화면 확인(글꼴, 스크롤바, 툴팁)은 CI가 못 한다. Windows 실기기 확인이 필요하다.
- 성능 기준(1만 개 저장 300ms)은 범위 밖. log는 200개 고정 조회만.
- 앱 이름 `hiGit`이 다른 제품과 겹치는지 공개 전에 확인(spec.md 열린 질문).

## 사람 확인 결과
모두 결정됨(2026-10-15).
- H-1 caution 이상 거부 → D-011 확정
- H-2 조회는 Query로 분리 → D-010 확정(D-005 대체)
- H-3 `higit-core` → D-012 확정
- H-4 도구 버전, `.nvmrc`, LF → D-013 확정
- H-5 이름 `hiGit` → D-009 확정
- H-6 `discover`·`version` 추가, 나머지 5개는 architecture.md에 반복 표시 → architecture.md 반영
- H-7 → 이후 변경: init을 반복 0의 첫 실제 Action으로(안내 문구 옆 "관리 시작" 버튼). roadmap 반영
- H-8 Pretendard 가변 1개, JetBrains Mono latin 400만 → D-014 확정
- H-9 명령 표기 → 17단계
- H-10 RC 고정, 바인딩 코드 한 파일 → D-019 확정
- H-11 비공개 원격, CI를 4단계로 앞당김
- H-12 `en.ts` → D-015 확정
- 추가: src-tauri를 app 크레이트 겸 루트로 → D-018 확정
