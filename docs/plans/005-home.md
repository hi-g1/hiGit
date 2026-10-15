# 005 홈 화면과 1b 남은 일의 순서
- 관련 결정: D-042~D-049(홈 화면, 확정). D-043이 D-007을 대체. D-025 갱신 줄. D-034, D-035, D-037~D-041(보안, 확정). D-036(제안), D-017(제안). D-031(노선도 옆 글자 색), D-033(단축키 표). 기대는 확정 결정: D-003, D-010, D-011, D-019~D-024, D-026, D-027, D-029, D-032.
- 관련 명세: spec.md "화면 구성"의 작업 공간·버전 기록·복구 행, 용어표의 `fetch`·`pull`·`push`·`branch`·`switch`·`commit`·`stash`, "오류 메시지 원칙".
- 참고 자료(결정 아님): `docs/research/quick-actions.md`, `docs/research/img/flow4/r-home.png`, `r-note.png`, `r-saved.png`, `r-local.png`, `r-branch.png`, `r-newbranch.png`, `r-switched.png`, `r-mergehint.png`, `r-log.png`. HTML 원본 `docs/research/mockups/home/`. 목업의 초록 "지금 여기" 글자는 D-031과 어긋나므로 따르지 않는다.
- 앞선 계획: 001(1b 남은 단계 6, 8, 9, 10 브랜치 조회, 10b, 12, 13, 19, 20, 다시 만드는 18), 003(시작 화면, 완료), 004(보안 1b). 이 계획은 홈 단계(H1~H18)를 새로 쓰고, 001·004의 남은 단계와 하나의 순서로 합친다.

## 목표
홈에서 버전 저장, 브랜치 갈아타기, 새 브랜치를 확인 창 없이 바로 하고 Ctrl+Z로 되돌린다. 버전 이름은 파일 목록 위에 문서 제목처럼 쓰고, 비워 두면 제안 이름으로 저장한다. 동기화 버튼 하나로 가져오고 올리며, 원격 확인은 저절로 한다. 위쪽 두 줄 노선도로 지금 위치가 보인다.

## 현재 상태
이 계획이 기대는 사실만 적는다.
- core 실행기(`crates/core/src/executor.rs`)는 위험도 `Caution` 이상만 미리보기 토큰을 확인하고 실행을 `NotYetSupported`로 거부한다(D-011). `ActionMeta`에 `undo`가 없다(`action.rs:23`). journal과 백업 ref는 없다.
- 등록된 액션: `init`, `stage`, `unstage`, `commit`(`actions/mod.rs:31`). `GitOp`는 `Init`, `Stage`, `Unstage`, `Commit`(`git/src/types.rs:143`). `GitBackend`에 `branches`, `resolve_ref`, `update_ref`가 없다(`backend.rs`).
- `Runner`는 git 출력을 모두 메모리에 읽고 시간 제한이 없다(`runner.rs:47`, `:67`). 공통 환경에 `PAGER`, `GIT_EDITOR`, `GIT_ALLOW_PROTOCOL`이 없다(004-1). `--end-of-options`를 쓰는 곳이 없다(004-3).
- 저장소 신뢰 확인은 빈 함수다(`bootstrap.rs:197`). IPC 명령은 화면이 준 아무 경로나 연다(`src-tauri/src/commands.rs:124-155`, 004-10).
- 화면: `Workspace.tsx`는 상단 바 + 왼쪽 탭(`LeftTabs`) "변경 사항 | 버전 기록" + 왼쪽 아래 `CommitBox`(입력칸 하나, 메시지가 비면 꺼짐) + 오른쪽 비교. `TopBar.tsx`는 프로젝트 메뉴, 브랜치 표시(누를 수 없음), 올릴·가져올 수 글자. 단축키 표(`src/lib/shortcuts.ts`)에는 `commit`(Ctrl+Enter), `openFolder`(Ctrl+O)만 있다. Radix는 `dropdown-menu`, `tooltip`만 쓴다.
- 이 PC와 Ubuntu 22.04 기본 git은 2.34.1이다. `merge-tree --write-tree`는 2.38부터다. 코드의 `MIN_GIT_VERSION`은 2.30이다.

## 완료 기준 (1b 전체)
- Ubuntu 22.04(git 2.34.1)와 Windows 11에서 아래가 모두 된다(수동, 두 OS).
- 고친 파일을 고르고 이름을 비운 채 Ctrl+Enter를 누르면 "화면.tsx 외 2개 고침" 같은 제안 이름으로 저장되고, 아래에 "저장했어요 · 되돌리기 · Ctrl+Z"가 뜬다. Ctrl+Z를 누르면 버전이 사라지고 고친 파일과 이름·설명이 그대로 돌아온다. Ctrl+Shift+Z로 다시 저장된다.
- 이름 칸에 글자를 쓰는 중에 Ctrl+Z를 누르면 글자만 되돌려진다.
- Ctrl+B로 브랜치 목록을 열고 다른 브랜치를 고르면 묻지 않고 갈아탄다. 고친 파일은 떠난 브랜치에 맡겨지고(새 파일 포함), 돌아오면 다시 나온다. Ctrl+Z로 갈아타기 전 상태가 된다.
- 찾기 칸에 없는 이름을 쓰고 Enter를 누르면 새 브랜치로 갈아타고 고친 파일이 따라온다. 한글 조합 중 Enter는 무시된다.
- 상단 바 오른쪽은 "동기화"와 "버전 기록"뿐이다. 동기화를 누르면 가져온 다음 올린다. 다른 사람 버전과 파일이 겹치지 않으면 저절로 합쳐서 올리고, 겹치면 합치기를 시작하지 않고 "겹친 파일 N개"를 보여준다. 이 판정은 git 2.34.1에서도 된다.
- 원격 확인은 몇 분마다 저절로 되고, 질문 창(SSH, Git Credential Manager)이 뜨지 않으며, 실패는 동기화 버튼 옆 작은 표시로만 보인다. 네트워크가 멈춰도 정해진 시간 뒤에 git 프로세스 트리가 끝난다.
- 홈 위쪽에 main과 지금 브랜치 두 줄 노선도가 보이고, 노선도 옆 글자는 `--text`·`--text-muted`다.
- 004의 완료 기준을 모두 만족한다.
- 두 OS CI 초록. `pnpm check:full` 통과. code-analyst 점검 위반 0건.

## 홈 단계
각 단계 = 커밋 하나. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`나 Tauri 명령이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`. 새 Tauri 명령은 004-9 뒤라면 `build.rs`와 capability에 함께 넣는다. git 호출은 004-3의 `args.rs`로 리비전을 붙인다. 실행 순서는 아래 "합친 순서"를 따른다.

### 되돌리기 바탕

H1. git: ref 읽기·쓰기와 git 폴더 (`git: add ref read/write, ancestry and git dir`)
- 목표: 되돌리기와 맡겨 두기에 필요한 ref 조작을 `GitBackend`에 둔다.
- 바꿀 파일: `crates/git/src/backend.rs`, `crates/git/src/cli/mod.rs`, `crates/git/src/cli/refs.rs`(신규), `crates/git/src/types.rs`(`Oid`), core 테스트의 가짜 백엔드들(`executor.rs` 테스트, `crates/core/tests/common`).
- 내용:
  - `resolve_ref(repo, name) -> Result<Option<Oid>>`: `rev-parse --verify -q --end-of-options <name>^{commit}`.
  - `update_ref(repo, name, new, expected_old: Option<Oid>)`, `delete_ref(repo, name, expected_old)`: `update-ref`에 이전 값을 함께 넘겨 그사이 바뀌었으면 실패한다. 이름은 `refs/`로 시작해야 하고 core가 만든 이름만 받는다.
  - `git_dir(repo) -> PathBuf`: `rev-parse --absolute-git-dir`. 작업 트리가 `.git` 파일인 경우(worktree, submodule)도 맞는 폴더를 준다.
  - `is_ancestor(repo, a, b) -> bool`: `merge-base --is-ancestor`. 종료 코드 0·1만 본다.
  - `count_between(repo, a, b) -> (u32, u32)`: `rev-list --left-right --count a...b`. 두 숫자를 탭으로 나눈 출력만 파싱한다.
  - `merge_base(repo, a, b) -> Option<Oid>`.
- 테스트: 실제 git 임시 저장소. 없는 ref → `None`, 이전 값이 다르면 `update_ref` 실패, 삭제 후 `resolve_ref`가 `None`, worktree에서 `git_dir`, 갈라진 두 브랜치의 `count_between`과 `merge_base`, `-x` 같은 이름이 옵션으로 읽히지 않음.
- 완료 기준: `cargo test -p git` 두 OS 통과.
- Windows/Ubuntu: `--absolute-git-dir`는 Windows에서 `C:/…` 형식을 준다. core가 `platform::normalize_repo_path`로 맞춘다.

H2. core: `UndoKind`, journal, 백업 ref (`core: add undo kind, journal and backup refs to executor`)
- 목표: 실행기가 D-042·D-043 흐름을 갖는다. 이 단계에서는 아직 되돌리는 동작이 없다.
- 바꿀 파일: `crates/core/src/action.rs`(`ActionMeta.undo`), `crates/core/src/undo/{mod.rs, journal.rs, backup.rs}`(신규), `crates/core/src/executor.rs`, `crates/core/src/actions/*.rs`(각 메타에 `undo`), `crates/core/src/view/mod.rs`(`ActionMetaView.undo`), 생성물, `docs/architecture.md` "되돌리기와 UndoJournal"·실행기 흐름.
- 내용:
  - `UndoKind`는 architecture.md의 여섯 가지에 `Nothing`을 더한다. `Nothing`은 되돌릴 것이 없는 동작(원격 확인)이고 journal에 남기지 않는다. 메타: `init` Nothing, `stage`·`unstage` RestoreIndex, `commit` RestoreRef.
  - 실행기: 미리보기 토큰 확인을 `risk >= Caution`에서 `undo == Irreversible`로 바꾼다(D-043). `Caution` 이상 거부(D-011)는 그대로 둔다. 1b에 `Caution` 액션은 없다.
  - journal: `<git_dir>/higit/journal.jsonl`. 줄마다 `{ v: 1, id, at, action, input, before, after, kind }`. 되돌리기·다시 하기는 기존 줄을 고치지 않고 `{ v: 1, undo: id }`, `{ v: 1, redo: id }` 줄을 덧붙인다. 마지막 줄이 깨졌으면(쓰다 끊김) 그 줄만 버린다. 쓸 때 `fsync`.
  - 1b에서 journal에 남기는 것은 되돌리기가 구현된 액션(`commit`, `switch`, `branch-create`)과 `sync`뿐이다. `stage`·`unstage`는 RestoreIndex가 반복 2라 남기지 않는다. 그래도 되돌리기 검증(H3)이 HEAD와 브랜치를 보므로 순서가 꼬이지 않는다.
  - `before`·`after`는 `UndoState { head: Option<Oid>, branch: Option<String>, parked: Vec<(String, Oid)> }`. 실행기가 실행 전후에 `resolve_ref("HEAD")`와 현재 브랜치를 읽고, 액션이 `Outcome`으로 맡겨 둔 oid를 더한다.
  - 백업 ref: `refs/higit/backup/<UTC 시각>-<순번>/<HEAD 이름>`. `Caution` 이상 실행 전과 모든 되돌리기 전에 만든다. 30일 정리는 반복 2.
  - journal 내용은 믿지 않는 입력이다(D-034). 읽을 때 ref 이름은 `validate_ref_like`(004-4)를, oid는 16진수 40·64자를 검사한다.
- 테스트: 가짜 백엔드로 `commit` 실행 → journal 한 줄, `before.head`·`after.head`가 맞음. `Nothing` 액션은 줄이 없음. `Irreversible` 가짜 액션은 토큰 없이 `PreviewRequired`. 깨진 마지막 줄을 무시. 백업 ref 이름 형식. 이름에 `-x`가 든 journal 줄은 무시.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 `undo`.
- Windows/Ubuntu: journal 경로는 `git_dir` 결과에 붙인다. 줄바꿈은 두 OS 모두 `\n`으로 쓴다.

H3. core: 버전 저장 되돌리기와 다시 하기 (`core: undo and redo last commit`)
- 목표: Ctrl+Z가 쓸 core 입구와 Tauri 명령을 만든다. 첫 대상은 버전 저장.
- 바꿀 파일: `crates/git/src/types.rs`(`GitOp::ResetSoft { oid }`), `crates/git/src/cli/mod.rs`, `crates/core/src/undo/run.rs`(신규), `crates/core/src/bootstrap.rs`(`Core::undo`, `Core::redo`), `crates/core/src/view/undo.rs`(신규 `UndoResult`), `crates/core/src/error.rs`, `view` `ErrorCode`(`nothingToUndo`, `undoStale`, `undoPushed`), `src-tauri/src/commands.rs`(`undo_last`, `redo_last`), `bindings.rs`, 생성물, `src/lib/api.ts`, `src/messages/{ko,en}.ts`.
- 내용:
  - `Core::undo(folder)`: 저장소 잠금을 잡고, 되돌리지 않은 마지막 journal 줄을 꺼낸다. 지금 HEAD와 브랜치가 그 줄의 `after`와 다르면 `undoStale`("앱 밖에서 바뀌었어요"). 같으면 백업 ref를 만들고 종류대로 되돌린 뒤 `undo` 줄을 덧붙인다.
  - RestoreRef(버전 저장): `after.head`가 upstream에 들어 있으면(`is_ancestor`) `undoPushed`로 막는다(D-042, 올린 기록을 바꾸지 않음). 아니면 `reset --soft <before.head>`. 첫 버전이라 `before.head`가 없으면 `delete_ref(refs/heads/<브랜치>, after.head)`. 고친 파일과 index는 그대로다.
  - `UndoResult { action, kind, branch, message: Option<String> }`. 버전 저장을 되돌리면 그때 쓴 메시지를 돌려줘 화면이 이름 칸에 다시 채운다.
  - `Core::redo(folder)`: 마지막 줄이 `undo` 표시이고 그 뒤 새 줄이 없을 때만. 그 줄의 input으로 액션을 실행기에 다시 넣고 `redo` 줄을 덧붙인다. 다른 작업을 하면 다시 하기는 사라진다.
- 테스트: 실제 git. 저장 → undo → log에 없고 status에 같은 파일이 넣은 상태로 남음 → redo → 같은 메시지의 버전. 첫 버전 undo. 저장 뒤 터미널에서 다른 버전 저장 → `undoStale`. 올린 뒤 undo → `undoPushed`. undo 뒤 새 저장 → redo는 `nothingToUndo`.
- 완료 기준: `cargo test --workspace`, `pnpm typecheck`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음. 사용자의 `post-rewrite` 등 hook은 `reset --soft`에서 돌지 않는다.

H4. 용어: 동기화와 맡겨 두기 (`glossary: add sync and park terms`)
- 목표: 화면에 쓸 새 용어를 먼저 넣는다(D-015, D-045).
- 바꿀 파일: `docs/spec.md` 용어표, `src/glossary/types.ts`(`TERM_IDS`), `src/glossary/ko.ts`, `src/glossary/en.ts`.
- 내용: 아래 "용어 변경"의 두 행을 `push` 다음과 `stash` 다음에 넣는다.
- 테스트: spec 동기화 테스트, `TERM_IDS` 순서 테스트, 문구 규칙 테스트(괄호 없음, D-027).
- 완료 기준: `pnpm test`.
- Windows/Ubuntu: 차이 없음.

H5. 단축키 표 넓히기 (`lib: add home shortcuts to the shortcut table`)
- 목표: quick-actions.md 표를 D-033 표에 넣고 겹침을 테스트로 막는다.
- 바꿀 파일: `src/lib/shortcuts.ts`, `src/lib/shortcuts.test.ts`, `src/lib/useShortcut.ts`.
- 내용:
  - 항목 더하기: `undo` Ctrl+Z, `redo` Ctrl+Shift+Z, `branches` Ctrl+B, `sync` Ctrl+Shift+S, `history` Ctrl+H, `nextFile` ↓, `prevFile` ↑.
  - 항목마다 `scope: "global" | "fileList"`와 `textField: "app" | "native"`를 둔다. ↑↓는 `fileList`(파일 목록에 포커스가 있을 때만). `undo`·`redo`는 `native`: 입력칸·글상자·`contenteditable`에 포커스가 있으면 처리하지 않고 기본 동작(글자 되돌리기)에 맡긴다(D-042). 나머지 Ctrl 조합은 지금처럼 입력칸 안에서도 동작한다.
  - 겹침 테스트를 범위별로 한다. 계획 중인 Ctrl+K, Ctrl+Shift+O 주석도 겹침 검사 대상 목록에 넣는다.
  - 웹뷰·OS 예약 키 목록 테스트: WebView2·WebKitGTK가 쓰는 키(Ctrl+F, Ctrl+G, Ctrl+P, Ctrl+R, F5, Ctrl+Plus·Minus·0, Ctrl+Shift+I·J·C, F12, Ctrl+U, Alt+←·→)와 겹치면 실패한다.
- 테스트: 위 두 테스트, `native` 키가 글상자 포커스에서 `preventDefault`되지 않음, `fileList` 키가 목록 밖에서 무시됨, 한글 상태(`key`가 `ㅠ`)에서 Ctrl+B 판정.
- 완료 기준: `pnpm test`. 두 OS 수동: 아래 키가 웹뷰 기본 동작 없이 앱에 온다. Ctrl+H(Chrome 계열의 기록, 일부 입력칸의 지우기), Ctrl+Shift+S(Edge의 웹 캡처), Ctrl+B. 막히면 키를 바꾸고 표만 고친다.
- Windows/Ubuntu: WebView2와 WebKitGTK가 각각 가로채는 키가 다르다. 글상자의 다시 하기는 Windows가 Ctrl+Y, Ubuntu가 Ctrl+Shift+Z에 익숙하다(Q5).

H6. 화면: 되돌리기 토스트와 Ctrl+Z (`features: undo toast and ctrl+z`)
- 목표: 바로 한 일의 결과와 "되돌리기"를 화면 아래에 잠깐 띄운다.
- 바꿀 파일: `src/ui/Toast.tsx`·`Toast.module.css`(신규, `@radix-ui/react-toast`), `src/features/undo/{useUndo.ts, UndoToast.tsx, undoStore.ts}`(신규), `src/screens/Workspace/Workspace.tsx`, `src/lib/api.ts`(`undoLast`, `redoLast`), `src/messages/{ko,en}.ts`, `src/styles/tokens.css`(토스트 바탕이 필요하면 토큰), 테스트, `package.json`.
- 내용:
  - 토스트: 화면 아래 가운데, 결과 문구, 굵은 글자 버튼 "되돌리기", 옅은 `Ctrl+Z`. 8초 뒤 사라지고 마우스를 올리면 멈춘다. `aria-live="polite"`. 색은 토큰만 쓴다.
  - `useUndo`: Ctrl+Z → `undoLast`, Ctrl+Shift+Z → `redoLast`. 결과로 status·log·branches 쿼리를 무효화하고 "되돌렸어요" 토스트. `undoStale`·`undoPushed`·`nothingToUndo`는 같은 자리에 짧은 문장으로.
  - 버전 저장 되돌리기는 `UndoResult.message`를 이름 칸에 돌려놓는다(H7과 이어짐).
- 테스트: 토스트 버튼과 Ctrl+Z가 같은 api를 부름, 글상자 포커스에서 Ctrl+Z는 api를 부르지 않음, 8초 뒤 사라짐, 오류 코드별 문구.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 토스트가 비교 칸 가로 스크롤 위에 겹쳐도 읽힌다.
- Windows/Ubuntu: Radix Toast의 포털 위치를 두 웹뷰에서 확인한다.

H7. 화면: 문서 제목형 버전 이름 (`features: document-title version name`) — 001-18을 다시 만든다
- 목표: D-044대로 고친 파일 목록 위에 큰 이름 칸과 설명 칸을 둔다.
- 바꿀 파일: `src/features/commit/{NameBox.tsx, NameBox.module.css, NameBox.test.tsx, suggest.ts, suggest.test.ts}`(`CommitBox`를 바꿔 씀), `src/screens/Workspace/Workspace.tsx`, `src/styles/tokens.css`(`--font-size-title`), `docs/design.md`(제목 글자 크기와 포커스 표시), `src/messages/{ko,en}.ts`.
- 내용:
  - 이름 칸: 테두리 없는 한 줄 입력, `--font-size-title`, 굵기 600. 제안 이름을 placeholder로 보여주고, 비워 두면 그 이름으로 저장한다. 설명 칸: 바로 아래 글상자, placeholder "설명 추가". 오른쪽 아래에 옅은 `Ctrl+Enter`와 주 버튼 `<Term id="commit" />`. 테두리가 없어도 `:focus-visible`에 `--accent` 밑줄을 남긴다.
  - 저장 메시지: 이름(비면 제안 이름) + 설명이 있으면 빈 줄 + 설명.
  - 제안 이름 규칙(`suggest.ts`, 문구는 messages에서): 이번 버전에 넣은 파일 기준, 목록 순서의 첫 파일 이름(경로 제외).
    - 1개: "<이름> 고침". 새 파일이면 "추가", 지운 파일이면 "삭제", 이름 바꿈이면 "<옛 이름> 이름 바꿈".
    - 여러 개: "<첫 이름> 외 N개 고침". 모두 새 파일이면 "추가", 모두 지운 파일이면 "삭제".
    - en: "Update <name>", "Update <name> and N more", "Add …", "Delete …", "Rename <old>".
  - 버튼이 꺼지는 경우는 넣은 파일 0개뿐이다(Q12의 답에 따라 바뀜). 꺼진 이유는 버튼 아래 둘째 줄 "넣은 파일 없음"(D-049).
  - 성공하면 두 칸을 비우고, 포커스를 파일 목록으로 옮긴다. 포커스가 글상자에 남으면 다음 Ctrl+Z가 글자 되돌리기로 가 버리기 때문이다. "저장했어요" 토스트(H6).
  - 한글 조합 중 Ctrl+Enter는 무시(D-033).
- 테스트: 제안 이름 표 전부(ko·en), 비우고 저장하면 제안 이름, 설명이 있으면 두 줄 메시지, 성공 후 포커스 위치, 조합 중 무시, 되돌리기 결과로 칸이 다시 채워짐.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 한글 IME로 이름 입력 중 Enter·Ctrl+Enter가 저장하지 않음, 800px 너비에서 긴 제안 이름이 말줄임.
- Windows/Ubuntu: WebKitGTK의 조합 중 `keyCode 229` 처리는 기존 `matches`가 한다.

### 브랜치

H8. git: 맡겨 두기와 꺼내기 (`git: add park and unpark ops`)
- 목표: 고친 파일(새 파일 포함)을 일반 임시 보관 목록과 섞지 않고 `refs/higit/parked/<브랜치>`에 둔다(D-047).
- 바꿀 파일: `crates/git/src/types.rs`(`GitOp::Park { message }`, `GitOp::Unpark { oid }`), `crates/git/src/cli/park.rs`(신규), `crates/git/src/failure.rs`.
- 내용:
  - Park: `stash push --include-untracked -m <메시지>` → `rev-parse --verify refs/stash`로 oid를 얻고 → `stash drop -q stash@{0}`(드롭 전 그 oid가 맞는지 다시 확인). 결과 oid를 돌려주고, ref 쓰기는 core가 H1로 한다. git에 "목록에 넣지 않는 `stash push`"가 없어서 이렇게 한다. 실패하면 stash 목록에 남으므로 파일은 잃지 않는다.
  - Unpark: `stash apply --index <oid>`. 실패를 `UnparkConflict`(겹침)와 `UntrackedWouldOverwrite`로 나눈다.
  - 무시된 파일(`.gitignore` 대상)은 맡기지 않는다.
- 테스트: 수정·새 파일·지운 파일·넣은 파일이 섞인 상태 → Park → 작업 폴더가 깨끗하고 `git stash list`가 그대로 → Unpark → index와 작업 폴더가 원래와 같다. 한글·공백 파일명. 사용자 stash가 이미 있어도 그 목록이 바뀌지 않음.
- 완료 기준: `cargo test -p git` 두 OS 통과.
- Windows/Ubuntu: `core.autocrlf=true`(Windows 기본)에서 CRLF 파일이 맡기고 꺼낸 뒤 그대로인지 테스트한다. Windows는 편집기가 파일을 잡고 있으면 Park의 정리 단계가 실패할 수 있다(위험).

H9. core: 맡겨 두는 갈아타기와 새 브랜치 (`core: switch with parked changes and create-and-switch`) — 001-12를 대체
- 목표: `switch`와 `branch-create` 액션을 D-047 동작으로 등록하고 되돌리기를 붙인다.
- 바꿀 파일: `crates/core/src/view/input.rs`(`Switch(SwitchInput { branch, remote: Option<String> })`, `BranchCreate(BranchCreateInput { name })`), `crates/core/src/actions/{switch.rs, branch_create.rs, park.rs}`(신규), `actions/mod.rs`, `crates/core/src/undo/run.rs`(SwitchBack, DeleteCreated), `Outcome`(`Switched { branch, parked, unparked, unpark_blocked }`, `BranchCreated { name }`), `view::ActionResult`, `ErrorCode`(`parkBlocked`), 생성물, `src/messages/{ko,en}.ts`.
- 내용:
  - 메타: `switch`/`switch`/Normal/SwitchBack, `branch-create`/`branch`/Safe/DeleteCreated. 위험도는 spec 표와 같다.
  - `switch` 흐름: 이름 검사(004-4) → 고친 파일이 있으면 Park 후 `refs/higit/parked/<떠나는 브랜치>`에 기록(이미 있으면 앞 것을 백업 ref로 옮기고 덮어씀) → `switch <브랜치>`(원격 브랜치면 `switch -c <이름> --track <원격>/<이름>`) → 도착 브랜치에 맡겨 둔 것이 있으면 꺼낸다. 꺼내는 규칙은 Q3 추천안: 맡길 때의 기준 버전이 지금 HEAD와 같으면 바로 꺼내고, 다르면 H11의 겹침 검사가 깨끗할 때만 꺼낸다. 겹치면 꺼내지 않고 ref를 그대로 두며 `unpark_blocked`로 알린다.
  - `switch` 중간 실패: Park 뒤 `switch`가 실패하면 같은 브랜치에 곧바로 다시 꺼내 원래대로 돌린다(기준이 같아 겹치지 않는다). 그것도 실패하면 ref를 남기고 `parkBlocked`와 원문을 보여준다.
  - 떠나는 쪽이 브랜치가 아닌 상태(특정 버전을 보는 중)면 맡기지 않고 git 기본 동작대로 갈아탄다. 덮어쓸 파일이 있으면 `wouldOverwrite`.
  - `branch-create`: `check-ref-format`(001-6) → `switch -c <이름>`. 시작점은 늘 지금 HEAD라 작업 폴더가 바뀌지 않고, 고친 파일이 따라간다. 001-12의 "화면이 두 액션을 차례로 부른다"는 없앤다. 한 액션이어야 Ctrl+Z 한 번으로 되돌아간다. 같은 이름(Windows·Ubuntu 모두 대소문자 무시, H-18)이면 `branchExists`.
  - 되돌리기: SwitchBack은 `before.branch`로 같은 `switch` 흐름을 실행한다(지금 고친 파일은 도착했던 브랜치에 다시 맡겨지고, 떠났던 브랜치의 맡긴 파일이 다시 나온다). 원격 브랜치에서 새로 만든 로컬 브랜치는 돌아간 뒤 `delete_ref`로 지운다(가리키는 버전이 그대로일 때만). DeleteCreated는 이전 브랜치로 `switch`(고친 파일이 따라옴) 후 `delete_ref(refs/heads/<새 이름>, 만든 때 oid)`.
- 테스트: 가짜 백엔드로 호출 순서. 실제 git 통합: A에서 고침 → B로 → 작업 폴더 깨끗, ref 있음 → A로 → 고친 파일 복원, ref 없음. 새 파일만 고친 경우. 맡긴 뒤 터미널로 A에 버전 저장(기준 다름) + 겹치지 않음 → 꺼냄, 겹침 → 꺼내지 않음. switch 실패 흉내(잠긴 index) → 원래대로. 새 브랜치 → undo → 원래 브랜치이고 새 브랜치 없음. switch → undo → redo.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션.
- Windows/Ubuntu: 브랜치 이름의 `/`는 ref 경로 폴더가 된다. Windows에서 `Feature`와 `feature`는 같은 파일이라 ref 쓰기가 부딪힌다(위 대소문자 검사로 막음).

H10. 화면: 브랜치 목록 (`features: branch list with search, switch and create`) — 001-19를 대체
- 목표: D-047대로 상단 바 브랜치 이름을 누르거나 Ctrl+B를 누르면 찾기 칸이 있는 목록이 열린다.
- 바꿀 파일: `src/features/branches/{BranchList.tsx, BranchList.module.css, BranchList.test.tsx, model.ts, model.test.ts}`(신규), `src/screens/Workspace/TopBar.tsx`, `src/lib/queries.ts`(`useBranches`, `useBranchNameCheck`), `src/ui/Popover.tsx`(신규, `@radix-ui/react-popover`), `src/features/branches/color.ts`(브랜치 색 고르기), `src/messages/{ko,en}.ts`, `package.json`.
- 내용:
  - 찾기 칸 placeholder "찾기 또는 새로 만들기". 구역 "지금"(지금 브랜치), "최근에 쓴 브랜치"(마지막 버전 시각 순 5개), "브랜치 N개 더"(펼치기). 찾는 중에는 일치하는 것만, 원격에만 있는 브랜치는 원격 이름을 붙여 보인다.
  - 줄 오른쪽 글자(D-049): 맡겨 둔 파일이 있으면 "맡겨 둔 파일 N개", 그 브랜치 upstream에 새 버전이 있으면 "새 버전 N개", 아니면 "작성자 · 날짜". 꺼내지 못한 맡긴 파일은 "맡겨 둔 파일 N개 · 겹침".
  - 고르면 곧바로 `switch` → 목록을 닫고 "<브랜치>로 갈아탔어요" 토스트(되돌리기 포함).
  - 일치하는 이름이 없으면 맨 아래 `+ "<이름>" 새로 만들기 Enter`. 입력 중 이름 검사(001-10 `query_branch_name`, 300ms 지연), 쓸 수 없으면 "쓸 수 없는 이름". Enter → `branch-create` → 토스트.
  - 키보드: ↑↓ 이동, Enter 실행, Esc 닫기. 조합 중 Enter 무시.
  - "main에 합치기"·"검토 요청"(목업)은 반복 2·4라 넣지 않는다.
  - 브랜치 색: main(기본 브랜치)은 `--graph-1`, 나머지는 이름 해시로 `--graph-2`~`--graph-6`. 상단 바 점, 목록 점, 노선도(H18)가 같은 함수를 쓴다. 글자 색은 `--text`·`--text-muted`(D-031).
- 테스트: 모델(구역 나누기, 정렬, 찾기, 오른쪽 글자 우선순위), 클릭과 Enter가 각 액션을 부름, 조합 중 Enter 무시, 쓸 수 없는 이름이면 Enter가 아무것도 안 함, 색 함수가 같은 이름에 같은 값.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 한글 브랜치 이름 만들기·갈아타기, 브랜치 50개에서 찾기.
- Windows/Ubuntu: WebKitGTK에서 팝오버 안 입력칸의 IME 후보 창 위치를 확인한다.

### 동기화

H11. git: 작업 폴더를 건드리지 않는 겹침 검사 (`git: add merge check that leaves worktree alone`)
- 목표: 두 버전을 합칠 때 겹치는 파일이 있는지 미리 안다. 동기화(H13)와 꺼내기(H9)가 쓴다.
- 바꿀 파일: `crates/git/src/backend.rs`(`merge_check(repo, base: Option<Oid>, ours, theirs) -> MergeCheck { Clean, Overlap { paths } }`), `crates/git/src/cli/merge_check.rs`(신규), `crates/git/src/cli/runner.rs`(호출 하나에 환경 변수를 더하는 `run_with_env`).
- 내용: git 버전(004-3의 `OnceLock`)에 따라 고른다.
  - 2.38 이상: `merge-tree --write-tree --name-only -z <ours> <theirs>`. 종료 코드 0은 Clean, 1은 Overlap이고 NUL로 나뉜 파일 이름을 읽는다. 기준을 따로 줘야 하면(꺼내기) 2.40의 `--merge-base`를 쓰고, 그보다 낮으면 아래 방식으로 떨어진다.
  - 2.38 미만(Q8 추천안): 임시 index로 3-way 읽기. 앱 임시 폴더의 index 파일을 `GIT_INDEX_FILE`로 주고 `read-tree -i -m --aggressive <base> <ours> <theirs>` → `ls-files -u -z`로 두 쪽 모두 바꾼 파일을 얻는다. 그 파일마다 세 blob을 `cat-file blob`으로 임시 파일에 쓰고 `merge-file -p --quiet`의 종료 코드로 겹침을 본다(0이면 깨끗). 이진 파일, 지움과 고침, 이름 바꿈은 겹침으로 본다. 실제 index와 작업 폴더는 건드리지 않는다.
  - 두 방식의 결과가 다를 수 있는 곳(이름 바꿈 감지)은 2.38 미만이 더 보수적이다. 깨끗하다고 했는데 실제 합치기가 겹치는 경우는 H13의 안전망이 막는다.
- 테스트: 같은 파일의 다른 줄 → Clean(두 방식 모두), 같은 줄 → Overlap, 다른 파일 → Clean, 한쪽 지움·한쪽 고침 → Overlap, 이진 파일 양쪽 고침 → Overlap. 버전 분기는 가짜 버전으로 두 경로를 모두 돌린다. 004-15b의 Ubuntu 22.04 작업에서 2.38 미만 경로가 실제로 돈다.
- 완료 기준: `cargo test -p git` 두 OS 통과. 테스트 뒤 `git status`와 `.git/index`가 그대로.
- Windows/Ubuntu: 임시 파일은 `std::env::temp_dir()` 아래 앱 폴더에 두고 끝나면 지운다. Windows 경로를 `GIT_INDEX_FILE`에 그대로 넘겨도 된다.

H12. git·platform: 시간 제한과 백그라운드 모드 (`git: add timeouts, process tree kill and background mode`) — D-041의 일부
- 목표: 자동 원격 확인(D-045) 전에 네트워크 git이 멈춰도 끝나게 하고, 질문 창을 띄우지 않는 호출 방식을 둔다.
- 바꿀 파일: `crates/git/src/cli/runner.rs`, `crates/git/src/cli/mod.rs`, `crates/git/src/error.rs`(`GitError::TimedOut`), `crates/platform/src/process.rs`(신규, `spawn_group`, `kill_tree`), `crates/platform/Cargo.toml`, `src-tauri/deny.toml`(새 의존성 확인), `docs/architecture.md` "git 실행 규칙".
- 내용:
  - `Runner`가 자식을 띄울 때 platform의 함수로 프로세스 그룹을 만든다. Linux는 `CommandExt::process_group(0)`, Windows는 Job Object. 시간이 지나면 `kill_tree`로 그룹째 끝낸다. Windows Job Object는 unsafe 없이 쓰는 crate(예: `win32job`)로 하고, 없으면 `taskkill /T /F /PID`를 부른다. 그래서 004-12의 `forbid(unsafe_code)`를 platform에서도 지킨다. 어떤 crate를 쓸지는 이 단계에서 `cargo deny check`로 확인한다.
  - 시간 제한: 조회 30초, 쓰기 2분, 네트워크 10분, 백그라운드 원격 확인 60초. 진행이 멈췄을 때 끊는 기능(`--progress` stderr 감시)과 취소 버튼은 반복 2.
  - 백그라운드 모드(자동 원격 확인 전용): D-023의 `BatchMode`에 더해 `GCM_INTERACTIVE=never`, `-c credential.interactive=never`, `SSH_ASKPASS_REQUIRE=never`, `GIT_ASKPASS`·`SSH_ASKPASS`를 지운다. 사용자가 누른 동기화는 지금처럼 자격 증명 도우미 창을 막지 않는다(D-023).
- 테스트: 잠자는 가짜 git(별칭으로 `sleep`)이 시간 제한에 `TimedOut`이고 자식·손자 프로세스가 남지 않음. 백그라운드 모드 환경 변수를 별칭으로 읽어 확인.
- 완료 기준: `cargo test -p git -p platform` 두 OS 통과. 수동: 응답 없는 원격 주소로 원격 확인 → 60초 안에 끝남.
- Windows/Ubuntu: 프로세스 트리 종료가 OS별로 완전히 다르다. 코드는 platform에만 둔다. Windows의 `git.exe`는 `sh`·`ssh` 손자 프로세스를 띄우므로 Job Object가 꼭 필요하다.

H13. core: 원격 확인·가져오기·올리기·동기화 액션 (`core: add fetch, pull, push and sync actions`) — 001-13을 바꿈
- 목표: fetch·pull·push 액션은 그대로 두고 동기화가 이를 묶는다(D-045).
- 바꿀 파일: `crates/core/src/view/input.rs`(`Fetch(FetchInput { background: bool })`, `Pull`, `Push`, `Sync`), `crates/core/src/actions/{fetch.rs, pull.rs, push.rs, sync.rs}`, `actions/mod.rs`, `crates/core/src/bootstrap.rs`(잠금 둘로 나누기), `Outcome`(`Fetched`, `Synced { received, sent, merged_from: Vec<String>, overlap: Vec<String> }`), `ErrorCode`(`overlap`, `timedOut`), 생성물, `src/messages/{ko,en}.ts`.
- 내용:
  - 메타: `fetch` Safe/Nothing, `pull` Normal/RestoreRef, `push` Normal/Inverse, `sync`/`sync`/Normal/Inverse. 1b에는 Inverse 되돌리기가 없으므로 이 줄에서 Ctrl+Z를 누르면 "올린 버전은 되돌릴 수 없어요". 앞의 버전 저장도 이미 올렸으니 맞는 동작이다.
  - 잠금: 지금 저장소 잠금 하나를 "작업 폴더 잠금"과 "네트워크 잠금"으로 나눈다. `fetch`는 네트워크 잠금만 잡아, 백그라운드 원격 확인 중에도 버전 저장이 `Busy`가 되지 않는다. `sync`·`pull`·`push`는 둘 다 잡는다. 백그라운드 `fetch`가 잠금을 못 잡으면 조용히 건너뛴다.
  - `fetch`: H-12대로 `fetch --all`, prune 안 함. `background`면 H12의 백그라운드 모드와 60초 제한.
  - `pull`: `merge --ff-only <upstream>`. 001-9의 `git pull --ff-only`를 바꾼다. `pull`은 사용자의 `pull.rebase` 설정을 따라 동작이 달라지므로 쓰지 않는다.
  - `sync` 흐름:
    1. 원격이 없으면 `NoRemote`(화면은 버튼을 숨김). upstream이 없으면 H-11대로 원격을 고르고 `push -u` 후 끝.
    2. `fetch`(그 원격만).
    3. 받을 것 0 → 4로. 보낼 것 0 → `merge --ff-only`. 둘 다 있으면 `merge_check(HEAD, upstream)`(H11). Overlap이면 합치기를 시작하지 않고 `Synced { overlap }`로 끝낸다(D-048). Clean이면 백업 ref를 만들고 `merge --no-edit -m <메시지> <upstream>`. 메시지는 들어온 버전의 작성자로 만든다: "민지의 버전 1개 가져와 합침", 여럿이면 "민지 외 1명의 버전 3개 가져와 합침"(앱 언어).
    4. 안전망: 합치기가 겹침으로 실패하면 곧바로 `merge --abort`하고 Overlap으로 돌려준다. 고친 파일이 들어올 파일과 겹쳐 git이 시작을 거부하면 `wouldOverwrite`.
    5. 보낼 것이 있으면 `push`. 거부되면(그사이 누가 올림) 한 번만 2부터 다시 한다.
  - journal: `sync`는 `before.head`를 남긴다. 반복 2의 Inverse 구현이 쓴다.
- 테스트: 로컬 bare 원격(`allow_local_remotes_for_tests`, 004-1). 받기만, 보내기만, 둘 다 + 다른 파일 → 합친 버전 메시지 확인 후 올림, 둘 다 + 같은 줄 → 합치기 시작 안 함·`MERGE_HEAD` 없음, upstream 없음 → `-u`, 원격 없음, push 경쟁 한 번 재시도, 백그라운드 fetch 중 commit이 `Busy`가 아님. 2.38 미만 경로는 가짜 버전으로.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 로컬 원격은 Windows에서 `C:\…` 경로 그대로(001-9). 합친 버전 메시지는 `commit -F -`처럼 stdin으로 넘겨 Windows 명령줄 인코딩을 피한다(`merge -F -`).

### 상단 바와 동기화 화면

H14. 화면: 상단 바 구성과 버전 기록 화면 (`screens: top bar per D-045 and history screen`)
- 목표: D-045의 상단 바와 홈 2단 배치를 만든다. 왼쪽 탭을 없앤다(Q7 추천안).
- 바꿀 파일: `src/screens/Workspace/{TopBar.tsx, TopBar.module.css, TopBar.test.tsx, Workspace.tsx, Workspace.module.css}`, `src/screens/Workspace/LeftTabs.*`(삭제), `src/screens/History/History.tsx`(신규, 기존 `HistoryList` 사용), `src/state/repoStore.ts`(`leftTab` → `view: "home" | "history"`), `src/screens/Workspace/ChangedFiles.tsx`(머리 줄 "고친 파일 N개"와 모두 체크, ↑↓), `src/messages/{ko,en}.ts`, `docs/spec.md` "화면 구성" 작업 공간·버전 기록 행.
- 내용:
  - 상단 바 왼쪽: 프로젝트 메뉴, 세로 구분선, 브랜치 단추(색 점 + 이름 + ▼, H10을 연다). 지금의 올릴·가져올 수 글자는 동기화 버튼(H15)으로 옮기므로 지운다.
  - 오른쪽: 동기화(H15), 글자 버튼 `<Term id="log" />`(Ctrl+H, 누르면 버전 기록 화면, 다시 누르거나 Esc면 홈). "작업 내역"은 반복 2, "설정"은 설정 화면이 생길 때 넣는다. 그 전에는 보이지 않는다.
  - 홈: 상단 바 → 노선도 줄(H18 전에는 없음) → 2단. 왼쪽 340px: 이름 칸(H7), 머리 줄 "고친 파일 N개", 파일 목록. 오른쪽: 비교. 고친 파일이 없으면 왼쪽 "고친 파일 없음", 오른쪽 가운데에 브랜치 이름, "안 올린 버전 N개", 동기화 버튼(r-saved). 목업의 "모두 저장했어요" 문장은 D-049대로 "고친 파일 없음"으로 바꾼다.
  - 홈에 들어오면 첫 파일의 비교가 열려 있다(quick-actions "고친 내용 보기").
  - 버전 기록 화면: 지금의 단순 목록을 창 전체 폭으로. 그래프와 작은 메뉴(r-log)는 D-017 뒤.
- 테스트: 상단 바에 오른쪽 컨트롤이 동기화·버전 기록 둘, Ctrl+H 전환과 Esc, 빈 상태 문구, ↑↓로 선택 파일 이동, 첫 파일 자동 선택.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 800px, 라이트·다크.
- Windows/Ubuntu: 차이 없음.

H15. 화면: 동기화 버튼과 자동 원격 확인 (`features: sync button and background fetch`) — 001-20을 대체
- 목표: 새로 고침·가져오기·올리기 세 버튼 대신 동기화 하나. 원격 확인은 저절로.
- 바꿀 파일: `src/features/sync/{SyncButton.tsx, SyncButton.module.css, SyncButton.test.tsx, useAutoFetch.ts, useAutoFetch.test.ts}`(신규), `src/screens/Workspace/TopBar.tsx`, `src/messages/{ko,en}.ts`.
- 내용:
  - 버튼: `<Term id="sync" />` + 옅은 `↑N ↓M`(0이면 빼고 표시). 원격이 없으면 숨긴다(r-local의 "GitHub 연결"은 반복 4). 실행 중에는 돌아가는 표시와 꺼짐. Ctrl+Shift+S.
  - 주 버튼 한 개 규칙: 고친 파일이 0개이고 보낼 것이나 받을 것이 있을 때만 `--primary` 채움(r-saved). 그 밖에는 테두리 버튼.
  - 결과: 성공은 "동기화했어요 · ↑2 ↓1" 토스트(되돌리기 없음). Overlap은 버튼 옆에 "겹친 파일 N개"를 남기고 툴팁에 파일 이름 5개까지. 다음 동기화가 성공하면 사라진다. 그 밖의 오류는 상단 바 아래 안내 줄 + "자세히 보기"(spec "오류 메시지 원칙"). `pushRejected`의 해결 버튼은 001의 [가져오기] 대신 [동기화].
  - 자동 원격 확인(`useAutoFetch`, Q4 추천안): 프로젝트를 열 때 한 번, 창이 보이는 동안 5분마다, 창 포커스가 돌아왔을 때 마지막 확인에서 1분이 지났으면. `fetch { background: true }`. 다른 작업 중이면 건너뛴다. 실패는 버튼 옆 작은 점과 툴팁 "원격 확인 못 함 · 3분 전"으로만 보이고, 인증 실패가 나면 사용자가 동기화를 누를 때까지 멈춘다. 신뢰 확인(T1~T4)을 넣는다면 신뢰 결정 전에는 돌리지 않는다.
- 테스트: 가짜 타이머로 주기·포커스·건너뛰기·인증 실패 후 멈춤, 버튼 숫자 표시, 원격 없음 숨김, 주 버튼 조건, Overlap 표시와 사라짐, Ctrl+Shift+S.
- 완료 기준: `pnpm test`. 두 OS 수동: GitHub 비공개 저장소에서 동기화(받기·보내기·둘 다·겹침), Windows에서 자동 원격 확인 때 Git Credential Manager 창이 뜨지 않음, SSH 원격에서 처음 접속 서버여도 질문 없이 점만 표시.
- Windows/Ubuntu: Git Credential Manager(Windows)와 libsecret 도우미(Ubuntu)의 비대화 동작이 다르다. 두 OS에서 자격 증명이 없는 상태로 한 번씩 확인한다.

H16. core: 오류 원문 속 자격 증명 가리기 (`core: mask credentials in error detail`) — D-040의 작은 일부
- 목표: 동기화 실패의 "자세히 보기"에 주소 속 아이디·토큰이 나오지 않게 한다.
- 바꿀 파일: `crates/core/src/redact.rs`(신규), `crates/core/src/view/mod.rs`(`ErrorView` 만드는 곳), core 테스트.
- 내용: `https://아이디:토큰@` → `https://***@`, `ghp_`·`gho_`·`github_pat_` 패턴, 홈 폴더 경로 → `~`. `ErrorView.detail`을 만드는 한 곳에서만 부른다. 로그 파일과 순환은 반복 2(1b에서 빠지는 것).
- 테스트: 표본 stderr 여러 개.
- 완료 기준: `cargo test -p higit-core`.
- Windows/Ubuntu: 홈 폴더는 `C:\Users\이름`과 `/home/이름`. 위치는 platform이 준다.

### 작은 노선도 (D-046)

H17. core: 홈 노선도 조회 (`core: add home map query`)
- 목표: 두 줄 노선도에 필요한 것만 한 번에 준다.
- 바꿀 파일: `crates/core/src/query.rs`(`home_map`), `crates/core/src/view/map.rs`(신규 `HomeMapView`), `src-tauri/src/commands.rs`(`query_home_map`), `bindings.rs`, 생성물, `src/lib/{api.ts, queries.ts}`.
- 내용: 기본 브랜치(D-048: 원격 HEAD, 없으면 main, master 순), 지금 브랜치, 갈라진 버전(`merge_base`), 지금 브랜치의 갈라진 뒤 버전(최근 3개의 요약 + 전체 수), 기본 브랜치의 갈라진 뒤 수와 마지막 버전, 지금 브랜치 upstream이 가리키는 위치(원격 이름, 주소가 github.com이면 "GitHub"), 고친 파일 수, 기본 브랜치 대비 앞·뒤 수(`count_between`, 반복 2의 합치기 제안이 쓴다). 지금 브랜치가 기본 브랜치면 한 줄. 버전이 없으면 빈 값.
- 테스트: 가짜 백엔드와 실제 git으로 한 줄, 두 줄, 기본 브랜치 없음, upstream 없음, 버전 없음.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음.

H18. 화면: 두 줄 노선도 (`features: two-line map on home`)
- 목표: 상단 바 아래 104px 줄에 두 줄 노선도를 그린다.
- 바꿀 파일: `src/features/history/layout/twoLine.ts`·`twoLine.test.ts`(신규, 순수 함수), `src/features/history/render/{Renderer.ts, SvgRenderer.tsx}`(신규), `src/screens/Workspace/HomeMap.tsx`(신규), `src/screens/Workspace/Workspace.tsx`, `src/lib/useRepoWatch.ts`(`.git` 변경 때 노선도 무효화).
- 내용:
  - architecture.md "버전 기록 그리기 계층"의 층을 그대로 따른다. 데이터(H17) → 배치(`twoLine`: 역 좌표, 곡선, 너비가 모자라면 앞쪽 역을 "…"로 접음) → `Renderer` 인터페이스 → `SvgRenderer`. 클릭 판정은 배치 결과로 한다. 전체 노선도(D-017)가 확정되면 같은 `Renderer`에 일반 배치를 더한다.
  - 색: 선과 점은 H10의 브랜치 색 함수(`--graph-*`). 글자("지금 여기", 브랜치 이름, 버전 이름, "고친 파일 N개", 원격 표시)는 `--text`·`--text-muted`(D-031). 목업의 초록 "지금 여기"는 쓰지 않는다. 고친 파일이 있으면 마지막 역 뒤 점선과 점선 원.
  - 버전 이름은 저장소에서 온 글자라 React 글자로만 그린다(D-037).
- 테스트: 배치 함수 표(한 줄, 두 줄, 역이 많아 접음, 너비 800px), 글자에 그래프 색 변수가 쓰이지 않음(렌더 결과의 `fill` 검사), 고친 파일 0일 때 점선 없음.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 라이트·다크, 800px.
- Windows/Ubuntu: WebKitGTK의 SVG 글자 기준선이 WebView2와 다를 수 있어 두 OS에서 글자 겹침을 본다.

### 신뢰 확인 (Q1을 "넣는다"로 정했을 때만)
D-039. 004 "1b 끝나기 전" 표의 크기(커밋 4~6)를 넷으로 나눈다. 004-1, 004-10이 먼저 있어야 한다. 자동 원격 확인(H15)보다 먼저 한다. 자동 원격 확인은 사용자가 누르지 않아도 저장소 설정의 `core.sshCommand`·`credential.helper`를 실행하기 때문이다.

T1. git: 로컬 설정 읽기 (`git: read local config entries`) — `config --local --list -z`(NUL 구분)를 읽는 조회, 저장소별 덮어쓰기 `-c` 목록을 `CliBackend`가 저장소 경로로 들고 있다가 모든 호출에 붙이는 기능. 테스트: 실행 키가 있는 설정, 덮어쓰기가 붙는지 별칭으로 확인.
T2. core: 실행 설정 찾기와 신뢰 기록 (`core: detect executing config and remember trust`) — D-039의 키 목록, 앱 데이터 `trust.json`(경로 + 설정 해시, 임시 파일 뒤 rename), `open_project`가 `NeedsTrust { keys }`를 돌려줌, "끄고 열기"면 덮어쓰기 등록. 빈 hooks 폴더는 platform 앱 데이터.
T3. 화면: 신뢰 묻기 (`screens: ask trust when opening`) — 설정 이름 목록, "믿고 열기", "끄고 열기". 대화상자는 Radix Dialog. 결정 전에는 감시·자동 원격 확인을 시작하지 않는다.
T4. core: 소유자가 다른 저장소 (`core: handle dubious ownership`) — 분류(D-021), 사용자가 고르면 `safe.directory`에 그 경로 하나만 추가(전역 설정 쓰기 = 새 GitOp). `*`는 쓰지 않는다.
- Windows/Ubuntu: Windows는 다른 사용자 폴더·네트워크 드라이브에서 dubious ownership이 자주 난다. 경로 표기(`C:/…`)를 git이 받는 형식으로 platform이 바꾼다.

## 합친 순서
001·004의 단계는 번호로 부른다. "바뀜"은 그 단계의 원래 내용을 이 계획대로 고쳐서 한다는 뜻이다. 004 "순서 요약"의 의존(1→001-9 전, 3→001-6·8·9 전, 4→001-12·13 전, 9·10→001-10 전, 10→신뢰 확인 전, 5→001-10b 전, 7→9·14 뒤, 15b→3 뒤)을 지킨다.

| 순번 | 단계 | 한 줄 | 바뀐 점 |
| --- | --- | --- | --- |
| 1 | 004-12 | 모든 crate에 `forbid(unsafe_code)` | 없음 |
| 2 | 004-13 | clippy `unwrap_used` 등 | 004 Q10에 달림. 추천: 넣는다 |
| 3 | 004-11 | ESLint로 innerHTML·eval 막기 | 없음 |
| 4 | 004-16 | Actions 권한과 SHA 고정 | 없음 |
| 5 | 004-15 | pnpm 설치 스크립트 제한 | 없음 |
| 6 | 004-14 | cargo-deny | 없음 |
| 7 | 004-1 | git 실행 환경 보강 | 없음. H12가 백그라운드 모드 환경을 뒤에 더한다 |
| 8 | 004-2 | git을 절대 경로에서만 찾기 | 없음 |
| 9 | 004-3 | 리비전 앞 `--end-of-options`, `args.rs` | 없음 |
| 10 | 004-15b | Ubuntu 22.04 git으로 git 크레이트 시험 | 004 Q11에 달림. 추천: 넣는다. H11의 2.38 미만 경로를 실제로 시험하기 때문 |
| 11 | 001-6 | 브랜치·원격 목록, 이름 검사 | 바뀜: `Branch`에 마지막 버전 시각·작성자, `for-each-ref`로 `refs/higit/parked/*` 목록, 기본 브랜치 찾기(`symbolic-ref refs/remotes/<원격>/HEAD`)를 더한다 |
| 12 | 004-4 | 옵션처럼 보이는 이름 막기 | 없음 |
| 13 | 001-8 | 브랜치 만들기·바꾸기 GitOp | 바뀜: `CreateBranch`를 `CreateAndSwitch { name }`(`switch -c`)로. 시작점 지정은 빼고 반복 3의 "여기서 새 브랜치"로 미룸. `SwitchTrack` 추가 |
| 14 | H1 | ref 읽기·쓰기, 조상 확인, git 폴더 | 새 단계. architecture.md에서 반복 2였던 `resolve_ref`·`update_ref`를 앞당김 |
| 15 | H8 | 맡겨 두기·꺼내기 GitOp | 새 단계 |
| 16 | H11 | 겹침 검사 | 새 단계 |
| 17 | 004-9 | CSP와 capability 좁히기 | 없음. 이후 새 명령(`undo_last`, `redo_last`, `query_home_map`, `query_branches`, `query_branch_name`)은 각 단계에서 허용 목록에 더한다 |
| 18 | 004-10 | 열린 저장소만 받기 | 004 Q9에 달림. 신뢰 확인(T1~T4)을 넣으면 필수. `undo_last`·`redo_last`도 같은 검사를 거친다 |
| 19 | 004-6 | 보안 수정 기준 판정 | 없음 |
| 20 | 004-7 | Git 업데이트 안내 | 없음 |
| 21 | 001-10 | 브랜치 조회와 오류 코드 | 바뀜: 브랜치 조회 부분만(diff는 1a에서 끝남). `BranchRow`에 `parkedCount`, `parkBlocked`, `lastAuthor`, `lastAt`, `isDefault`. 오류 코드는 이 단계에 001 목록과 004-1의 `protocolNotAllowed`, 004-4의 다룰 수 없는 이름을 넣고, H3·H9·H13의 코드는 그 단계에서 더한다 |
| 22 | 004-5 | 링크를 따라가지 않는 비교 | 없음 |
| 23 | 001-10b | 큰 새 파일 비교 상한, `run_capped` | 없음. H12가 같은 `runner.rs`를 고치므로 바로 뒤에 둔다 |
| 24 | H12 | 시간 제한, 프로세스 트리 종료, 백그라운드 모드 | 새 단계. 004 "1b 끝나기 전"의 "시간 제한과 취소" 중 시간 제한만. 취소는 반복 2 |
| 25 | 001-9 | 원격 GitOp | 바뀜: `Pull`을 `MergeFf { rev }`로 바꾸고 `Merge { rev, message }`(`merge -F -`), `MergeAbort`를 더한다. `Fetch`에 `background`. 테스트 로컬 원격은 `allow_local_remotes_for_tests` |
| 26 | H16 | 오류 원문 속 자격 증명 가리기 | 새 단계 |
| 27 | H2 | `UndoKind`, journal, 백업 ref | 새 단계. 반복 2에서 앞당김 |
| 28 | H3 | 버전 저장 되돌리기·다시 하기 | 새 단계 |
| 29 | H4 | 용어 `sync`, `park` | 새 단계 |
| 30 | H5 | 단축키 표 넓히기 | 새 단계 |
| 31 | H6 | 되돌리기 토스트와 Ctrl+Z | 새 단계 |
| 32 | H7 | 문서 제목형 버전 이름 | 001-18을 다시 만든다 |
| 33 | H9 | 맡겨 두는 갈아타기, 새 브랜치 | 001-12를 대체 |
| 34 | H10 | 브랜치 목록 | 001-19를 대체 |
| 35 | T1~T4 | 저장소 신뢰 확인 | Q1에 달림. 넣으면 커밋 넷 |
| 36 | H13 | 원격 확인·가져오기·올리기·동기화 액션 | 001-13을 바꿈 |
| 37 | H14 | 상단 바 구성과 버전 기록 화면 | 새 단계. 001-15의 상단 바·왼쪽 탭을 고친다 |
| 38 | H15 | 동기화 버튼과 자동 원격 확인 | 001-20을 대체. 새로 고침 버튼은 없앤다 |
| 39 | H17 | 홈 노선도 조회 | 새 단계. Q11에 달림 |
| 40 | H18 | 두 줄 노선도 | 새 단계. Q11에 달림 |
| 41 | 004-8 | 거부 기준 2.38 | D-036 확정 뒤에만. Q8 추천안이면 하지 않는다 |
| 42 | 마무리 | 문서와 점검 | 001-22와 004-17을 합친다. 아래 |

42. 마무리 (`docs: wrap up iteration 1b`)
- 바꿀 파일: `docs/architecture.md`(GitBackend 목록에 H1·H8·H11 메서드, `UndoKind::Nothing`, journal 줄 형식, 잠금 둘, 시간 제한 표, `args.rs`와 버전 표, 링크 규칙, 명령 목록에 `undo_last`·`redo_last`·`query_home_map`), `docs/spec.md`(화면 구성의 작업 공간·버전 기록·복구 행, 위험도 설명의 `caution`을 D-043에 맞춤), `docs/design.md`(토스트, 제목 글자, 브랜치 색 함수), `docs/roadmap.md`(1b 완료 표시), `CLAUDE.md`(D-043에 맞춘 규칙 문장, 아직이면), `.claude/agents/code-analyst.md`(004에서 더한 줄 확인).
- code-analyst 점검: 001-22와 004-17의 항목 전부. 더해서 화면에서 `undo_last`를 `src/lib` 밖에서 부르지 않음, 단축키가 `shortcuts.ts` 밖에 없음(`rg "e\.code ===|ctrlKey" src --glob '!src/lib/**'`), 노선도 글자에 `--graph-` 없음, `refs/higit/`를 만드는 코드가 core의 undo·park 모듈에만 있음, `Command::new`가 `runner.rs`와 platform에만 있음, 새 명령이 capability에 모두 있음.
- 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full` 통과, 위 "완료 기준(1b 전체)"를 두 OS에서 손으로 확인.

## 1b에서 빠지는 것
| 항목 | 가는 곳 | 까닭 |
| --- | --- | --- |
| 합치기 제안 "main 가져와 합치기"·"main에 합치기"(r-mergehint, 브랜치 목록) | 반복 2 | D-048. 충돌 해결 화면과 함께 |
| 충돌 해결 화면, 동기화 중 겹침 해결, 꺼내다 겹친 맡긴 파일 해결 | 반복 2 | D-028. 1b는 멈춤 표시까지 |
| 이번 버전에 넣기·빼기, 새 브랜치 밖의 되돌리기(RestoreIndex, Inverse) | 반복 2 | D-042. 1b는 버전 저장·갈아타기·새 브랜치만 |
| 작업 내역(복구) 화면, 상단 바 "작업 내역" | 반복 2 | D-045 |
| 백업 ref 30일 정리 | 반복 2 | 1b의 백업 ref는 되돌리기와 맡긴 파일 덮어쓰기 때만 생겨 적다 |
| 고친 것 버리기(목업 오른쪽 아래), 임시 보관, 변경 취소 | 반복 2 | D-043 확인 흐름, D-011 |
| 네트워크 진행 멈춤 감지, 취소 버튼, `op-progress` | 반복 2 | H12는 시간 제한만 |
| index.lock 정리, 다음 실행에서 끝나지 않은 작업 알리기 | 반복 2 | D-041의 나머지 |
| 비밀 파일 경고, 로그 파일과 로그 가리기 | 반복 2 앞부분 | D-040. 1b는 오류 원문 가리기(H16)만 |
| 저장소 ID로 명령 시그니처 바꾸기 | 미정 | D-037 영향. 004-10이 그 전 단계 |
| 버전 기록 노선도, 역 작은 메뉴(r-log) | D-017 확정 뒤 | 1b는 단순 목록을 화면으로 옮기기만 |
| 시작 화면의 큰·작은 노선도 | D-017 확정 뒤(003 N1~N3) | H18의 `Renderer`를 나중에 같이 씀 |
| 이 버전으로 되돌리기, 여기서 새 브랜치(시작점 지정) | 반복 3 | 로드맵 |
| 검토 요청, GitHub 연결(r-local) | 반복 4 | 로드맵 |
| 설정 화면, 상단 바 "설정", 덮어쓰며 올리기 | 설정 화면이 생길 때 | D-043 |
| 비교의 "나란히 보기", "변경 없는 N줄 · 펼치기", 파일 줄의 `+12 −3` | 1b 뒤 별도 | 결정에 없고 홈 동작과 독립 |
| 브랜치 지우기·이름 바꾸기 | 반복 2 이후 | Q2의 답을 그때 구현 |

## 용어 변경
- spec.md 용어표에 추가(H4).
  - `push` 다음: `| sync | 동기화 | sync · \`git pull\` + \`git push\` | 원격 저장소의 새 버전을 가져온 다음 내 버전을 올리기 | normal | 매일 |`
  - `stash` 다음: `| park | 맡겨 두기 | stash · \`git stash -u\` | 브랜치를 바꿀 때 고친 파일을 그 브랜치에 두었다가 돌아오면 꺼내기 | safe | 자주 |`
- `src/glossary/types.ts`(`TERM_IDS`), `ko.ts`, `en.ts`에 같은 항목. en: "Sync" / "Get new versions from the remote, then upload yours", "Park" / "Keep changed files on a branch when you switch away, and bring them back when you return".
- 확인할 것(Q9, Q10): `fetch`의 쉬운 이름 "새로 고침"을 "원격 확인"으로 바꿀지, 홈 머리 줄 "고친 파일"을 `status`("변경 사항")와 맞출지. 바꾸면 D-024·D-025에 새 결정이 필요하다.
- "되돌리기"·"다시 하기"(토스트, Ctrl+Z)는 앱 동작 문구라 용어표가 아니라 `src/messages`에 둔다. 용어표의 `revert`도 "되돌리기"라 같은 말이 두 뜻으로 쓰인다(위험).

## 위험
- journal은 `.git/higit/`에 있어 받은 zip 저장소에 미리 만든 journal이 들어 있을 수 있다. H2의 검증과 H3의 "지금 HEAD가 `after`와 같을 때만"으로 피해를 좁히지만, 줄마다 앱 데이터의 비밀 키로 서명해 남이 만든 줄을 무시하는 편이 더 안전하다(Q13).
- Windows에서 편집기가 파일을 잡고 있으면 `stash push -u`의 정리나 `switch`가 중간에 실패할 수 있다. H9는 곧바로 원래대로 꺼내고, 그것도 실패하면 맡긴 ref와 stash 목록에 파일이 남아 잃지는 않는다. 사용자에게는 "자세히 보기"와 다시 시도만 준다.
- `stash push`가 잠시 사용자의 stash 목록을 건드린다. 앱이 그 순간 죽으면 목록에 "higit parked …" 항목이 남는다(다음 실행 알림은 반복 2).
- 2.38 미만 겹침 검사는 이름 바꿈을 감지하지 못해 2.38 이상보다 겹침을 더 자주 말한다. 반대로 깨끗하다고 했는데 실제로 겹치면 H13이 `merge --abort`로 되돌린다. 이 안전망은 "겹칠 때 합치기를 시작하지 않는다"(D-048)를 드물게 어긴다.
- 자동 합친 버전의 메시지가 앱 언어로 남는다. 팀원이 다른 언어를 쓰면 기록에 두 언어가 섞인다.
- `pull` 대신 `merge`를 쓰므로 사용자의 `pull.rebase=true` 설정을 따르지 않는다. D-003 "사용자 설정과 같은 동작"에서 벗어난다. 반복 2에서 리베이스 동기화를 고를 수 있게 할지 정한다.
- 자동 원격 확인은 노트북 배터리와 회사 프록시 기록에 영향이 있다. 창이 보일 때만 돌린다.
- Radix Toast·Popover 두 의존성이 늘어난다(cargo-deny가 아니라 pnpm 쪽, 004-15 허용 목록 확인).
- 단계 수가 많다(42행, 신뢰 확인을 넣으면 커밋 45개 안팎). 1b를 둘로 나눈다면 26번(H16)까지를 "1b-1 바탕", 그 뒤를 "1b-2 홈"으로 끊을 수 있다.

## 열린 질문
- Q1. 004 "1b 끝나기 전" 보안 항목을 1b에 넣을까요?
  - 추천: 저장소 신뢰 확인(T1~T4)과 시간 제한(H12)은 넣고, 오류 원문 가리기는 작은 판(H16)만 넣는다. 비밀 파일 경고, 로그 가리기 전체, index.lock 처리는 반복 2 앞부분으로 미룬다.
  - 이유: 자동 원격 확인이 사용자가 누르지 않아도 저장소 설정의 프로그램을 실행하게 만들고, 시간 제한은 D-041·D-045가 요구한다.
- Q2. 맡겨 둔 파일이 있는 브랜치를 지우거나 이름을 바꿀 때 어떻게 할까요?
  - 추천: 맡긴 ref는 저절로 지우지 않는다. 앱에서 이름을 바꾸면 ref도 함께 옮기고, 앱에서 지우면 ref를 백업 ref로 옮겨 journal에 남긴다(복구 화면에서 살림). 앱 밖에서 지운 브랜치의 ref는 브랜치 목록 끝에 "맡겨 둔 파일만 남음"으로 보여준다. 1b에는 지우기·이름 바꾸기 액션이 없으므로 표시만 1b 뒤에 넣는다.
  - 이유: 맡긴 파일은 아직 버전이 아닌 작업이라 사라지면 되살릴 길이 없다.
- Q3. 돌아왔을 때 맡긴 파일을 꺼내다 겹치면 어떻게 할까요?
  - 추천: 맡길 때 기준 버전과 지금 버전이 같으면 바로 꺼낸다. 다르면 겹침 검사(H11)가 깨끗할 때만 꺼내고, 겹치면 꺼내지 않고 브랜치 목록과 상단 바 브랜치 단추에 "맡겨 둔 파일 N개 · 겹침"을 보여준다. 해결은 반복 2 충돌 화면에서 한다.
  - 이유: 작업 폴더에 `<<<<<<<` 표시를 남기지 않고(D-028), 맡긴 파일도 잃지 않는다.
- Q4. 자동 원격 확인 간격은?
  - 추천: 프로젝트를 열 때, 창이 보이는 동안 5분마다, 창 포커스가 돌아올 때 1분이 지났으면. 인증 실패 뒤에는 사용자가 동기화를 누를 때까지 멈춘다.
  - 이유: VS Code(3분)와 GitHub Desktop(더 김) 사이이고, 실패하는 원격을 계속 두드리지 않는다.
- Q5. 단축키 표를 quick-actions.md대로 채택할까요?
  - 추천: 채택한다. ↑↓는 파일 목록 안에서만, Ctrl+Z·Ctrl+Shift+Z는 입력칸에서 글자 되돌리기. 다시 하기에 Ctrl+Y를 두 OS 공통 별칭으로 더한다. Ctrl+H·Ctrl+Shift+S는 두 웹뷰에서 손으로 확인하고 막히면 그 키만 바꾼다.
  - 이유: Windows 사용자는 Ctrl+Y가 익숙하고, 겹침은 H5 테스트가 막는다.
- Q6. 시작 화면의 상태 문장에도 D-049(이름과 숫자만)를 적용할까요?
  - 추천: 적용하지 않는다. 결과 토스트("저장했어요")와 오류 안내도 짧은 문장으로 둔다.
  - 이유: 시작 화면은 하루에 한 번 읽는 화면이라 D-031이 문장을 고른 이유가 그대로 남고, 토스트는 결과를 알리는 곳이다.
- Q7. 버전 기록은 왼쪽 탭(D-025)인가요, 상단 바 "버전 기록" 화면인가요?
  - 추천: 상단 바 화면(H14). 왼쪽 탭은 없앤다. D-025의 "버전 기록은 왼쪽 탭"을 바꾸는 갱신 줄이 필요하다.
  - 이유: D-045가 상단 바 오른쪽에 "버전 기록"을 넣었고, 왼쪽 칸은 이름 칸과 파일 목록으로 꽉 차며, 지금도 버전 기록 탭은 창 전체 폭을 쓴다.
- Q8. git 2.38 미만에서 겹침 검사를 어떻게 할까요?
  - 가(추천): 거부 기준은 2.30으로 두고(D-036 거부 부분은 확정하지 않음), 2.38 이상은 `merge-tree`, 미만은 임시 index의 `read-tree` + `merge-file`(H11). 이유: Ubuntu 22.04 기본 git에서 앱이 켜지고, 작업 폴더를 건드리지 않으며, 같은 파일의 다른 줄도 합쳐진다.
  - 나: 미만이면 파일 단위로만 본다(양쪽이 바꾼 파일 이름이 겹치면 멈춤). 단순하지만 같은 파일만 고쳐도 늘 멈춘다.
  - 다: 미만이면 실제로 합쳐 보고 겹치면 `merge --abort`. 정확하지만 D-048의 "합치기를 시작하지 않는다"를 어긴다.
  - 라: D-036대로 2.38 거부. Ubuntu 22.04에서 PPA 없이는 앱이 켜지지 않는다(D-002와 부딪힘).
- Q9. `fetch`의 쉬운 이름 "새로 고침"을 "원격 확인"으로 바꿀까요?
  - 추천: 바꾼다. 실패 표시 "원격 확인 못 함"이 용어표에서 나오게 된다(D-029).
  - 이유: 새로 고침 버튼이 없어졌고(D-045), "새로 고침"은 화면 다시 읽기로 오해된다.
- Q10. 홈 머리 줄 "고친 파일 N개"와 용어 `status`("변경 사항")가 다릅니다. 어떻게 할까요?
  - 추천: 머리 줄은 문구 "고친 파일 N개"로 두고, 그 글자에 `status` 용어 툴팁을 붙인다. 용어표는 그대로.
  - 이유: D-046·D-049 예시가 "고친 파일"을 쓰고, 호버로 원래 이름을 배우는 길(D-006)은 남는다.
- Q11. 작은 두 줄 노선도(H17, H18)를 1b에 넣을까요?
  - 추천: 넣되 맨 끝에 둔다. 밀리면 1b 뒤로 넘겨도 다른 단계에 영향이 없다.
  - 이유: D-046은 확정이고, 두 줄 배치는 D-017의 미확정 부분(확대·축소, 선 굵기, 전체 지도)이 필요 없다. 같은 층으로 만들어 두면 D-017 확정 뒤 일반 배치를 더하기만 하면 된다. 003이 "노선도는 D-017 뒤"라고 한 것은 시작 화면 노선도에만 남긴다.
- Q12. 목업처럼 고친 파일을 처음부터 모두 "이번 버전에 넣기"로 할까요?
  - 추천: 저절로 넣지 않는다. 대신 넣은 파일이 0개일 때 버전 저장을 누르면 고친 파일을 모두 넣고 저장한다. 버튼은 고친 파일이 0개일 때만 꺼진다.
  - 이유: 화면에 들어오기만 해도 index가 바뀌면 터미널을 같이 쓰는 사람이 놀라고, 한 번에 저장하는 흐름은 그대로 얻는다.
- Q13. journal 줄을 앱 데이터의 비밀 키로 서명할까요?
  - 추천: 서명한다(H2에 포함, HMAC 한 함수).
  - 이유: journal은 저장소와 함께 옮겨 다니므로 남이 만든 줄을 되돌리기에 쓰지 않게 하는 가장 작은 방법이다.
- 004의 Q1~Q12와 D-036 확정 여부는 그대로 열려 있다. Q8의 답이 D-036과 004 Q3의 답이 된다.

## 검토 반영 (code-analyst, 2026-10-15)
구현 전 검토에서 나온 것. 아래가 위 본문보다 앞선다. 관련 자동 결정 D-064~D-067.

### git 2.34.1 실험 결과 (`--end-of-options`)
- 됨: `rev-parse --verify`, `log`, `diff`, `switch`, `switch -c --track`, `branch`, `push`, `merge --ff-only`, `merge-base`, `rev-list`, `update-ref`, `cat-file`, `show-ref`, `for-each-ref`, `symbolic-ref`, `fetch`, `ls-remote`, `stash`, `read-tree`, `commit-tree`, `show`.
- 안 됨: `checkout`(경로로 오해석), `reset`(옵션 위치 오류), `check-ref-format`(usage 오류).
- 004-3의 버전 표는 "2.34.1에서 확인한 명령" 허용 목록으로 만든다. 목록 밖 명령과 2.34 미만 버전은 `--end-of-options`를 넣지 않고 core 이름 검사(004-4)로 막는다. checkout·reset·check-ref-format에는 절대 넣지 않는다는 테스트를 둔다.
- `check-ref-format --branch '@{-1}'`은 이전 브랜치 이름을 돌려주며 통과한다. 이름 검사는 `check-ref-format refs/heads/<이름>` 전체 형식으로 하고 `-` 시작과 `HEAD`는 따로 막는다.

### 단계별 고칠 점
- 004-1(됨): 테스트 보강은 f5f2ffe. `ext::`는 환경 변수 없이도 막혀 `file`·로컬 경로 테스트가 실제 증명이다.
- 004-6: `dpkg-query -S` 출력에서 패키지 이름을 읽는다(`git`으로 고정하지 않음).
- 004-7(순번 20): 시작 화면 줄과 platform·Rust 쪽만 한다. 작업 공간 안내 줄(D-053)과 "닫은 버전" 기록은 H14에서 만든다.
- 004-9: CSP는 D-052 값(`object-src`, `frame-src`, `base-uri`, `form-action` 모두 `'none'`). `asset:`은 asset 프로토콜이 꺼져 있어 넣지 않는다. dev에는 CSP가 없으므로 확인은 릴리스 실행 파일로. 이후 새 Tauri 명령은 `commands.rs`·`build.rs`·capability 세 곳에 같은 커밋으로 넣는다.
- 004-10(순번 18): `Queries`가 열린 경로 집합을 보도록 생성자에 공유 집합을 넘긴다. 집합 등록 시점은 T2의 신뢰 결정 뒤로 맞춘다(T2 전에는 `Opened`·`NotARepo` 때 등록하고, T2에서 `NeedsTrust`는 등록하지 않게 바꾼다). `project_summary`는 최근 목록에 있고, T2 뒤에는 신뢰 기록의 해시가 맞을 때만 `status`를 돌린다(`core.fsmonitor`는 `status`에서 실행된다).
- 004-11 보강(별도 커밋 가능): 저장소에서 온 URL이 `href`·`src`에 들어가지 않게, 링크는 https 검사 부품 하나로만.
- 004-15b: 컨테이너에서 `cargo test --workspace --exclude app`을 돌린다(H11, H13의 2.38 미만 경로는 core 테스트에 있다).
- IPC 길이 검사(D-037): 명령 인자에 상한을 둔다. 메시지 1MB, 경로 목록 10만 개, 이름 255바이트, `LogQuery.limit` 500. 004-10과 같은 커밋.
- H1: `update_ref`의 이전 값은 `Expect { Any, Absent, Is(Oid) }`. `Absent`는 영(0) oid로 넘긴다. journal은 worktree별(`--absolute-git-dir`), `refs/higit/*`는 저장소 공통이라는 점을 architecture.md에 적는다.
- H2: stage·unstage처럼 journal에 남지 않는 동작을 하면 "되돌림 경계" 줄을 남기고, 그 앞 줄은 되돌리지 않는다(D-065). `UndoState`에 `index_tree: Option<Oid>`(`write-tree`)를 둔다. 되돌리기는 `Executor`의 메서드로 두어 백업 ref를 같은 흐름으로 남긴다. 백업 ref 쓰기가 실패하면(사용자 `reference-transaction` hook 등) 본 동작을 하지 않는다. journal 서명(D-063)은 `hmac`+`sha2` crate, 키 파일 만들기·권한은 platform.
- H3: 다시 하기는 액션을 다시 실행하지 않고 `update-ref <브랜치> <after.head> <before.head>`로 같은 버전을 되살린다(D-065). "모두 넣고 저장"(D-062)을 되돌리면 `index_tree`로 index도 되돌린다. 사용자 hook은 `reset --soft`·`update-ref`에서도 `reference-transaction`으로 돈다(본문의 "hook이 돌지 않는다"는 틀림).
- H5: 단축키 표가 항목 하나에 키 여러 개를 가질 수 있게 바꾼다(Ctrl+Y 별칭, D-056).
- H7: commit 입력에 `stage_all: bool`(D-062).
- H4: `fetch` 쉬운 이름 "원격 확인"(D-059)을 같은 커밋에서. 용어표의 `pull` 명령 열은 `git merge --ff-only`, `sync`는 `git fetch` + `git merge` + `git push`.
- H8(맡겨 두기): 순서는 ① `refs/stash` oid를 읽음 ② `stash push --include-untracked -m <표지>` ③ `refs/stash`가 바뀌지 않았으면 맡길 것이 없음(사용자 stash를 건드리지 않음) ④ 바뀌었으면 core가 `refs/higit/parked/<이름>`을 먼저 쓰고 ⑤ 그 뒤 `stash@{0}`이 그 oid일 때만 drop. git 쪽은 `park_push`(oid 또는 없음)와 `stash_drop_if(oid)` 두 함수로 나눈다. 합치기·충돌 중이면 맡기지 않는다(git이 거부함).
- H8(꺼내기): 꺼내기는 작업 폴더가 깨끗할 때만 한다(갈아탄 직후). 적용 전에 stash 세 번째 부모(새 파일)의 경로가 작업 폴더에 이미 있는지 본다. `stash apply --index`가 실패하면 `reset --hard HEAD`와 stash가 만든 새 파일 삭제로 깨끗한 상태로 되돌리고 ref는 남긴다(D-054). 테스트: 겹침 apply 실패 뒤 작업 폴더가 깨끗함.
- H9: 맡긴 ref 이름은 브랜치 이름을 바꿔 적는다(D-064). 브랜치 `a`와 `a/b`가 함께 있어도 ref가 부딪히지 않는다.
- H11: 기준이 없으면(무관한 기록) 빈 트리를 기준으로. `merge-file`은 1 이상(255 포함)이면 겹침. 임시 index는 호출마다 고유 파일. 2.38 이상 경로 테스트는 버전을 보고 건너뛴다.
- H12: D-023의 `BatchMode`가 아직 코드에 없으므로 H12에서 함께 넣는다. 강제 종료는 네트워크·백그라운드 작업에만 건다(D-066). 로컬 쓰기는 시간 제한 없이 기다린다. 백그라운드 fetch는 `-c gc.auto=0 -c maintenance.auto=false`. Linux 프로세스 그룹 종료는 `rustix`의 안전한 API로(platform `forbid(unsafe_code)` 유지). `platform/src/process.rs`는 이미 있으니 거기에 더한다.
- H13: 합친 버전 메시지는 `-m`으로 넘긴다(2.34.1의 `merge`는 `-F -`를 읽지 못한다). 갈라진 경우 `--no-ff`를 명시(사용자 `merge.ff=only`). `merge --abort`는 `MERGE_HEAD`가 있을 때만. 동기화는 upstream 원격만 fetch한다(`--all` 아님). 테스트에 `commit.gpgsign` 켬, identity 없음, upstream 강제 갱신, upstream 사라짐을 넣는다.
- H16: 주소 속 자격 증명은 콜론 없는 형태도 가린다(`://[^/@\s]+@` → `://***@`). 001-9(순번 25) 바로 뒤라 순서는 그대로.
- T2: `<git_dir>/hooks`의 `.sample`이 아닌 파일도 "실행되는 설정"으로 본다(D-067). "끄고 열기"는 `core.hooksPath=<빈 폴더>`를 붙인다.

### 테스트에 더할 실패 경우
- 맡기기: 바뀐 것 없음 + 사용자 stash 있음(사용자 stash가 그대로), 무시된 파일만 있음, 합치기 중, 꺼내기 겹침 후 깨끗함, ref 이름 `a`와 `a/b`, 한글 브랜치.
- 되돌리기: Ctrl+Z 두 번, 앱 밖에서 브랜치 삭제·이름 바꿈, 특정 버전을 보는 중, 서명이 다른 journal 줄, hook이 백업 ref를 거부.
- `--end-of-options`: `-x.txt`, `--output=x` 파일 이름(`--` 뒤), 태그 `refs/tags/-t`.
- 링크: 대상 없는 링크, 링크 순환, 안쪽 폴더 링크 아래 파일.

## 1b-1 점검 반영 (code-analyst, 2026-10-16)
1b-1 점검의 위반은 df4b108, 1b72a92, 9c6e0cf에서 고쳤다. 1b-2에서 지킬 것:
- 백그라운드 fetch에 `--no-recurse-submodules`, `--no-write-fetch-head`(없으면 FETCH_HEAD가 매번 바뀌어 감시 이벤트가 난다). 동기화·자동 확인은 upstream 원격 하나만 지정한다(`Fetch { remote: None }`은 `--all`).
- 갈아타기 전에 `validate_ref_like`와 `check_branch_name`을 꼭 부른다. git 2.30~2.33과 버전 읽기 실패(0.0.0)에서는 core 검사가 유일한 방어다.
- 꺼내기(`Unpark`)는 갈아탄 직후, 합치기·리베이스 중이 아닐 때만 부른다(실패 경로가 `reset --hard`).
- 저장이 없는 저장소는 `stash push`가 거부한다. 그때 맡기기를 건너뛴다.
- 맡긴 ref 이름은 `%2F` 때문에 길어질 수 있다. 브랜치 이름 200바이트를 넘으면 맡기지 않고 알린다.
- 신뢰 확인(T2): `NeedsTrust`는 열린 목록에 넣지 않는다. `project_summary`와 `watch`도 신뢰 기록을 확인한다. "끄고 열기" 덮어쓰기는 `Runner::command` 한 곳에서 붙인다.
- NotOpened 부정 테스트를 preview, diff, branches, check_branch_name, watch에도 둔다.
- `Queries::branches`는 맡긴 ref마다 git을 한 번씩 부른다. 많아지면 묶는다.
- H13 테스트에 `commit.gpgsign` 켬, identity 없음, upstream 강제 갱신, upstream 사라짐을 넣는다.
