# 구조 설계 (Architecture)

목표는 **기능을 계속 붙여도 구조가 무너지지 않는 것**이다. 핵심은 세 가지다.
1. Git 처리는 `GitBackend` 트레이트 뒤에 숨긴다(D-003).
2. 상태를 바꾸는 기능은 `Action`으로 등록하고 화면은 레지스트리에서 만든다. 조회는 core의 Query로 제공한다(D-010).
3. 화면과 백엔드는 정해진 명령(command)과 이벤트로만 대화한다.

## 계층

```
┌──────────────────────── 프론트엔드 (React, src/) ────────────────────────┐
│ screens/      시작 화면, 작업 공간, 버전 기록, 작업 내역, 기록 정리, 충돌 해결, 검토 요청, 설정 │
│ features/     액션별 UI 조각 (미리보기 패널, 확인 대화상자)              │
│ ui/           디자인 시스템 부품 (Button, Term, Tooltip, Dialog …)      │
│ lib/api.ts    백엔드 명령 호출과 타입 (자동 생성 바인딩)                │
└───────────────────────────────▲──────────────────────────────────────────┘
                     Tauri command / event (타입 공유)
┌───────────────────────────────┴── 백엔드 (Rust, src-tauri/) ────────────┐
│ app        Tauri 진입점. 명령은 얇게: 입력 검사 → core 호출 → 결과 변환  │
│ core       Action 레지스트리, Query, 프로젝트 열기·최근 목록, 작업 내역, view 타입 │
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
│  ├─ lib/                   api.ts(백엔드 호출, invoke는 lib/ 안에서만), events.ts(이벤트 구독), queries.ts(TanStack Query), useRepoWatch.ts, dialog.ts, shortcuts.ts·useShortcut.ts(단축키 표와 판정은 여기만, D-056), bindings.ts·action-meta.json(생성물)
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
    // 반복 0~1a
    fn version(&self) -> Result<GitVersion>;                          // 2.30 이상 확인(D-003, D-050)
    fn discover(&self, path: &Path) -> Result<Option<RepoPath>>;      // 고른 폴더의 저장소 최상위 찾기
    fn status(&self, repo: &RepoPath) -> Result<Status>;
    fn log(&self, repo: &RepoPath, query: &LogQuery) -> Result<Vec<Commit>>;
    fn excludes_file(&self, repo: &RepoPath) -> Result<Option<PathBuf>>; // 전역 무시 규칙 파일(감시용)
    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff>; // 마지막 버전과 지금을 합쳐 비교
    // 반복 1b: 브랜치·원격
    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>>;       // 로컬·원격, 원격 HEAD 별칭 뺌
    fn remotes(&self, repo: &RepoPath) -> Result<Vec<Remote>>;
    fn remote_head(&self, repo: &RepoPath, remote: &str) -> Result<Option<String>>; // 원격 기본 브랜치
    fn check_branch_name(&self, name: &str) -> Result<bool>;          // check-ref-format + `-`·HEAD 막기
    fn current_branch(&self, repo: &RepoPath) -> Result<Option<String>>;
    // 반복 1b: ref (core가 만든 refs/higit/* 와 되돌리기)
    fn list_refs(&self, repo: &RepoPath, prefix: &str) -> Result<Vec<RefEntry>>;
    fn resolve_ref(&self, repo: &RepoPath, name: &str) -> Result<Option<Oid>>;
    fn update_ref(&self, repo: &RepoPath, name: &str, new: &Oid, expect: &RefExpect) -> Result<()>; // 그사이 바뀌면 실패
    fn delete_ref(&self, repo: &RepoPath, name: &str, expect: &RefExpect) -> Result<()>;
    fn git_dir(&self, repo: &RepoPath) -> Result<PathBuf>;             // journal 위치. worktree면 .git/worktrees/<이름>
    fn common_dir(&self, repo: &RepoPath) -> Result<PathBuf>;          // worktree끼리 함께 쓰는 git 폴더(hooks 위치)
    fn write_tree(&self, repo: &RepoPath) -> Result<Option<Oid>>;      // index 트리(되돌리기). 충돌 중이면 None
    // 반복 1b: 기록 비교 (홈·동기화)
    fn is_ancestor(&self, repo: &RepoPath, ancestor: &str, descendant: &str) -> Result<bool>;
    fn count_between(&self, repo: &RepoPath, left: &str, right: &str) -> Result<(u32, u32)>;
    fn merge_base(&self, repo: &RepoPath, a: &str, b: &str) -> Result<Option<Oid>>;
    fn merge_check(&self, repo: &RepoPath, base: Option<&str>, ours: &str, theirs: &str) -> Result<MergeCheck>; // 겹침 미리 보기(D-050)
    // 반복 1b: 맡겨 두기·신뢰
    fn parked_paths(&self, repo: &RepoPath, oid: &Oid) -> Result<Vec<String>>;
    fn local_config(&self, repo: &RepoPath) -> Result<Vec<(String, String)>>; // local·worktree 범위만. 신뢰 확인용(D-039)
    fn set_repo_overrides(&self, repo: &RepoPath, pairs: Vec<String>); // "끄고 열기"의 -c 덮어쓰기
    // 반복 2: 충돌·임시 보관·작업 내역
    fn reflog(&self, repo: &RepoPath, limit: u32) -> Result<Vec<ReflogEntry>>;   // HEAD reflog, 최근 것부터
    fn stash_list(&self, repo: &RepoPath) -> Result<Vec<StashEntry>>;           // refs/stash 목록
    fn index_entries(&self, repo: &RepoPath) -> Result<Vec<IndexEntry>>;        // ls-files -s -z. 충돌 중이면 stage 1·2·3
    fn read_blobs(&self, repo: &RepoPath, oids: &[Oid], max_bytes: usize) -> Result<Vec<BlobHead>>; // cat-file --batch 프로세스 둘
    fn merge_segments(&self, repo: &RepoPath, base: &[u8], ours: &[u8], theirs: &[u8]) -> Result<Vec<MergeSegment>>; // 충돌 구간(D-086)
    // 반복 2: 진행·취소·잠금 (D-080, D-081)
    fn request_cancel(&self, repo: &RepoPath);   // 이 저장소에서 지켜보는 git을 끝낸다(취소·강제로 멈추기)
    fn clear_cancel(&self, repo: &RepoPath);     // 액션을 시작·마칠 때 취소 표시를 지운다
    fn set_observer(&self, observer: Option<ProgressObserver>); // 1초마다 (저장소, 경과, 마지막 출력 뒤 시간)
    fn other_git_running(&self) -> bool;         // 앱이 띄우지 않은 git이 시스템에 있는지. 모르면 true
    // 쓰기 작업은 전부 여기로
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;
}
```

- 코드에는 해당 반복에 필요한 메서드만 구현해 넣는다. 위는 반복 2까지 구현한 것이다. 메서드를 더하면 가짜 구현 둘(`executor.rs` 테스트의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)도 함께 고친다.
- `GitOp`(반복 2까지): `Init`, `Stage`, `Unstage`, `StageAll`, `Commit`, `CreateAndSwitch`, `Switch`, `SwitchTrack`, `Park`, `DiscardPaths`, `StashDropIf`, `StashDropOid`, `Unpark`, `UnparkKeep`, `UnparkAbort`, `Fetch`, `MergeFf`, `Merge`, `MergeAbort`, `Push`, `ResetSoft`, `ResetKeep`, `ReadTree`, `ResolveFile`, `ResolveDelete`, `ResolveSide`, `SnapshotFiles`, `Revert`, `AddSafeDirectory`.
- `merge_check`는 git 2.38 이상이면 `merge-tree --write-tree`, 그보다 낮으면 임시 index에 `read-tree -m`으로 겹침만 본다(D-050). 사용자의 index와 작업 폴더는 건드리지 않는다.
- `merge_segments`는 임시 폴더에 세 쪽을 쓰고 `merge-file -p --diff3 --marker-size=40`의 표시 줄만 읽는다. 입력 어디에든 길이 40의 표시 같은 줄이 있거나 이진 파일이면 오류로 돌려 화면이 파일 단위 선택으로 떨어진다. 표시 문자는 화면으로 나가지 않는다(D-086).
- 충돌 해결: `ResolveFile`(블롭을 만들어 index에 넣고 `checkout-index`로 작업 폴더를 사용자 줄바꿈 설정대로 씀, 모드는 100644·100755·120000만), `ResolveDelete`(지우는 쪽), `ResolveSide`(이진·링크·큰 파일을 한쪽 그대로). 덮어쓰기 전 백업은 `SnapshotFiles`(작업 폴더 파일을 stash 모양 커밋으로, index·stash 목록은 그대로).
- `LogQuery`는 나중에 확대·축소 단계(버전만 / 합치기까지 / 전체)를 받을 자리를 둔다(D-017, 반복 2b).

- `GitOp`는 enum(Init, Commit, Merge, Rebase, Reset …). 새 쓰기 작업은 enum 변형 하나 + `CliBackend` 매핑 하나를 추가한다.
- `GitOp::Init`은 아직 저장소가 아닌 폴더에서 실행하므로 `repo` 인자는 정규화한 대상 폴더다.
- CLI 출력은 기계용 형식만 파싱한다: `status --porcelain=v2 -z`, `log --format=%x1e…%x00…`, unified diff, `for-each-ref --format`, `ls-files -s -z`, `cat-file --batch`, `reflog -z`. 원격 작업의 `--progress` 줄은 해석하지 않고 출력이 왔는지만 본다(D-080). 실패 종류는 `crates/git/src/failure.rs` 한 곳에서만 영어 원문으로 가린다(D-021).
- 경로 목록은 `--pathspec-from-file=- --pathspec-file-nul`로 stdin에 넘기고 `--literal-pathspecs`를 붙인다.
- 모든 git 호출에 `LC_ALL=C`(그리고 `LANGUAGE` 지움), `GIT_TERMINAL_PROMPT=0`, `GIT_PAGER=cat`, `GIT_OPTIONAL_LOCKS=0`, `-c core.quotepath=off -c color.ui=never`를 준다. `GIT_OPTIONAL_LOCKS=0`은 조회가 index를 쓰지 않게 해 파일 감시의 되먹임 고리를 막는다. OS별 설정(Windows 콘솔 창 숨기기)은 platform이 먼저 붙이고 위 값이 마지막에 이긴다.

#### git 실행 규칙 (D-035)
- 인자는 배열로 넘기고 셸을 거치지 않는다. 문자열을 이어 붙여 명령줄을 만들지 않는다.
- 리비전 앞에는 `--end-of-options`, 경로 앞에는 `--`. 경로 목록은 위처럼 stdin으로. 리비전은 `crates/git/src/cli/args.rs`의 `push_revs`로만 붙인다(git 크레이트에서 `--end-of-options` 글자는 이 함수에만 있다). 이 표시는 git 2.34 이상에서 확인한 명령(`args::Cmd`)에만 넣고, `checkout`·`reset`·`check-ref-format`에는 넣지 않는다(2.34.1에서 오동작). 넣지 못하는 경우는 core의 이름 검사가 막는다. 새 명령에 리비전을 넘기면 `Cmd`에 변형을 더하고 Ubuntu 22.04 기본 git(2.34.1) CI 작업에서 확인한다.
- 공통 환경에 `PAGER=cat`, `GIT_EDITOR=:`(편집기를 띄우지 않는 특별 값), `GIT_ALLOW_PROTOCOL=https:ssh:git`도 둔다. 상위 프로세스의 `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` 등 저장소 위치 변수는 지운다(D-052). 로컬 경로 원격은 테스트에서만 `CliBackend::allow_local_remotes_for_tests()`로 켠다. 조회는 `--no-ext-diff`, `--no-textconv`.
- 새 브랜치·태그 이름은 `check-ref-format`으로 검사하고 `-`로 시작하면 막는다. 원격 이름과 원격에서 온 브랜치 이름도 git에 넘기기 전에 core의 `actions/names.rs` 검사를 거친다. 원격 주소는 `https://`, `ssh://`, `git@host:path`만.
- 저장소 파일을 읽을 때 심볼릭 링크는 따라가지 않고 링크 대상 글자만 보여준다(D-052). 경로는 저장소 루트 안만 받는다.
- git 실행 파일은 시작할 때 절대 경로로 한 번 찾아 그 경로만 쓴다.
- 원격 작업은 `CliBackend::run_network`로만 한다. hook·서명이 오래 붙잡을 수 있는 로컬 쓰기(`Commit`·`Merge`·`Revert`)는 `run_local_watched`로 돌려 진행 알림과 사람이 확인한 강제 멈춤만 받는다. 그 밖의 로컬 조회·쓰기는 끝내지 않는다(D-066, D-080).

| 작업 | 시간 제한 | 끝내는 방법 |
| --- | --- | --- |
| 사용자가 누른 원격 작업(fetch·pull·push·sync) | 처음부터 10분(`NETWORK_TIMEOUT`). 첫 출력 뒤 90초 동안 출력이 없으면 멈춤(`STALL`, `Stalled`). 연결 전(인증 창 대기)에는 10분 제한만. 동기화 버튼의 "취소"(`Cancelled`) | 프로세스 트리째(`ProcessControl`). 멈춤·취소는 먼저 정상 종료(`terminate_tree`), 2초 뒤 강제(`kill_tree`). 10분 제한은 바로 `kill_tree` |
| 저절로 하는 원격 확인(D-055) | 60초(`BACKGROUND_TIMEOUT`). 진행 알림·취소 없음 | 같음. 묻는 창을 띄우지 않는 백그라운드 모드 |
| 로컬 쓰기 `Commit`·`Merge`·`Revert` | 없음. 60초가 지나면 화면이 "응답 없음 · 강제로 멈추기"를 보이고 확인을 받는다(D-080) | 사람이 확인할 때만 멈춤·취소와 같은 방법 |
| 그 밖의 로컬 조회·쓰기 | 없음 | 끝내지 않는다(index를 쓰는 중에 끊으면 깨진다). 남은 잠금 파일은 "멈춘 작업 정리"(D-081) |

- 사용자가 SSH 명령을 정하지 않았으면 `GIT_SSH_COMMAND=ssh -o BatchMode=yes`(D-023). 백그라운드는 `GCM_INTERACTIVE=never`, `credential.interactive=never`, `SSH_ASKPASS_REQUIRE=never`, `gc.auto=0`.
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
  잠금: 작업 폴더 잠금, 원격 잠금, 또는 둘 다 (아래 "잠금")
  if undo == Irreversible: 프론트가 확인을 받았는지 확인 (확인 토큰)
  if risk >= Caution && action.moves_head(): HEAD를 백업 ref로  refs/higit/backup/<UTC 시각>-<순번>/<액션 id>. 실패하면 실행하지 않음
  tracked = action.tracked_refs(input)        (refs/heads/*, refs/stash)
  before = UndoState::capture  (HEAD, 브랜치, index 트리, tracked의 값)   ← undo가 기록하는 종류일 때만
  result = action.execute()
  journal: result.journal_mode()가
    Journal → is_recorded면 Action 줄(after를 캡처해 before와 같으면 남기지 않음), is_barrier면 Barrier 줄, Nothing이면 없음
    Barrier → Barrier 줄 (맡긴 파일 겹침 해결을 마침)
    Skip    → 없음 (합치기를 시작만 함, 꺼내기, 잠금 정리, 맡긴 항목 복구)
    Always  → 상태가 그대로여도 Action 줄 (제외할 파일)
  (화면 갱신은 파일 변경 감시가 맡는다. 실행기가 이벤트를 보내지 않는다)
```

- `moves_head()`가 거짓인 액션(`discard`, `clear-lock`, `take-out`, `recover-parked`, `stash`, `ignore`, `merge-into`, `conflict-abort`)은 실행기가 HEAD 백업을 만들지 않는다(작업 내역에 쓸모없는 줄이 쌓이지 않게, D-072). 지우거나 덮어쓰는 액션은 스스로 그 대상을 백업 ref로 먼저 남긴다(`discard-<브랜치>`, `conflict-files-<브랜치>`, `restore-<브랜치>`, `sync`).
- 반복 2까지 등록한 액션(`actions::register_all` 순서): `init`, `stage`, `unstage`, `commit`, `ignore`, `merge`, `merge-into`, `conflict-finish`, `conflict-abort`, `stash`, `take-out`, `discard`, `restore`, `clear-lock`, `recover-parked`, `switch`, `branch-create`, `fetch`, `pull`, `push`, `sync`. 위험도와 `undo`는 생성물 `src/lib/action-meta.json`에 있다.

#### 잠금 둘
- `Core.busy`(작업 폴더): index·작업 폴더를 쓰는 작업. 같은 저장소에서 둘이 겹치면 index.lock이 부딪히므로 나중 것은 `Busy`로 거절한다.
- `Core.net_busy`(원격): 원격과 통신하는 작업. 원격 확인(fetch)과 올리기(push)는 이것만 잡아 버전 저장을 막지 않는다. pull·sync와 `clear-lock`(앱의 다른 작업이 하나도 없을 때만 지운다, D-081)은 둘 다 잡고, 되돌리기·다시 하기는 작업 폴더 잠금을 잡는다.
- 저절로 하는 원격 확인이 원격 잠금을 쥐고 있으면(`Core.bg_fetch`) 사람이 누른 원격 작업은 거절하지 않고 끝날 때까지 기다린다(최대 `BACKGROUND_TIMEOUT` + 5초, D-068). 그 밖의 겹침은 바로 `Busy`.
- 잠금은 저장소 경로별 `HashSet`이고, `BusyGuard`가 오류여도 푼다. 프로세스 사이 잠금은 git의 index.lock에 맡긴다.
- `Core.running`(저장소 → 액션 id와 마지막 진행 알림의 경과): 저절로 하는 원격 확인을 뺀 액션과 되돌리기·다시 하기(`undo`·`redo`)가 도는 동안 들어 있다. 진행 알림과 취소가 이 표를 본다. 시작과 끝에 같은 잠금 안에서 `clear_cancel`을 불러 끝난 작업 뒤에 취소 표시가 남지 않는다.

#### 진행과 취소 (D-080)
- `Core::set_progress_sink`가 `GitBackend::set_observer`에 관찰자를 걸고, `Core.running`에 있는 저장소의 알림만 `view::OpProgress { repo, action, elapsed_secs, idle_secs }`로 바꿔 넘긴다. 앱은 이를 `op-progress` 이벤트로 보낸다(`src-tauri/src/lib.rs`).
- 알림은 지켜보는 git 프로세스(원격 작업과 `Commit`·`Merge`·`Revert`)가 도는 동안 1초마다 나간다. 저절로 하는 원격 확인은 보내지 않는다. 페이로드에 파일 이름이나 원격 주소를 담지 않는다.
- `action_cancel` → `Core::cancel`은 원격 작업(fetch·pull·push·sync)이면 언제든, 로컬 쓰기면 진행 알림의 경과가 60초를 넘었을 때만 `request_cancel`을 부른다(화면만 믿지 않는다). 화면은 원격 작업에 "취소"(3초 뒤)를 두고, 로컬 쓰기는 60초가 지나면 "강제로 멈추기"를 확인 뒤 같은 명령으로 부른다(`features/progress`).
- `Cancelled`는 오류 줄 없이 토스트 "취소했어요", `Stalled`는 안내 줄 "진행이 멈춰 그만뒀어요"와 동기화 단추.

### Query (crates/core) — 조회 (D-010)

```rust
pub struct Queries { /* GitBackend를 품는다 */ }
impl Queries {
    pub fn status(&self, repo: &Path) -> Result<view::StatusView>;            // 반복 0
    pub fn log(&self, repo: &Path, q: view::LogQuery) -> Result<view::LogPage>; // 반복 0
    pub fn diff(&self, repo: &Path, r: &view::DiffRequest) -> Result<view::FileDiffView>; // 반복 1a. 경로는 저장소 안만. 링크는 대상만(D-052)
    pub fn branches(&self, repo: &Path) -> Result<view::BranchListView>;        // 반복 1b. 맡긴 파일 수, 기본 브랜치
    pub fn check_branch_name(&self, repo: &Path, name: &str) -> Result<view::BranchNameCheck>; // 반복 1b
    pub fn home_map(&self, repo: &Path) -> Result<view::HomeMapView>;          // 반복 1b. 홈 두 줄 노선도(D-060)
    pub fn project_summary(&self, repo: &Path) -> Result<view::ProjectSummaryView>; // 시작 화면
    pub fn secret_check(&self, repo: &Path, r: &view::SecretCheckRequest) -> Result<view::SecretCheckView>; // 반복 2. 경로·규칙·줄 번호만(D-077)
    pub fn merge_hint(&self, repo: &Path) -> Result<view::MergeHintView>;      // 반복 2. 방향 하나(D-048, D-071)
    pub fn conflict_state(&self, repo: &Path) -> Result<view::ConflictStateView>; // 반복 2. 맥락 Merge·Unpark, 충돌 파일 200개까지
    pub fn conflict_file(&self, repo: &Path, path: &str) -> Result<view::ConflictFileView>; // 반복 2. 구간(text) 또는 파일 단위(whole)
    pub fn stashes(&self, repo: &Path) -> Result<view::StashListView>;        // 반복 2. 맡겨 두기가 잠깐 만든 항목은 뺌
}
```

- 조회는 미리보기·위험도·백업이 없으므로 실행기를 거치지 않는다. 레지스트리에도 등록하지 않는다.
- 작업 내역(`Core::recovery` → `Executor::recovery` → `core::recovery::collect`)은 journal 서명 키가 필요해 실행기가 맡는다. 끝나지 않은 작업(`Core::pending_work`)은 앱의 잠금 상태를 봐야 해 `Core`가 맡는다. 둘 다 `query_*` 명령으로 부르고 잠금을 잡지 않는다.
- `merge_hint`는 결과를 `(HEAD 해시, 기본 브랜치 해시)`로 기억하고, 겹침 검사가 1초를 넘으면 그 저장소에서는 겹친 파일 표시를 뺀다. `conflict_file`은 세 쪽이 모두 UTF-8 텍스트이고 1MB 이하일 때만 구간을 주고, 이진·한쪽 삭제·링크·큰 파일·모드·서브모듈은 파일 단위(`Whole`)로 각 쪽 상태와 앞부분만 준다. 읽는 것은 object database의 블롭뿐이다.
- 결과는 `core::view` 타입으로 돌려준다. `crates/git` 타입은 core 안에서 view 타입으로 옮긴다(D-019).
- 화면은 Query도 core를 통해서만 부른다. `app`의 `query_*` 명령이 `Queries`를 호출한다.
- 용어 호버는 glossary id로 동작하므로 조회 화면도 `<Term id="status" />`처럼 쓴다.

### 파일 변경 감시 (crates/watcher, core::watch)
- `watcher::RepoWatcher`가 저장소 루트를 재귀 감시한다(notify, 300ms 디바운스). 다른 내부 크레이트를 모른다.
- 작업 폴더 변경과 `.git` 안의 의미 있는 파일(HEAD, index, refs/*, packed-refs, FETCH_HEAD …)만 알린다. `.gitignore`(하위 폴더 포함)·`.git/info/exclude`·`core.excludesFile` 대상, `.git/objects`·잠금 파일, 읽기 알림, 폴더 자체의 수정 알림은 거른다.
- 감시 오류나 저장소 폴더 삭제는 한 번만 "멈춤"으로 알린다. 폴더 삭제는 0.5초마다 따로 확인한다(알림을 주지 않는 OS가 있다).
- `Core::watch(repo, sink)`가 한 번에 한 저장소만 감시하고, 알림을 `RepoChanged`·`WatchStatus` view 타입으로 바꾼다. `app`이 Tauri 이벤트로 내보낸다.
- 화면(`useRepoWatch`)은 작업 폴더 변경이면 변경 사항·비교, `.git` 변경이면 거기에 버전 기록·브랜치·노선도·충돌 상태·합치기 제안·작업 내역·임시 보관·끝나지 않은 작업을 다시 읽는다(1초에 한 번). 감시가 꺼지면 안내 줄과 "다시 읽기"를 보여주고 창 포커스 새로 고침으로 대신한다(D-025).
- 알려진 제약: 무시된 폴더에도 감시 등록은 된다(Ubuntu inotify 한도에 걸리면 멈춤으로 떨어짐). `core.excludesFile`이 없을 때 git의 기본 전역 무시 파일은 읽지 않는다.

### 되돌리기와 UndoJournal (crates/core/src/undo, D-042, D-063, D-065)
- 앱이 한 작업 목록을 `<git 폴더>/higit/journal.jsonl`에 남긴다(저장소마다 따로, 저장소를 옮겨도 따라감. worktree는 자기 git 폴더).
- 레지스트리의 Action마다 `undo`를 정한다. 되돌리기는 별도 Action을 새로 만들지 않고 journal 항목과 그 Action의 `undo`로 `undo::run`이 처리한다. 명령은 `undo_last`, `redo_last`.

```rust
pub enum UndoKind {
    Nothing,         // 되돌릴 것이 없음. journal에 남기지 않음(원격 확인, 관리 시작, 합치기 그만두기, 꺼내기, 맡긴 항목 복구)
    RestoreRef,      // HEAD·브랜치를 before로. 고친 파일은 그대로(버전 저장. 모두 넣고 저장이면 index도). HEAD가 그대로면 refs만("main에 합치기")
    RestoreIndex,    // index를 before의 tree로(이번 버전에 넣기·빼기). 충돌 중이라 트리가 없으면 경계 줄
    SwitchBack,      // 이전 브랜치로 갈아타고 맡겨 둔 stash를 다시 꺼냄(브랜치 갈아타기)
    RestoreTree,     // 지금 버전을 백업하고 reset --keep으로 before.head로. 고친 파일은 둠(합치기, 합치기 마치기, 되살리기)
    DeleteCreated,   // 만든 ref를 지움(새 브랜치)
    Inverse,         // 액션별 반대 동작. 임시 보관은 꺼내고 목록에서 빼기, 제외할 파일은 덧붙인 줄 지우기. 올리기·동기화는 UndoPushed, 가져오기는 NothingToUndo
    Irreversible,    // 되돌릴 수 없음. 실행 전 확인(고친 것 버리기, 멈춘 작업 정리, 덮어쓰며 올리기)
    Barrier,         // 되돌릴 수 없지만 확인 없이 실행하고 경계 줄만(D-072). 지금은 결과의 journal_mode로만 쓴다
}
```

journal 줄 형식(한 줄에 JSON 하나, 덧붙이기만 한다):

```
{"v":1,"type":"action","id":…,"at":unix초,"action":"commit","input":{…},"before":UndoState,"after":UndoState,"kind":"restoreRef","sig":…}
{"v":1,"type":"barrier","at":…,"action":"…","sig":…}     // 이 앞 줄은 되돌리지 않는다(D-065). 되돌리는 새 버전 뒤에는 "action":"revert"
{"v":1,"type":"undo","target":<action id>,"at":…,"sig":…}
{"v":1,"type":"redo","target":<action id>,"at":…,"sig":…}
```

- `sig`는 앱 데이터 폴더의 키(`journal.key`)로 만든 HMAC-SHA256이다(D-063). 서명이 맞지 않거나 형식이 이상한 줄은 되돌리기에 쓰지 않는다(`Journal::read`). 작업 내역은 형식과 `is_valid`만 통과한 줄도 서명 여부와 함께 읽는다(`Journal::read_all`). 받은 zip 저장소에 남이 만든 줄이 있어도 되돌리기가 그 줄을 따르지 않는다. 4MB를 넘으면 뒤쪽만 읽는다.
- `UndoState = { head, branch, parked: [(브랜치, stash 해시)], index_tree, refs: [(ref 이름, 해시 또는 없음)] }`. `refs`는 액션의 `tracked_refs`가 채우고 `is_valid`는 `refs/heads/*`와 `refs/stash`만 받는다.
- 반복 2의 새 필드(`refs` 등)는 비어 있으면 직렬화하지 않는다. 옛 줄을 다시 직렬화해도 같은 글자라 서명이 그대로 맞는다(`VERSION`은 1, D-072). 새 `kind`를 모르는 옛 앱은 그 줄을 버린다.
- `input`에는 `ActionInput::journal_copy()`를 넣는다. 경로 목록은 200개까지, 합치기 마치기의 직접 고친 글은 빼고 선택 종류만 남긴다. 되돌리기·다시 하기는 입력이 아니라 `before`/`after`로 한다.
- 넣기·빼기 한 번마다 `capture`가 두 번 돈다(git 6회 안팎). 넣기·빼기 되돌리기는 토스트 없이 Ctrl+Z로만 한다(D-072).
- 되돌리기(Ctrl+Z, 토스트 "되돌리기")는 마지막 경계 뒤의 되돌리지 않은 Action 줄을 `kind`대로 되돌리고 `undo` 줄을 덧붙인다. 다시 하기(Ctrl+Shift+Z)는 같은 버전을 ref로 되살린다(D-065. 버전을 새로 만들지 않는다). 종류별 되돌리기를 마치면 `before.refs`의 값으로 다른 ref를 되돌린다(다시 하기는 `after.refs`). 그사이 바뀌었으면 실패한다.
- 합치기·리베이스·cherry-pick·revert 중이면 되돌리기·다시 하기 모두 `OperationInProgress`로 거절한다(`refuse_during_operation`).
- 되돌리기 전에 지금 HEAD·브랜치가 줄의 `after`와 같은지 확인한다. 다르면(앱 밖에서 git을 썼음) `UndoStale`로 거절하고, 화면은 토스트의 "작업 내역" 단추로 안내한다.
- 버전을 뒤로 옮기는 되돌리기(`RestoreRef`·`RestoreTree`)는 옮기기 전에 지금 버전을 백업 ref(`undo-<브랜치>`)로 남긴다. 백업을 못 쓰면 되돌리지 않는다.
- 되돌리면 이미 올린 버전이 빠지는 경우(`RestoreRef`·`RestoreTree`)에는 기록을 지우지 않고 그 변경을 취소하는 새 버전을 만든다(`GitOp::Revert`, 합치기 버전은 `-m 1`, D-073). 되돌릴 버전이 `before` 바로 위가 아니면 `UndoPushed`, 넣은 파일이 있으면 `StagedChanges`. 만든 뒤 경계 줄을 덧붙여 그 앞 줄과 다시 하기를 닫고, `UndoResult.reverted`로 화면이 "되돌리는 새 버전을 만들었어요"를 띄운다.
- 브랜치별로 맡겨 둔 고친 파일은 `refs/higit/parked/<브랜치>`의 stash 커밋이다(D-047). 브랜치 이름의 `/`는 `%2F`로 적는다(D-064). 일반 stash 목록과 섞지 않는다(만든 뒤 `StashDropIf`로 뺀다). 꺼낸 맡긴 ref는 지우기 전에 백업 ref(`parked-<브랜치>`)로 옮긴다.
- 백업 ref는 늘 `undo::backup::backup_oid`로 만든다. `refs/higit/`를 쓰거나 지우는 코드는 core의 `undo`, `actions/switch`(맡기기), `actions/stash`·`actions/conflict`(꺼낸 맡긴 ref 지우기), `actions/recover_parked`(남은 항목을 맡긴 ref로)에만 둔다. ref 이름 규칙은 `parked`(`ref_name`)와 `undo::backup`(`ref_name`)에 있다.

#### 작업 내역 (D-085)
- 화면(`screens/Activity`)은 `query_recovery`가 주는 `RecoveryView { items }`를 보여준다. `core::recovery::collect`가 다섯 원천을 최신 먼저 200개로 합친다: journal의 Action 줄(되돌린 줄은 `undone`, 서명이 맞지 않는 줄은 `restore: None`), HEAD reflog(같은 해시·2초 안의 journal 줄과 겹치면 뺌), 백업 ref, 맡긴 ref, 임시 보관 목록(맡겨 두기가 잠깐 만든 항목은 뺌).
- 백업 ref의 종류는 이름이 아니라 객체 모양으로 가른다(`recovery::kind_of`). 둘째 부모의 메시지가 `index on `으로 시작하면 `Changes`(꺼내기), 아니면 `Version`(이 시점으로 되돌리기).
- `Version`은 `restore`(`reflog`/Safe/`RestoreTree`): 지금 브랜치를 `ResetKeep`으로 옮기고 그 전에 지금 버전을 `restore-<브랜치>` 백업 ref로 남긴다. `Changes`는 `take-out`(`TakeOutSource::Stash { oid }`, `Ref { refname }`): 백업 ref는 남기고, 맡긴 ref는 꺼낸 뒤 백업으로 옮겨 지운다. core는 받은 해시·ref가 reflog·백업 ref·목록에 있는지 다시 확인한다.
- 정리: 프로젝트를 열고 신뢰 확인을 마친 뒤 `undo::backup::prune`이 이름의 시각이 30일(`KEEP_DAYS`) 지난 백업 ref를 한 번에 20개(`PRUNE_AT_ONCE`)까지 지운다(D-079). 시각을 못 읽는 ref와 맡긴 ref는 두고, 정리가 실패해도 열기는 성공이다.

#### 끝나지 않은 작업 (D-041, D-081, D-082)
- `Core::pending_work`(`query_pending_work`, `view::PendingWorkView { stale_locks, leftover_parked, other_operation }`)는 journal이 아니라 저장소에 남은 흔적으로 알아낸다. 앱의 작업이 그 저장소에서 돌고 있으면(`repo_busy`) 잠금과 맡긴 항목은 비워 둔다. 합치는 중이면 화면은 알림 줄 대신 충돌 해결 화면을 연다.
- 오래된 잠금(`core::locks`): git 폴더(작업 트리별·공용) 최상위와 공용 폴더 `refs/` 아래(깊이 6, 500개까지)의 일반 파일 `*.lock`. 모두 수정한 지 60초(`STALE_AFTER`)가 지났고 앱이 띄우지 않은 git 프로세스가 없을 때(`GitBackend::other_git_running`: platform의 `running_git_pids`에서 runner가 기록한 앱의 git pid를 뺀다)만 돌려준다. 하나라도 어긋나면 빈 목록이다. 지우기 직전(`locks::remove`)에 링크·이름·위치·나이를 다시 본다.
- `clear-lock`(`lock`/Caution/`Irreversible`, 잠금 둘 다): 확인 토큰을 받은 뒤 다시 조사해 같은 조건일 때만 지운다. 지우기 직전에 링크가 아니고 이름이 `.lock`으로 끝나며 실제 위치가 git 폴더 안인지 본다(`locks::remove`). HEAD를 바꾸지 않아 실행기 백업과 journal 줄이 없다. `IndexLocked` 오류의 "정리하기"도 같은 액션이다.
- 남은 맡긴 항목(`parked::leftovers`): stash 목록에서 메시지가 `higit parked <브랜치>`이고 브랜치 이름과 든 파일 경로 검사를 통과한 항목. `recover-parked`(`park`/Safe/`Nothing`)가 맡긴 ref로 옮기고(이미 있으면 그 ref를 `parked-leftover-<브랜치>` 백업으로) 목록에서 뺀다.
- 다른 도구의 작업(`conflict::other_operation`): `rebase-merge`·`rebase-apply`·`CHERRY_PICK_HEAD`·`REVERT_HEAD`. 앱이 끝내지 않고 안내만 한다.

#### 로그 파일 (D-078)
- `core::log::Logger`가 `<앱 데이터>/logs/higit.log`에 쓴다. 1MB가 넘으면 `higit.1.log`·`higit.2.log`로 돌리고, 열 때 14일 지난 파일을 지운다. 파일은 `platform::open_private_append`로 연다(Unix 0600).
- `Core::execute`·`undo`·`redo`가 결과마다 한 줄을 남긴다: 시각·종류·작업·액션 id·오류 코드·git 하위 명령 이름·종료 코드. git 오류 원문, 파일·브랜치 이름, 버전 메시지는 쓰지 않고, 모든 칸은 글자·숫자·`-`·`_`·`.`만 남긴다(줄 위조 방지).
- 앱 데이터 폴더를 못 쓰면 로그 없이 계속한다. 폴더 열기는 `app_open_log_folder`가 Rust에서 opener로 한다.

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
| 악성 저장소를 복제하거나 받은 zip을 연다 | 저장소 설정, 서브모듈, 심볼릭 링크로 명령 실행 | Git 보안 수정 기준 안내(D-050), 신뢰 확인(D-039, D-067), 위험한 설정 끄기 |
| 이상한 브랜치 이름이나 파일 이름 | git 옵션으로 해석되게 만들기 | `--end-of-options`와 `--`, 이름 검사(D-035) |
| 저장소 파일 내용에 섞인 HTML이나 스크립트 | 웹뷰에서 실행되어 IPC 호출 | 이스케이프, CSP, 좁은 capability(D-037) |
| 같은 PC의 다른 사용자나 악성 프로그램 | 토큰, 설정 파일 | OS 키체인, 평문 저장 금지(D-040) |
| 오염된 의존성 | 빌드 결과물에 코드 심기 | lockfile, cargo-deny, 설치 스크립트 제한(D-038) |
| 사용자의 실수 | 비밀 파일을 버전에 넣어 올리기 | 버전 저장을 누르면 넣을 파일을 이름·내용 규칙과 50MB 크기로 검사해 저장을 멈추는 알림 줄(D-040, D-077, `core::secrets`, `query_secret_check`). 결과에는 경로·규칙·줄 번호만 |
| 중간에 끊기는 작업 | 저장소 깨짐, 잠금 파일 남음, 맡긴 파일 유실 | 시간 제한·멈춤 90초·취소, 사람이 확인한 로컬 쓰기 강제 멈춤(D-080), 백업 ref, journal(D-041), 오래된 잠금 정리(D-081), 열 때 끝나지 않은 작업 알림과 남은 맡긴 항목 복구(D-082) |

- 새 Action과 새 IPC 명령을 만들 때 위 표의 해당 줄을 확인한다. 점검 항목은 `.claude/agents/code-analyst.md` "보안 점검".
- 비밀 검사(`Queries::secret_check`)는 넣은 파일(넣은 파일이 없고 `stage_all`이면 고친 파일 전부)을 200개까지, 파일마다 앞 256KB만 본다. 넣은 파일은 index 블롭을, 안 넣은 파일은 작업 폴더 파일을 읽고 링크는 읽지 않는다. 정규식 crate 없이 접두사·글자 종류·길이로 맞추고, 일치한 글자는 결과·로그 어디에도 담지 않는다.
- 로그 파일에는 코드만 남긴다(D-078, 위 "로그 파일").
- 웹뷰가 부를 수 있는 앱 명령은 `build.rs`의 `COMMANDS`와 `capabilities/default.json`에 적은 것뿐이다. 새 명령은 `src/commands.rs`, `src/bindings.rs`와 함께 네 곳에 넣고, `src-tauri/tests/security_config.rs`가 네 곳과 CSP 문자열을 확인한다. dev 서버에는 CSP가 없으므로 CSP 확인은 릴리스 실행 파일로 한다.
- 조회·액션·감시는 `Core::open_project`를 거친 경로만 받는다(`core::access::OpenedRepos`, 아니면 `NotOpened`). 시작 화면 상태 요약은 최근 목록의 경로도 받는다. 신뢰 확인(D-039, `core::trust`)을 거쳐 신뢰하거나 "끄고 열기"를 고른 뒤에만 열린 것으로 기록한다. 실행 설정(hooks 포함, D-067)이 있는 저장소는 `NeedsTrust`를 돌려주고, 신뢰하기 전에는 그 저장소에서 status 같은 git 명령을 돌리지 않는다. 소유자가 다른 폴더(dubious ownership)는 사용자가 고를 때만 그 폴더 하나를 `safe.directory`에 더한다.
- 입력 상한: 경로 4096바이트, 경로 목록 10만 개, 버전 메시지 1MB, 이름 255바이트(맡긴 ref에 쓰는 브랜치 이름은 200바이트), 버전 기록 한 번에 500개, 제외할 파일 한 번에 200개. 합치기 마치기(`ConflictFinishInput`)는 파일 1000개, 직접 고친 글 하나 1MB, 전체 8MB이고, 경로는 남은 충돌 경로에 있는 것만 받는다.
- 화면이 돌려보내는 해시·ref 이름(작업 내역의 되살리기·꺼내기, 임시 보관 꺼내기)은 core가 목록을 다시 계산해 그 안에 있을 때만 받는다.
- IPC 명령 인자는 serde 타입으로 받고 길이와 형식을 검사한다. 파일 경로는 저장소 루트 기준 상대 경로만 받아 백엔드가 정규화하고, 루트 밖이면 거부한다. 심볼릭 링크는 따라가지 않는다.

## 프론트와 백엔드 계약
- 프로젝트 열기는 한 곳: 앱을 켤 때 열기, 최근 목록, 폴더 고르기, 프로젝트 메뉴는 모두 `project_open` 명령 → `Core::open_project` 하나를 거친다(D-032). 저장소 신뢰 확인은 그 함수 안에 넣는다. 화면은 `useOpenProject` 훅 하나로 연다.
- Tauri 명령 이름은 `<영역>_<동사>`. 목록의 원본은 `src-tauri/build.rs`의 `COMMANDS`다. 반복 2까지: `app_status`, `app_open_git_download`, `app_open_log_folder`, `project_open`, `project_trust`, `project_allow_owner`, `recent_list`, `recent_remove`, `action_list`, `action_preview`, `action_execute`, `action_cancel`, `undo_last`, `redo_last`, `query_status`, `query_log`, `query_diff`, `query_branches`, `query_branch_name`, `query_home_map`, `query_secret_check`, `query_merge_hint`, `query_stashes`, `query_recovery`, `query_pending_work`, `query_conflict_state`, `query_conflict_file`, `query_project_summary`, `repo_watch`, `repo_unwatch`.
- 명령 등록과 TS 내보내기는 `src-tauri/src/bindings.rs` 한 파일에 모은다. 바인딩 라이브러리를 바꾸면 이 파일과 `core::view`만 고친다(D-019).
- 액션 입력은 `core::view::ActionInput` enum으로 보낸다(D-020). 액션 id는 enum 태그에서 얻으므로 `action_preview(folder, input)`, `action_execute(folder, input, previewToken)`에는 id 인자가 없다. 입력이 있는 액션은 전용 UI(`src/features/<id>/`)가 입력 구조체를 채운다.
- 실패한 명령의 Promise는 `ErrorView { code, detail }`로 reject된다. 화면은 `src/lib/api.ts`가 바꿔 주는 `ApiError`만 다룬다.
- 타입은 Rust에서 정의하고 `tauri-specta`로 TS 바인딩을 생성한다. 프론트에서 타입을 손으로 쓰지 않는다.
- 이벤트: `repo-changed { repo, worktree, git }`, `watch-status { repo, active, reason }`, `op-progress { repo, action, elapsedSecs, idleSecs }`. 이름은 Rust 상수를 바인딩으로 내보내고(`REPO_CHANGED_EVENT`, `WATCH_STATUS_EVENT`, `OP_PROGRESS_EVENT`), 화면은 `src/lib/events.ts`로만 구독한다. `op-progress`는 사람이 누른 액션의 지켜보는 git 프로세스가 도는 동안 1초마다 온다(위 "진행과 취소"). 저절로 하는 원격 확인은 보내지 않고, 파일 이름이나 원격 주소는 담지 않는다.
- 프론트는 `action_list`로 받은 메타데이터로 버튼, 메뉴, 명령 팔레트(Ctrl+K)를 만든다.

## 용어와 호버 (D-006)
- `src/glossary/ko.ts`가 `docs/spec.md` 용어표를 그대로 옮긴 데이터. `id → { friendly, gitName, command, description }`.
- 화면에서 용어는 항상 `<Term id="commit" />`로 쓴다. 호버(키보드 포커스 포함) 시 툴팁: 원래 이름 · 명령 · 한 줄 설명.
- 설정 "Git 원래 이름으로 보기"가 켜지면 `Term`이 원래 이름을 본문에 보여주고 툴팁에 쉬운 이름을 보여준다.
- 테스트: 레지스트리의 모든 `glossary_id`가 glossary에 있는지 확인하는 테스트를 둔다.

## 버전 기록 그리기 계층 (D-017, 반복 2b)

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
| 로그 폴더(D-078) | 앱 데이터 폴더 아래 `logs` | 같음. 파일 권한 0600 |
| git 끝내기(D-080) | `taskkill /T`, 2초 뒤 `taskkill /T /F` | 새 프로세스 그룹에 SIGTERM, 2초 뒤 SIGKILL |
| 도는 git 찾기(D-081) | `tasklist`의 `git.exe`·`git-*.exe` | `/proc/*/comm`이 `git`이거나 `git-`로 시작 |
| 비밀 저장 | Credential Manager (`keyring` 크레이트) | Secret Service (`keyring` 크레이트) |
| 웹뷰 | WebView2 | WebKitGTK 4.1 |

우클릭 메뉴 등록·해제는 설정 화면 토글로 하고, 설치 프로그램에서는 기본 등록한다.

## 새 액션 추가 절차
1. `docs/spec.md` 용어표에 행 추가(id, 쉬운 이름, 원래 이름, 설명, 위험도).
2. `src/glossary/ko.ts`와 `en.ts`에 같은 항목 추가.
3. 필요하면 `crates/git`의 `GitOp`에 변형 추가 + `CliBackend` 매핑 + 파서 테스트.
4. `core::view::ActionInput`에 변형을 더하고(입력이 있으면 `<Id>Input` 구조체), `crates/core/actions/<id>.rs`에 `Action` 구현, 레지스트리에 등록. `ActionMeta.undo`로 되돌리는 방법을 정한다(되돌릴 수 없으면 `Irreversible`). HEAD를 바꾸지 않으면 `moves_head`를 거짓으로, 다른 ref를 옮기면 `tracked_refs`를, 결과에 따라 journal에 남기는 방식이 다르면 `Outcome::journal_mode`를 고친다. 조회 기능이면 Action 대신 `core::query`에 메서드와 `core::view` 타입을 추가한다(D-010).
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
