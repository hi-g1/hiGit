# 구조 설계 (Architecture)

목표는 **기능을 계속 붙여도 구조가 무너지지 않는 것**이다. 핵심은 세 가지다.
1. Git 처리는 `GitBackend` 트레이트 뒤에 숨긴다(D-003).
2. 상태를 바꾸는 기능은 `Action`으로 등록하고 화면은 레지스트리에서 만든다. 조회는 core의 Query로 제공한다(D-010).
3. 화면과 백엔드는 정해진 명령(command)과 이벤트로만 대화한다.

## 계층

```
┌──────────────────────── 프론트엔드 (React, src/) ────────────────────────┐
│ screens/      시작 화면, 작업 공간, 버전 기록, 기록 정리, 충돌 해결, 검토 요청, 설정 │
│ features/     액션별 UI 조각 (미리보기 패널, 확인 대화상자)              │
│ ui/           디자인 시스템 부품 (Button, Term, Tooltip, Dialog …)      │
│ lib/api.ts    백엔드 명령 호출과 타입 (자동 생성 바인딩)                │
└───────────────────────────────▲──────────────────────────────────────────┘
                     Tauri command / event (타입 공유)
┌───────────────────────────────┴── 백엔드 (Rust, src-tauri/) ────────────┐
│ app        Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환  │
│ core       Action 레지스트리, Query, 프로젝트 열기·최근 목록, 복구, view 타입 │
│ git        GitBackend 트레이트 + CliBackend 구현, 출력 파서              │
│ hosting    HostingProvider 트레이트 + GitHub 구현 (PR, 인증)            │
│ platform   OS별 코드: 우클릭 메뉴 등록, git 실행 파일 찾기, 키체인      │
│ watcher    파일 변경 감시(.gitignore 거름). core가 이벤트로 바꾼다     │
└──────────────────────────────────────────────────────────────────────────┘
```

의존 방향은 위에서 아래로만. `git`, `hosting`, `platform`, `watcher`는 서로 모른다. `core`만 이들을 조합한다.

## 폴더 구조

```
/
├─ CLAUDE.md
├─ docs/                     decision.md, spec.md, architecture.md, design.md, roadmap.md, plans/, research/
├─ .claude/agents/           planner.md, code-analyst.md
├─ src/                      프론트엔드
│  ├─ screens/
│  ├─ features/<action-id>/
│  ├─ ui/
│  ├─ glossary/              ko.ts, en.ts  (용어 원본은 docs/spec.md 표)
│  ├─ lib/                   api.ts(백엔드 호출, invoke는 lib/ 안에서만), events.ts(이벤트 구독), queries.ts(TanStack Query), useRepoWatch.ts, dialog.ts, bindings.ts(생성물)
│  ├─ messages/              ko.ts, en.ts 화면 문구(copy-rules.test.ts가 문구 규칙 확인)
│  └─ styles/tokens.css      docs/design.md 토큰
└─ src-tauri/               app 크레이트 겸 워크스페이스 루트(D-018)
   ├─ Cargo.toml             [package] app + [workspace] members = ["crates/*"]
   ├─ tauri.conf.json, build.rs, capabilities/, icons/
   ├─ src/                   app: main.rs, lib.rs, commands.rs, bindings.rs(D-019), bin/gen-bindings.rs
   └─ crates/
      ├─ core/              패키지 이름 higit-core(D-012)
      ├─ git/
      ├─ hosting/
      ├─ platform/
      └─ watcher/
```

Rust 크레이트를 나누면 컴파일 경계가 생겨 의존 방향이 강제되고, `git`과 `core`는 Tauri 없이 테스트할 수 있다.

## 핵심 인터페이스

### GitBackend (crates/git)

```rust
pub trait GitBackend: Send + Sync {
    fn version(&self) -> Result<GitVersion>;                          // 반복 0. 2.30 이상 확인(D-003). 2.38은 D-036 제안
    fn discover(&self, path: &Path) -> Result<Option<RepoPath>>;      // 반복 0. 고른 폴더의 저장소 최상위 찾기
    fn status(&self, repo: &RepoPath) -> Result<Status>;              // 반복 0
    fn log(&self, repo: &RepoPath, query: &LogQuery) -> Result<Vec<Commit>>; // 반복 0
    fn excludes_file(&self, repo: &RepoPath) -> Result<Option<PathBuf>>; // 반복 1a. 전역 무시 규칙 파일(감시용)
    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff>; // 반복 1a. 마지막 버전과 지금을 합쳐 비교
    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>>;       // 반복 1b에서 구현
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;   // 반복 1a까지 Init, Stage, Unstage, Commit. 쓰기 작업은 전부 여기로
    fn resolve_ref(&self, repo: &RepoPath, r: &str) -> Result<Oid>;   // 반복 2에서 구현(백업 참조)
    fn update_ref(&self, repo: &RepoPath, name: &str, oid: Oid) -> Result<()>; // 반복 2에서 구현(백업 참조)
}
```

- 이 트레이트는 목표 설계다. 코드에는 해당 반복에 필요한 메서드만 구현해 넣는다.
- `LogQuery`는 나중에 확대·축소 단계(버전만 / 합치기까지 / 전체)를 받을 자리를 둔다(D-017 제안).

- `GitOp`는 enum(Init, Commit, Merge, Rebase, Reset …). 새 쓰기 작업은 enum 변형 하나 + `CliBackend` 매핑 하나를 추가한다.
- `GitOp::Init`은 아직 저장소가 아닌 폴더에서 실행하므로 `repo` 인자는 정규화한 대상 폴더다.
- CLI 출력은 기계용 형식만 파싱한다: `status --porcelain=v2 -z`, `log --format=%x1e…%x00…`, unified diff, `for-each-ref --format`. 실패 종류는 `crates/git/src/failure.rs` 한 곳에서만 영어 원문으로 가린다(D-021).
- 경로 목록은 `--pathspec-from-file=- --pathspec-file-nul`로 stdin에 넘기고 `--literal-pathspecs`를 붙인다.
- 모든 git 호출에 `LC_ALL=C`(그리고 `LANGUAGE` 지움), `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, `GIT_OPTIONAL_LOCKS=0`, `-c core.quotepath=off -c color.ui=never`를 준다. `GIT_OPTIONAL_LOCKS=0`은 조회가 index를 쓰지 않게 해 파일 감시의 되먹임 고리를 막는다. OS별 설정(Windows 콘솔 창 숨기기)은 platform이 먼저 붙이고 위 값이 마지막에 이긴다.

#### git 실행 규칙 (D-035)
- 인자는 배열로 넘기고 셸을 거치지 않는다. 문자열을 이어 붙여 명령줄을 만들지 않는다.
- 리비전 앞에는 `--end-of-options`, 경로 앞에는 `--`. 경로 목록은 위처럼 stdin으로. 리비전은 `crates/git/src/cli/args.rs`의 `push_revs`로만 붙인다. 이 표시는 git 2.34 이상에서 확인한 명령에만 넣고, `checkout`·`reset`·`check-ref-format`에는 넣지 않는다(2.34.1에서 오동작). 넣지 못하는 경우는 core의 이름 검사가 막는다.
- 공통 환경에 `PAGER=cat`, `GIT_EDITOR=:`(편집기를 띄우지 않는 특별 값), `GIT_ALLOW_PROTOCOL=https:ssh:git`도 둔다. 상위 프로세스의 `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` 등 저장소 위치 변수는 지운다(D-052). 로컬 경로 원격은 테스트에서만 `CliBackend::allow_local_remotes_for_tests()`로 켠다. 조회는 `--no-ext-diff`, `--no-textconv`.
- 새 브랜치·태그 이름은 `check-ref-format`으로 검사하고 `-`로 시작하면 막는다. 원격 주소는 `https://`, `ssh://`, `git@host:path`만.
- git 실행 파일은 시작할 때 절대 경로로 한 번 찾아 그 경로만 쓴다.
- 믿지 않기로 한 저장소(D-039)는 모든 호출에 실행 설정을 끄는 `-c` 덮어쓰기를 붙인다.

### Action (crates/core) — 상태를 바꾸는 기능

```rust
pub trait Action: Send + Sync {
    fn meta(&self) -> &ActionMeta;                       // id, risk, glossary_id, 단축키
    fn available(&self, state: &RepoState) -> Availability; // 지금 쓸 수 있나, 아니면 쉬운 이유
    fn preview(&self, ctx: &ExecCtx, input: &ActionInput) -> Result<Preview>; // 무엇이 바뀌나
    fn execute(&self, ctx: &ExecCtx, input: &ActionInput) -> Result<Outcome>; // ExecCtx = { git, state }
}

pub struct ActionMeta {
    pub id: &'static str,          // "commit"
    pub glossary_id: &'static str, // 용어표 id. 쉬운 이름·원래 이름·설명은 여기서 온다
    pub risk: Risk,                // Safe | Normal | Caution | Danger
    pub undo: UndoKind,            // 되돌리는 방법(D-042). Irreversible이면 실행 전 확인(D-043)
}

// core::view (D-020). 화면은 태그 붙은 enum으로 액션과 입력을 함께 보낸다.
// TS: { id: "init" } | { id: "commit"; input: CommitInput } …
#[serde(tag = "id", content = "input", rename_all = "kebab-case")]
pub enum ActionInput { Init, /* Commit(CommitInput), … */ }
```

`RepoState`는 열린 폴더와 저장소 여부를 함께 담는다(`folder`, `repo: Option<RepoPath>`). 그래서 저장소가 아닌 폴더에서만 쓸 수 있는 init도 같은 `available`로 판단한다.

`core`의 실행기(`Executor`)가 위험도와 `undo`에 따라 공통 흐름을 강제한다. 액션 각각이 안전장치를 따로 구현하지 않는다(D-043).

```
execute(action, input):
  if undo == Irreversible: 프론트가 확인을 받았는지 확인 (확인 토큰)
  if risk >= Caution: 백업 참조 생성  refs/higit/backup/<timestamp>/<HEAD 이름>
  before = 되돌리는 데 필요한 상태 (HEAD, 브랜치, 맡겨 둔 stash oid 등. undo 종류가 정함)
  result = action.execute()
  if undo != Irreversible: journal에 {action, input, before, 백업 참조, 시각} 추가 → 화면에 되돌리기 띄움
  (화면 갱신은 파일 변경 감시가 맡는다. 실행기가 이벤트를 보내지 않는다)
```

### Query (crates/core) — 조회 (D-010)

```rust
pub struct Queries { /* GitBackend를 품는다 */ }
impl Queries {
    pub fn status(&self, repo: &RepoPath) -> Result<view::Status>;            // 반복 0
    pub fn log(&self, repo: &RepoPath, q: view::LogQuery) -> Result<view::LogPage>; // 반복 0
    pub fn diff(&self, repo: &Path, r: &view::DiffRequest) -> Result<view::FileDiffView>; // 반복 1a. 경로는 저장소 안만
    // branches 등은 해당 반복에서 추가
}
```

- 조회는 미리보기·위험도·백업이 없으므로 실행기를 거치지 않는다. 레지스트리에도 등록하지 않는다.
- 결과는 `core::view` 타입으로 돌려준다. `crates/git` 타입은 core 안에서 view 타입으로 옮긴다(D-019).
- 화면은 Query도 core를 통해서만 부른다. `app`의 `query_*` 명령이 `Queries`를 호출한다.
- 용어 호버는 glossary id로 동작하므로 조회 화면도 `<Term id="status" />`처럼 쓴다.

### 파일 변경 감시 (crates/watcher, core::watch)
- `watcher::RepoWatcher`가 저장소 루트를 재귀 감시한다(notify, 300ms 디바운스). 다른 내부 크레이트를 모른다.
- 작업 폴더 변경과 `.git` 안의 의미 있는 파일(HEAD, index, refs/*, packed-refs, FETCH_HEAD …)만 알린다. `.gitignore`(하위 폴더 포함)·`.git/info/exclude`·`core.excludesFile` 대상, `.git/objects`·잠금 파일, 읽기 알림, 폴더 자체의 수정 알림은 거른다.
- 감시 오류나 저장소 폴더 삭제는 한 번만 "멈춤"으로 알린다. 폴더 삭제는 0.5초마다 따로 확인한다(알림을 주지 않는 OS가 있다).
- `Core::watch(repo, sink)`가 한 번에 한 저장소만 감시하고, 알림을 `RepoChanged`·`WatchStatus` view 타입으로 바꾼다. `app`이 Tauri 이벤트로 내보낸다.
- 화면(`useRepoWatch`)은 작업 폴더 변경이면 변경 사항·비교, `.git` 변경이면 변경 사항·버전 기록·비교를 다시 읽는다(1초에 한 번). 감시가 꺼지면 안내 줄과 "다시 읽기"를 보여주고 창 포커스 새로 고침으로 대신한다(D-025).
- 알려진 제약: 무시된 폴더에도 감시 등록은 된다(Ubuntu inotify 한도에 걸리면 멈춤으로 떨어짐). `core.excludesFile`이 없을 때 git의 기본 전역 무시 파일은 읽지 않는다.

### 되돌리기와 UndoJournal (crates/core, D-042)
- 앱이 한 작업 목록을 `.git/higit/journal.jsonl`에 남긴다(저장소마다 따로, 저장소를 옮겨도 따라감).
- 레지스트리의 Action마다 `undo`를 정한다. 되돌리기는 별도 Action을 새로 만들지 않고 journal 항목과 그 Action의 `undo`로 실행기가 처리한다.

```rust
pub enum UndoKind {
    RestoreRef,      // HEAD·브랜치를 before의 oid로 되돌림. 고친 파일은 그대로(버전 저장, 이 버전으로 돌아가기)
    RestoreIndex,    // index를 before의 tree로(이번 버전에 넣기·빼기)
    SwitchBack,      // 이전 브랜치로 갈아타고 맡겨 둔 stash를 다시 꺼냄(브랜치 갈아타기)
    DeleteCreated,   // 만든 ref를 지움(새 브랜치, 태그)
    Inverse,         // 기록을 바꾸지 않는 반대 동작(원격에 올린 뒤의 되돌리기, revert)
    Irreversible,    // 되돌릴 수 없음. 실행 전 확인(덮어쓰며 올리기, 고친 것 버리기)
}
```

- 되돌리기(Ctrl+Z, 화면 아래 "되돌리기")는 journal 마지막 항목을 꺼내 `undo` 종류대로 되돌리고 항목에 `undone`을 적는다. 다시 하기(Ctrl+Shift+Z)는 `undone` 항목의 input으로 Action을 다시 실행한다.
- 되돌리기 전에 HEAD가 journal의 결과와 같은지 확인한다. 다르면(앱 밖에서 git을 썼음) 되돌리지 않고 "복구" 화면으로 안내한다.
- 되돌리는 동작도 실행기를 거쳐 백업 ref를 남긴다. 그래서 되돌리기를 잘못 눌러도 살릴 수 있다.
- 원격에 올린 버전을 바꾸는 되돌리기는 `RestoreRef` 대신 `Inverse`로 떨어지거나 막는다(올린 기록을 몰래 바꾸지 않음).
- 브랜치별로 맡겨 둔 고친 파일은 `refs/higit/parked/<브랜치>`의 stash 커밋이다(D-047). 일반 stash 목록과 섞지 않는다.
- "복구" 화면 = journal + `git reflog` + `refs/higit/backup/*`를 시간순으로 합쳐 보여준다.

### HostingProvider (crates/hosting)

```rust
pub trait HostingProvider {
    fn detect(remote_url: &str) -> bool;
    fn auth_status(&self) -> AuthState;
    fn list_prs(&self, repo: &RemoteRepo, filter: PrFilter) -> Result<Vec<PullRequest>>;
    fn create_pr(&self, repo: &RemoteRepo, req: NewPr) -> Result<PullRequest>;
}
```
GitHub 먼저. 토큰은 `platform`의 키체인(Windows 자격 증명 관리자, Ubuntu Secret Service)에 저장한다.

## 보안 (D-034)
| 공격자나 사고 | 노리는 것 | 주된 대책 |
| --- | --- | --- |
| 악성 저장소를 복제하거나 받은 zip을 연다 | 저장소 설정, 서브모듈, 심볼릭 링크로 명령 실행 | 최소 Git 버전(D-036), 신뢰 확인(D-039), 위험한 설정 끄기 |
| 이상한 브랜치 이름이나 파일 이름 | git 옵션으로 해석되게 만들기 | `--end-of-options`와 `--`, 이름 검사(D-035) |
| 저장소 파일 내용에 섞인 HTML이나 스크립트 | 웹뷰에서 실행되어 IPC 호출 | 이스케이프, CSP, 좁은 capability(D-037) |
| 같은 PC의 다른 사용자나 악성 프로그램 | 토큰, 설정 파일 | OS 키체인, 평문 저장 금지(D-040) |
| 오염된 의존성 | 빌드 결과물에 코드 심기 | lockfile, cargo-deny, 설치 스크립트 제한(D-038) |
| 사용자의 실수 | 비밀 파일을 버전에 넣어 올리기 | 버전 저장 전 경고(D-040) |
| 중간에 끊기는 작업 | 저장소 깨짐, index.lock 남음 | 시간 제한, 취소, 백업 ref, journal(D-041) |

- 새 Action과 새 IPC 명령을 만들 때 위 표의 해당 줄을 확인한다. 점검 항목은 `.claude/agents/code-analyst.md` "보안 점검".
- 웹뷰가 부를 수 있는 앱 명령은 `build.rs`의 `COMMANDS`와 `capabilities/default.json`에 적은 것뿐이다. 새 명령은 `src/commands.rs`, `src/bindings.rs`와 함께 네 곳에 넣고, `src-tauri/tests/security_config.rs`가 네 곳과 CSP 문자열을 확인한다. dev 서버에는 CSP가 없으므로 CSP 확인은 릴리스 실행 파일로 한다.
- 조회·액션·감시는 `Core::open_project`를 거친 경로만 받는다(`core::access::OpenedRepos`, 아니면 `NotOpened`). 시작 화면 상태 요약은 최근 목록의 경로도 받는다. 신뢰 확인(D-039)이 생기면 신뢰한 뒤에만 열린 것으로 기록한다.
- 입력 상한: 경로 4096바이트, 경로 목록 10만 개, 버전 메시지 1MB, 이름 255바이트, 버전 기록 한 번에 500개.
- IPC 명령 인자는 serde 타입으로 받고 길이와 형식을 검사한다. 파일 경로는 저장소 루트 기준 상대 경로만 받아 백엔드가 정규화하고, 루트 밖이면 거부한다. 심볼릭 링크는 따라가지 않는다.

## 프론트와 백엔드 계약
- 프로젝트 열기는 한 곳: 앱을 켤 때 열기, 최근 목록, 폴더 고르기, 프로젝트 메뉴는 모두 `project_open` 명령 → `Core::open_project` 하나를 거친다(D-032). 저장소 신뢰 확인은 그 함수 안에 넣는다. 화면은 `useOpenProject` 훅 하나로 연다.
- Tauri 명령 이름은 `<영역>_<동사>`: `app_status`, `repo_open`(1b에서 `project_open`으로 바뀜), `recent_list`·`recent_remove`·`query_project_summary`(1b), `repo_watch`, `repo_unwatch`, `action_list`, `action_preview`, `action_execute`, `query_status`, `query_log`, `query_diff`, `undo_last`(반복 2).
- 명령 등록과 TS 내보내기는 `src-tauri/src/bindings.rs` 한 파일에 모은다. 바인딩 라이브러리를 바꾸면 이 파일과 `core::view`만 고친다(D-019).
- 액션 입력은 `core::view::ActionInput` enum으로 보낸다(D-020). 액션 id는 enum 태그에서 얻으므로 `action_preview(folder, input)`, `action_execute(folder, input, previewToken)`에는 id 인자가 없다. 입력이 있는 액션은 전용 UI(`src/features/<id>/`)가 입력 구조체를 채운다.
- 실패한 명령의 Promise는 `ErrorView { code, detail }`로 reject된다. 화면은 `src/lib/api.ts`가 바꿔 주는 `ApiError`만 다룬다.
- 타입은 Rust에서 정의하고 `tauri-specta`로 TS 바인딩을 생성한다. 프론트에서 타입을 손으로 쓰지 않는다.
- 이벤트: `repo-changed { repo, worktree, git }`, `watch-status { repo, active, reason }`. 이름은 Rust 상수를 바인딩으로 내보내고(`REPO_CHANGED_EVENT` 등), 화면은 `src/lib/events.ts`로만 구독한다. `op-progress`는 아직 없다.
- 프론트는 `action_list`로 받은 메타데이터로 버튼, 메뉴, 명령 팔레트(Ctrl+K)를 만든다.

## 용어와 호버 (D-006)
- `src/glossary/ko.ts`가 `docs/spec.md` 용어표를 그대로 옮긴 데이터. `id → { friendly, gitName, command, description }`.
- 화면에서 용어는 항상 `<Term id="commit" />`로 쓴다. 호버(키보드 포커스 포함) 시 툴팁: 원래 이름 · 명령 · 한 줄 설명.
- 설정 "Git 원래 이름으로 보기"가 켜지면 `Term`이 원래 이름을 본문에 보여주고 툴팁에 쉬운 이름을 보여준다.
- 테스트: 레지스트리의 모든 `glossary_id`가 glossary에 있는지 확인하는 테스트를 둔다.

## 버전 기록 그리기 계층 (D-017 제안, 반복 0은 단순 목록)

노선도 보기는 아직 제안이다. 반복 0은 목록만 만들되, 나중에 그리기를 붙일 때 데이터와 그리기가 섞이지 않게 아래 네 층으로 나눈다.

```
query_log (core::view::LogPage)          데이터: 버전, 부모, 브랜치·태그 이름표
   ↓
features/history/model.ts                화면 모델: HistoryRow[] (반복 0의 목록도 이것만 쓴다)
   ↓
features/history/layout/                 배치 계산: 순수 함수 (rows, 단계) → 역·노선 좌표. DOM 없음, 단위 테스트
   ↓
features/history/render/                 그리기: Renderer 인터페이스 하나에 SvgRenderer(먼저), CanvasRenderer(나중)
```

- `Renderer`는 배치 결과와 보이는 영역만 받는다. 색은 tokens.css의 `--graph-1`~`--graph-6`을 읽어 쓴다.
- 클릭·호버 판정은 DOM이 아니라 배치 결과 좌표로 한다. 그래야 Canvas로 바꿔도 상호작용 코드가 그대로다.
- 확대·축소 단계별로 필요한 저장만 백엔드에서 걸러 받는다(`LogQuery`).
- 시작 화면의 큰·작은 노선도 미리보기도 같은 layout·render를 축소해 쓴다(D-031).

## OS별 처리 (crates/platform)
| 항목 | Windows | Ubuntu |
| --- | --- | --- |
| 폴더 우클릭 "Git으로 관리" | `HKCU\Software\Classes\Directory\shell\higit` (+ `Directory\Background\shell`) 등록, 관리자 권한 불필요 | `~/.local/share/nautilus/scripts/` 스크립트, 또는 nautilus-python 확장 |
| 실행 인자 | `higit.exe "<경로>"` | `higit "<경로>"` |
| 단일 인스턴스 | `tauri-plugin-single-instance` | 같음 |
| git 찾기 | PATH → `C:\Program Files\Git\cmd\git.exe` | PATH |
| 앱 데이터 폴더(최근 목록, D-032) | `%LOCALAPPDATA%\dev.higit.desktop` | `$XDG_DATA_HOME/dev.higit.desktop` 또는 `~/.local/share/dev.higit.desktop` |
| 경로 같음 비교 | 대소문자 무시 | 바이트 비교 |
| 비밀 저장 | Credential Manager (`keyring` 크레이트) | Secret Service (`keyring` 크레이트) |
| 웹뷰 | WebView2 | WebKitGTK 4.1 |

우클릭 메뉴 등록·해제는 설정 화면 토글로 하고, 설치 프로그램에서는 기본 등록한다.

## 새 액션 추가 절차
1. `docs/spec.md` 용어표에 행 추가(id, 쉬운 이름, 원래 이름, 설명, 위험도).
2. `src/glossary/ko.ts`와 `en.ts`에 같은 항목 추가.
3. 필요하면 `crates/git`의 `GitOp`에 변형 추가 + `CliBackend` 매핑 + 파서 테스트.
4. `core::view::ActionInput`에 변형을 더하고(입력이 있으면 `<Id>Input` 구조체), `crates/core/actions/<id>.rs`에 `Action` 구현, 레지스트리에 등록. `ActionMeta.undo`로 되돌리는 방법을 정한다(되돌릴 수 없으면 `Irreversible`). 조회 기능이면 Action 대신 `core::query`에 메서드와 `core::view` 타입을 추가한다(D-010).
5. 기본 버튼·메뉴는 자동. 전용 UI가 필요하면 `src/features/<id>/`.
6. 테스트: 임시 저장소에서 실행 → 결과 확인 → 되돌리기 확인.

## 테스트 전략
- `crates/git`: 실제 git으로 임시 저장소를 만들어 파서와 명령을 검증(`tempfile`). CI에서 Windows·Ubuntu 둘 다.
- `crates/core`: `GitBackend` 가짜 구현으로 실행기 흐름(미리보기 → 백업 → 실행 → 되돌리기) 검증.
- 프론트: Vitest + Testing Library로 부품, `Term` 호버, 위험도별 확인 흐름.
- 끝단 테스트는 기능이 안정된 뒤 `tauri-driver`로 추가.

## 성능 기준(초기 목표)
- 버전 1만 개 저장소에서 버전 기록 첫 화면 300ms 이내(가상 스크롤, 200개씩 불러오기).
- 파일 변경 감시는 300ms 디바운스, `.git/` 내부 변경은 별도 처리.
