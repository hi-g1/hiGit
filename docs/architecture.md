# 구조 설계 (Architecture)

목표는 **기능을 계속 붙여도 구조가 무너지지 않는 것**이다. 핵심은 세 가지다.
1. Git 처리는 `GitBackend` 트레이트 뒤에 숨긴다(D-003).
2. 상태를 바꾸는 기능은 `Action`으로 등록하고 화면은 레지스트리에서 만든다. 조회는 core의 Query로 제공한다(D-010).
3. 화면과 백엔드는 정해진 명령(command)과 이벤트로만 대화한다.

## 계층

```
┌──────────────────────── 프론트엔드 (React, src/) ────────────────────────┐
│ screens/      홈, 작업 공간, 버전 기록, 기록 정리, 충돌 해결, 검토 요청, 설정 │
│ features/     액션별 UI 조각 (미리보기 패널, 확인 대화상자)              │
│ ui/           디자인 시스템 부품 (Button, Term, Tooltip, Dialog …)      │
│ lib/api.ts    백엔드 명령 호출과 타입 (자동 생성 바인딩)                │
└───────────────────────────────▲──────────────────────────────────────────┘
                     Tauri command / event (타입 공유)
┌───────────────────────────────┴── 백엔드 (Rust, src-tauri/) ────────────┐
│ app        Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환  │
│ core       Action 레지스트리, Query, 복구(백업·되돌리기), view 타입      │
│ git        GitBackend 트레이트 + CliBackend 구현, 출력 파서              │
│ hosting    HostingProvider 트레이트 + GitHub 구현 (PR, 인증)            │
│ platform   OS별 코드: 우클릭 메뉴 등록, git 실행 파일 찾기, 키체인      │
│ watcher    파일 변경 감시 → "repo-changed" 이벤트                       │
└──────────────────────────────────────────────────────────────────────────┘
```

의존 방향은 위에서 아래로만. `git`, `hosting`, `platform`은 서로 모른다. `core`만 이들을 조합한다.

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
│  ├─ lib/api.ts             tauri-specta 생성 바인딩 import (invoke는 lib/ 안에서만)
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
    fn version(&self) -> Result<GitVersion>;                          // 반복 0. 2.30 이상 확인(D-003)
    fn discover(&self, path: &Path) -> Result<Option<RepoPath>>;      // 반복 0. 고른 폴더의 저장소 최상위 찾기
    fn status(&self, repo: &RepoPath) -> Result<Status>;              // 반복 0
    fn log(&self, repo: &RepoPath, query: &LogQuery) -> Result<Vec<Commit>>; // 반복 0
    fn diff(&self, repo: &RepoPath, target: DiffTarget) -> Result<Vec<FileDiff>>; // 반복 1에서 구현
    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>>;       // 반복 1에서 구현
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;   // 반복 0은 GitOp::Init만, 나머지는 반복 1부터. 쓰기 작업은 전부 여기로
    fn resolve_ref(&self, repo: &RepoPath, r: &str) -> Result<Oid>;   // 반복 2에서 구현(백업 참조)
    fn update_ref(&self, repo: &RepoPath, name: &str, oid: Oid) -> Result<()>; // 반복 2에서 구현(백업 참조)
}
```

- 이 트레이트는 목표 설계다. 코드에는 해당 반복에 필요한 메서드만 구현해 넣는다.
- `LogQuery`는 나중에 확대·축소 단계(버전만 / 합치기까지 / 전체)를 받을 자리를 둔다(D-017 제안).

- `GitOp`는 enum(Init, Commit, Merge, Rebase, Reset …). 새 쓰기 작업은 enum 변형 하나 + `CliBackend` 매핑 하나를 추가한다.
- `GitOp::Init`은 아직 저장소가 아닌 폴더에서 실행하므로 `repo` 인자는 정규화한 대상 폴더다.
- CLI 출력은 기계용 형식만 파싱한다: `status --porcelain=v2 -z`, `log --format=...%x00`, `for-each-ref --format`.
- 모든 git 호출에 `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `core.quotepath=off`를 준다(한글 파일명, 영어 오류 파싱).

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
}

// core::view (D-020). 화면은 태그 붙은 enum으로 액션과 입력을 함께 보낸다.
// TS: { id: "init" } | { id: "commit"; input: CommitInput } …
#[serde(tag = "id", content = "input", rename_all = "kebab-case")]
pub enum ActionInput { Init, /* Commit(CommitInput), … */ }
```

`RepoState`는 열린 폴더와 저장소 여부를 함께 담는다(`folder`, `repo: Option<RepoPath>`). 그래서 저장소가 아닌 폴더에서만 쓸 수 있는 init도 같은 `available`로 판단한다.

`core`의 실행기(`Executor`)가 위험도에 따라 공통 흐름을 강제한다. 액션 각각이 안전장치를 따로 구현하지 않는다.

```
execute(action, input):
  if risk >= Caution: 프론트가 preview를 먼저 받아 보여줬는지 확인 (preview 토큰)
  if risk >= Caution: 백업 참조 생성  refs/higit/backup/<timestamp>/<HEAD 이름>
  result = action.execute()
  기록: UndoJournal에 {action, 이전 HEAD, 백업 참조, 시각} 추가
  이벤트: "repo-changed"
```

### Query (crates/core) — 조회 (D-010)

```rust
pub struct Queries { /* GitBackend를 품는다 */ }
impl Queries {
    pub fn status(&self, repo: &RepoPath) -> Result<view::Status>;            // 반복 0
    pub fn log(&self, repo: &RepoPath, q: view::LogQuery) -> Result<view::LogPage>; // 반복 0
    // diff, branches 등은 해당 반복에서 추가
}
```

- 조회는 미리보기·위험도·백업이 없으므로 실행기를 거치지 않는다. 레지스트리에도 등록하지 않는다.
- 결과는 `core::view` 타입으로 돌려준다. `crates/git` 타입은 core 안에서 view 타입으로 옮긴다(D-019).
- 화면은 Query도 core를 통해서만 부른다. `app`의 `query_*` 명령이 `Queries`를 호출한다.
- 용어 호버는 glossary id로 동작하므로 조회 화면도 `<Term id="status" />`처럼 쓴다.

### UndoJournal (crates/core)
- 앱이 한 작업 목록을 `.git/higit/journal.jsonl`에 남긴다(저장소마다 따로, 저장소를 옮겨도 따라감).
- "방금 작업 취소" = 마지막 항목의 이전 상태로 복구(필요하면 stash로 작업 중 변경 보호).
- "복구" 화면 = journal + `git reflog`를 시간순으로 합쳐 보여준다.

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

## 프론트와 백엔드 계약
- Tauri 명령 이름은 `<영역>_<동사>`: `app_status`, `repo_open`, `action_list`, `action_preview`, `action_execute`, `query_status`, `query_log`, `undo_last`.
- 명령 등록과 TS 내보내기는 `src-tauri/src/bindings.rs` 한 파일에 모은다. 바인딩 라이브러리를 바꾸면 이 파일과 `core::view`만 고친다(D-019).
- 액션 입력은 `core::view::ActionInput` enum으로 보낸다(D-020). 액션 id는 enum 태그에서 얻으므로 `action_preview(folder, input)`, `action_execute(folder, input, previewToken)`에는 id 인자가 없다. 입력이 있는 액션은 전용 UI(`src/features/<id>/`)가 입력 구조체를 채운다.
- 실패한 명령의 Promise는 `ErrorView { code, detail }`로 reject된다. 화면은 `src/lib/api.ts`가 바꿔 주는 `ApiError`만 다룬다.
- 타입은 Rust에서 정의하고 `tauri-specta`로 TS 바인딩을 생성한다. 프론트에서 타입을 손으로 쓰지 않는다.
- 이벤트: `repo-changed { repo, kinds }`, `op-progress { op_id, percent, message }`.
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

## OS별 처리 (crates/platform)
| 항목 | Windows | Ubuntu |
| --- | --- | --- |
| 폴더 우클릭 "Git으로 관리" | `HKCU\Software\Classes\Directory\shell\higit` (+ `Directory\Background\shell`) 등록, 관리자 권한 불필요 | `~/.local/share/nautilus/scripts/` 스크립트, 또는 nautilus-python 확장 |
| 실행 인자 | `higit.exe "<경로>"` | `higit "<경로>"` |
| 단일 인스턴스 | `tauri-plugin-single-instance` | 같음 |
| git 찾기 | PATH → `C:\Program Files\Git\cmd\git.exe` | PATH |
| 비밀 저장 | Credential Manager (`keyring` 크레이트) | Secret Service (`keyring` 크레이트) |
| 웹뷰 | WebView2 | WebKitGTK 4.1 |

우클릭 메뉴 등록·해제는 설정 화면 토글로 하고, 설치 프로그램에서는 기본 등록한다.

## 새 액션 추가 절차
1. `docs/spec.md` 용어표에 행 추가(id, 쉬운 이름, 원래 이름, 설명, 위험도).
2. `src/glossary/ko.ts`와 `en.ts`에 같은 항목 추가.
3. 필요하면 `crates/git`의 `GitOp`에 변형 추가 + `CliBackend` 매핑 + 파서 테스트.
4. `core::view::ActionInput`에 변형을 더하고(입력이 있으면 `<Id>Input` 구조체), `crates/core/actions/<id>.rs`에 `Action` 구현, 레지스트리에 등록. 조회 기능이면 Action 대신 `core::query`에 메서드와 `core::view` 타입을 추가한다(D-010).
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
