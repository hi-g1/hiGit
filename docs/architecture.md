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
│ hosting    HostingProvider + GitHub 구현(로그인·검토 요청), HttpTransport │
│ platform   OS별 코드: git 찾기, 프로세스, 키체인, 앱 실행 파일 위치     │
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
│  ├─ ui/                    공용 부품(Button, Term, Tooltip, DialogFrame(대화상자 틀·키 차단·`composing`) …)
│  ├─ glossary/              ko.ts, en.ts  (용어 원본은 docs/spec.md 표)
│  ├─ lib/                   api.ts(백엔드 호출, invoke는 lib/ 안에서만), events.ts(이벤트 구독), queries.ts(TanStack Query), useRepoWatch.ts, dialog.ts, copy.ts(클립보드), shortcuts.ts·useShortcut.ts(단축키 표와 판정은 여기만, D-056), bindings.ts·action-meta.json(생성물)
│  ├─ messages/              ko.ts, en.ts 화면 문구(copy-rules.test.ts가 문구 규칙 확인)
│  └─ styles/tokens.css      docs/design.md 토큰
└─ src-tauri/               app 크레이트 겸 워크스페이스 루트(D-018)
   ├─ Cargo.toml             [package] app + [workspace] members = ["crates/*"]
   ├─ tauri.conf.json, build.rs, capabilities/, icons/
   ├─ src/                   app: main.rs, lib.rs, commands.rs, bindings.rs(D-019), bin/gen-bindings.rs
   └─ crates/
      ├─ core/              패키지 이름 higit-core(D-012). src/github/(연결 서비스·helper·검토 요청 대상)
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
    fn log_graph(&self, repo: &RepoPath, q: &GraphLogQuery) -> Result<Vec<Commit>>; // 2b: 여러 끝점(해시만)에서 --topo-order, ^제외
    fn tags(&self, repo: &RepoPath) -> Result<Vec<TagRef>>; // 2b: 버전을 가리키는 태그 1000개까지. 태그를 가리키는 태그는 git 버전에 따라 빠진다
    fn commit_stats(&self, repo: &RepoPath, oids: &[Oid]) -> Result<Vec<CommitStat>>; // 2b: 버전별 파일·줄 수(--numstat -z, 해시는 stdin). 2MB·10초 상한(D-096)
    fn count_unpublished(&self, repo: &RepoPath, tip: &Oid, base: Option<&Oid>) -> Result<Option<u32>>; // 3: 어떤 refs/remotes/*에도 없는 조상 수(끝점은 stdin). 원격 끝점 1000개 넘으면 None(모름, D-097)
    fn reachable_from_branches(&self, repo: &RepoPath, oid: &Oid) -> Result<bool>; // 3: 로컬 브랜치 끝에서 닿는지(D-099)
    fn commit_messages(&self, repo: &RepoPath, oids: &[Oid]) -> Result<Vec<CommitMessage>>; // 3: 전체 메시지 100개까지(해시는 stdin, 1MB·10초)
    fn check_tag_name(&self, name: &str) -> Result<bool>; // 3: check-ref-format refs/tags/<이름>, `-` 시작 금지(D-104)
    fn commit_files(&self, repo: &RepoPath, oid: &Oid) -> Result<CommitFiles>; // 3: 그 버전에서 바뀐 파일(diff-tree -z --name-status, 첫 부모나 빈 트리와) 500개까지. diff의 DiffKind::Commit은 같은 기준으로 한 파일(D-105)
    fn rebase_state(&self, repo: &RepoPath) -> Result<Option<RebaseState>>; // 3: 멈춘 기록 정리(rebase-merge/의 head-name·onto·orig-head·남은 할 일·한 일, edit 멈춤인지)
    fn has_identity(&self, repo: &RepoPath) -> Result<bool>; // 3: git var GIT_COMMITTER_IDENT가 되는지(이름·이메일)
    fn excludes_file(&self, repo: &RepoPath) -> Result<Option<PathBuf>>; // 전역 무시 규칙 파일(감시용)
    fn diff(&self, repo: &RepoPath, target: &DiffTarget) -> Result<FileDiff>; // 마지막 버전과 지금을 합쳐 비교
    // 반복 1b: 브랜치·원격
    fn branches(&self, repo: &RepoPath) -> Result<Vec<Branch>>;       // 로컬·원격, 원격 HEAD 별칭 뺌
    fn remotes(&self, repo: &RepoPath) -> Result<Vec<Remote>>;
    // 반복 4: git이 실제로 쓸 받기·올리기 주소들(insteadOf·pushInsteadOf·pushurl 적용, `remote get-url --all [--push]`)
    fn remote_urls(&self, repo: &RepoPath, remote: &str) -> Result<RemoteUrls>;
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
    fn set_helper_program(&self, path: Option<PathBuf>); // 반복 4: credential helper로 다시 실행할 앱 실행 파일(D-111)
    // 반복 2: 충돌·임시 보관·작업 내역
    fn reflog(&self, repo: &RepoPath, limit: u32) -> Result<Vec<ReflogEntry>>;   // HEAD reflog, 최근 것부터
    fn stash_list(&self, repo: &RepoPath) -> Result<Vec<StashEntry>>;           // refs/stash 목록
    fn index_entries(&self, repo: &RepoPath) -> Result<Vec<IndexEntry>>;        // ls-files -s -z. 충돌 중이면 stage 1·2·3
    fn read_blobs(&self, repo: &RepoPath, oids: &[Oid], max_bytes: usize) -> Result<Vec<BlobHead>>; // cat-file --batch 프로세스 둘
    fn merge_segments(&self, repo: &RepoPath, base: &[u8], ours: &[u8], theirs: &[u8]) -> Result<Vec<MergeSegment>>; // 충돌 구간(D-086)
    // 반복 2: 진행·취소·잠금 (D-080, D-081)
    fn request_cancel(&self, repo: &RepoPath);   // 이 저장소에서 사람이 시킨 git을 끝낸다(취소·강제로 멈추기). 저절로 하는 원격 확인은 깃발이 따로라 건드리지 않는다(D-117)
    fn clear_cancel(&self, repo: &RepoPath);     // 액션을 시작·마칠 때 취소 표시를 지운다
    fn set_observer(&self, observer: Option<ProgressObserver>); // 1초마다 (저장소, 경과, 마지막 출력 뒤 시간)
    fn other_git_running(&self) -> bool;         // 앱이 띄우지 않은 git이 시스템에 있는지. 모르면 true
    // 쓰기 작업은 전부 여기로
    fn run(&self, repo: &RepoPath, op: &GitOp) -> Result<OpOutput>;
    fn identity(&self) -> Result<GitIdentity>; // 전역(없으면 시스템) user.name·user.email, 저장소 밖에서 읽음(D-125)
    fn run_global(&self, op: &GlobalOp) -> Result<OpOutput>; // 저장소 밖 전역 설정 쓰기: SetIdentity(D-125)
}
```

- 코드에는 해당 반복에 필요한 메서드만 구현해 넣는다. 위는 반복 2까지 구현한 것이다. 메서드를 더하면 가짜 구현 둘(`executor.rs` 테스트의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)도 함께 고친다.
- `GitOp`(반복 2까지): `Init`, `Stage`, `Unstage`, `StageAll`, `Commit`, `CreateAndSwitch`, `Switch`, `SwitchTrack`, `Park`, `DiscardPaths`, `StashDropIf`, `StashDropOid`, `Unpark`, `UnparkKeep`, `UnparkAbort`, `Fetch`, `MergeFf`, `Merge`, `MergeAbort`, `Push`, `ResetSoft`, `ResetKeep`, `ReadTree`, `ResolveFile`, `ResolveDelete`, `ResolveSide`, `SnapshotFiles`, `Revert`, `AddSafeDirectory`. 반복 3: `Amend`(`--cleanup=whitespace`, 메시지는 stdin), `CherryPickStart`·`RevertStart`(멈추면 정리하지 않음, core가 상태를 봄), `OperationContinue`·`OperationSkip`·`OperationAbort`(`OperationKind`: 복사·되돌리기·기록 정리). `TagCreate`(가벼운 태그, `tag.gpgSign=false`)·`TagDelete`, `BranchDelete`·`BranchRename`, `UnsetUpstream`·`SetUpstream`, `CreateAndSwitchAt`(이름은 core가 검사한 뒤 `-c`의 값으로, 해시는 `--end-of-options` 뒤). `RewriteHistory { base, steps }`(`rebase -i`, 할 일 목록은 해시와 `pick`·`fixup`·`edit`·`drop`만, 상수 편집기 명령 `printf %s "$HIGIT_TODO" >`와 환경 변수로 넘기고 `rebase.*` 설정을 고정, D-100). 반복 4: `Clone { url, name, auth }`(`repo`는 부모 폴더, `clone --progress --no-recurse-submodules --origin origin --end-of-options <주소> <이름>`). `Fetch`·`Push`에 `auth: AuthMode`(`UserDefault` | `Helper`, core가 정함). `Helper`이고 helper 실행 파일이 있으면 그 호출에만 `-c credential.helper=`와 상수 `-c credential.helper=!"$HIGIT_CREDENTIAL_HELPER" --git-credential`, 환경 변수 `HIGIT_CREDENTIAL_HELPER=<경로>`를 붙인다(D-111). 조회 `rebase_state`(멈춘 정리의 `rebase-merge/`), `has_identity`(`git var GIT_COMMITTER_IDENT`). 반복 4b(원격 ref 쓰기, `cli/refpush.rs`): `PushForce { remote, branch, src, expect }`(`--force-with-lease=refs/heads/<브랜치>:<기대 값>`과 `<src>:refs/heads/<브랜치>`, 확인한 버전만 올라감), `PushDelete { expect }`(`:refs/heads/<브랜치>`, 이미 없으면 git이 `(stale info)`), `PushNewRef { oid }`(빈 기대 값 = 원격에 없어야 함), `PushTag { name }`(강제 없음). 값 있는 lease만 쓰고 `--force` 단독·`+` refspec은 없다. 조회 `ls_remote(remote, refname)`(전체 ref 이름이 정확히 같은 줄만)과 `ls_remote_head(remote)`(`ls-remote --symref <원격> HEAD`). 둘 다 네트워크라 `run_network`(Foreground, 10분)로 돈다. 실패 종류 `LeaseStale`·`RemoteRefused`·`RemoteRefExists`·`RemoteRefMissing`은 일반 올리기 거절(`PushRejected`)보다 먼저 가린다.
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
- 원격 작업은 `CliBackend::run_network`로만 한다. hook·서명이 오래 붙잡을 수 있는 로컬 쓰기(`Commit`·`Merge`·`Revert`, 반복 3의 `Amend`·`CherryPickStart`·`RevertStart`·`RewriteHistory`·`OperationContinue`·`OperationSkip`)는 `run_local_watched`로 돌려 진행 알림과 사람이 확인한 강제 멈춤만 받는다. 그 밖의 로컬 조회·쓰기는 끝내지 않는다(D-066, D-080).
- 셸을 거치는 곳은 둘이다(D-100, D-111, D-035의 예외). ① 기록 정리에서 git이 상수 편집기 명령 `GIT_SEQUENCE_EDITOR='printf %s "$HIGIT_TODO" >'`을 셸로 부른다. 사용자 데이터는 명령에 들어가지 않고 환경 변수 `HIGIT_TODO`(검증한 해시와 `pick`·`fixup`·`edit`·`drop`만)에만 든다. 할 일 목록에 `exec` 줄은 없다. ② hiGit credential helper: git이 상수 설정 `credential.helper=!"$HIGIT_CREDENTIAL_HELPER" --git-credential`을 셸로 부른다. 환경 변수에는 앱 실행 파일 경로만 든다(토큰 없음).

| 작업 | 시간 제한 | 끝내는 방법 |
| --- | --- | --- |
| 사용자가 누른 원격 작업(fetch·pull·push·sync) | 처음부터 10분(`NETWORK_TIMEOUT`). 첫 출력 뒤 90초 동안 출력이 없으면 멈춤(`STALL`, `Stalled`). 연결 전(인증 창 대기)에는 10분 제한만. 동기화 버튼의 "취소"(`Cancelled`) | 프로세스 트리째(`ProcessControl`). 멈춤·취소는 먼저 정상 종료(`terminate_tree`), 2초 뒤 강제(`kill_tree`). 10분 제한은 바로 `kill_tree` |
| 복제(D-115) | 처음부터 1시간(`CLONE_TIMEOUT`). 멈춤 90초·취소는 위와 같다. cwd는 부모 폴더, 진행 알림·취소 표시의 키는 대상 폴더. SSH 명령은 환경 변수와 전역·시스템 설정만 본다(부모가 다른 저장소 안이어도 그 설정을 읽지 않음) | 위와 같음 |
| GitHub API(D-110) | 연결 5초·전체 15초, 응답 2MB(`UreqTransport`). `pr-create`는 HTTP 하나라 취소 대상이 아니다(D-123) | 요청을 끊는다 |
| 저절로 하는 원격 확인(D-055) | 60초(`BACKGROUND_TIMEOUT`). 진행 알림·취소 없음 | 같음. 묻는 창을 띄우지 않는 백그라운드 모드 |
| 로컬 쓰기 `Commit`·`Merge`·`Revert`·`Amend`·`CherryPickStart`·`RevertStart`·`RewriteHistory`·이어가기 | 없음. 60초가 지나면 화면이 "응답 없음 · 강제로 멈추기"를 보이고 확인을 받는다(D-080) | 사람이 확인할 때만 멈춤·취소와 같은 방법 |
| 그 밖의 로컬 조회·쓰기 | 없음 | 끝내지 않는다(index를 쓰는 중에 끊으면 깨진다). 남은 잠금 파일은 "멈춘 작업 정리"(D-081) |

- 사용자가 SSH 명령을 정하지 않았으면 `GIT_SSH_COMMAND=ssh -o BatchMode=yes`(D-023). 백그라운드는 `GCM_INTERACTIVE=never`, `credential.interactive=never`, `SSH_ASKPASS_REQUIRE=never`, `gc.auto=0`.
- 믿지 않기로 한 저장소(D-039)는 모든 호출에 실행 설정을 끄는 `-c` 덮어쓰기를 붙인다.
- GitHub 인증(D-111): 원격 작업마다 core가 `AuthMode`를 정한다. git이 실제로 쓸 받기·올리기 주소(`remote_urls`)가 모두 사용자 정보 없는 `https://github.com/…`이고, 저장소 로컬·워크트리 설정에 `http.*`·`remote.*.proxy`가 없고, 이번 실행에 키체인에서 토큰을 읽었거나 썼을 때만 `Helper`(사람이 시킨 작업은 키체인을 읽어 보고, 저절로 하는 확인은 읽지 않는다). `Helper` 호출에만 `-c credential.helper=`와 상수 helper 설정을 붙이고 앱 실행 파일 경로를 `HIGIT_CREDENTIAL_HELPER`로 넘긴다. git이 앱을 `--git-credential get`으로 다시 실행하면 `main`이 창·플러그인보다 먼저 갈라져 `higit_core::git_credential_main`이 답한다: `protocol=https`·`host=github.com`(포트 없음)·사용자 이름 없음일 때만 키체인을 읽어 `username=x-access-token`·`password=<토큰>`을 표준 출력에 쓰고, 그 밖(`store`·`erase`·다른 호스트·입력 8KB·64줄 초과)은 아무것도 쓰지 않고 0으로 끝난다(전체 10초). helper를 쓴 호출이 인증·권한 실패면 토큰을 한 번 확인해 401이면 연결을 지우고 `githubExpired`로 알린다.

### Action (crates/core) — 상태를 바꾸는 기능

```rust
pub trait Action: Send + Sync {
    fn meta(&self) -> &ActionMeta;                       // id, risk, glossary_id, 단축키
    fn available(&self, state: &RepoState) -> Availability; // 지금 쓸 수 있나, 아니면 쉬운 이유
    fn preview(&self, ctx: &ExecCtx, input: &ActionInput) -> Result<Preview>; // 무엇이 바뀌나
    fn execute(&self, ctx: &ExecCtx, input: &ActionInput) -> Result<Outcome>; // ExecCtx = { git, state, key(표식 서명) }
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
  tracked = action.tracked_refs(input)        (refs/heads/*, refs/tags/*, refs/stash)
  before = action.journal_before(UndoState::capture)  (HEAD, 브랜치, index 트리, tracked의 값)   ← undo가 기록하는 종류일 때만. 기본은 그대로, 멈췄다 이어가는 작업이 시작 때 상태로 바꾼다
  result = action.execute()
  journal: result.journal_mode()가
    Journal → is_recorded면 Action 줄(after를 캡처해 before와 같으면 남기지 않음), is_barrier면 Barrier 줄, Nothing이면 없음
    Barrier → Barrier 줄 (맡긴 파일 겹침 해결을 마침)
    Skip    → 없음 (합치기를 시작만 함, 꺼내기, 잠금 정리, 맡긴 항목 복구)
    Always  → 상태가 그대로여도 Action 줄 (제외할 파일)
  (화면 갱신은 파일 변경 감시가 맡는다. 실행기가 이벤트를 보내지 않는다)
```

- `moves_head()`가 거짓인 액션(`discard`, `clear-lock`, `take-out`, `recover-parked`, `stash`, `ignore`, `merge-into`, `conflict-abort`)은 실행기가 HEAD 백업을 만들지 않는다(작업 내역에 쓸모없는 줄이 쌓이지 않게, D-072). 지우거나 덮어쓰는 액션은 스스로 그 대상을 백업 ref로 먼저 남긴다(`discard-<브랜치>`, `conflict-files-<브랜치>`, `restore-<브랜치>`, `sync`).
- 등록한 액션 32개(`actions::register_all` 순서). 반복 2까지: `init`, `stage`, `unstage`, `commit`, `ignore`, `merge`, `merge-into`, `conflict-finish`, `conflict-abort`, `stash`, `take-out`, `discard`, `restore`, `clear-lock`, `recover-parked`, `switch`, `branch-create`, `fetch`, `pull`, `push`, `sync`. 반복 3: `amend`(Caution/`RestoreRef`. `require_branch` → `refuse_during_operation` → `expect_head` = 지금 위치 → 지금 위치가 안 올린 버전 → 충돌 없음 → 메시지·넣은 파일이 그대로면 `NoChange`. 메시지는 stdin, 1MB, journal에는 비운 채로), `revert`·`cherry-pick`(Normal/`RestoreTree`, `actions/pick.rs`. `require_branch` → `refuse_during_operation` → 충돌 없음 → `refuse_staged` → `verified_commit`(되돌리기 `HeadAncestor`, 합친 버전은 `-m 1` / 복사 `BranchReachable`, 합친 버전·지금 위치의 조상은 거절) → 신원 확인(`IdentityMissing`) → 표식 쓰기 → 시작. 끝까지 가면 표식을 지우고 `Reverted`·`Picked`. 멈추면 비었으면 건너뛰고 `NoChange`, 키가 이번 실행만의 것이면 그만두고 오류, 아니면 표식을 두고 `OperationConflicts { context, paths }`(journal 없음)), `reset`(Danger/`RestoreTree`, 되돌릴 수 있어 확인 없음. `require_branch` → `refuse_during_operation` → `verified_commit(HeadAncestor)`, 지금 위치와 같으면 거절 → `refuse_published(HEAD, base = 고른 버전)` → 실행기가 `reset` 백업 → `ResetKeep`. 결과 `Reset { dropped }`), `rebase-i`(Danger/`RestoreTree`, `actions/rewrite.rs`. 입력 `RewriteInput { steps }`는 적용 순서(옛 것부터)의 `pick`·`squash`·`drop`과 묶음 맨 아래 카드의 메시지. 입력 검사(50개, 메시지 16KB·전체 24KB, 첫 단계·빼기 뒤 묶기 금지) → `rewrite_chain` 다시 계산, 해시 집합이 구간과 같아야 함 → 순서·메시지 그대로면 `NoChange` → 신원 → 표식(할 일 목록의 해시 모두와 고친 메시지) → `RewriteHistory`(구간 위 버전은 `pick`으로 이어 붙임). `edit` 멈춤은 `rewrite::drive`가 표식의 메시지로 `Amend` 뒤 이어간다. 충돌이면 표식을 두고 `OperationConflicts`, 충돌이 아닌 멈춤(hook 등)이나 이번 실행만의 키면 그만두고 오류. 결과 `Rewritten { kept, dropped, joined }`), `tag`·`tag-delete`(둘 다 용어 `tag`/Safe/`RestoreRef`, HEAD를 바꾸지 않고 `tracked_refs = [refs/tags/<이름>]`. 이름은 앞뒤 공백을 떼고 `validate_ref_like`·`check_tag_name`, 대소문자만 다른 같은 이름도 `TagExists`. 만들기는 `verified_commit(BranchOrActivity)`, 지우기는 태그가 화면이 본 버전을 가리킬 때만, 주석 태그는 `AnnotatedTag`), `branch-delete`(Caution/`RestoreRef`, HEAD를 바꾸지 않음, `tracked_refs = [refs/heads/<이름>]`. 지금 브랜치·기본 브랜치(`query::default_branch`)·다른 작업 폴더가 쓰는 브랜치는 `BranchProtected` → `refuse_during_operation` → 끝 버전 `branch-delete-<이름>` 백업 → `BranchDelete` → 맡긴 ref를 `parked-<이름>` 백업으로(`parked::retire_to_backup`). Ctrl+Z는 ref를 되살리고 같은 이름의 원격 추적 브랜치가 하나뿐이면 다시 잇는다), `branch-rename`(Normal/`Inverse`, `tracked_refs = [옛, 새]`. 새 이름이 다른 브랜치와 대소문자까지 겹치면 `BranchExists`, 대소문자만 바꾸면 `-M`, 합치는 중·unpark 표식이 있으면 거절 → `BranchRename` → upstream이 있었으면 끊음 → 맡긴 ref를 새 이름으로(`parked::move_ref`). 되돌리기는 `undo_branch_rename`이 이름과 맡긴 ref를 되돌리고 `restore_refs`는 건너뛴다. 다시 하기 없음). `branch-create`는 `from`(노선도의 역·작업 내역의 버전)이 있고 지금 위치와 다르면 `verified_commit(BranchOrActivity)` → `refuse_dirty` → `CreateAndSwitchAt`, 다시 하기도 같은 해시에서 만든다. 반복 4: `clone`(Safe/`Nothing`, D-115. `Core::execute`가 열린 경로 대신 대상 검사·대상 경로 잠금·실패 정리를 하고 액션은 git만 부른다), `pr-create`(Safe/`Nothing`, 용어 `pr`, D-113·D-123. 올리기는 하지 않고 올라가 있지 않으면 `notPushed`, 원격 잠금만, journal 없음). 위험도와 `undo`는 생성물 `src/lib/action-meta.json`에 있다.

#### 잠금 둘
- `Core.busy`(작업 폴더): index·작업 폴더를 쓰는 작업. 같은 저장소에서 둘이 겹치면 index.lock이 부딪히므로 나중 것은 `Busy`로 거절한다.
- `Core.net_busy`(원격): 원격과 통신하는 작업. 원격 확인(fetch)과 올리기(push)는 이것만 잡아 버전 저장을 막지 않는다. pull·sync와 `clear-lock`(앱의 다른 작업이 하나도 없을 때만 지운다, D-081)은 둘 다 잡고, 되돌리기·다시 하기는 작업 폴더 잠금을 잡는다.
- 저절로 하는 원격 확인이 원격 잠금을 쥐고 있으면(`Core.bg_fetch`) 사람이 누른 원격 작업은 거절하지 않고 끝날 때까지 기다린다(최대 `BACKGROUND_TIMEOUT` + 5초, D-068). 그 밖의 겹침은 바로 `Busy`.
- 잠금은 저장소 경로별 `HashSet`이고, `BusyGuard`가 오류여도 푼다. 프로세스 사이 잠금은 git의 index.lock에 맡긴다.
- `Core.running`(저장소 → 액션 id와 마지막 진행 알림의 경과): 저절로 하는 원격 확인을 뺀 액션과 되돌리기·다시 하기(`undo`·`redo`)가 도는 동안 들어 있다. 진행 알림과 취소가 이 표를 본다. 시작과 끝에 같은 잠금 안에서 `clear_cancel`을 불러 끝난 작업 뒤에 취소 표시가 남지 않는다.

#### 진행과 취소 (D-080)
- `Core::set_progress_sink`가 `GitBackend::set_observer`에 관찰자를 걸고, `Core.running`에 있는 저장소의 알림만 `view::OpProgress { repo, action, elapsed_secs, idle_secs }`로 바꿔 넘긴다. 앱은 이를 `op-progress` 이벤트로 보낸다(`src-tauri/src/lib.rs`).
- 알림은 지켜보는 git 프로세스(원격 작업과 `Commit`·`Merge`·`Revert`, 복제)가 도는 동안 1초부터 1초마다 나가고, 화면은 3초가 지나면 "취소"를 보인다. 저절로 하는 원격 확인은 보내지 않는다. 페이로드에 파일 이름이나 원격 주소를 담지 않는다.
- 취소 깃발은 (저장소, 사람이 시킨 작업 | 저절로 하는 확인)으로 나뉜다(D-117). 복제는 대상 폴더 경로가 키이고, `Core::cancel(폴더)`는 그 경로와 정확히 같을 때만(열린 폴더가 아니어도) 복제를 취소한다.
- `action_cancel` → `Core::cancel`은 원격 작업(fetch·pull·push·sync·clone)이면 언제든, 로컬 쓰기면 진행 알림의 경과가 60초를 넘었을 때만 `request_cancel`을 부른다(화면만 믿지 않는다). 화면은 원격 작업에 "취소"(3초 뒤)를 두고, 로컬 쓰기는 60초가 지나면 "강제로 멈추기"를 확인 뒤 같은 명령으로 부른다(`features/progress`).
- `Cancelled`는 오류 줄 없이 토스트 "취소했어요", `Stalled`는 안내 줄 "진행이 멈춰 그만뒀어요"와 동기화 단추.

### Query (crates/core) — 조회 (D-010)

```rust
pub struct Queries { /* GitBackend를 품는다 */ }
impl Queries {
    pub fn status(&self, repo: &Path) -> Result<view::StatusView>;            // 반복 0
    pub fn log(&self, repo: &Path, q: view::LogQuery) -> Result<view::LogPage>; // 반복 0. 2b: 범위(All·Current), 끝점 해시 스냅샷(최대 201개, 약 8KB라 Windows 명령줄 한도 안), 이름표, 첫 쪽만 지금 위치·기본 브랜치·안 올림(D-088)
    pub fn diff(&self, repo: &Path, r: &view::DiffRequest) -> Result<view::FileDiffView>; // 반복 1a. 경로는 저장소 안만. 링크는 대상만(D-052)
    pub fn branches(&self, repo: &Path) -> Result<view::BranchListView>;        // 반복 1b. 맡긴 파일 수, 기본 브랜치
    pub fn check_branch_name(&self, repo: &Path, name: &str) -> Result<view::BranchNameCheck>; // 반복 1b
    pub fn home_map(&self, repo: &Path) -> Result<view::HomeMapView>;          // 반복 1b. 홈 두 줄 노선도(D-060)
    pub fn route_preview(&self, repo: &Path, limit: u32) -> Result<view::HomeMapView>; // 2b: 시작 화면 노선도. 홈과 같은 요약, 지금 줄 limit개(최대 20). 접근은 project_summary와 같은 summary_root(D-093)
    pub fn project_summary(&self, repo: &Path) -> Result<view::ProjectSummaryView>; // 시작 화면
    pub fn secret_check(&self, repo: &Path, r: &view::SecretCheckRequest) -> Result<view::SecretCheckView>; // 반복 2. 경로·규칙·줄 번호만(D-077)
    pub fn merge_hint(&self, repo: &Path) -> Result<view::MergeHintView>;      // 반복 2. 방향 하나(D-048, D-071)
    pub fn commit_detail(&self, repo: &Path, oid: &str) -> Result<view::CommitDetailView>; // 반복 3. 버전 상세(메시지 64KB, 첫 부모와 비교한 파일 500개). `diff`는 `DiffRequest.commit`이 있으면 그 버전 안의 비교(블롭만, 작업 폴더와 무관). 해시는 둘 다 `verified_commit(BranchOrActivity)`
    pub fn rewrite_plan(&self, repo: &Path, oids: Vec<String>) -> Result<view::RewritePlanView>; // 반복 3. 기록 정리 카드(새 것 먼저, 전체 메시지), 막힌 까닭, 옛 버전에 남을 이름표 수. 50개까지
    pub fn conflict_state(&self, repo: &Path) -> Result<view::ConflictStateView>; // 반복 2. 맥락 Merge·Unpark(반복 3: CherryPick·Revert·Rebase), 충돌 파일 200개까지. 명령은 `Core::conflict_state`를 거쳐 앱 작업이 돌면 `Busy`(화면은 받은 상태를 둔다)
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
- `UndoState = { head, branch, parked: [(브랜치, stash 해시)], index_tree, refs: [(ref 이름, 해시 또는 없음)] }`. `refs`는 액션의 `tracked_refs`가 채우고 `is_valid`는 `refs/heads/*`·`refs/tags/*`(이름 규칙을 지킨 것)와 `refs/stash`만 받는다.
- 반복 2의 새 필드(`refs` 등)는 비어 있으면 직렬화하지 않는다. 옛 줄을 다시 직렬화해도 같은 글자라 서명이 그대로 맞는다(`VERSION`은 1, D-072). 새 `kind`를 모르는 옛 앱은 그 줄을 버린다.
- `input`에는 `ActionInput::journal_copy()`를 넣는다. 경로 목록은 200개까지, 합치기 마치기의 직접 고친 글은 빼고 선택 종류만 남긴다. 되돌리기·다시 하기는 입력이 아니라 `before`/`after`로 한다.
- 넣기·빼기 한 번마다 `capture`가 두 번 돈다(git 6회 안팎). 넣기·빼기 되돌리기는 토스트 없이 Ctrl+Z로만 한다(D-072).
- 되돌리기(Ctrl+Z, 토스트 "되돌리기")는 마지막 경계 뒤의 되돌리지 않은 Action 줄을 `kind`대로 되돌리고 `undo` 줄을 덧붙인다. 다시 하기(Ctrl+Shift+Z)는 같은 버전을 ref로 되살린다(D-065. 버전을 새로 만들지 않는다). 종류별 되돌리기를 마치면 `before.refs`의 값으로 다른 ref를 되돌린다(다시 하기는 `after.refs`). 그사이 바뀌었으면 실패한다.
- 합치기·리베이스·cherry-pick·revert 중이면 되돌리기·다시 하기 모두 `OperationInProgress`로 거절한다(`refuse_during_operation`).
- 되돌리기 전에 지금 HEAD·브랜치가 줄의 `after`와 같은지 확인한다. 다르면(앱 밖에서 git을 썼음) `UndoStale`로 거절하고, 화면은 토스트의 "작업 내역" 단추로 안내한다.
- 버전을 뒤로 옮기는 되돌리기(`RestoreRef`·`RestoreTree`)는 옮기기 전에 지금 버전을 백업 ref(`undo-<브랜치>`)로 남긴다. 백업을 못 쓰면 되돌리지 않는다.
- "올린 버전"은 어떤 원격 추적 브랜치(`refs/remotes/*`, upstream만이 아님)에 닿는 버전이다. 원격 끝점이 1000개를 넘어 판정하지 못하면 올린 것으로 본다(`guard::is_published`, D-097).
- 다시 하면 버전이 기록에서 빠지는 액션(`amend`·`reset`·`rebase-i`)은 다시 하기 전에 빠질 버전(`after`에 없고 지금 있는 것)이 하나라도 올라갔으면 `PublishedHistory`로 거절한다.
- 되돌리면 이미 올린 버전이 빠지는 경우(`RestoreRef`·`RestoreTree`)에는 기록을 지우지 않고 그 변경을 취소하는 새 버전을 만든다(`GitOp::Revert`, 합치기 버전은 `-m 1`, D-073). 되돌릴 버전이 `before` 바로 위가 아니면 `UndoPushed`, 넣은 파일이 있으면 `StagedChanges`. 만든 뒤 경계 줄을 덧붙여 그 앞 줄과 다시 하기를 닫고, `UndoResult.reverted`로 화면이 "되돌리는 새 버전을 만들었어요"를 띄운다.
- 브랜치별로 맡겨 둔 고친 파일은 `refs/higit/parked/<브랜치>`의 stash 커밋이다(D-047). 브랜치 이름의 `/`는 `%2F`로 적는다(D-064). 일반 stash 목록과 섞지 않는다(만든 뒤 `StashDropIf`로 뺀다). 꺼낸 맡긴 ref는 지우기 전에 백업 ref(`parked-<브랜치>`)로 옮긴다.
- 백업 ref는 늘 `undo::backup::backup_oid`로 만든다. `refs/higit/`를 쓰거나 지우는 코드는 core의 `undo`, `actions/switch`(맡기기), `actions/stash`·`actions/conflict`(꺼낸 맡긴 ref 지우기), `actions/recover_parked`(남은 항목을 맡긴 ref로), 그리고 `parked`의 `retire_to_backup`·`move_ref`(브랜치 지우기·이름 바꾸기와 그 되돌리기가 부른다)에만 둔다. ref 이름 규칙은 `parked`(`ref_name`)와 `undo::backup`(`ref_name`)에 있다.

#### 기록을 바꾸는 액션의 검사 (`core::guard`, D-097, D-099)
- 화면이 준 해시·이름은 낡거나 조작될 수 있어 core가 다시 확인한다. 순서는 `require_branch`(특정 버전을 보는 중이면 거절) → `refuse_during_operation` → `verified_commit(oid, Scope)`(전체 해시이고 커밋인지, `HeadAncestor`·`BranchReachable`·`BranchOrActivity`) → 상태 검사(`refuse_staged`: 넣은 파일 → `StagedChanges`, `refuse_dirty`: 추적 중인 파일을 고쳤거나 넣은 파일 → `HasChanges`, 새 파일은 둠, `refuse_published(tip, base)`: `base`에 없고 `tip`에 있는 버전이 하나라도 올라갔으면 → `PublishedHistory`).
- 새 오류 코드: `PublishedHistory`, `HasChanges`, `TagExists`, `BranchProtected`, `NoChange`(하려는 결과가 지금과 같음. `NothingToCommit`과 따로).
- 올림 검사 규칙(`guard::PublishedRule`, D-127·D-129): `published_rule(allow)`가 앱 설정 "올린 버전도 바꾸기"와 지금 브랜치로 정한다. 꺼졌거나, 특정 버전을 보는 중이거나, 지금 브랜치나 그 upstream이 기본 브랜치(`guard::protected_branch`: `main`·`master`, `init.defaultBranch`, 로컬에 기록된 원격 HEAD, 네트워크 없음)면 `Refuse`(지금 동작), 아니면 `Allow`. `Allow`는 `refuse_published`·`is_published`·`rewrite_chain`의 `Published` 막힘만 건너뛰고 나머지 검사·백업 ref·journal은 그대로다. 고치기·돌아가기·기록 정리·그 다시 하기와 기록 정리 카드 지도 조회(`Core::rewrite_plan`)가 같은 규칙을 쓴다. 되돌리기(Ctrl+Z)는 규칙과 무관하다.
- 덮어쓰며 올리기(`actions/force_push.rs`, D-128, `Irreversible`·`danger`·원격 잠금): 지키는 것 넷. ① 설정 스위치 ② 확인 토큰(입력 `{ remote, branch, expectRemote, expectHead }` 전체에 묶임, 이름은 core가 upstream에서 다시 구해 비교) ③ 원격 기본 브랜치 거절(로컬 `protected_branch`와 실행 때 서버 `ls_remote_head`, 읽지 못하면 거절) ④ 확인 창이 본 원격 끝에만 거는 lease(`GitOp::PushForce`). 그 밖에 HEAD·추적 ref가 기대 값과 같고 덮어쓸 것이 있어야 하며, 원격 옛 끝을 백업 ref(`force-push-<브랜치>`)로 남긴 뒤 올린다(못 남기면 올리지 않음). 성공하면 표식을 지우고 경계 줄을 남긴다. 확인 창 값은 `Core::force_push_plan`(사라질 버전 200개까지, 다른 사람 = 작성자 이메일이 저장소·전역 `user.email`과 다름, 모르면 모두). 원격 작업 목록 `REMOTE_ACTIONS`는 core 상수를 바인딩으로 내보내 화면이 그대로 쓴다.
- 원격 브랜치 지우기·이름 바꾸기(`actions/remote_branch.rs`, D-130, `Irreversible`·`danger`, 지우기는 원격 잠금·이름 바꾸기는 둘 다): 원격은 목록에 있고 추적 ref는 목록에서 정확히 같은 이름만, 값이 확인 창이 본 `expectTip`, 원격 기본 브랜치·`HEAD`는 막는다(로컬 규칙과 서버 `ls_remote_head`, 모르면 막음). 끝 버전을 백업 ref로 남긴 뒤 값 있는 lease로 지운다(`(stale info)`면 서버에 다시 물어 이미 없으면 성공). 이름 바꾸기는 새 이름이 추적 ref·서버 모두 없고 옛 이름이 서버에서 본 값일 때 빈 lease로 새 이름을 올리고 옛 이름을 지운 뒤 로컬 upstream을 다시 잇는다. 옛 이름 지우기부터 실패하면 되돌리지 않고 `RemoteRenamePartial`. 로컬 상태를 바꾸지 않아 journal 줄을 남기지 않는다.
- GitHub에 올리기(`actions/publish.rs`, D-132, `normal`, 잠금 둘 다): 지금 브랜치·버전 있음·원격 없음·이름(`[A-Za-z0-9._-]` 1~100자)·설명(350바이트)·공개면 `acknowledgePublic`·연결됨을 본 뒤 `create_repo`. 응답 소유자가 연결된 로그인과 같아야 하고, `origin` 주소는 검사한 소유자·이름으로 core가 `https://github.com/<소유자>/<이름>.git`을 만들어 `GitOp::RemoteAdd`(`remote add --end-of-options`). 올리기는 화면이 이어서 `push`. 만든 뒤 잇기가 실패해도 저장소는 지우지 않는다. 422 이름 중복은 `GithubRepoExists`, 그 밖은 `GithubRepoInvalid`. 저장소 페이지는 `HostingPage::Repository`.
- 태그 올리기(`actions/tag_push.rs`, D-131, `normal`·되돌리기 없음·확인 없음·원격 잠금): 태그 이름 검사(D-104)와 그 태그가 버전을 가리키는지 본 뒤 `pick_remote`로 고른 원격에 `GitOp::PushTag`(강제 없음). 같은 이름이 다른 버전으로 있으면 `RemoteRefExists`.
- "올린 기록을 바꿨다" 표식(`core::rewrote`, D-129): `Allow`로 올린 버전의 기록을 바꾸기 **직전에**(`guard::touches_published`가 참일 때) git 폴더의 서명 표식 `higit/rewrote.json`(`core::marker`, journal 키, git 폴더 실제 경로에 묶임)에 브랜치·upstream·그때 upstream 끝(`old_tip`)을 적는다. 못 적으면 바꾸지 않는다. 표식이 있으면 `sync`는 원격이 앞섰을 때(갈라짐·돌아가기 뒤 앞으로 감기 모두) 받지 않고 `RewrotePublished`로 멈추고, `SyncInput.mergeAnyway`("그래도 합치기", 거짓이면 직렬화하지 않아 옛 journal 줄 서명이 맞음)면 받는다. `pull`은 늘 멈춘다. HEAD가 `old_tip`을 품게 되거나 upstream이 바뀌면 저절로 지우고, 브랜치 지우기·덮어쓰며 올리기 성공도 지운다. 설정을 꺼도 표식은 남아 보호한다.
- 기록 정리 구간(`guard::rewrite_chain`, D-100, D-106): 지금 위치부터 기록을 1001개까지 읽어 고른 버전이 첫 부모 선 위의 연속 구간인지 본다. 구간 위·안에 합친 버전이 있으면 `HasMerge`(부모가 하나뿐인 버전만 이어지면 그 사이에 곁가지가 없다), 첫 버전이 들어 있으면 `IncludesFirstVersion`, 그 밖에 `Detached`·`OtherOperation`·`TooMany`(50개 또는 지금 위치까지 1000개)·`NotOnBranchLine`·`Published`(바탕 ~ 지금 위치)·`HasChanges`. 구간을 찾았으면 막혀도 카드를 준다. 조회와 액션이 같은 함수를 쓰고 액션은 실행 때 다시 계산한다.
- 작업 내역 안의 버전인지(`guard::in_activity`)는 reflog 200줄과 버전 모양 백업 ref로 본다. `restore`도 같은 함수를 쓴다.

#### 작업 내역 (D-085)
- 화면(`screens/Activity`)은 `query_recovery`가 주는 `RecoveryView { items }`를 보여준다. `core::recovery::collect`가 다섯 원천을 최신 먼저 200개로 합친다: journal의 Action 줄(되돌린 줄은 `undone`, 서명이 맞지 않는 줄은 `restore: None`), HEAD reflog(같은 해시·2초 안의 journal 줄과 겹치면 뺌), 백업 ref, 맡긴 ref, 임시 보관 목록(맡겨 두기가 잠깐 만든 항목은 뺌).
- 서명이 맞지 않는 journal 줄은 최근 50개만 보이고(`restore: None`), 같은 순간의 reflog 줄을 가리지 못한다. 모든 원천을 시각순으로 200개로 자른 뒤 남은 백업 ref에만 `kind_of`(git log)를 부른다.
- 꺼내기(`take-out`: 임시 보관·백업 ref·맡긴 ref)는 stash 모양 버전에 든 경로가 모두 상대 경로이고 `.git`(Windows 변형 포함)이 아닐 때만 꺼낸다(`parked::refuse_unsafe`).
- 백업 ref의 종류는 이름이 아니라 객체 모양으로 가른다(`recovery::kind_of`). 둘째 부모의 메시지가 `index on `으로 시작하면 `Changes`(꺼내기), 아니면 `Version`(이 시점으로 되돌리기).
- `Version`은 `restore`(`reflog`/Safe/`RestoreTree`): 지금 브랜치를 `ResetKeep`으로 옮기고 그 전에 지금 버전을 `restore-<브랜치>` 백업 ref로 남긴다. `Changes`는 `take-out`(`TakeOutSource::Stash { oid }`, `Ref { refname }`): 백업 ref는 남기고, 맡긴 ref는 꺼낸 뒤 백업으로 옮겨 지운다. core는 받은 해시·ref가 reflog·백업 ref·목록에 있는지 다시 확인한다.
- 정리: 프로젝트를 열고 신뢰 확인을 마친 뒤 `undo::backup::prune`이 이름의 시각이 30일(`KEEP_DAYS`) 지난 백업 ref를 한 번에 20개(`PRUNE_AT_ONCE`)까지 지운다(D-079). 시각을 못 읽는 ref와 맡긴 ref는 두고, 정리가 실패해도 열기는 성공이다.

#### 끝나지 않은 작업 (D-041, D-081, D-082)
- `Core::pending_work`(`query_pending_work`, `view::PendingWorkView { stale_locks, leftover_parked, other_operation }`)는 journal이 아니라 저장소에 남은 흔적으로 알아낸다. 앱의 작업이 그 저장소에서 돌고 있으면(`repo_busy`) 잠금과 맡긴 항목은 비워 둔다. 합치는 중이면 화면은 알림 줄 대신 충돌 해결 화면을 연다.
- 오래된 잠금(`core::locks`): git 폴더(작업 트리별·공용) 최상위와 공용 폴더 `refs/` 아래(깊이 6, 500개까지)의 일반 파일 `*.lock`. 모두 수정한 지 60초(`STALE_AFTER`)가 지났고 앱이 띄우지 않은 git 프로세스가 없을 때(`GitBackend::other_git_running`: platform의 `running_git_pids`에서 runner가 기록한 앱의 git pid를 뺀다)만 돌려준다. 하나라도 어긋나면 빈 목록이다. 지우기 직전(`locks::remove`)에 링크·이름·위치·나이를 다시 본다.
- `clear-lock`(`lock`/Caution/`Irreversible`, 잠금 둘 다): 확인 토큰을 받은 뒤 다시 조사해 같은 조건일 때만 지운다. 지우기 직전에 링크가 아니고 이름이 `.lock`으로 끝나며 실제 위치가 git 폴더 안인지 본다(`locks::remove`). HEAD를 바꾸지 않아 실행기 백업과 journal 줄이 없다. `IndexLocked` 오류의 "정리하기"도 같은 액션이다.
- 남은 맡긴 항목(`parked::leftovers`): stash 목록에서 메시지가 `higit parked <브랜치>`이고 브랜치 이름과 든 파일 경로 검사를 통과한 항목. `recover-parked`(`park`/Safe/`Nothing`)가 맡긴 ref로 옮기고(이미 있으면 그 ref를 `parked-leftover-<브랜치>` 백업으로) 목록에서 뺀다.
- 다른 도구의 작업(`conflict::other_operation`): `rebase-merge`·`rebase-apply`·`CHERRY_PICK_HEAD`·`REVERT_HEAD`. 앱이 끝내지 않고 안내만 한다.
- 앱이 시작한 복사·되돌리기·기록 정리(`core::operation`, D-101): 시작 전에 `OpMarker { kind, branch, head_before, base, oids, started_at }`를 표식으로 쓴다. `verify_state`가 맞을 때만 앱 작업으로 본다: 복사·되돌리기는 같은 브랜치에서 `CHERRY_PICK_HEAD`·`REVERT_HEAD`가 `oids[0]`, 기록 정리는 `rebase-merge/`의 `head-name`·`orig-head`·`onto`가 표식의 브랜치·`head_before`·`base`. 맞으면 충돌 맥락 `Operation`이고 `pending_work`·`conflict_state`의 "다른 도구의 작업"에서 뺀다(`conflict::foreign_operation`). 기록 정리는 git의 stage 2(정리된 기록)·3(옮기는 버전)을 바꿔 담아(`conflict_paths(entries, flip)`) 화면의 "내 쪽"이 옮기는 버전이다. 이름표: 복사·되돌리기는 내 쪽 = 지금 브랜치, 합칠 쪽 = 고른 버전, 기록 정리는 내 쪽 = `REBASE_HEAD`, 합칠 쪽 = 지금 HEAD(이름표는 브랜치). 충돌 화면의 `conflict-finish`·`conflict-abort`는 복사·되돌리기 맥락이면 고른 것을 쓴 뒤 `verify_state`를 한 번 더 보고 `OperationContinue`(해결 결과가 지금과 같으면 `OperationSkip`, `NoChange`), 그만두기는 충돌 파일 백업 뒤 `OperationAbort`, 둘 다 끝나면 표식을 지운다. 쪽을 뒤집은 맥락에서는 `ResolveSide`의 git 쪽도 뒤집는다. 기록 정리 맥락의 마치기는 먼저 `verify_state`와 `verify_todo`(남은 할 일 줄이 모두 `pick`·`fixup`·`drop`·`edit` + 표식의 해시 앞 7자 이상, `exec`·`break`·`label`·`reset`·`merge`·`update-ref` 등이 있으면 `OperationInProgress`)를 통과해야 고른 것을 쓰고 `OperationContinue`를 부른다. `edit`으로 멈춘 채면(앱이 꺼졌던 경우) 표식의 메시지로 고쳐 이어가고, 해결 결과가 비면 `OperationSkip`. 다시 겹치면 `ConflictContinued { paths }`(journal 없음), 끝나면 `ConflictFinished`이고 journal의 `before`는 `journal_before`가 표식의 `head_before`·브랜치로 바꾼다. 그만두기는 충돌 파일과 추적 중인 모든 변경(1000개)을 백업한 뒤 `OperationAbort`. 멈춘 작업 없이 남은 표식은 `pending_work`가 앱 작업이 돌지 않을 때만 지운다(`operation::tidy`).
- 앱이 시작한 작업의 표식(`core::marker`, D-102): `<git 폴더>/higit/<이름>.json`(`unpark`, `operation`) = `{ v, dir: git 폴더 실제 경로, body: 내용 JSON 글자, sig }`. 서명은 journal 키로 `higit-marker`·`v`·이름·`dir`·`body`에 건다. 서명이 없거나 틀리거나 `dir`이 지금 실제 경로와 다르면 없는 것으로 본다(저장소 폴더를 옮기면 진행 중 표식은 "다른 도구의 작업"으로 떨어진다). `higit` 폴더나 표식이 링크면 쓰지도 읽지도 않고, 256KB를 넘으면 읽지 않는다. 키는 `ExecCtx.key`(액션)와 `Queries`(조회)가 같은 것을 받는다. 앱 데이터 폴더를 못 써 키가 이번 실행만의 것이면(`JournalKey::is_persistent`가 거짓) 멈출 수 있는 작업은 충돌 때 바로 그만둔다(계획 008 E11~).

#### 로그 파일 (D-078)
- `core::log::Logger`가 `<앱 데이터>/logs/higit.log`에 쓴다. 1MB가 넘으면 `higit.1.log`·`higit.2.log`로 돌리고, 열 때 14일 지난 파일을 지운다. 파일은 `platform::open_private_append`로 연다(Unix 0600).
- `Core::execute`·`undo`·`redo`가 결과마다 한 줄을 남긴다: 시각·종류·작업·액션 id·오류 코드·git 하위 명령 이름·종료 코드. git 오류 원문, 파일·브랜치 이름, 버전 메시지는 쓰지 않고, 모든 칸은 글자·숫자·`-`·`_`·`.`만 남긴다(줄 위조 방지).
- 앱 데이터 폴더를 못 쓰면 로그 없이 계속한다. 폴더 열기는 `app_open_log_folder`가 Rust에서 opener로 한다.

### HostingProvider (crates/hosting)

```rust
pub trait HostingProvider: Send + Sync {          // 블로킹. core가 작업 스레드에서 부른다
    fn id(&self) -> &'static str;                  // "github"
    fn detect(&self, url: &str) -> Option<RemoteRepo>; // 호스트가 정확히 github.com일 때만
    fn begin_login(&self, scope: LoginScope) -> Result<LoginChallenge, HostingError>; // device flow 시작(Repo | RepoWorkflow, D-135)
    fn poll_login(&self, c: &LoginChallenge) -> Result<LoginPoll, HostingError>;
    fn current_user(&self, token: &SecretString) -> Result<HostUser, HostingError>;
    fn list_prs(&self, token: &SecretString, repo: &RemoteRepo, q: &PrQuery) -> Result<PrPage, HostingError>;
    fn pr_detail(&self, token: &SecretString, repo: &RemoteRepo, number: u64) -> Result<PrDetail, HostingError>;
    fn create_pr(&self, token: &SecretString, repo: &RemoteRepo, new: &NewPr) -> Result<PullRequest, HostingError>;
    fn create_repo(&self, token: &SecretString, new: &NewRepo) -> Result<CreatedRepo, HostingError>; // POST /user/repos, 빈 저장소(D-132)
    fn list_repos(&self, token: &SecretString, page: u32, per_page: u32) -> Result<RepoPage, HostingError>; // 50개씩(D-133)
}
// pr_detail은 리뷰 요약(사람마다 마지막 승인·변경 요청, 100개 넘거나 볼 수 없으면 None)과 댓글 수를 함께 읽는다(D-134).
// LoginPoll::Granted { token, workflow }: 받은 권한에 workflow가 들었는지만 본다.

pub trait HttpTransport: Send + Sync {             // 실제는 UreqTransport, 시험은 testing::FakeTransport
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, TransportError>;
}
```
- `GitHub::new(transport, GitHubConfig)`. 주소는 설정 상수(`https://github.com`, `https://api.github.com`)만 쓰고 서버가 준 URL을 열거나 따라가지 않는다(`verification_uri`는 상수와 같은지만 본다). 리디렉트는 `api_base` 바로 아래일 때만 같은 요청으로 3번까지.
- `UreqTransport`: https만, 상태 코드를 오류로 바꾸지 않음, 리디렉트 0, 연결 5초·전체 15초·본문 2MB, 환경 변수 프록시, rustls(ring) + OS 인증서 저장소. `Authorization: Bearer`는 `HttpRequest.bearer`에서만 만든다. 요청·응답·오류의 `Debug`·`Display`에 토큰과 본문이 없다.
- `HostingError`는 종류(`HostingErrorKind`)와 앞 200자 설명. core가 한 함수(`github::from_hosting`)에서 `ErrorCode`로 바꾸고 서비스가 쥔 토큰·`device_code`를 설명에서 지운다.

### GitHub 연결 (core::github, D-109·D-111·D-121)
- `GitHubService`: 상태(`github.json`만 읽음), device flow(간격 안 폴링은 네트워크 없음, 만료는 시작 시각 + `expires_in`), 승인되면 `current_user` → 키체인(`platform::Keychain`, 서비스 `dev.higit.desktop`, 계정 `github.com`) → `github.json` 순으로 저장, 키체인을 못 쓰면 이번 실행에만 메모리. 401은 그 토큰 세대일 때만 연결을 지운다. `call(|provider, token| …)`이 토큰 읽기와 401 처리를 한 곳에서 한다.
- `auth_for(git, repo, remote, background)`: 받기·올리기 주소가 모두 사용자 정보 없는 `https://github.com/…`, 저장소 로컬·워크트리 설정에 `http.*`·`remote.*.proxy` 없음, 이번 실행에 키체인에서 읽었거나 쓴 토큰(저절로 하는 확인은 키체인을 새로 읽지 않음)일 때만 `AuthMode::Helper`.
- helper(`git_credential_main`): `main`이 첫 인자 `--git-credential`이면 창·플러그인보다 먼저 갈라진다. `get`이고 `protocol=https`·`host=github.com`·사용자 이름 없음일 때만 키체인을 읽어 표준 출력으로 답한다(입력 8KB·64줄, 10초).
- 검토 요청 대상(`github::pr::pr_target`): 지금 브랜치 upstream의 원격(없으면 하나뿐·origin)의 올리기 주소가 github.com일 때만. head는 upstream 원격 브랜치 이름.

## 보안 (D-034)
| 공격자나 사고 | 노리는 것 | 주된 대책 |
| --- | --- | --- |
| 악성 저장소를 복제하거나 받은 zip을 연다 | 저장소 설정, 서브모듈, 심볼릭 링크로 명령 실행 | Git 보안 수정 기준 안내(D-050), 신뢰 확인(D-039, D-067), 위험한 설정 끄기 |
| 이상한 브랜치 이름이나 파일 이름 | git 옵션으로 해석되게 만들기 | `--end-of-options`와 `--`, 이름 검사(D-035) |
| 저장소 파일 내용에 섞인 HTML이나 스크립트 | 웹뷰에서 실행되어 IPC 호출 | 이스케이프, CSP, 좁은 capability(D-037) |
| 같은 PC의 다른 사용자나 악성 프로그램 | 토큰, 설정 파일 | OS 키체인, 평문 저장 금지(D-040) |
| 오염된 의존성 | 빌드 결과물에 코드 심기 | lockfile, cargo-deny, 설치 스크립트 제한(D-038) |
| 토큰 노출(반복 4) | 화면·로그·오류·명령줄·주소·환경 변수에 GitHub 토큰이 새기 | 토큰은 OS 키체인과 `SecretString`에만, 화면 타입에 칸이 없음, `Debug`가 가림, 로그는 코드만, 오류 설명은 정확한 글자 지우기 + `redact`, git으로는 helper의 표준 출력으로만(D-040, D-109, D-111) |
| 악성 저장소가 토큰을 다른 호스트로 보내게 함 | `insteadOf`·`pushurl`·`http.proxy`·`http.sslVerify=false`로 helper 답을 가로채기 | git이 실제로 쓸 주소가 모두 github.com https이고 저장소 로컬 `http.*`·`remote.*.proxy`가 없을 때만 helper, helper는 `host=github.com` 정확 일치·사용자 이름 없음일 때만 키체인을 읽음(D-111, D-118) |
| 서버가 준 URL | 앱이 피싱 주소를 열거나 따라가게 만들기 | 여는 페이지는 core가 상수와 검사한 값으로 만들고(`app_open_hosting_page`는 주소 인자가 없음), API 리디렉트는 같은 API 주소 아래만(D-110, D-122) |
| 사용자의 실수 | 비밀 파일을 버전에 넣어 올리기 | 버전 저장을 누르면 넣을 파일을 이름·내용 규칙과 50MB 크기로 검사해 저장을 멈추는 알림 줄(D-040, D-077, `core::secrets`, `query_secret_check`). 결과에는 경로·규칙·줄 번호만 |
| 받은 저장소에 심어 둔 멈춘 작업(`rebase-merge/` 할 일 목록의 `exec`, `CHERRY_PICK_HEAD`, 가짜 `operation.json`) | 앱의 "마치기"로 명령을 실행하게 만들기 | 앱이 시작한 작업만 이어간다: 서명한 표식(journal 키, 저장소 실제 경로에 묶임, D-102) + git 상태 일치(`verify_state`) + 남은 할 일 줄이 모두 `pick`·`fixup`·`drop`·`edit` + 표식의 해시(`verify_todo`)일 때만 `--continue`. 그 밖은 "다른 도구의 작업"으로 안내만 |
| 올린 기록을 바꾸는 실수 | 다음 동기화가 옛 버전을 다시 들여와 같은 변경이 둘 | 고치기·돌아가기·기록 정리와 그 다시 하기가 모든 원격 추적 브랜치 기준으로 빠질 버전을 확인(판정 못 하면 거절, D-097) |
| 중간에 끊기는 작업 | 저장소 깨짐, 잠금 파일 남음, 맡긴 파일 유실 | 시간 제한·멈춤 90초·취소, 사람이 확인한 로컬 쓰기 강제 멈춤(D-080), 백업 ref, journal(D-041), 오래된 잠금 정리(D-081), 열 때 끝나지 않은 작업 알림과 남은 맡긴 항목 복구(D-082) |

- 새 Action과 새 IPC 명령을 만들 때 위 표의 해당 줄을 확인한다. 점검 항목은 `.claude/agents/code-analyst.md` "보안 점검".
- 비밀 검사(`Queries::secret_check`)는 넣은 파일(넣은 파일이 없고 `stage_all`이면 고친 파일 전부)을 200개까지, 파일마다 앞 256KB만 본다. 넣은 파일은 index 블롭을, 안 넣은 파일은 작업 폴더 파일을 읽고 링크는 읽지 않는다. 정규식 crate 없이 접두사·글자 종류·길이로 맞추고, 일치한 글자는 결과·로그 어디에도 담지 않는다.
- 로그 파일에는 코드만 남긴다(D-078, 위 "로그 파일").
- 웹뷰가 부를 수 있는 앱 명령은 `build.rs`의 `COMMANDS`와 `capabilities/default.json`에 적은 것뿐이다. 새 명령은 `src/commands.rs`, `src/bindings.rs`와 함께 네 곳에 넣고, `src-tauri/tests/security_config.rs`가 네 곳과 CSP 문자열을 확인한다. dev 서버에는 CSP가 없으므로 CSP 확인은 릴리스 실행 파일로 한다.
- 조회·액션·감시는 `Core::open_project`를 거친 경로만 받는다(`core::access::OpenedRepos`, 아니면 `NotOpened`). 시작 화면 상태 요약은 최근 목록의 경로도 받는다. 신뢰 확인(D-039, `core::trust`)을 거쳐 신뢰하거나 "끄고 열기"를 고른 뒤에만 열린 것으로 기록한다. 실행 설정(hooks 포함, D-067)이 있는 저장소는 `NeedsTrust`를 돌려주고, 신뢰하기 전에는 그 저장소에서 status 같은 git 명령을 돌리지 않는다. 소유자가 다른 폴더(dubious ownership)는 사용자가 고를 때만 그 폴더 하나를 `safe.directory`에 더한다.
- 입력 상한: 경로 4096바이트, 경로 목록 10만 개, 버전 메시지 1MB, 이름 255바이트(맡긴 ref에 쓰는 브랜치 이름은 200바이트), 버전 기록 한 번에 500개(끝점 해시 201개), 줄 수 조회 한 번에 100개, 제외할 파일 한 번에 200개. 합치기 마치기(`ConflictFinishInput`)는 파일 1000개, 직접 고친 글 하나 1MB, 전체 8MB이고, 경로는 남은 충돌 경로에 있는 것만 받는다. 반복 3: 기록 정리 50개(구간 위까지 1000개), 고친 메시지 하나 16KB·전체 24KB, 고치기 메시지 1MB, 태그 이름 255바이트, 표식 파일 256KB, 버전 상세 메시지 64KB·파일 500개. 반복 4: 복제 주소 2048바이트·폴더 이름 255바이트·대상 경로 200자, 검토 요청 제목 256바이트·본문 64KB·목록 한 쪽 30개(hosting 상한 50), helper 입력 8KB·64줄, GitHub 응답 2MB·연결 5초·전체 15초, client id 영숫자 8~64자.
- 화면이 돌려보내는 해시·ref 이름(작업 내역의 되살리기·꺼내기, 임시 보관 꺼내기)은 core가 목록을 다시 계산해 그 안에 있을 때만 받는다. 반복 3의 액션(되돌리기·복사·돌아가기·고치기·기록 정리·태그·새 브랜치의 시작)이 받는 해시도 `core::guard`가 다시 확인한다(D-099). 꺼내기는 stash 모양 버전의 경로가 모두 안전할 때만 한다.
- IPC 명령 인자는 serde 타입으로 받고 길이와 형식을 검사한다. 파일 경로는 저장소 루트 기준 상대 경로만 받아 백엔드가 정규화하고, 루트 밖이면 거부한다. 심볼릭 링크는 따라가지 않는다.

## 프론트와 백엔드 계약
- 프로젝트 열기는 한 곳: 앱을 켤 때 열기, 최근 목록, 폴더 고르기, 프로젝트 메뉴는 모두 `project_open` 명령 → `Core::open_project` 하나를 거친다(D-032). 저장소 신뢰 확인은 그 함수 안에 넣는다. 화면은 `useOpenProject` 훅 하나로 연다.
- Tauri 명령 이름은 `<영역>_<동사>`. 목록의 원본은 `src-tauri/build.rs`의 `COMMANDS`다. 반복 2까지: `app_status`, `app_open_git_download`, `app_open_log_folder`, `project_open`, `project_trust`, `project_allow_owner`, `recent_list`, `recent_remove`, `action_list`, `action_preview`, `action_execute`, `action_cancel`, `undo_last`, `redo_last`, `query_status`, `query_log`, `query_log_stats`, `query_route_preview`, `query_diff`, `query_branches`, `query_branch_name`, `query_home_map`, `query_secret_check`, `query_merge_hint`, `query_stashes`, `query_recovery`, `query_pending_work`, `query_conflict_state`, `query_conflict_file`, `query_project_summary`, `repo_watch`, `repo_unwatch`. 반복 3: `query_rewrite_plan`, `query_commit_detail`. 반복 4b: `settings_get`, `settings_set`(앱 설정 `settings.json`, 고정 칸 패치, D-127. 다듬기에서 언어·글자 크기·화면 모드·용어 설명·줄 바꿈 칸, D-148, 칸 나눔선 폭 `workspacePane`·`conflictPane`·`filesPane` rem, D-164), `query_force_push_plan`(덮어쓰며 올리기 확인 창의 값, 네트워크 없음, D-128), `query_remote_branch_plan(repo, remote, branch)`(원격 브랜치 지우기·이름 바꾸기 확인 창의 값, 네트워크 없음, D-130), `query_publish_draft(repo)`(GitHub에 올리기 대화상자의 처음 값, D-132), `query_github_repos(page)`(내 GitHub 저장소 목록 한 쪽, 저장소 없음, D-133), `github_connect_start(workflow)`(워크플로 권한을 함께 요청할지 bool 하나, D-135, 상태 파일 `workflow`). 다듬기: `app_display_hints`(OS 글자 크기·배율·다크, 인자 없음, Core 없이 돎, D-147). 다듬기(UX 높음, 010-H4 앞당김): `query_identity`(전역 이름·이메일, 저장소 없음, D-125)와 액션 `identity-set`(저장소 없이 `Core::execute`가 따로 부름, 로그에는 id·결과 코드만). 반복 4(11개): `app_open_hosting_page`, `github_status`, `github_connect_start`, `github_connect_poll`, `github_connect_cancel`, `github_disconnect`, `query_clone_defaults`, `query_hosting_context`, `query_pr_list`, `query_pr_detail`, `query_pr_draft`. 모두 `async` + `blocking()`(`app_display_hints`는 Core 없이 blocking 스레드)이고 인자에 주소·토큰이 없다. 저장소를 받는 것은 열린 저장소만.
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

## 화면 배율과 설정 시작 (D-146, D-147, D-149, D-153, D-162)
- 크기 토큰은 모두 rem이다. 루트 글자 크기 = 16px × 배율이고, 배율은 `src/lib/fontScale.ts`의 `computeUiScale`이 OS 본문 px / 14 × OS 글자 배율 × 설정 단계(작게·보통·크게·아주 크게)로 계산해 하한(가장 작은 글자 12px)~2.0으로 자른다. OS 값은 `app_display_hints`가 준다(Ubuntu는 배율을 곱하지 않음, D-162).
- 첫 화면 전에 `src/lib/preload.ts`의 `preloadDisplay`가 설정과 OS 값을 각각 0.8초까지 기다려 `applyDisplay`(`src/lib/display.ts`)로 루트 글자 크기·`lang`·`data-theme`과 `uiStore`(배율·언어·용어 설명)를 정한다. 늦게 온 값과 이후 바뀐 값은 `useDisplaySync`가 같은 함수로 적용한다. Ctrl+=·Ctrl+-·Ctrl+0은 설정의 글자 크기 단계를 바꾼다(`useFontShortcuts`, 웹뷰 자체 확대는 꺼 둠, D-152).
- 노선도처럼 SVG를 px로 그리는 곳은 장면을 단위(배율 1)로 배치하고 그릴 때 `useUiScale()`을 곱한다(D-153). 화면 언어는 `locale.ts`만 `navigator.language(s)`를 읽고, 시각은 `time.ts`와 `<Time>`만 `Intl.DateTimeFormat`을 쓴다.
- 화면 공용 부품: 긴 이름은 `ui/Truncate`(잘렸을 때만 툴팁), 불러오는 표시는 `useDelayedFlag`(300ms)와 `<Loading>`, 포커스 테두리는 `base.css`의 전역 `:focus-visible` 하나, 칸 폭은 `ui/Splitter`와 `usePaneWidth`(설정에 rem으로 저장), 확인은 `ui/ConfirmDialog`(열린 동안 `useShortcut`이 쉰다), 오류 토스트는 `showError`.

## 버전 기록 그리기 계층 (D-017, 반복 2b)

버전 기록·홈·시작 화면 노선도는 같은 배치와 그리기로 만든다(D-017, D-090). 데이터와 그리기가 섞이지 않게 층을 나눈다.

```
query_log (LogPage, 쪽마다)                데이터: 버전·부모(위상 순서), 브랜치·태그 이름표, 첫 쪽의 끝점·지금 위치·안 올림
query_home_map / query_route_preview      홈·시작 화면: 요약(HomeMapView)
   ↓
features/history/model.ts                 화면 모델: toRouteNodes(쪽들) → RouteNode[]
features/history/layout/horizontal.ts     homeToRouteNodes(요약) → RouteNode[] (갈라진 곳·main 끝·접힌 구간·고친 파일)
   ↓
layout/simplify.ts                        확대·축소 접기(모든 버전 / 주요 버전 / 태그만, D-087)
   ↓
layout/lanes.ts                           레인 배치 assignLanes: 0번 레인은 기본 브랜치, 색 키(D-091)
   ↓
layout/route.ts  verticalScene           세로 장면(버전 기록): 행 48·레인 24·굵기(D-089)
layout/horizontal.ts layoutHorizontal     가로 장면(홈 home, 시작 화면 preview·mini)
   ↓
layout/scene.ts (Scene) → render/Renderer → SvgRenderer   그리기: 색 키는 palette(branch·mono)가 CSS 색으로
```

- `Renderer`는 장면과 `palette`, 보이는 영역(`viewport`)만 받는다. 색은 tokens.css의 `--graph-1`~`--graph-6`을 읽어 쓴다. 비싼 레인 배치(`buildRoute`)는 쪽·단계가 바뀔 때만, 장면은 줄 수가 올 때 다시 만든다.
- 버전 기록 목록(`RouteList`)은 행 높이가 고정된 가상 스크롤이다. DOM 행은 보이는 것 ± 10행과 고른 행뿐이고 SVG도 보이는 구간만 그린다. 포커스는 목록 하나(`listbox`, `aria-activedescendant`)가 받는다.
- 줄 수(선 굵기)는 보이는 버전만 `query_log_stats`로 150ms 뒤 100개씩 묻는다(`stats.ts`). 기록 무효화는 `invalidateLog`로 첫 쪽만 다시 받는다(useInfiniteQuery는 모든 쪽을 다시 받으므로).
- 클릭·호버 판정은 DOM이 아니라 배치 결과 좌표로 한다. 그래야 Canvas로 바꿔도 상호작용 코드가 그대로다.
- 확대·축소 단계(모든 버전 / 주요 버전 / 태그만)는 백엔드가 걸러 주지 않는다. 받은 버전을 화면의 순수 함수가 접는다(D-087). `LogQuery`는 단계를 받지 않는다.
- 홈 두 줄 노선도와 시작 화면의 큰·작은 노선도도 같은 배치 함수와 Renderer로 그린다. 홈 요약(`HomeMapView`)은 어댑터가 노선도 노드로 바꾼다(D-090, D-093). 시작 화면 노선도는 상태 요약이 성공한 프로젝트만, 줄이 보일 때, 동시에 3개까지 부른다.
- 역 메뉴(D-092): 항목은 `features/history/menu/items.tsx`의 표 하나(id, 이름표(`ReactNode`, 용어는 `<Term focusable={false}>`), 묶음, `availability(역 정보, 화면 상태)`, `run(역 정보, ctx)`)이고 고른 역은 `StationInfo` 하나로 넘긴다. 화면 상태 `MenuEnv`는 지금 브랜치·고친 파일 수·넣은 파일 수·`operation`("other"면 다른 도구의 작업 중)·upstream·기본 브랜치·확대 단계. `run`은 `menu/run.ts`의 `runAction(ctx, input, { done, onError })`으로 core 액션을 부른다: 성공이면 `ctx.onDone(문구)`(되돌리기 단추), `…Conflicts` 결과면 충돌 상태를 다시 읽어 작업 화면이 충돌 해결 화면을 열고, 실패는 오류 코드 문구(올린 기록 등은 작업 내역 단추), 끝나면 기록·상태·브랜치·작업 내역·정리 계획 조회를 새로 읽는다. 대화상자가 필요한 항목은 `ctx.ask(StationDialog)`(`menu/ask.ts`)로 열고 버전 기록 화면이 `StationDialogs`로 그린다. 액션 입력에 담긴 해시는 core가 다시 확인한다(D-085와 같은 방식). 액션 항목의 `actionId`는 `action-meta.json`에 있어야 한다(테스트).

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
| 비밀 저장 | 자격 증명 관리자 (`keyring-core` + `windows-native-keyring-store`) | Secret Service (`keyring-core` + `zbus-secret-service-keyring-store`, 순수 Rust) |
| 글자 크기·다크 읽기(D-147, `display_hints`) | 본문 14px. `%SystemRoot%\System32\reg.exe query HKCU\Software\Microsoft\Accessibility /v TextScaleFactor`(100~225%). 다크는 읽지 않음(WebView2가 스스로 따름) | 본문 15px. 글자 배율은 WebKitGTK가 Xft.dpi로 이미 화면 전체에 반영하므로 곱하지 않는다(D-162). 다크만 `/usr/bin/gsettings get org.gnome.desktop.interface color-scheme`. 둘 다 절대 경로·1.5초·출력 256바이트까지, 실패하면 배율 100·다크 모름 |
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
- 버전 1만 개 저장소에서 버전 기록 첫 화면 300ms 이내(가상 스크롤, 200개씩 불러오기). 반복 2b 측정(Ubuntu, 릴리스, `perf_ten_thousand_versions`): 첫 쪽 40ms, 깊은 쪽(9800번째부터) 36ms, 25쪽 이어 받기 906ms, 줄 수 100개 10ms. 화면 배치(1만 행, 모든 버전): `buildRoute` 23ms, 장면 10ms.
- 파일 변경 감시는 300ms 디바운스, `.git/` 내부 변경은 별도 처리.
