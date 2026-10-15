# 006 합치기와 복구 (반복 2)
- 관련 결정: D-011(이 반복에서 이행), D-068, D-028, D-040, D-041, D-042, D-043, D-045, D-048, D-049, D-050, D-051, D-054, D-057, D-058, D-062~D-067. 새로 정할 것은 아래 "자동 결정 후보" C1~C18이고, 구현 전에 Claude가 `docs/decision.md` 맨 아래에 D-069부터 "(자동 결정)"으로 올린다(R1).
- 관련 명세: spec.md "화면 구성"의 합치기/충돌·복구 행, "충돌 해결 화면", 용어표(`merge`·`conflict`·`stash`·`discard`·`reflog`·`ignore`), 위험도 설명. architecture.md "되돌리기와 UndoJournal", "잠금 둘", "git 실행 규칙"의 시간 제한 표, "보안" 표.
- 참고 자료(결정 아님): `docs/research/img/flow4/r-mergehint.png`(합치기 제안, 맨 아래 "고친 것 버리기"), `r-local.png`, `r-log.png`(상단 바 "작업 내역"), `quick-actions.md`(동작별 방식 표의 합치기 제안 두 줄), `design-direction.md` "충돌 해결(conflict)" 1안, `git-visualization.md` ⑥ 세이브 카드("불러와도 지금 버전은 남아요"), `security.md`("비밀을 올리지 않게", "신뢰성", "개인정보와 로그").
- 앞선 계획: 005(이 계획의 입력은 005 "1b에서 빠지는 것" 표), 004(보안 1b), 001(남은 것 없음). 진행: 사람 확인 없이 Claude가 아래 순서대로, 한 단계 = 한 커밋.

## 목표
겹치지 않으면 버튼 하나로 합치고, 겹치면 한 곳씩 고르는 충돌 해결 화면으로 끝낸다. 되돌리기(Ctrl+Z)가 이번 버전에 넣기·빼기와 합치기까지 넓어지고, 올린 뒤에도 기록을 지우지 않는 되돌리기로 이어진다. "작업 내역" 화면에서 앱 작업·reflog·백업을 시간순으로 보고 되살리며, 고친 것 버리기·임시 보관·멈춘 작업 정리가 안전하게 된다.

## 현재 상태
이 계획이 기대는 사실만 적는다.
- `crates/core/src/executor.rs:122-125`는 `risk >= Caution`이면 `NotYetSupported`로 거부한다(D-011). 되돌리기는 `RestoreRef`·`SwitchBack`·`DeleteCreated`만 구현(`undo/run.rs`), `RestoreIndex`와 `Irreversible`은 경계 줄만 남기고(`undo/mod.rs` `is_barrier`), `Inverse`는 `pull`이면 `NothingToUndo`, 그 밖에는 `UndoPushed`다. `UndoState`는 `head·branch·parked·index_tree`뿐이다.
- `GitOp`에 `Merge`, `MergeFf`, `MergeAbort`, `Park`, `StashDropIf`, `Unpark`, `ResetSoft`, `ReadTree`가 있다. `merge_check`(2.38 이상 `merge-tree`, 미만 `read-tree`+`merge-file`)와 임시 폴더 `Scratch`(`cli/merge_check.rs`)가 있고, `ls-files -u -z` 파서 `unmerged()`는 그 파일 안에서만 쓴다. 충돌 상태를 읽는 조회와 해결 GitOp는 없다.
- `actions/remote.rs`의 `sync`는 겹치면 합치기를 시작하지 않고 `SyncReport.overlap`만 돌려주며, 합치기가 실패하면 `merge --abort`한다. `actions/switch.rs`의 `unpark_arrived`는 겹치면 `unpark_blocked`로 알리고 맡긴 ref를 그대로 둔다. 화면에는 "맡겨 둔 파일 N개 · 겹침" 글자만 있다.
- `undo/backup.rs`는 백업 ref(`refs/higit/backup/<UTC>-<순번>/<이름>`)를 만들 뿐 정리하지 않는다. `RefEntry`는 이름·해시만 가진다. `GitBackend`에 reflog·stash 목록 조회가 없다.
- `Runner::run_limited`는 시간 제한만 있다(출력을 끝까지 모은 뒤 해석, 진행 감시·취소 없음). 이벤트는 `repo-changed`, `watch-status` 둘뿐이다(`architecture.md`에 "`op-progress`는 아직 없다").
- `redact()`(`crates/core/src/redact.rs`)는 오류 화면용으로만 쓰이고 로그 파일은 없다. 비밀 파일 검사 없음. 실패 분류(`failure.rs`)에 잠금 파일·취소·멈춤이 없다.
- 화면: `WorkspaceView`는 `home | history`(`state/repoStore.ts`), 상단 바 오른쪽은 동기화와 버전 기록, `HomeMap`은 노선도만 그린다. 단축키 표의 `scope`는 `global | field | fileList`.
- 이 PC와 Ubuntu 22.04 기본 git은 2.34.1이다(D-050으로 거부 기준 2.30 유지). 새로 쓰는 git 명령은 `git-min` CI(Ubuntu 22.04 컨테이너)에서 확인한다.

## 완료 기준 (반복 2 전체)
두 OS(Ubuntu 22.04 git 2.34.1, Windows 11)에서 손으로, 그리고 가능한 것은 자동 테스트로 확인한다.
- main에서 갈라진 브랜치에서 main에 새 버전이 있으면 노선도 옆에 "main에 새 버전 N개"와 "main 가져와 합치기"가 보인다. 겹치지 않으면 누르는 즉시 합치고 "합쳤어요 · 되돌리기"가 뜨며 Ctrl+Z로 합치기 전 상태(고친 파일 그대로)로 돌아온다. 이번 버전에 넣은 파일이 있으면 시작하지 않고 이유를 알린다.
- 겹치면 같은 버튼이 충돌 해결 화면을 연다. 위아래 카드 두 장에 "내 브랜치 · 사람 · 날짜 · 이름"과 "합칠 브랜치 · …", 한 곳씩 "3곳 중 2곳 남음", 고르면 "저장될 모습"이 바뀌고, `<<<<<<<` 표시가 화면 어디에도 없다. 모두 고르면 주 버튼이 "합치기 마치기"가 되어 합친 버전이 생기고, "합치기 그만두기"는 늘 보이며 누르면 합치기 전으로 돌아간다. 앱을 껐다 켜도 합치는 중이면 이 화면이 열린다.
- 동기화에서 겹치면 같은 화면으로 이어진다(합치기가 시작된 상태).
- 앞서 있고 뒤처지지 않은 브랜치의 브랜치 목록 "지금" 줄에 "main에 합치기"가 있고, 누르면 main으로 갈아타 합쳐지며 Ctrl+Z로 원래 브랜치와 main 위치가 돌아온다.
- 체크 상자로 넣기·빼기를 한 뒤 Ctrl+Z가 그 한 번을 되돌린다("버전 저장 → 체크 → Ctrl+Z"는 체크만 되돌린다). 올린 버전을 저장한 뒤 Ctrl+Z는 기록을 지우지 않고 "되돌리는 새 버전"을 만든다. 올리기·동기화는 "올린 것은 되돌릴 수 없어요"와 "작업 내역" 버튼을 보여준다.
- 꺼내다 겹친 맡긴 파일은 브랜치 목록의 "겹침 해결"로 같은 화면에서 풀고, 마치면 고친 파일로 나온다. 그만두면 꺼내기 전 그대로다.
- "임시 보관"으로 고친 파일을 보관하고 꺼낸다. "고친 것 버리기"는 파일 이름을 보여주는 확인을 거친 뒤 버리고, 작업 내역에서 꺼낼 수 있다. 되돌릴 수 없는 일(`Irreversible`)만 확인을 묻는다.
- 상단 바 "작업 내역"(Ctrl+Shift+H)이 앱 작업·reflog·백업·맡긴 파일·임시 보관을 시간순으로 합쳐 보여주고, "이 시점으로 되돌리기"와 "꺼내기"가 되며 되돌린 뒤에도 "지금 버전은 남아요". Ctrl+Z가 `UndoStale`이면 토스트의 "작업 내역" 버튼으로 이 화면이 열린다. 30일 지난 백업 ref는 프로젝트를 열 때 정리된다.
- 동기화·원격 확인 중 "취소"가 되고, 첫 출력 뒤 90초 동안 아무것도 오지 않으면 끝낸다. 중간에 끊겨 남은 잠금 파일과 임시 보관 목록에 남은 "맡겨 둔" 항목은 프로젝트를 열 때 알림 줄로 알리고 확인 뒤 정리한다.
- 버전 저장 때 `.env`·키 파일·토큰이 든 파일이 있으면 알림 줄이 뜬다(일치한 글자는 어디에도 나오지 않음). 로그 파일(`<앱 데이터>/logs/`)에 토큰·이메일·홈 경로가 없다.
- 두 OS CI 초록(`git-min` 포함). `pnpm check:full` 통과. code-analyst 점검 위반 0건.

## 단계
각 단계 = 커밋 하나, 메시지는 `<영역>: <요약>`. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령·액션 목록이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`. 새 Tauri 명령은 `src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣는다(`tests/security_config.rs`가 확인). 새 액션은 architecture.md "새 액션 추가 절차"대로 spec 용어표·glossary → `GitOp` → `ActionInput`·`actions/<id>.rs`·레지스트리 → 전용 UI → 테스트(임시 저장소에서 실행 → 결과 → 되돌리기). 테스트 저장소는 `core.autocrlf false`(`git::testutil`). `GitBackend`에 메서드를 더하면 가짜 구현 둘(`executor.rs` 테스트의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)도 같은 커밋에서 고친다. 새 git 명령에 리비전을 넘기면 `args::Cmd` 변형을 더하고 `git-min`에서 확인한다(`Oid`는 16진수만 받아 옵션으로 읽히지 않으니 표시를 못 넣는 명령은 `Oid`만 넘긴다). 새 의존성은 없다(비밀 규칙·시각 계산은 직접 구현).

### A. 용어와 바탕

R1. 문서·용어: 반복 2의 결정과 용어 (`docs: add iteration 2 decisions and terms`)
- 목표: 이후 단계가 쓰는 결정과 용어를 먼저 확정한다(D-015: spec 표, glossary ko·en 함께).
- 바꿀 파일: `docs/decision.md`(C1~C18을 D-069부터 "(자동 결정)", 상태 확정), `docs/spec.md`(용어표 행 추가·이름 변경 아래 "용어 변경", "화면 구성"의 합치기/충돌 행 진입 문구를 "홈의 합치기 제안·동기화 중 겹침", 복구 행 이름을 "작업 내역"으로, "충돌 해결 화면" 5번의 위험도 문장을 "합치기 그만두기는 `normal`, 되돌릴 것이 없음"으로), `src/glossary/{types,ko,en}.ts`(`TERM_IDS`는 spec 표 순서), `src/messages/*`는 옛 이름 사용처 없음을 확인.
- 테스트: `src/glossary/spec-sync.test.ts`, `TERM_IDS` 순서, `src/messages/copy-rules.test.ts`(괄호·옛 용어·해요체), `terms-in-messages.test.ts`.
- 완료 기준: `pnpm test`, `pnpm lint`.
- Windows/Ubuntu: 차이 없음.

R2. core: 실행기가 `caution` 이상을 실행한다 (`core: run caution actions with backup refs`)
- 목표: D-011을 이행해 `Caution` 이상 액션을 백업 ref와 journal 흐름 안에서 실행한다(D-043).
- 바꿀 파일: `crates/core/src/executor.rs`(NotYetSupported 분기 삭제), `crates/core/src/undo/mod.rs`(`UndoKind::Barrier` 추가), `crates/core/src/action.rs`(`Outcome::journal_mode() -> Journal | Barrier | Skip`, 기본 `Journal`), `crates/core/src/error.rs`·`view/mod.rs`(`NotYetSupported`·`ErrorCode::NotYetSupported` 삭제, `UndoKindView::Barrier`), `src/messages/{ko,en}.ts`(해당 문구 삭제), 생성물.
- 내용:
  - 흐름: `Irreversible`이면 토큰 확인 → `risk >= Caution`이면 HEAD를 `backup::backup_oid(label = 액션 id)`로 남기고 실패하면 실행하지 않는다(HEAD가 없는 첫 버전 전은 건너뜀) → `before` 캡처 → 실행 → `outcome.journal_mode()`에 따라 Action 줄·Barrier 줄·없음.
  - `UndoKind::Barrier`: 되돌릴 수 없지만 확인 없이 실행하고 경계 줄을 남긴다("이 앞 줄은 되돌리지 않는다", D-065). `is_barrier`는 `Irreversible | Barrier`. (`RestoreIndex`는 R3에서 일반 줄로 바뀐다.)
  - `Outcome::journal_mode`는 이후 단계의 결과(합치는 중, 합치기 마침의 맡긴 파일 쪽)가 쓴다.
- 테스트: 가짜 백엔드로 `Caution`+`Irreversible`은 토큰 없이 `PreviewRequired`, 토큰이 있으면 백업 ref가 실행보다 먼저 생기고 Barrier 줄이 남는다. 백업 ref 쓰기가 실패하면(가짜가 오류) 액션이 실행되지 않는다. HEAD 없음이면 백업 없이 실행. `Safe`·`Normal` 흐름은 그대로.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음.
- 보안: "지우거나 덮어쓰는 일은 백업 ref와 journal을 먼저"를 실행기 한 곳이 강제한다. `refs/higit/`는 기존 `undo::backup`만 만든다.

R3. git·core: 되돌리기 넓히기 — 넣기·빼기, 합치기, 다른 ref (`core: undo stage, merge and moved refs`)
- 목표: 이번 버전에 넣기·빼기를 Ctrl+Z로 되돌리고(D-065 이행), 합치기와 다른 브랜치를 옮기는 액션이 쓸 되돌리기 바탕을 만든다.
- 바꿀 파일: `crates/git/src/types.rs`·`cli/mod.rs`(`GitOp::ResetKeep { oid }` = `reset --keep -q <oid>`), `crates/core/src/undo/mod.rs`(`UndoKind::RestoreTree`, `UndoState.refs: Vec<(String, Option<String>)>` serde default, `capture(git, repo, tracked)`, `is_valid`에 ref 이름·해시 검사), `crates/core/src/action.rs`(`Action::tracked_refs(&self, &ActionInput) -> Vec<String>` 기본 빈 목록), `executor.rs`(실행 전후에 추적 ref 값을 `resolve_ref`로 캡처, `RestoreIndex`도 Action 줄로 기록하되 `before.index_tree`가 없으면 Barrier), `undo/run.rs`, `view/mod.rs`(`UndoKindView`).
- 내용:
  - `RestoreIndex`: 지금 HEAD·브랜치와 `index_tree`가 `after`와 같을 때만(다르면 `UndoStale`) `ReadTree { before.index_tree }`. 다시 하기는 `after.index_tree`로.
  - `RestoreTree`: 지금 HEAD가 `after.head`일 때만. 지금 HEAD를 백업 ref(`undo-<브랜치>`)로 남기고 `ResetKeep { before.head }`(고친 파일은 두고, 겹치면 git이 실패하므로 `WouldOverwrite`). 다시 하기는 `ResetKeep { after.head }`.
  - `refs`: 종류별 되돌리기를 끝낸 뒤 `before.refs`의 (ref, 이전 값)을 `update_ref(…, Is(after 값))`/`delete_ref`로 되돌린다(경쟁이 있으면 실패).
  - 합치기·충돌이 진행 중이면 `undo`·`redo` 모두 `OperationInProgress`로 거절한다(`refuse_during_operation`).
  - `stage`·`unstage` 되돌리기는 토스트를 띄우지 않고 Ctrl+Z로만 한다(체크 상자 하나마다 알림이 쌓이지 않게). `useUndo`의 "되돌렸어요" 토스트는 그대로.
- 테스트: 실제 git. 넣기 → undo → 넣기 전, 빼기 → undo, 넣기 두 번 → Ctrl+Z 두 번, "저장 → 넣기 → Ctrl+Z"는 넣기만 되돌린다, 터미널에서 `git add` 하면 `UndoStale`, 충돌 중이면 Barrier(되돌리기 없음). `RestoreTree`: 가짜 버전 둘을 만들고 `ResetKeep` 뒤 파일 내용이 이전이고 고친 파일(겹치지 않음)이 그대로, 겹치면 실패하고 아무것도 안 바뀐다. 이름이 `-x`인 ref가 든 journal 줄은 무시. 이전 형식(`refs` 없음)의 journal 줄이 읽힌다.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: `reset --keep`은 Windows에서 편집기가 파일을 잡으면 실패할 수 있다(오류로 알림). `reset`은 `--end-of-options`를 넣지 않는다(`args::Cmd::Reset` 기존 규칙).
- 보안: journal은 믿지 않는 입력(`UndoState::is_valid`로 ref 이름·해시 검사, D-063 서명 유지).

R4. git·core: 올린 뒤 되돌리기는 되돌리기(revert) 새 버전 (`core: undo a pushed version with a revert`)
- 목표: 올린 버전을 저장한 뒤 Ctrl+Z가 기록을 바꾸지 않는 반대 동작으로 이어진다(D-042). push·pull·sync는 되돌릴 수 없음을 알리고 작업 내역으로 안내한다(C5).
- 바꿀 파일: `crates/git/src/types.rs`·`cli/mod.rs`·신규 `cli/revert.rs`(`GitOp::Revert { oid, mainline: Option<u32> }`: `revert --no-edit [-m N] <oid>`, 실패하면 `revert --abort`(안 되면 `reset --merge -q`)로 정리하고 `MergeConflict`·`WouldOverwrite` 분류), `cli/args.rs`(`Cmd::Revert`), `crates/core/src/undo/{run.rs, journal.rs}`(`Body::Undo`에 `reverted: bool` serde default, 되돌린 줄이 `reverted`면 다시 하기 대상에서 뺌), `error.rs`·`view/mod.rs`(`CoreError::StagedChanges`, `ErrorCode::StagedChanges`), `view/mod.rs`(`UndoResult.reverted`), `src/messages/{ko,en}.ts`, `src/features/undo/useUndo.ts`.
- 내용:
  - `undo_commit`·`RestoreTree` 되돌리기가 "이미 올림"이면 `UndoPushed` 대신 되돌리기 새 버전을 만든다: 합치기 버전(부모 2개)은 `-m 1`. 메시지는 git 기본 형식(`Revert "<이름>"`)이라 undo 입력에 앱 언어가 필요 없다(기록에 언어가 섞이는 점은 위험에 적음). 이번 버전에 넣은 파일이 있으면 시작하지 않고 `StagedChanges`.
  - `stale` 검사로 HEAD가 그 버전일 때만 하므로 내용 충돌은 없고, 고친 파일이 겹치면 git이 거절한다(`WouldOverwrite`).
  - `Inverse` 종류(push·pull·sync)는 지금처럼 `UndoPushed`/`NothingToUndo`이고 문구만 "올린 것은 되돌릴 수 없어요"로 바꾼다. 화면의 "작업 내역" 버튼은 R20.
  - 토스트: "되돌리는 새 버전을 만들었어요"(`reverted`). 동기화를 눌러야 원격에 반영되므로 안 올린 버전 수가 늘어난다.
- 테스트: 로컬 bare 원격(`allow_local_remotes_for_tests`)에 올린 뒤 undo → 새 버전이 생기고 파일 내용이 이전, 기록에 원래 버전도 남음. 합치기 버전은 `-m 1`로 첫 부모 쪽 내용으로. 다시 하기는 `NothingToUndo`. 넣은 파일이 있으면 `StagedChanges`. 고친 파일과 겹치면 `WouldOverwrite`이고 작업 폴더가 그대로. `commit.gpgsign`·identity 없음. `revert` 실패 후 정리로 index가 깨끗.
- 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신.
- Windows/Ubuntu: 메시지를 앱이 만들지 않아 명령줄 인코딩 문제가 없다. 2.34.1에서 `revert --no-edit -m`·`revert --abort`를 `git-min`에서 확인(`GIT_EDITOR=:`라 편집기가 뜨지 않음).
- 보안: 되돌림이 원격 기록을 바꾸지 않는다(새 버전을 더할 뿐). 입력은 `Oid`와 정수뿐이라 옵션으로 읽히지 않는다.

### B. 보안·신뢰성 앞부분 (D-040, D-051이 "반복 2 앞부분"으로 정한 것)

R5. core·platform·app: 로그 파일과 로그 가리기 (`core: write redacted log file`)
- 목표: D-040의 로그 부분. 앱 데이터 폴더에만 남기고 쓰기 전에 한 함수로 가린다(C10).
- 바꿀 파일: 신규 `crates/core/src/log.rs`(`Logger::open(dir)`, `info/warn/error`), `crates/core/src/redact.rs`(`redact`를 `pub(crate)` 그대로, 파일 경로 접두사 처리 추가는 하지 않음), `crates/platform/src/app_dir.rs`(`open_private_append(path)`: Unix 0600), `crates/core/src/bootstrap.rs`(`Core::logger()`), `src-tauri/src/commands.rs`(`blocking()`이 실패를 로그), `app_open_log_folder` 명령(네 곳, opener는 Rust에서만), `src/lib/api.ts`(`openLogFolder`).
- 내용: `<앱 데이터>/logs/higit.log`, 1MB가 넘으면 `higit.1.log`→`higit.2.log`로 순환(셋까지), 시작할 때 14일 지난 파일 삭제. 남기는 것은 시각·종류·액션 id·오류 코드·`redact`한 "자세히 보기" 원문뿐이고 파일 이름·버전 메시지·비교 내용은 쓰지 않는다. 앱 데이터 폴더를 못 쓰면 로그 없이 계속한다(최근 목록과 같은 방식).
- 테스트: 토큰·`https://user:pass@`·이메일·홈 경로가 든 오류를 로그에 쓰면 파일에 그 글자가 없다. 순환(1MB 초과 → 새 파일), 오래된 파일 삭제, 폴더를 못 쓰면 무시. Unix에서 권한 0600(platform 테스트).
- 완료 기준: `cargo test --workspace`, 바인딩·`security_config` 통과.
- Windows/Ubuntu: 위치는 `%LOCALAPPDATA%\dev.higit.desktop\logs`와 `$XDG_DATA_HOME/dev.higit.desktop/logs`(platform이 정함). 열기는 opener(`xdg-open`/탐색기).
- 보안: "비밀: 토큰·주소 속 비밀번호가 로그에 들어가지 않는다"를 파일 로그까지 넓힌다. 폴더 열기 주소는 Rust가 정한다(웹뷰 opener 권한 없음).

R6. core: 비밀 파일 검사와 제외할 파일 추가 (`core: scan files for secrets and add ignore action`)
- 목표: 버전 저장 전에 "키나 비밀번호로 보이는 파일"을 알아내고, 제외할 파일에 넣을 수 있게 한다(D-040, C9).
- 바꿀 파일: `crates/git/src/backend.rs`·`cli/mod.rs`(`index_entries(repo) -> Vec<IndexEntry { path, mode, stage, oid }>` = `ls-files -s -z`, `read_blob(repo, oid, max) -> Blob { bytes, size }` = `cat-file -s`·`cat-file blob`을 `run_capped`로), `crates/core/src/secrets.rs`(규칙 데이터와 직접 만든 일치 함수), `crates/core/src/query.rs`(`secret_check`), `crates/core/src/view/secret.rs`(`SecretCheckRequest { stage_all }`, `SecretCheckView`, `SecretFinding { path, rule, line }`), `crates/core/src/actions/ignore.rs`, `view/input.rs`(`Ignore(PathsInput)`), `undo/run.rs`(`Inverse` 중 `ignore` 처리기), `src-tauri`(`query_secret_check` 네 곳), `src/lib/{api,queries}.ts`, 생성물, 용어는 R1에서 끝남.
- 내용:
  - 대상: 넣은 파일(index와 HEAD가 다른 것). 넣은 파일이 없고 `stage_all`이면 모든 고친 파일. 스캔은 최대 200개, 파일당 앞 256KB.
  - 규칙: 이름(`.env`, `.env.*`(`.example`·`.sample`·`.template` 제외), `id_rsa`·`id_ed25519`·`id_ecdsa`, `*.pem`·`*.key`·`*.p12`·`*.pfx`·`*.keystore`, `credentials.json`, `service-account*.json`), 내용(`-----BEGIN … PRIVATE KEY-----`, `ghp_`·`gho_`·`github_pat_`, `AKIA` + 16자, `xoxb-`·`xoxp-`, `sk-` + 20자 이상), 크기(50MB 이상, GitHub 100MB 한도 전). 정규식 crate 없이 접두사 + 문자 종류 + 길이로 구현한다.
  - 결과에는 경로·규칙 이름·줄 번호만 담고 일치한 글자는 담지 않는다. 링크는 읽지 않는다(`symlink_metadata`). 넣은 파일은 index 블롭을, 안 넣은 파일은 작업 폴더 파일을 읽는다.
  - `ignore`: 입력 `PathsInput`. 새 파일(Untracked)이거나 새로 넣은 파일(index `Added`)만 받는다(이미 추적 중이면 `InvalidInput`). 저장소 루트 `.gitignore`에 `/경로` 줄을 덧붙이고(`\`·`*?[`·앞의 `#`·`!`·끝 공백을 이스케이프, 줄바꿈 든 이름은 거절, 파일의 줄바꿈 방식 유지, 임시 파일 뒤 rename) 넣은 파일은 `Unstage`한다. 메타: `ignore`/`ignore`/Safe/`Inverse`. 되돌리기: 덧붙인 줄을 지우고 `ReadTree { before.index_tree }`.
- 테스트: 규칙마다 걸리는 예와 안 걸리는 예(`.env.example`, 짧은 `ghp_x`), 결과를 JSON으로 만들어 비밀 글자가 없음을 확인, 한글 파일명, 링크는 건너뜀, 200개 넘으면 `skipped`. `ignore`: 새 파일 → `.gitignore`에 줄, `git status`에서 사라짐, 이스케이프 표본, CRLF `.gitignore`, undo로 줄과 넣은 상태 복원, 추적 중인 파일은 거절.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `registry-glossary` 테스트(`ignore` 용어 있음).
- Windows/Ubuntu: `.gitignore` 줄바꿈은 파일에서 읽어 따른다. 경로 구분자는 `/`.
- 보안: 새 IPC 명령은 열린 저장소만(`opened_repo`)·경로는 `paths::validate`. 결과에 비밀 원문 없음. 새 의존성 없음.

R7. 화면: 비밀 파일 알림 줄 (`features: secret warning before saving`)
- 목표: 버전 저장을 누르는 순간 걸리면 알림 줄을 띄운다(모달 아님).
- 바꿀 파일: `src/features/secrets/{SecretNotice.tsx, SecretNotice.module.css, SecretNotice.test.tsx}`(신규), `src/features/commit/NameBox.tsx`, `src/messages/{ko,en}.ts`.
- 내용: 저장하면 먼저 `secretCheck`를 부르고 걸린 것이 없으면 그대로 저장한다. 걸리면 저장하지 않고 이름 칸 아래에 "키나 비밀번호로 보이는 파일 N개"와 파일 이름·규칙 이름(최대 3개)을 보여준다. 버튼: `<Term id="unstage" />`("이번 버전에서 빼기", 넣은 파일일 때), `<Term id="ignore" />에 추가`(모두 새 파일일 때만), "그래도 저장"(저장). 같은 파일 묶음으로 두 번째 저장(Ctrl+Enter 포함)은 바로 저장한다(C9). 빼기·제외하기가 끝나면 알림을 지우고 토스트 "제외했어요 · 되돌리기".
- 테스트: 걸림 → 저장 안 됨, 두 번째 저장은 저장됨, 빼기·제외 버튼이 맞는 액션을 부름, 한글 조합 중 Enter 무시(기존), 저장소 글자는 React 글자로만 그림(`innerHTML` 없음).
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: `.env`를 만들고 저장.
- Windows/Ubuntu: 차이 없음.
- 보안: 화면에는 경로·규칙 이름만. `dangerouslySetInnerHTML` 없음.

### C. 합치기와 충돌 해결

R8. git: 충돌을 읽고 해결하는 조각 (`git: add unmerged entries, blobs and merge segments`)
- 목표: 충돌 해결 화면과 마치기가 쓸 git 조각을 `GitBackend`에 둔다.
- 바꿀 파일: `crates/git/src/backend.rs`·`cli/mod.rs`(R6의 `index_entries`로 `stage > 0`만 걸러 충돌 목록을 얻으므로 새 조회 하나: `merge_segments(repo, base, ours, theirs) -> Vec<MergeSegment>`), 신규 `cli/segments.rs`, `cli/merge_check.rs`(`unmerged` 파서를 `index_entries`와 공유), `cli/runner.rs`(종료 코드를 실패로 보지 않는 `run_status`), `types.rs`(`MergeSegment::{Same(Vec<u8>), Conflict { ours, base, theirs }}`, `GitOp::ResolveFile { path, mode, bytes }`, `GitOp::ResolveDelete { path }`), `cli/pathcheck.rs`(`park.rs`의 `safe_relative`를 옮겨 둘이 쓴다).
- 내용:
  - `merge_segments`: 임시 폴더 `Scratch`에 세 파일을 쓰고 `merge-file -p --diff3 --marker-size=31 -L O<무작위> -L B<무작위> -L T<무작위>`를 돌려 앱이 정한 이름표 줄만 표시로 읽는다(내용에 `<<<<<<<` 줄이 있어도 속지 않음). 줄 끝(`\r\n`)을 그대로 두어 이어 붙이면 원본과 같다. 종료 코드 0~127은 충돌 수, 255(이진 파일·오류)는 실패. 표시를 해석하지 못하면 오류로 돌려 호출 쪽이 파일 단위 선택으로 떨어진다.
  - `ResolveFile`: `hash-object -w --no-filters --stdin` → `update-index -z --index-info`(stdin `모드 해시<TAB>경로NUL`) → `checkout-index -f -z --stdin`. 해결 내용은 index용 정리된 형식(LF)이고 작업 폴더 파일은 git이 사용자의 줄바꿈 설정으로 만든다. 모드는 100644·100755·120000만 받는다.
  - `ResolveDelete`: `git rm -f -q`(`--literal-pathspecs`, 경로는 stdin).
  - 모든 경로는 `safe_relative`를 거친다.
- 테스트: 임시 저장소에서 갈라진 두 브랜치의 같은 줄 수정 → `Merge`가 `MergeConflict`로 실패하고 `index_entries`에 stage 1·2·3, `read_blob`이 세 내용, `merge_segments`가 기대한 구간. 표시처럼 보이는 줄(`<<<<<<<`, `=======`, 마크다운 밑줄)이 든 파일, 한글·공백 파일명, 파일 끝 줄바꿈 없음, 한쪽이 지운 파일. `ResolveFile` 뒤 충돌 항목이 사라지고 내용이 기대와 같다. CRLF: `.gitattributes`로 `text eol=crlf`를 주고 `core.autocrlf false`에서 해결한 파일이 작업 폴더에 CRLF, index 블롭은 LF. 이진 파일은 `merge_segments` 오류. `ResolveFile` 경로가 `../x`·`.git/config`면 거절. 테스트 뒤 `git status`가 기대와 같다.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min` 통과.
- Windows/Ubuntu: `merge-file`·`checkout-index`의 줄바꿈 처리가 OS 설정에 따라 다르므로 위 CRLF 테스트를 두 OS에서 돌린다. 임시 파일은 `std::env::temp_dir()` 아래 앱 폴더. 2.34.1에서 `merge-file --marker-size/--diff3`, `update-index -z --index-info`, `checkout-index -z --stdin`을 `git-min`에서 확인한다.
- 보안: 인자 배열·`--literal-pathspecs`·경로 stdin. 경로가 작업 폴더 밖이면 거절(`checkout-index`는 링크를 건너 쓰지 않음을 테스트).

R9. core: 합치기 제안과 합치기 액션 (`core: add merge hint query and merge actions`)
- 목표: D-048의 방향 하나 제안과 두 액션을 만든다. 겹치면 합치기를 시작해 충돌 상태로 둔다(C1, C3).
- 바꿀 파일: `crates/core/src/query.rs`(`merge_hint`), `view/merge.rs`(`MergeHintView { kind: None|Bring|Into, branch, behind, ahead, overlap: Vec<String>(5개까지), overlap_count }`), `actions/merge.rs`·`actions/mod.rs`(등록), `view/input.rs`(`Merge(MergeInput { branch, locale })`, `MergeInto(MergeIntoInput { branch })`), `action.rs`(`Outcome::Merged { branch, into, fast_forward }`, `Outcome::MergeConflicts { paths }`, `journal_mode`: 충돌은 `Skip`), `view/mod.rs`(`ActionResult`), `actions/remote.rs`(sync 변경), `error.rs`·`view/mod.rs`(`StagedChanges`는 R4에서 있음), `src-tauri`(`query_merge_hint` 네 곳), `src/lib/{api,queries}.ts`, `src/messages/{ko,en}.ts`(`notFastForward` 문구에서 "아직 준비 중" 삭제), 생성물.
- 내용:
  - `merge_hint`: 기본 브랜치(`default_branch`)와 지금 브랜치(브랜치일 때, 기본 브랜치가 아닐 때)에서 `count_between`. 기본 브랜치에 새 버전이 있으면 `Bring`(내 쪽도 앞서 있을 때만 `merge_check(HEAD, 기본 브랜치)`로 겹친 파일), 아니고 내 쪽만 앞서면 `Into`, 둘 다 0이면 `None`. 기준은 `home_map`과 같은 로컬 기본 브랜치.
  - `merge`(`merge`/Normal/`RestoreTree`): 브랜치 이름 검사(`validate_ref_like`, 있는 로컬 브랜치, 지금과 다름) → `refuse_during_operation` → 넣은 파일이 있으면 `StagedChanges`(합치기 커밋에 섞이거나 겹침 판정이 흐려지는 것을 막음) → HEAD 백업 ref → 뒤처지기만 하면 `MergeFf`, 아니면 `Merge`(메시지 `"<브랜치> 가져와 합침"`/`Merge <branch>`, `locale`). 겹쳐서 `MergeConflict`로 실패하면 `merge --abort`하지 않고 `MERGE_HEAD`가 있는지 확인해 `MergeConflicts { paths }`(충돌 파일 목록)로 돌려준다. 다른 실패는 그대로 오류.
  - `merge-into`(`merge`/Normal/`SwitchBack`, `tracked_refs = [refs/heads/<브랜치>]`): 뒤처지지 않았을 때만(`is_ancestor(브랜치, HEAD)`, 아니면 `NotFastForward`). `switch_with_parking(브랜치)`로 갈아탄 뒤 `MergeFf { 이전 브랜치 }`. 실패하면 이전 브랜치로 돌아가고 오류. 되돌리기는 `undo_switch`가 `MergeInto` 입력도 받도록 넓혀(원격 브랜치 처리 없음) 이전 브랜치로 돌아가고, R3의 `refs`가 기본 브랜치 ref를 원래 값으로 되돌린다. 다시 하기는 지원하지 않는다(`NothingToUndo`).
  - sync 변경: 겹치면(미리 본 판정이 맞든 틀리든) 합치기를 시작해 충돌 상태로 두고 `SyncReport { merge_started: true, overlap }`을 돌려준다(`merge --abort` 안전망 삭제). 시작할 때 `refuse_during_operation`, 겹치는데 넣은 파일이 있으면 `StagedChanges`. `Outcome::Synced`는 `merge_started`이면 `journal_mode = Skip`. 합치기가 끝나도 자동으로 올리지 않는다(동기화를 다시 누름).
- 테스트: 실제 git. 힌트: 뒤처짐/앞섬/갈라짐/같음/기본 브랜치 없음/겹침 파일 5개 한도. `merge`: 겹치지 않는 갈라짐 → 합친 버전(메시지 확인), 뒤처지기만 → fast-forward, 겹침 → `MERGE_HEAD` 있고 `MergeConflicts`, 넣은 파일 있음 → `StagedChanges`, 고친 파일이 겹침 → `WouldOverwrite`, 합치는 중에 또 부르면 `OperationInProgress`, 없는 브랜치·`-x`. 되돌리기: 합친 뒤 Ctrl+Z → 합치기 전 HEAD이고 고친 파일 그대로. `merge-into`: main으로 갈아타 ff, undo → 원래 브랜치와 main 위치 복원, 맡긴 파일이 있어도 잃지 않음, 뒤처졌으면 `NotFastForward`. sync 겹침 → `MERGE_HEAD` 있음(005 H13 테스트의 "합치기 시작 안 함" 기대를 이 단계에서 고침). git 2.38 미만 경로는 가짜 버전으로.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션.
- Windows/Ubuntu: 로컬 원격은 Windows `C:\…` 그대로. 메시지는 `-m` 인자 배열(2.34.1의 `merge`는 `-F -`를 읽지 못함).
- 보안: 브랜치 이름은 `validate_ref_like`·`push_revs`로, 힌트 조회는 `opened_repo`를 거친다.

R10. core: 충돌 조회 (`core: add conflict state and file queries`)
- 목표: 충돌 해결 화면이 쓰는 두 조회. 표시 문자는 화면으로 나가지 않는다.
- 바꿀 파일: 신규 `crates/core/src/conflict.rs`(구간 계산·조립), `query.rs`(`conflict_state`, `conflict_file`), `view/conflict.rs`(`ConflictStateView { active, context: Merge, ours, theirs: ConflictSide { label, author, at, summary, short_oid }, files: Vec<ConflictFileSummary { path, kind: Text|Whole, hunks }>, clean_files, other_operation: bool }`, `ConflictFileView { path, kind, hunks: Vec<ConflictHunkView { index, line, ours, base, theirs, before: Vec<String>, after: Vec<String> }>, ours_side, theirs_side: SideState }`, `SideState::{Text, Binary, Deleted, Link(String)}`, `FileKind`), `src-tauri`(`query_conflict_state`, `query_conflict_file` 네 곳), `src/lib/{api,queries}.ts`(키 `["mergeState", repo]`), 생성물.
- 내용:
  - 상태: `MERGE_HEAD`(`resolve_ref`)가 있으면 합치는 중(context `Merge`). 충돌 파일은 `index_entries`의 `stage > 0`을 경로별로 묶은 것(내 쪽 = stage 2, 합칠 쪽 = stage 3, 원래 = stage 1). 내 쪽 이름표는 지금 브랜치와 HEAD 버전의 사람·날짜·이름, 합칠 쪽은 `MERGE_HEAD` 버전과 그 oid를 가리키는 브랜치 이름(없으면 짧은 해시). 합치지 않은 상태에서 충돌 항목만 있고 앱이 다루는 맥락이 아니면(리베이스·cherry-pick·revert 중) `other_operation`으로만 알린다(C18).
  - `clean_files`: 충돌이 아닌, 이미 index에 합쳐 넣어진 파일 수(`status`의 넣은 파일 중 충돌 아님).
  - 파일: 세 쪽이 모두 UTF-8 텍스트이고 NUL 없음·각 1MB 이하이면 `Text`로 `merge_segments`의 `Conflict` 구간마다 `ConflictHunkView`(내 쪽 시작 줄 번호, 앞뒤 3줄 맥락). 이진·한쪽 삭제·링크·큰 파일·모드 충돌·서브모듈(160000)은 `Whole`로 각 쪽의 `SideState`와 앞 40줄 미리보기만. 읽는 것은 object database의 블롭뿐이라 작업 폴더 링크를 따라가지 않는다.
  - 충돌 파일이 200개를 넘으면 앞 200개만 보내고 나머지 수를 `more_files`로 준다.
- 테스트: 실제 git으로 합치는 중이 아님(`active: false`), 합치는 중 + 충돌 파일 둘(텍스트 하나·이진 하나), 구간 3개 파일의 `line`·맥락, 한쪽이 지운 파일, 표시처럼 보이는 줄이 든 파일에서 구간이 맞음, 응답 JSON 어디에도 `<<<<<<<`·무작위 이름표가 없음, 열지 않은 저장소는 `NotOpened`, 리베이스 중은 `other_operation`.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음(줄 끝은 그대로 담고 화면용 문자열만 줄 끝 제거).
- 보안: 새 IPC 명령 네 곳·열린 저장소 검사·경로 `paths::validate`. 저장소에서 온 글자는 데이터로만 보낸다.

R11. core: 충돌 마치기와 그만두기 (`core: add conflict finish and abort actions`)
- 목표: 화면에서 고른 것을 index·작업 폴더에 쓰고 합친 버전을 만들거나, 합치기를 그만둔다.
- 바꿀 파일: `actions/conflict.rs`·`actions/mod.rs`, `view/input.rs`(`ConflictFinish(ConflictFinishInput { resolutions: Vec<FileResolution { path, choice }>, locale })`, `FileChoice::{Hunks { choices: Vec<HunkChoice> }, Ours, Theirs}`, `HunkChoice::{Ours, Theirs, Both, Custom { text }}`, `ConflictAbort`), `action.rs`(`Outcome::ConflictFinished { merged: bool }`, `ConflictAborted`), `view/mod.rs`(`ActionResult`, `ErrorCode::ConflictChanged`), 생성물, `src/messages/{ko,en}.ts`.
- 내용:
  - `conflict-finish`(`conflict`/Normal/`RestoreTree`): 합치는 중이어야 한다. `resolutions`가 남은 충돌 경로를 정확히 한 번씩 덮는지, 구간 수가 다시 계산한 것과 같은지 확인하고(다르면 `ConflictChanged`) 각 파일을 조립한다 — `Both`는 내 쪽 다음 합칠 쪽, `Custom`은 내 쪽 줄바꿈이 CRLF이면 CRLF로 바꿔 넣는다. `Whole`은 `Ours`/`Theirs`로 그 쪽 블롭을 쓰거나 그 쪽이 지웠으면 `ResolveDelete`. 모두 쓰면 `Commit { 메시지 }`(합치는 중이라 병합 버전이 된다, 메시지 `"<합칠 쪽> 가져와 합침"`/`Merge <name>`, `locale`). 충돌이 하나도 남지 않았는데 `MERGE_HEAD`만 있으면(앞서 마치기가 커밋에서 멈춤) 쓰기 없이 커밋만 한다. 입력 상한: 파일 1000개, `Custom` 하나 1MB, 전체 8MB.
  - `conflict-abort`(`merge`/Normal/`Nothing`, 위험도는 C2): `MergeAbort`. 앱이 보여준 선택은 화면 상태일 뿐 git에 쓰지 않았으므로 잃을 것이 없고, 외부 편집기로 충돌 파일을 직접 고쳤다면 그 고침은 사라진다(위험에 적음). 합치는 중이 아니면 `InvalidInput`.
  - 둘 다 `refuse` 대신 리베이스 등 다른 작업 중이면 `OperationInProgress`. 작업 폴더 잠금.
- 테스트: R8 저장소로 구간 3개 파일을 `Ours`/`Theirs`/`Both`/`Custom`으로 마치면 파일 내용·병합 버전의 부모 둘·`git status` 깨끗, 이진 파일 `Whole` 양쪽, 한쪽 삭제 선택, 다 고르지 않으면 `InvalidInput`이고 아무것도 안 바뀜, 구간 수가 달라지면 `ConflictChanged`, 커밋 hook 실패 뒤 다시 마치기, Ctrl+Z로 합치기 전(고친 파일 그대로), `conflict-abort` 뒤 `MERGE_HEAD` 없고 HEAD 그대로, `Custom`에 CRLF·한글, 입력 상한 초과 거절. `.gitattributes` CRLF 파일.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션.
- Windows/Ubuntu: Windows에서 편집기가 파일을 잡고 있으면 `checkout-index`가 실패할 수 있다(오류로 알리고 다시 시도하면 이어짐: 이미 쓴 파일은 index가 해결됨).
- 보안: 입력은 serde 타입·길이 검사, 경로는 남은 충돌 경로 집합에 있는 것만(화면이 임의 경로를 쓰게 못 함). 커밋은 사용자 hook을 따른다(D-039 신뢰 확인을 거친 저장소).

R12. 화면: 합치기 제안 (`features: merge hint beside the map and in branch list`)
- 목표: r-mergehint대로 노선도 옆에 "main에 새 버전 N개 · main 가져와 합치기", 브랜치 목록 "지금" 줄에 "main에 합치기".
- 바꿀 파일: `src/features/merge/{MergeHint.tsx, MergeHint.module.css, MergeHint.test.tsx, useMergeAction.ts}`(신규), `src/screens/Workspace/HomeMap.tsx`(오른쪽 블록), `src/features/branches/BranchList.tsx`(지금 줄에 형제 버튼), `src/lib/queries.ts`(`useMergeHint`), `src/lib/useRepoWatch.ts`(`.git` 변경 때 힌트 무효화), `src/messages/{ko,en}.ts`.
- 내용: 노선도 줄 오른쪽에 굵은 글자 "main에 새 버전 N개"와 테두리 버튼 "main 가져와 <Term merge />"(D-049: 이름과 숫자만). 겹친 파일이 있으면 버튼 아래 옅게 "겹친 파일 N개"(툴팁에 5개까지). `Into`이면 노선도 옆에는 아무것도 없고 브랜치 목록 "지금" 줄에 `<Term id="merge" />` 버튼 "main에 합치기"(`aria-keyshortcuts` 없음, Tab으로 닿음). 결과: `merged` → 토스트 "합쳤어요 · 되돌리기"(`notifyDone`), `mergeConflicts` → 상태 쿼리 무효화(충돌 화면으로, R14), `StagedChanges` 등은 토스트 한 줄. GitHub 연결 프로젝트의 "검토 요청" 우선은 반복 4.
- 테스트: 종류별 표시(Bring/Into/None), 버튼이 `merge`·`merge-into` 입력(`locale` 포함)을 부름, 실패 코드별 문구, 겹침 표시, 800px에서 노선도와 겹치지 않음(말줄임).
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 라이트·다크, 800px.
- Windows/Ubuntu: 차이 없음. 색은 토큰만(주 버튼 규칙: 이 화면의 주 버튼은 버전 저장 하나이므로 이 버튼은 테두리).
- 보안: 브랜치 이름은 React 글자로만.

R13. 화면: 충돌 모델·상태·단축키 (`features: conflict model and shortcuts`)
- 목표: 충돌 화면의 계산을 DOM 없이 만들고 시험한다.
- 바꿀 파일: `src/features/conflict/{model.ts, model.test.ts, conflictStore.ts}`(신규), `src/lib/shortcuts.ts`·`shortcuts.test.ts`, `src/messages/{ko,en}.ts`, `docs/design.md`는 R14.
- 내용:
  - `model.ts` 순수 함수: `progress(files, choices)`(전체·남은 곳, 파일별 ✓/●), `savedPreview(hunk, choice)`(맥락 + 고른 내용의 줄과 각 줄이 어느 쪽 것인지), `toResolutions(files, choices, locale)`, `nextUnresolved(...)`, `canFinish`. `Both`는 내 쪽 다음 합칠 쪽.
  - `conflictStore`(zustand): 고른 것(파일별 구간 선택, `Whole` 선택, 직접 고친 글), 현재 파일·구간, 원래 코드 펼침. 합치기가 끝나면 비운다.
  - 단축키 표: `chooseOurs`(`Digit1`), `chooseTheirs`(`Digit2`), `chooseBoth`(`Digit3`), `nextHunk`(Ctrl+↓), `prevHunk`(Ctrl+↑), `activity`(Ctrl+Shift+H, R20이 씀). `scope`에 `"conflict"` 추가(충돌 화면이 열려 있을 때만 훅이 켜짐), 모두 `inTextField: "native"`(직접 고치기 글상자에서는 처리하지 않음). 겹침 테스트에 conflict 범위 키가 global 키와 겹치지 않는 검사를 더한다. Ctrl+Shift+H는 웹뷰 예약 키 목록과 겹치지 않음을 테스트.
- 테스트: 진행 계산 표(파일 여럿, 일부만 고름, 모두 고름), `savedPreview` 세 선택과 직접 고침, 이진 파일 선택, `toResolutions`가 입력 형식과 같음(타입), 한글 조합 중 Digit 키 무시.
- 완료 기준: `pnpm test`, `pnpm typecheck`.
- Windows/Ubuntu: Ctrl+Shift+H가 두 웹뷰에서 앱에 오는지 수동 확인(막히면 키만 바꾸고 표만 고침).

R14. 화면: 충돌 해결 화면 (`screens: conflict resolution screen`)
- 목표: D-028 1안 그대로 "한 곳씩 고르기" 화면과 합치는 중 전환.
- 바꿀 파일: `src/screens/Conflict/{Conflict.tsx, Conflict.module.css, Conflict.test.tsx}`, `src/features/conflict/{FileList.tsx, HunkCards.tsx, SavedPreview.tsx, WholeFile.tsx, useConflictState.ts, useFinish.ts}`(신규), `src/screens/Workspace/Workspace.tsx`(합치는 중이면 홈·기록 대신 이 화면), `TopBar.tsx`(합치는 중에는 동기화·브랜치 목록·버전 기록 단추를 숨기거나 끔), `src/features/sync/SyncButton.tsx`(합치기 시작이면 겹침 글자 대신 화면 전환), `src/ui/Button.tsx`(`danger` 변형은 R18), `src/styles/tokens.css`(`--conflict-ours-bg`, `--conflict-theirs-bg`: `--graph-2`·`--graph-1`의 옅은 바탕, 라이트·다크), `src/styles/contrast.test.ts`, `docs/design.md`(충돌 화면 색 규칙), `src/messages/{ko,en}.ts`.
- 내용(그림: design-direction.md의 충돌 1안, 색은 spec "색 규칙"):
  - 윗줄: "● 합칠 브랜치 → ● 내 브랜치"(색 점은 고정: 내 브랜치 `--graph-2`, 합칠 브랜치 `--graph-1`) · "충돌 3곳 중 2곳 남음" · 오른쪽 "합치기 그만두기"(글자 버튼, 늘 보임, `<Term id="merge" />` 툴팁). 모두 골랐을 때만 주 버튼 "합치기 마치기"가 "합치기 그만두기" 옆에 나타난다(화면의 주 버튼 하나).
  - 왼쪽(220px) 파일 목록: "겹친 파일", 줄마다 ✓(다 고름)·●(남음, 곳 수), 아래 "문제없이 합쳐진 N개"(펼치기 없이 숫자만). 진행 표시는 먹색 `--text`.
  - 오른쪽: 경로·"14번째 줄", 질문 한 줄 "두 브랜치가 같은 줄을 다르게 고쳤어요. 어느 쪽을 남길까요?", 위아래 카드 둘 — "<Term ours /> · 사람 · 날짜 · 이름"과 "<Term theirs /> · …", 카드 왼쪽 띠와 코드 바탕은 그 쪽 색 토큰, 고른 카드의 "이걸로"는 `--primary` 채움(고르지 않은 카드는 `--text-muted` 글자). 아래 "둘 다 쓰기"·"직접 고치기"(표시 문자 없는 글상자, 고른 내용으로 시작), "고치기 전 원래 코드 보기"(`<details>`, 툴팁 `<Term base />`), "저장될 모습"(맥락 줄 + 고른 내용, 줄 바탕이 고른 쪽 색), 이전·다음.
  - `Whole` 파일: 카드 둘 대신 각 쪽 상태("이 쪽에서는 지웠어요", "이진 파일", 링크 대상, 앞 40줄)와 "이걸로" 두 개.
  - 키: 1·2·3, Ctrl+↑↓. 모든 곳을 고르면 포커스가 "합치기 마치기"로 간다. 마치면 `conflict-finish` → 토스트 "합쳤어요 · 되돌리기", 홈으로. `ConflictChanged`·`WouldOverwrite` 오류는 윗줄 아래 안내 줄 + "자세히 보기".
  - 상태 쿼리가 `active`이면 앱을 켜자마자 이 화면(신뢰 확인 뒤). `other_operation`이면 홈 위에 안내 줄만(리베이스는 반복 3).
- 테스트: 질문·이름표·"3곳 중 2곳 남음", 고르면 저장될 모습이 바뀜, git이 만든 표시 줄(`<<<<<<<`·`=======`·`>>>>>>>`)이 조회 결과에 없으므로 DOM에도 없음(파일 내용에 원래 그 글자가 든 줄은 그대로 보임), 모두 고르면 주 버튼, 그만두기가 `conflict-abort`를 부름, 이진 파일 선택, 한글 조합 중 단축키 무시, 대비 테스트에 새 토큰, 저장소 글자는 `<pre>`에 React 글자로만.
- 완료 기준: `pnpm test`, `pnpm lint`(`innerHTML` 없음, 색 값 직접 쓰기 없음). 두 OS 수동: 실제 충돌 만들어 끝까지, 800px, 라이트·다크, 앱을 중간에 껐다 켜기.
- Windows/Ubuntu: `<details>`·글상자 IME 후보 창 위치를 WebKitGTK·WebView2에서 확인. 색만으로 알리지 않고 ✓·●와 "이걸로" 글자를 함께 쓴다.
- 보안: 화면: `innerHTML` 없음, 저장소 글자는 그대로 그림, 파일 경로를 입력으로 보낼 때 목록에서 받은 것만.

### D. 임시 보관, 맡긴 파일 해결, 변경 취소, 작업 내역

R15. git·core: 임시 보관 (`core: add stash and take-out actions`)
- 목표: 고친 파일을 보관하고 꺼내는 `stash`·`take-out`(C8). 맡긴 파일(`park`)과 목록이 섞이지 않는다.
- 바꿀 파일: `crates/git/src/backend.rs`·`cli/mod.rs`(`stash_list(repo) -> Vec<StashEntry { oid, message, at }>` = `reflog show --format=…%x00… -z refs/stash`), `types.rs`(`GitOp::StashDropOid { oid }` = 목록에서 그 항목의 위치를 찾아 `stash drop -q`), `cli/park.rs`(`park_push` 재사용), `crates/core/src/actions/stash.rs`, `view/input.rs`(`Stash`, `TakeOut(TakeOutInput { source })`, `TakeOutSource::Stash { oid }`), `undo/run.rs`(`Inverse` 중 `stash` 처리기), `query.rs`(`stashes`: 목록 + 항목별 파일 수 `parked_paths`, "higit parked …"로 시작하는 남은 항목은 뺌), `view/stash.rs`, `src-tauri`(`query_stashes` 네 곳), `src/lib/{api,queries}.ts`, `action.rs`(`Outcome::Stashed`, `TookOut`), 생성물.
- 내용:
  - `stash`(`stash`/Safe/`Inverse`, `tracked_refs = [refs/stash]`): 합치기 중이면 `OperationInProgress`. 고친 파일(새 파일 포함, 무시된 파일 제외)을 `Park`(`stash push --include-untracked -m "higit stash <브랜치>"`)으로 만들고 목록에 남긴다. 만들 것이 없으면 `InvalidInput`. 되돌리기(Inverse 처리기): `refs/stash`가 `after` 값 그대로일 때만(`UndoStale`) 작업 폴더가 깨끗하면 `Unpark` + `StashDropOid`.
  - `take-out`(`stash`/Normal/`Barrier`): `Stash { oid }`는 목록에 그 항목이 있고 작업 폴더가 깨끗하며 합치는 중이 아닐 때만(아니면 `WouldOverwrite`·`OperationInProgress`) 기존 `Unpark`(겹치면 깨끗이 되돌리고 `UnparkConflict`)로 꺼내고 `StashDropOid`. 꺼낸 파일은 고친 파일로 나온다.
- 테스트: 수정·새 파일·지운 파일이 섞인 상태 → stash → 작업 폴더가 깨끗하고 목록에 한 줄, 사용자 stash가 이미 있어도 그 항목이 그대로, 한글·공백 파일명, 꺼내기 → 원래와 같고 목록에서 빠짐, 고친 파일이 있으면 거절, 겹치면 `UnparkConflict`이고 작업 폴더·목록이 그대로, Ctrl+Z로 stash 되돌림, "higit parked" 항목은 목록에서 빠짐, 되돌리기 전에 터미널로 `stash pop` 하면 `UndoStale`. CRLF 파일(`.gitattributes`)이 보관·꺼낸 뒤 그대로.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `registry-glossary` 테스트.
- Windows/Ubuntu: Windows 편집기가 파일을 잡으면 `stash push` 정리가 실패할 수 있다(오류로 알림, 파일은 잃지 않음). `reflog`의 `-z` 구분자 파싱을 두 OS에서.
- 보안: stash 경로는 `safe_relative`(기존), 꺼낼 oid는 목록에 있는 것만(화면이 임의 oid를 못 줌).

R16. core·화면: 겹친 맡긴 파일을 충돌 화면으로 해결 (`core: resolve parked files through the conflict screen`) — D-054의 해결 부분
- 목표: "맡겨 둔 파일 N개 · 겹침"을 누르면 같은 충돌 화면에서 풀고 고친 파일로 꺼낸다(C15).
- 바꿀 파일: `crates/git/src/types.rs`·`cli/park.rs`(`GitOp::UnparkKeep { oid }` = `stash apply <oid>`(`--index` 없음)로 충돌을 남긴 채 끝내기, `GitOp::UnparkAbort { oid }` = 지금 바뀐 파일이 모두 그 stash의 경로 안일 때만 `reset --hard -q` + 그 stash가 만든 새 파일 삭제), `core/src/view/input.rs`(`TakeOutSource::Parked { branch }`), `actions/stash.rs`, `conflict.rs`·`query.rs`·`actions/conflict.rs`(context `Unpark`), `view/conflict.rs`(`context`), 화면: `BranchList.tsx`("지금" 줄의 "겹침 해결" 단추), `Conflict.tsx`(윗줄 문구가 context에 따라), `src/messages`.
- 내용:
  - `take-out`의 `Parked { branch }`: 지금 브랜치가 그 브랜치이고 작업 폴더가 깨끗하며 합치는 중이 아닐 때만. 맡긴 ref가 가리키는 stash를 `UnparkKeep`으로 적용한다 — 깨끗이 적용되면 `Unstage`하지 않고 ref 삭제, 겹치면 `MergeConflicts`(충돌 상태는 `MERGE_HEAD` 없이 충돌 항목만 있고, 지금 브랜치의 맡긴 ref가 있으며 충돌 경로가 그 stash의 경로 안이면 context `Unpark`로 읽는다).
  - 충돌 화면의 `Unpark` 맥락: 내 쪽 = 지금 브랜치 버전, 합칠 쪽 = "맡겨 둔 파일"(stash 버전의 사람·날짜). `conflict-finish`는 병합 커밋 대신 해결한 파일을 `Unstage`(고친 파일로)하고 맡긴 ref를 `Is(oid)`로 지운 뒤 `journal_mode = Barrier`. `conflict-abort`는 `UnparkAbort`(ref는 그대로).
  - `unpark_arrived`는 겹치면 지금처럼 꺼내지 않고 알리기만 한다.
- 테스트: A에서 고침 → B로(맡김) → B에 버전 저장(같은 줄) → A로 돌아옴 → 겹침 → "겹침 해결" → 화면에서 고름 → 마치면 고친 파일 내용이 기대와 같고 ref 없음. 그만두면 작업 폴더가 깨끗하고 ref 그대로. 새 파일끼리 겹침(add/add), 앱을 껐다 켜도(상태 재계산) 같은 화면. 다른 고친 파일이 있으면 `UnparkAbort`는 거절. 이상한 경로의 stash는 거절(기존 `safe_relative`).
- 완료 기준: `cargo test --workspace`, `pnpm test`, 바인딩 최신.
- Windows/Ubuntu: `stash apply`의 충돌 표시·줄바꿈은 git이 사용자 설정으로 만든다. 맡긴 ref 이름은 D-064 규칙.
- 보안: 맡긴 ref와 그 안의 경로는 믿지 않는 입력(검사 유지). `refs/higit/parked`를 지우는 코드는 `parked` 모듈 함수만 부른다.

R17. core: 고친 것 버리기 (`core: add discard action with backup`)
- 목표: `Irreversible` 확인 흐름 안에서 고른 파일의 고친 내용을 버리되, 가능하면 되살릴 백업을 남긴다(C6).
- 바꿀 파일: `crates/git/src/types.rs`·`cli/park.rs`(`GitOp::DiscardPaths { paths, message }` = `--literal-pathspecs stash push --include-untracked -m … --pathspec-from-file=- --pathspec-file-nul`, 만든 해시 반환), `crates/core/src/actions/discard.rs`, `view/input.rs`(`Discard(PathsInput)`), `action.rs`(`Outcome::Discarded { count, backed_up }`), `view/mod.rs`(`ErrorCode::BackupFailed`), 생성물, 용어는 R1.
- 내용: 메타 `discard`/`discard`/`Caution`/`Irreversible`(미리보기 토큰 필요, 실행기가 HEAD 백업과 Barrier를 처리). 실행: 경로 검증 → 충돌 중인 경로·합치는 중이면 거절 → 버전이 하나도 없으면 `InvalidInput`(버릴 기준이 없음) → `DiscardPaths`가 stash로 만들며 작업 폴더를 마지막 버전으로 되돌린다(새 파일은 지워지고 이름 바꿈은 두 경로) → `undo::backup::backup_oid(oid, "discard-<브랜치>")`로 백업 ref → `StashDropOid`. 백업 ref 쓰기가 실패하면 이미 버려졌으므로 stash를 목록에 남기고 `BackupFailed`(파일은 임시 보관 목록에 있음). 바뀐 것이 없으면 `count: 0`.
- 테스트: 수정·새 파일·넣은 새 파일·지운 파일을 골라 버림 → 작업 폴더와 index가 마지막 버전, 안 고른 파일은 그대로, 사용자 stash 목록 그대로, 백업 ref가 있고 `take-out`으로 되살아남, 토큰 없이는 `PreviewRequired`(파일 변화 없음), 토큰은 한 번만, 다른 입력으로는 `PreviewMismatch`, 한글·공백 이름, CRLF 파일, 버전 없는 저장소, 충돌 파일 거절, `BackupFailed`(가짜가 ref 쓰기 거부).
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: `stash push --pathspec-from-file`은 git 2.26부터(`git-min`에서 확인). Windows는 파일을 다른 프로그램이 잡고 있으면 지우기가 실패할 수 있다(오류, 아무것도 안 버려짐 — stash가 실패하면 파일도 그대로).
- 보안: 되돌릴 수 없는 일은 토큰 확인 후 실행(D-043), 지우기 전에 백업 ref가 먼저(`DiscardPaths`가 만든 뒤 지움). 경로는 stdin과 `--literal-pathspecs`.

R18. 화면: 임시 보관과 고친 것 버리기 (`features: stash and discard in compare pane footer`)
- 목표: r-mergehint 오른쪽 아래 "고친 것 버리기"와 임시 보관 단추. 확인 대화상자.
- 바꿀 파일: `src/features/discard/{DiscardButton.tsx, ConfirmDialog.tsx, ConfirmDialog.test.tsx}`, `src/features/stash/{StashFooter.tsx, StashList.tsx, StashFooter.test.tsx}`(신규), `src/screens/Workspace/Workspace.tsx`(비교 칸 아래 줄), `src/ui/Button.tsx`·`Button.module.css`(`danger` 변형: `--deleted` 채움, 확인 대화상자에서만), `src/styles/tokens.css`(`--on-deleted`)·`contrast.test.ts`, `docs/design.md`, `src/messages/{ko,en}.ts`.
- 내용: 비교 칸 맨 아래 오른쪽에 글자 버튼 `<Term id="stash" />`, "임시 보관 N개"(있을 때, 누르면 목록 팝오버: 줄마다 브랜치·시각·파일 수와 "꺼내기"), 그리고 `<Term id="discard" />`("고친 것 버리기", `--text-muted`, 호버 `--deleted`). 버리기는 고른 파일(이름 바꿈이면 두 경로)에 적용: 누르면 `previewAction` → Radix Dialog "화면.tsx의 고친 내용을 버려요."와 "버리기"(danger 채움)·"닫기"(주 버튼은 안전한 쪽, TrustDialog 방식) → `executeAction(input, token)` → 토스트 "버렸어요 · 작업 내역"(백업이 있으면 그 버튼이 작업 내역 화면을 연다. Ctrl+Z가 아님). 임시 보관: 누르면 바로 보관하고 토스트 "보관했어요 · 되돌리기". 꺼내기는 고친 파일이 있으면 단추를 끄고 이유를 툴팁으로.
- 테스트: 확인 없이는 `executeAction`을 부르지 않음, 파일 이름이 대화상자에 보임, 실패 코드별 문구, 임시 보관 목록과 꺼내기, 버튼이 파일이 없으면 꺼짐, 한글 파일명, 대비 테스트.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 버리고 작업 내역(R20 뒤)에서 꺼내기.
- Windows/Ubuntu: 대화상자 포커스 이동·Esc를 두 웹뷰에서.
- 보안: 화면이 되돌릴 수 없는 일을 확인 없이 부르지 않는다(실행기가 한 번 더 강제).

R19. core: 백업 정리와 작업 내역 조회, 되살리기 (`core: prune backups, recovery timeline and restore`)
- 목표: D-043의 30일 정리, D-045·spec의 "작업 내역" 데이터, 되살리기 액션(C11, C17).
- 바꿀 파일: `crates/git/src/backend.rs`·`cli/mod.rs`·`cli/branches.rs`(`reflog(repo, limit) -> Vec<ReflogEntry { oid, at, subject, summary }>` = `reflog show --format=%H%x00%ct%x00%gs%x00%s -z HEAD`; `RefEntry`에 `summary`·`time` 추가 = `for-each-ref`의 `%(contents:subject)`·`%(creatordate:unix)`, 가짜 구현과 `parse_refs` 테스트 갱신), `crates/core/src/undo/backup.rs`(`prune(git, repo, now, keep_days)`, 이름의 시각을 읽는 `parse_stamp`), `bootstrap.rs`(`open_project`가 열린 뒤 한 번 호출, 최대 200개, 실패해도 열기는 성공), 신규 `crates/core/src/recovery.rs`·`view/recovery.rs`(`RecoveryView { items }`, `RecoveryItem { id, at, source: Journal|Reflog|Backup|Parked|Stash, action: Option<String>, label, summary, oid, restore: None|Version|Changes, undone }`), `query.rs`(`recovery`), `actions/restore.rs`, `view/input.rs`(`Restore(RestoreInput { oid })`, `TakeOutSource::Backup { refname }`), `src-tauri`(`query_recovery` 네 곳), 생성물.
- 내용:
  - 정리: `refs/higit/backup/` 아래 이름의 시각(`<UTC>-<순번>`)이 30일(`BACKUP_KEEP_DAYS`, 설정 화면이 생기면 설정으로)보다 오래된 ref만 `delete_ref(…, Is(oid))`. 시각을 못 읽는 ref와 `refs/higit/parked/*`는 지우지 않는다.
  - `recovery`: journal의 Action 줄(되돌린 줄은 `undone`), HEAD reflog(같은 해시·2초 안의 journal 줄과 겹치면 reflog 쪽을 뺌), 백업 ref(이름의 label `undo-…`·`sync`·`merge`·`discard-…`로 종류, `discard-`는 `Changes`, 나머지는 `Version`), 맡긴 ref, 임시 보관 항목을 시간순(최신 먼저, 200개)으로 합친다. journal은 서명이 맞는 줄만(`read()`), 다른 컴퓨터의 journal은 보이되 `restore`는 `None`(D-063).
  - `restore`(`reflog`/Caution/`RestoreTree`): 지금 브랜치(특정 버전을 보는 중이면 `InvalidInput`, 합치는 중이면 `OperationInProgress`)를 그 버전으로 `ResetKeep`한다. 실행기가 지금 HEAD를 백업 ref로 먼저 남기므로 "지금 버전은 남는다"(작업 내역에 새 행으로 보임). 지운 브랜치 되살리기(새 브랜치로)는 반복 3.
  - `take-out`의 `Backup { refname }`: `refs/higit/backup/` 아래이고 stash 모양(부모 둘 이상)일 때만 적용(ref는 남김). 맡긴 ref(`Parked`)와 stash는 R15·R16.
- 테스트: 정리(31일 전 이름은 삭제, 29일은 유지, 이름 형식이 이상한 것·parked는 유지, 200개 한도). 조회: 앱 작업·reflog·백업·맡긴·임시 보관이 시간순으로 섞이고 중복이 빠짐, `undone` 표시, 이름이 `-x`인 journal 줄 무시, 다른 키로 서명된 줄은 `restore: None`. `restore`: 버전 저장 두 번 뒤 첫 버전으로 → 파일이 그 버전이고 고친 파일 그대로, 그 직전 HEAD의 백업 ref가 생겨 다시 되살릴 수 있음, 겹치면 `WouldOverwrite`, 버전 보는 중 거절, 올린 뒤에도 가능(다음 동기화가 다시 가져옴, 위험에 적음).
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `registry-glossary`(reflog 용어).
- Windows/Ubuntu: reflog `-z`와 `%x00` 파싱, 한글 요약. 시각은 UTC로 읽고 화면이 지역 시각으로 바꾼다.
- 보안: journal·reflog·ref 이름·메시지는 모두 믿지 않는 입력(React 글자로만, 이름 검사). 새 IPC 명령 네 곳. 정리는 `refs/higit/backup/` 접두사만 지운다.

R20. 화면: 작업 내역 (`screens: activity screen`)
- 목표: D-045 상단 바 "작업 내역"과 세이브 카드 화면(D-017 제안의 복구 화면 부분, 화면 전환은 D-058과 같은 방식).
- 바꿀 파일: `src/screens/Activity/{Activity.tsx, Activity.module.css, Activity.test.tsx}`, `src/features/activity/{model.ts, model.test.ts, Card.tsx, useRestore.ts}`, `src/state/repoStore.ts`(`WorkspaceView`에 `"activity"`), `src/screens/Workspace/{TopBar.tsx, Workspace.tsx}`(오른쪽: 동기화 · 버전 기록 · 작업 내역 — 컨트롤 3개), `src/features/undo/useUndo.ts`(`undoStale`·`undoPushed`이면 토스트에 "작업 내역" 버튼), `src/lib/{api,queries,events}.ts`, `src/messages/{ko,en}.ts`, `docs/design.md`(작업 내역 카드).
- 내용: 창 전체 폭. 맨 위 한 줄 "되살려도 지금 버전은 남아요", 아래 날짜별 묶음의 카드(한 줄 높이 56px, 왼쪽 띠는 브랜치 색 `branchColor`, 모르면 `--border-strong`): 시각, 제목(앱 작업은 `<Term>`(액션의 glossary id), 백업·reflog·맡긴·보관은 종류 글자), 버전 이름, 오른쪽 단추. `Version`은 "이 시점으로 되돌리기"(`restore`), `Changes`는 "꺼내기"(`take-out`의 출처별), 되돌린 줄은 글자색을 `--text-muted`로. reflog의 git 원문(`gs`)은 "자세히 보기"로 접는다. 하단에 "로그 폴더 열기"(R5). Ctrl+Shift+H·상단 바 단추로 열고 닫고 Esc는 홈. 되살리면 토스트 "되돌렸어요 · 되돌리기"(`restore`는 `RestoreTree`라 Ctrl+Z로도). 합치는 중에는 단추를 끈다.
- 테스트: 카드 모델(시간순·날짜 묶음·종류별 단추), 단추가 올바른 액션·입력을 부름, 원문 접기, `undoStale` 토스트 버튼이 화면을 엶, Ctrl+Shift+H 전환과 Esc, 800px, 빈 상태, 저장소 글자는 React 글자로만.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 앱 밖에서 `git commit` 한 뒤 Ctrl+Z → "작업 내역" 안내 → 되살리기.
- Windows/Ubuntu: 날짜 형식은 기존 `lib/time.ts`. 차이 없음.
- 보안: 행의 oid·ref 이름은 `recovery` 조회가 준 것만 입력으로 되돌려 보낸다.

### E. 신뢰성 (D-041의 나머지)

R21. git·platform: 진행 감시와 취소 (`git: add stall detection and cancel for network commands`)
- 목표: 원격 작업이 멈추면 끝내고 사용자가 취소할 수 있게 한다(C12).
- 바꿀 파일: `crates/git/src/cli/runner.rs`(`run_limited`가 stderr를 읽는 스레드에서 "마지막 출력 시각"과 "첫 출력을 봤는지"를 갱신, 반복 중 취소 깃발·멈춤 판정, 1초마다 관찰자 호출), `cli/network.rs`(`fetch`·`push`에 `--progress`, `NetControl`), `cli/mod.rs`(`running` 표, `GitBackend::{request_cancel(repo), clear_cancel(repo), set_observer(…)}`), `error.rs`·`failure.rs`(`GitError::{Cancelled, Stalled}`, `FailureKind::{Cancelled, Stalled}`), `crates/platform/src/process.rs`(`terminate_tree`: Unix 프로세스 그룹에 SIGTERM, Windows `taskkill /T`; `ProcessControl.terminate`), 가짜 구현 둘.
- 내용: 첫 stderr 출력(연결·진행 줄, 내용은 해석하지 않음) 뒤 90초 동안 출력이 없으면 `Stalled`, 처음부터 10분(백그라운드 60초) 제한은 그대로. 연결 전(인증 창 대기)에는 멈춤 판정을 하지 않는다. 끝낼 때는 먼저 `terminate_tree`로 git이 잠금 파일을 지우게 하고 2초 안에 안 끝나면 `kill_tree`. 취소 요청은 실행 중이면 바로, 아니면 다음 `run_network` 시작 때(`clear_cancel`로 액션 시작 때 지움) `Cancelled`. 로컬 작업에는 걸지 않는다(D-066).
- 테스트: git 별칭으로 "한 번 출력하고 잠자는" 가짜(`!printf x >&2; sleep 30`)가 짧게 줄인 멈춤 제한에 `Stalled`이고 자식·손자가 남지 않음, 출력 없이 잠자는 것은 10분 제한 전까지 멈춤으로 보지 않음(제한을 줄인 테스트로), 취소 깃발이 `Cancelled`, 관찰자가 1초마다 불림, `--progress`가 인자에 있음, 백그라운드 모드 환경은 기존대로.
- 완료 기준: `cargo test -p git -p platform` 두 OS, 수동: 응답 없는 원격에서 취소.
- Windows/Ubuntu: 종료가 OS마다 다르다(코드는 platform에만). Windows `taskkill /T`의 정상 종료는 콘솔 프로세스에 잘 안 먹으므로 2초 뒤 `/F`가 사실상 기본이다(잠금 파일이 남을 수 있음 → R24).
- 보안: 프로세스는 platform 함수로만 끝낸다. `Command::new`는 runner·platform에만.

R22. core·app: 진행 이벤트와 취소 명령 (`core: op-progress event and cancel command`)
- 목표: 화면이 진행과 취소를 쓸 수 있게 한다.
- 바꿀 파일: `crates/core/src/bootstrap.rs`(`running` 표, `set_progress_sink`, 관찰자를 `view::OpProgress`로 바꿔 보냄, `execute`가 원격 잠금 작업 시작·끝에 `clear_cancel`, `Core::cancel(folder)`), `view/mod.rs`(`OpProgress { repo, action, elapsed_secs, idle_secs }`, `ErrorCode::{Cancelled, Stalled}`), `src-tauri/src/{commands.rs, bindings.rs, lib.rs}`(`action_cancel` 네 곳, 이벤트 `op-progress` 상수와 `.typ::<OpProgress>()`, 시작 때 sink 등록), `src/lib/{api,events}.ts`(`cancelAction`, `onOpProgress`), `src/messages/{ko,en}.ts`(`cancelled`, `stalled` 문구), `docs/architecture.md`는 R26.
- 내용: 원격 잠금을 잡은 작업(fetch·pull·push·sync)이 도는 동안 1초마다 `op-progress`를 보낸다(저장소 경로, 액션 id, 경과, 마지막 출력 뒤 시간). `action_cancel`은 `Core::cancel`을 부른다. `Cancelled`는 오류 배너가 아니라 토스트 "취소했어요", `Stalled`는 안내 줄 "진행이 멈춰 그만뒀어요" + [동기화].
- 테스트: 가짜 백엔드로 관찰자 → 이벤트 변환, 잠금이 없는 로컬 작업에는 이벤트 없음, `cancel`이 `request_cancel`을 부름, 취소된 sync 뒤 잠금이 풀림(다른 작업이 `Busy`가 아님), `security_config` 테스트 통과(네 곳).
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음.
- 보안: 새 명령 네 곳·열린 저장소 검사. 이벤트 페이로드에 파일 이름이나 원격 주소를 담지 않는다.

R23. 화면: 진행과 취소 (`features: progress and cancel in sync button`)
- 목표: 동기화·원격 확인이 오래 걸릴 때 보이고 멈출 수 있다.
- 바꿀 파일: `src/features/progress/{progressStore.ts, useProgress.ts}`, `src/features/sync/{SyncButton.tsx, SyncButton.module.css, SyncButton.test.tsx}`, `src/lib/events.ts`, `src/messages/{ko,en}.ts`.
- 내용: 동기화 중 버튼은 "동기화 중 · N초"(D-049, 이름과 숫자), 3초가 지나면 글자 버튼 "취소"가 옆에 나타난다. 취소는 `cancelAction`. 마지막 출력 뒤 시간이 30초를 넘으면 숫자 옆에 옅게 "응답 없음 · N초". 저절로 하는 원격 확인(`useAutoFetch`)은 표시하지 않는다.
- 테스트: 가짜 이벤트로 표시·취소 단추 시점·취소 호출, `cancelled` 오류는 토스트만, `stalled`는 안내 줄과 [동기화], 이벤트 구독 정리.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 비공개 저장소 동기화를 중간에 취소.
- Windows/Ubuntu: 차이 없음.

R24. git·platform·core: 멈춘 작업 알아내기와 정리 (`core: detect and clear stale git locks`)
- 목표: index.lock 정리와 "끝나지 않은 작업 알리기"의 데이터와 액션(D-041, C13, C14).
- 바꿀 파일: `crates/git/src/failure.rs`(`FailureKind::Locked`: `.lock': File exists`, 잠금 파일을 만들어 실제로 재현하는 테스트), `crates/platform/src/process.rs`(`running_git_pids()`: Linux는 `/proc/*/comm`, Windows는 `tasklist /FI "IMAGENAME eq git.exe" /FO CSV /NH`), 신규 `crates/core/src/locks.rs`, `query.rs`(`pending_work`), `view/pending.rs`(`PendingWorkView { stale_locks: Vec<String>, leftover_parked: u32, other_operation: Option<OtherOperation(Rebase|CherryPick|Revert)> }`), `actions/{clear_lock.rs, recover_parked.rs}`, `parked.rs`(맡긴 ref를 쓰는 함수), `view/input.rs`(`ClearLock`, `RecoverParked`), `view/mod.rs`(`ErrorCode::IndexLocked`), `src-tauri`(`query_pending_work` 네 곳), `src/lib/{api,queries}.ts`, 생성물, 용어는 R1(`lock`).
- 내용:
  - 잠금 조사: git 폴더 최상위의 `*.lock`(`index.lock`, `HEAD.lock`, `packed-refs.lock`, `shallow.lock`, `config.lock`)과 `refs/` 아래 `*.lock`(깊이 6, 500개까지, 링크는 건너뜀). 오래된 잠금 = 수정한 지 60초 이상 + 앱의 작업이 이 저장소에서 돌고 있지 않음(`busy`·`net_busy`) + 시스템에 `git` 프로세스가 없음. 하나라도 어긋나면 "오래된 잠금 없음"(`Locked` 오류는 "다른 Git 작업이 진행 중이에요"로).
  - `clear-lock`(`lock`/Caution/`Irreversible`): 다시 조사해 같은 조건이면 그 파일들(일반 파일, 이름이 `.lock`으로 끝남, git 폴더 안)만 지운다. 확인 토큰 필요.
  - 남은 맡긴 항목: stash 목록에서 메시지가 `higit parked <브랜치>`인 것. `recover-parked`(`park`/Normal/`Barrier`): 브랜치 이름 검사(`validate_ref_like`, 200바이트)를 거쳐 맡긴 ref가 없으면 쓰고, 있으면 그 ref를 백업 ref(`parked-leftover-<브랜치>`)로 옮겨 둔 뒤 항목을 목록에서 뺀다.
  - `other_operation`: `rebase-merge`·`rebase-apply`·`CHERRY_PICK_HEAD`·`REVERT_HEAD`(앱이 끝낼 수 없음, 안내만).
  - journal에 시작·끝 줄을 더하지 않고 저장소 흔적으로 알아낸다(C14).
- 테스트: 잠금 파일을 만들고(수정 시각을 과거로) → 조사가 오래된 것으로 보고 `clear-lock`이 지움, 60초 안 된 것·git이 도는 중(가짜 프로세스 목록)·링크 `.lock`·이름이 `.lock`이 아닌 파일은 안 지움, `Locked` 분류를 실제 `git add`로 재현, `recover-parked`(ref 없음/있음/이상한 이름 `-x`), `other_operation`, 열지 않은 저장소는 `NotOpened`, 토큰 없이 `clear-lock`은 `PreviewRequired`.
- 완료 기준: `cargo test --workspace`(platform은 `cargo test -p platform`), 바인딩 최신, `registry-glossary`.
- Windows/Ubuntu: 프로세스 조회가 OS마다 다르다(platform에만). Windows는 git이 파일을 잡고 있으면 지우기가 실패한다(오류). Linux도 같은 사용자의 IDE git을 `comm`으로 알아본다.
- 보안: 지우는 대상은 git 폴더 안 `*.lock`뿐(`canonicalize` 뒤 접두사 확인, 링크 안 따라감). `refs/higit/`를 쓰는 코드는 `parked` 모듈 함수로만.

R25. 화면: 끝나지 않은 작업 알림 (`features: pending work notice and parked recovery`)
- 목표: 프로젝트를 열면 위쪽에 알림 줄을 보여주고 확인 뒤 정리한다.
- 바꿀 파일: `src/features/pending/{PendingNotice.tsx, ClearLockDialog.tsx, PendingNotice.test.tsx}`, `src/screens/Workspace/Workspace.tsx`(오류 줄 위), `src/ui/ErrorNotice.tsx`(`indexLocked`이면 "정리하기" 단추), `src/lib/useRepoWatch.ts`, `src/messages/{ko,en}.ts`.
- 내용: 줄마다 짧은 문장과 단추 하나. "멈춘 작업 표시 파일 N개" [정리하기](확인 대화상자 → `previewAction` → `executeAction`, TrustDialog 방식), "맡겨 둔 파일 N개가 임시 보관 목록에 남았어요" [복구하기](`recover-parked`), "다른 도구에서 시작한 리베이스 중이에요"(단추 없음). 닫으면 이번 실행에서는 다시 안 보인다(저장하지 않음). 합치는 중이면 충돌 화면이 대신 알린다. 신뢰 확인 전(`NeedsTrust`)에는 조회하지 않는다.
- 테스트: 줄 종류별 표시, 정리하기가 확인 없이는 실행되지 않음, 복구하기, 닫기, `indexLocked` 오류의 단추, 알림 없을 때 아무것도 안 그림.
- 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 동기화를 취소한 뒤 잠금이 남았을 때 정리.
- Windows/Ubuntu: 차이 없음.

### F. 마무리

R26. 문서와 점검 (`docs: wrap up iteration 2`)
- 바꿀 파일: `docs/architecture.md`(GitBackend 목록에 `index_entries`·`read_blob`·`merge_segments`·`stash_list`·`reflog`·`request_cancel`·`clear_cancel`·`set_observer`, `GitOp` 목록, 등록 액션 목록, `UndoKind`(`RestoreTree`·`Barrier`, `RestoreIndex`는 일반 줄, `Inverse` 처리), `UndoState.refs`, journal 줄 형식 변화, 되돌리기 규칙(`reverted`, 합치는 중 거절), 잠금 둘 그대로, 시간 제한 표에 "멈춤 90초"·취소, 명령 목록과 이벤트 `op-progress`, 입력 상한(충돌 입력), 로그·잠금 정리·백업 정리, "복구 화면" 문단을 구현에 맞춤, 보안 표의 "비밀 파일"·"중간에 끊기는 작업" 줄), `docs/spec.md`(화면 구성의 합치기/충돌·작업 내역 행, 충돌 해결 화면 5번 위험도, 용어표는 R1에서 끝), `docs/design.md`(충돌 화면 색 토큰, 합치기 제안 블록, 작업 내역 카드, 비교 칸 아래 줄, 알림 줄, danger 버튼·`--on-deleted`), `docs/roadmap.md`(반복 2 완료 표시, 반복 3에 이월 항목), `docs/progress.md`, `.claude/agents/code-analyst.md`(새 점검 줄: `refs/higit/` 위치에 `parked` 쓰기 함수 포함, 새 명령 네 곳은 이미 있음, 비밀 원문 금지), `CLAUDE.md`는 규칙이 이미 맞으므로 바꾸지 않는다.
- code-analyst 점검: 3절 보안 점검 전부와 2절 전부. 더해서 `rg "undo_last|cancel" src --glob '!src/lib/**'`로 화면이 `api.ts`만 쓰는지, `rg '"--end-of-options"' src-tauri/crates/git/src`가 `args.rs`에만, `rg "Command::new" src-tauri --glob '!**/tests/**'`가 runner·platform에만, `rg "refs/higit" src-tauri/crates/core/src`가 `undo`·`parked`·`actions/switch`(+`recover_parked`가 `parked` 함수 호출)에만, 새 명령 9개(`app_open_log_folder`, `query_secret_check`, `query_merge_hint`, `query_conflict_state`, `query_conflict_file`, `query_stashes`, `query_recovery`, `query_pending_work`, `action_cancel`)가 네 곳에 모두, 충돌·결과 화면에 `<<<<<<<`가 렌더되지 않음, 로그에 비밀이 남지 않음, 색 값을 직접 쓴 곳 없음(`tokens.css` 밖), 단축키가 `shortcuts.ts` 밖에 없음, `unsafe`·`unwrap` 없음, `cargo deny check` 통과(새 의존성 없음).
- 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full` 통과, "완료 기준(반복 2 전체)"을 두 OS에서 손으로 확인(확인 못 하는 것은 `docs/progress.md` "사람이 할 일"에 적는다).

## 용어 변경
- spec.md 용어표와 `src/glossary`(`ko.ts`·`en.ts`, `TERM_IDS` 순서는 표 순서)에 R1에서 넣는다.
  - 추가(`conflict` 다음): `| ours | 내 브랜치 | ours | 합치는 중 지금 브랜치 쪽 내용. 리베이스 중에도 같은 뜻으로 보여줘요. | safe | 가끔 |`, `| theirs | 합칠 브랜치 | theirs | 합치는 중 가져오는 쪽 내용 | safe | 가끔 |`, `| base | 고치기 전 원래 코드 | base · merge base | 두 브랜치가 갈라지기 전의 코드 | safe | 가끔 |` (명령 열은 `conflict` 행처럼 이름만). en: "Your branch", "Incoming branch", "Original code".
  - 추가(표 맨 끝): `| lock | 멈춘 작업 정리 | index.lock | 중간에 끊긴 Git 작업이 남긴 잠금 파일 지우기. 실행 중인 Git이 없을 때만 해요. | caution | 드묾 |`. en: "Clear stuck work".
  - 이름 변경(C7): `discard` "변경 취소" → "고친 것 버리기"(en "Discard changes" 유지), `reflog` "복구" → "작업 내역"(en "Recover" → "Activity"). 설명·명령은 그대로. spec 화면 구성의 "복구(작업 내역)" 행도 "작업 내역"으로.
  - 그대로: `merge`, `conflict`, `stash`, `park`, `ignore`는 이름을 바꾸지 않고 화면 문구 "main 가져와 합치기"·"겹침 해결"처럼 용어 + 조사 조합은 `src/messages`가 용어표에서 꺼내 쓴다(D-029).
- "되돌리기"·"다시 하기"·"합치기 마치기"·"합치기 그만두기"·"임시 보관"(보관 단추)은 앱 동작 문구라 `src/messages`에 둔다. 용어표 `revert`("되돌리기")와 같은 말이 두 뜻으로 쓰이는 위험은 005에서 적은 그대로이고, 올린 뒤 되돌리기가 실제로 되돌리기(revert) 새 버전이 되면서 두 뜻이 이어진다.

## 자동 결정 후보
구현 전(R1)에 Claude가 `docs/decision.md`에 D-069부터 "(자동 결정)"으로 올린다. 기존 확정 결정을 바꾸지 않는다(근거 열).

| 번호 | 결정(추천안) | 까닭 | 근거·부딪힘 없음 확인 |
| --- | --- | --- | --- |
| C1 | 겹치는 합치기는 실제 `git merge`를 시작해 `MERGE_HEAD` 상태로 두고, 충돌 파일의 표시 문자는 화면에 보이지 않는다. 고른 내용은 "합치기 마치기" 때 core가 블롭으로 만들어 index에 넣고 작업 폴더에 쓴다. 시작·마치기 전에 이번 버전에 넣은 파일이 있으면 시작하지 않는다(`StagedChanges`). 충돌이 없는 합치기는 `merge_check`로 미리 보고 바로 합친다. | `--abort`와 마치기를 git이 이미 갖고 있고 앱을 껐다 켜도 상태가 git에 있다. 넣은 파일이 합치기 버전에 섞이는 것을 막는다. 작업 폴더의 표시 문자는 사용자가 외부 편집기로 열 때만 보인다. | D-048(겹칠 때만 충돌 해결로), D-028(화면에 표시 문자 없음), D-050 |
| C2 | `conflict-abort`(합치기 그만두기)는 위험도 `normal`, 되돌리기 `Nothing`(확인 없음). `conflict-finish`는 `normal`/`RestoreTree`(Ctrl+Z로 합치기 전, 올린 뒤에는 되돌리기 새 버전). | 그만두기는 시작 전 상태로 돌아가는 것이 곧 되돌리기이고 화면의 선택은 git에 쓰지 않아 잃을 것이 없다. spec이 "반복 2에서 정한다"고 한 항목. | D-043(확인은 되돌릴 수 없는 일만) |
| C3 | "main에 합치기"는 main으로 갈아타며(고친 파일은 맡김) 앞으로 감기만 한다. 뒤처지지 않을 때만 제안하므로 늘 앞으로 감기다. 기준은 로컬 기본 브랜치(홈 노선도와 같음). GitHub 프로젝트에서 "검토 요청"을 우선하는 것은 반복 4. | 방향은 앱이 고른다(D-048). 이후 흐름은 `switch`·ref 되돌리기를 재사용한다. | D-048 |
| C4 | `UndoKind`에 `RestoreTree`(합치기·되살리기, `reset --keep`)와 `Barrier`(확인 없이 경계만)를 더하고, `RestoreIndex`는 경계 줄 대신 일반 줄로 기록한다. `UndoState.refs`로 다른 ref의 이전 값을 되돌린다. 넣기·빼기 되돌리기는 토스트 없이 Ctrl+Z만. | D-065가 "넣기·빼기 되돌리기가 생기면 경계 대신 일반 줄"이라고 예약한 그대로. 합치기는 파일도 되돌려야 해 `reset --soft`로는 안 된다. | D-042, D-065 영향 |
| C5 | 올린 뒤 Ctrl+Z: `commit`·합치기 버전은 되돌리기(revert) 새 버전으로(합치기는 `-m 1`), `push`·`pull`·`sync`(`Inverse`)는 되돌리지 않고 "올린 것은 되돌릴 수 없어요 · 작업 내역"으로 안내. revert 액션과 버전 기록 메뉴의 "이때로 되돌리기"는 반복 3(이 단계의 `GitOp::Revert`를 재사용). | D-042 "올린 뒤에는 기록을 바꾸지 않는 방법만". 올리기 자체는 덮어쓰며 올리기 없이 되돌릴 수 없다. R4 한 단계만 빼면 이 후보를 반복 3으로 미룰 수 있다. | D-042, D-043 |
| C6 | `discard`는 `Caution`/`Irreversible` 확인을 유지하되 고른 파일 단위로, 실행 전에 `stash push`로 스냅샷 stash를 만들어 `refs/higit/backup/<시각>/discard-<브랜치>`에 남기고(실패하면 stash를 목록에 남기고 `BackupFailed`) 작업 내역에서 꺼낼 수 있다. 전체 버리기 버튼은 없다. 토스트 "버렸어요 · 작업 내역". | 사용자가 "실행 전 백업 가능하면 남기기"를 요구했다. 스냅샷은 best-effort라 D-043의 확인은 유지한다. stash 한 번으로 버리기와 백업이 동시에 되어 파일을 잃을 틈이 없다. | D-043 |
| C7 | 용어 이름: `discard` "고친 것 버리기", `reflog` "작업 내역"(spec·ko·en 함께), 새 행 `ours`·`theirs`·`base`·`lock`. | 홈이 "고친 파일"(D-060)을 쓰고 상단 바가 "작업 내역"(D-045)을 쓴다. D-059와 같은 방식. | D-024, D-045, D-059 |
| C8 | "임시 보관"은 일반 `refs/stash` 목록(터미널과 호환)을 쓰고, 단추 한 번에 고친 파일 전부를 보관, "꺼내기"는 작업 폴더가 깨끗할 때만. 이름 붙이기·지우기는 반복 3. 맡긴 파일(`park`)은 계속 `refs/higit/parked`. | D-011이 반복 2로 둔 stash의 최소 판. | D-011, D-047 |
| C9 | 비밀 파일 경고는 저장을 누른 순간 한 번 알림 줄(모달 아님)로 알리고 저장은 멈춘다. 줄의 "그래도 저장"이나 같은 묶음의 두 번째 저장 누름이면 저장한다. 결과에는 경로·규칙·줄 번호만(일치 글자 없음). 크기 경고 50MB. D-040 영향의 "저장을 멈추지 않는"은 "확인 창을 띄우지 않는"으로 읽는다. | D-040 본문이 세 버튼(빼기·제외·그래도 저장)을 정했다. 줄이라 D-043의 확인 피로와 겹치지 않는다. | D-040, D-043 |
| C10 | 로그: `<앱 데이터>/logs/higit.log` 1MB × 3개 순환, 14일 지난 파일 삭제, 모든 줄을 `redact`로 가린 뒤 쓴다. 남기는 것은 시각·종류·액션 id·오류 코드·가린 원문뿐(파일 이름·버전 메시지 없음). 로그 폴더 열기 명령을 둔다. | D-040 "로그에 쓰기 전 한 함수". 보고서에 경로·이름을 담지 않기 위해. | D-034, D-040 |
| C11 | 백업 ref 정리: 프로젝트를 열 때 한 번, `refs/higit/backup/` 아래 이름의 시각이 30일 지난 것만 최대 200개 삭제. 기간은 상수(설정 화면이 생기면 설정). parked ref는 지우지 않는다. | D-043 "30일 뒤 자동 정리(설정 가능)". 설정 화면이 아직 없다. | D-043, D-054 |
| C12 | 진행 감시: `--progress`를 주되 stderr 내용은 해석하지 않고 "출력이 왔는지"만 본다. 첫 출력 뒤 90초 무출력이면 `Stalled`, 연결 전에는 10분 제한만. 취소 버튼은 원격 작업(동기화·원격 확인)에만 있고 로컬 작업 취소는 두지 않는다. 끝낼 때 SIGTERM(Windows는 `taskkill /T`) → 2초 뒤 강제 종료. `op-progress`는 경과·무출력 시간만 싣는다. | 진행 줄을 해석하면 D-003(기계용 형식만)을 어긴다. 로컬 쓰기를 죽이면 반쯤 바뀐 상태가 남아 D-066이 금지한다(D-066 영향의 "로컬 작업 취소 버튼"은 이 결정으로 원격 작업으로 좁힌다). | D-003, D-041, D-066 |
| C13 | 멈춘 잠금 정리: 오래된 잠금 = 60초 이상 + 앱 작업이 이 저장소에서 안 돎 + 시스템에 `git` 프로세스 없음. 대상은 git 폴더 최상위와 `refs/` 아래 `*.lock`. `clear-lock`은 `Caution`/`Irreversible`로 확인 뒤 삭제. `Locked` 오류에는 "정리하기" 단추. | D-041 "살아 있는 git 프로세스가 없고 일정 시간이 지났을 때만 묻고 지운다". 취소로 SIGKILL이 되면 ref 잠금도 남는다. | D-041 |
| C14 | "끝나지 않은 작업 알리기"는 journal에 시작·끝 줄을 더하지 않고 저장소 흔적(합치는 중, 잠금 파일, 목록에 남은 "higit parked" 항목, 리베이스 등)으로 알아내 열 때 알림 줄로 보인다. `recover-parked`로 남은 맡긴 항목을 ref로 옮긴다. | 흔적이 실제 위험(합치기·잠금·맡긴 파일 유실)을 모두 덮고, 모든 액션에 fsync 쓰기를 더하지 않는다. | D-041("journal을 보고": 맡긴 항목은 journal이 쓴 stash 메시지 규칙으로 판별) |
| C15 | 맡긴 파일 겹침은 충돌 화면의 `Unpark` 맥락으로 푼다: `stash apply`로 충돌 상태를 만들고(깨끗한 작업 폴더에서만), 마치면 해결한 파일을 고친 파일로 두고 맡긴 ref를 지우며, 그만두면 꺼내기 전으로 되돌린다(ref 유지). 임시 보관·백업 꺼내기는 충돌 화면 없이 `UnparkConflict`로 거절한다. | D-054 "해결은 반복 2의 충돌 화면에서". 상태를 git에서 다시 계산하므로 앱 쪽 상태 파일이 없다. | D-054, D-028 |
| C16 | 브랜치 지우기·이름 바꾸기는 반복 2에 넣지 않고 반복 3으로 보낸다(D-054의 맡긴 ref 옮기기·백업 처리를 그때 함께). 앱 밖에서 지운 브랜치의 맡긴 파일은 작업 내역에서 "꺼내기"로 지금 브랜치에 꺼낼 수 있다. | 반복 2가 이미 크고, 원격 브랜치 삭제(`push --delete`는 `Irreversible`)·upstream 연결·이름 바꿈의 원격 처리가 맞물린다. | D-054(시기만 정함), 005 표 |
| C17 | 작업 내역은 journal·HEAD reflog·백업 ref·맡긴 ref·임시 보관 다섯 원천을 최신 먼저 200개로 합치고, 같은 해시·2초 안의 reflog는 journal 줄에 흡수한다. "이 시점으로 되돌리기"(`restore`, `Caution`/`RestoreTree`)는 지금 브랜치를 옮기고 지금 HEAD를 먼저 백업한다. 단축키 Ctrl+Shift+H. 지운 브랜치를 새 브랜치로 되살리기는 반복 3. | spec의 "journal + reflog + 백업 ref를 시간순으로", 세이브 카드 "지금 버전은 남아요". | D-017(제안), D-045, D-058 |
| C18 | 충돌 화면 세부: 이진·삭제·링크·큰 파일(1MB)·모드·서브모듈 충돌은 파일 단위 "이걸로" 선택만, 이름 바꿈 충돌도 경로별 선택, `Both`는 내 쪽 다음 합칠 쪽, 리베이스·cherry-pick·revert 충돌은 안내 줄만(반복 3). 단축키 1·2·3, Ctrl+↑↓. 색 토큰 `--conflict-ours-bg`·`--conflict-theirs-bg` 추가. | 1안의 질문이 "같은 줄을 다르게 고쳤어요"인 텍스트 충돌에 맞춘 범위. | D-028 |

## 반복 2에서 빠지는 것
| 항목 | 가는 곳 | 까닭 |
| --- | --- | --- |
| 브랜치 지우기·이름 바꾸기 | 반복 3 | C16 |
| 로컬 작업 취소 버튼 | 두지 않음 | C12, D-066 |
| revert 액션, 버전 기록 메뉴의 "이때로 되돌리기"·"여기서 새 브랜치", 지운 브랜치를 새 브랜치로 되살리기 | 반복 3 | `GitOp::Revert`는 R4에서 먼저 들어온다 |
| 리베이스·cherry-pick·revert 중 충돌 해결 | 반복 3 | 앱이 시작하지 않는 상태는 안내 줄만 |
| 충돌 화면 2안(둘 다 살린 제안), 문서처럼 보기, 단어 단위 비교 | 나중 후보 | spec "나중 후보" |
| 브랜치를 끌어다 놓아 합치기 | D-017 확정 뒤 | spec 진입 문구는 R1에서 맞춤 |
| 임시 보관 이름 붙이기·지우기 | 반복 3 | C8 |
| GitHub 프로젝트의 "검토 요청" 우선, GitHub 연결 | 반복 4 | D-048 |
| 설정 화면(백업 보존 기간, 로그 보관, 덮어쓰며 올리기, Git 원래 이름으로 보기) | 설정 화면이 생길 때 | D-043 |
| 오류 보고 전송 | 설정 화면과 함께 | D-034 5번 |
| 저장소 ID로 명령 시그니처 바꾸기 | 반복 5 전 | D-051 |
| push·pull·sync 되돌리기 | 하지 않음(안내만) | C5 |
| 서브모듈·이름 바꿈 충돌의 세부 해결 | 파일·경로 단위 선택만 | C18 |
| 로그 파일 전송·로그 화면 | 나중 | C10(폴더 열기만) |

## 위험과 열린 질문
- 충돌 중에는 작업 폴더 파일에 git의 표시 문자가 실제로 들어 있다(화면에는 안 보임). 사용자가 외부 편집기로 고친 뒤 "합치기 그만두기"를 누르면 그 고침이 사라진다. 화면을 쓰는 동안 같은 파일을 직접 고치지 않는 것이 전제다(문구는 넣지 않음, D-049).
- `reset --keep`(합치기·되살리기 되돌리기)은 고친 파일과 겹치면 실패한다. 오류 코드 `WouldOverwrite`로 알리고 아무것도 안 바뀐다. 편집기가 파일을 잡은 Windows에서도 실패할 수 있다.
- 합치기 힌트는 열 때마다 `merge_check`를 돌린다(2.34.1 경로는 `read-tree`+`merge-file`로 느리고 이름 바꿈을 못 알아 더 자주 겹침이라 한다). 갈라진 두 쪽이 모두 있을 때만 계산해 비용을 줄이고, 너무 느리면 힌트의 겹침 표시만 뺀다.
- 로컬 기본 브랜치가 `origin/main`보다 뒤처졌을 수 있다("main에 새 버전"이 로컬 기준). 동기화로 원격을 가져온 뒤 맞춰진다.
- 올린 뒤 "이 시점으로 되살리기"는 원격에 반영되지 않아 다음 동기화가 다시 가져올 수 있다. 되살리기 행에 따로 경고는 두지 않는다(원격 기록은 바뀌지 않음). 올린 뒤 되돌리기(revert) 버전의 메시지는 git 기본 형식(영어)이라 기록에 언어가 섞인다(동기화 합친 버전 메시지와 같은 문제).
- `stash push`·`DiscardPaths`는 사용자의 stash 목록을 잠깐 건드린다. 그 사이 앱이 죽으면 목록에 항목이 남는다(R24가 맡긴 항목은 알리고, 보관·버리기 항목은 사용자가 쓰는 목록에 있다).
- 90초 멈춤 판정은 서버 쪽 hook이 오래 걸리는 push를 잘못 끊을 수 있다. 취소와 같은 경로라 잠금 파일은 R24가 정리한다.
- journal에 넣기·빼기 줄이 많아져 4MB 상한에 닿으면 앞쪽이 잘린다(되돌릴 수 있는 범위는 최근 것만).
- 비밀 규칙은 오탐·미탐이 있다. 알림 줄이고 저장을 막지 않으며 "그래도 저장"이 있다.
- Q1. C5(올린 뒤 revert)를 반복 2에 넣을까? 추천: 넣는다(R4 한 단계, 안 하면 올린 뒤 Ctrl+Z가 막혀 있어 D-042가 비어 있음). 빼려면 R4만 삭제하고 `UndoPushed`를 유지한다.
- Q2. `restore`(작업 내역 되살리기)가 움직일 대상을 지금 브랜치로만 둘까? 추천: 그렇다. 새 브랜치로 되살리기는 반복 3의 "여기서 새 브랜치".
- Q3. Ctrl+Shift+H가 두 OS 웹뷰에서 앱에 오는가? 안 오면 키만 바꾸고 표만 고친다(005 H5와 같은 방식).
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md`에 "(자동 결정)"으로 올린 뒤 이어 간다.

## 검토 반영 (code-analyst, 2026-10-16)
구현은 아래를 위 단계 설명보다 우선한다. 결정 번호는 D-068이 이미 쓰였으므로(원격 확인 기다리기·올리기 잠금) C1~C18은 **D-069부터** 올린다. D-068의 "취소 버튼은 반복 2"는 R21~R23이 이행한다.

### 바뀐 순서
R1 → R2 → R3a → R3b → R4 → R5 → R6 → R7 → R8 → R9a → R9b → R10 → R11a → R11b → R11c → R13 → R14 → R12 → R9c → R15 → R16 → R17 → R19a → R19b → R19c → R20 → R18 → R21 → R22 → R23 → R24a → R24b → R25 → R26.
- R12(합치기 제안 화면)와 R9c(동기화가 겹치면 합치기를 시작)는 충돌 화면(R14) 뒤로 옮긴다. 그 전까지 `MERGE_HEAD`가 남는 길이 화면 없이 생기지 않게 한다.
- R18(버리기·임시 보관 화면)은 작업 내역 화면(R20) 뒤로 옮겨 토스트의 "작업 내역" 버튼이 처음부터 동작하게 한다.

### 나눈 단계
- R3a `RestoreIndex`(넣기·빼기 되돌리기)와 journal 호환. R3b `RestoreTree`·`UndoState.refs`·`Barrier`.
- R9a `merge_hint` 조회와 `merge` 액션. R9b `merge-into`. R9c 동기화 변경(겹치면 합치기 시작, `merge --abort` 안전망 삭제).
- R11a 조립 순수 함수(`conflict.rs`)와 테스트. R11b `conflict-finish`. R11c `conflict-abort`.
- R19a 백업 정리와 `RefEntry` 확장. R19b `recovery` 조회와 journal 읽기 API. R19c `restore`와 `take-out` 백업 꺼내기.
- R24a `pending_work` 조회와 `Locked` 분류. R24b `clear-lock`·`recover-parked`.
- R14가 너무 크면 틀·파일 목록 / 카드·저장될 모습·`Whole` / 홈 연결·토큰 셋으로 나눠 커밋해도 된다.

### 단계별 고침
- **journal 호환(R3a, R4)**: 서명 검증이 읽은 줄을 다시 직렬화하므로 새 필드(`UndoState.refs`, `Body::Undo.reverted` 등)는 모두 `#[serde(default, skip_serializing_if = …)]`로 넣어 옛 줄이 같은 글자로 직렬화되게 한다. `VERSION`은 1 그대로. 옛 구조로 서명한 고정 JSON 문자열을 읽는 테스트를 둔다. 새 `kind`·`type`을 모르는 옛 앱은 그 줄을 버린다(다운그레이드만 해당, 감수). 저장소 바인딩 서명·줄 순번은 이번에도 하지 않는다(progress의 이월 항목 유지).
- **journal 입력 줄이기(R3a부터)**: `Body::Action.input`에는 `ActionInput::journal_copy()`를 넣는다. 경로 목록은 200개까지 자르고 `ConflictFinish`의 `Custom` 글은 빼며 선택 종류만 남긴다. 되돌리기·다시 하기는 입력이 아니라 `before`/`after` 상태로 한다(D-065).
- **R2**: `core/tests/executor.rs`의 `NotYetSupported` 기대 테스트를 새 흐름으로 다시 쓴다. HEAD 백업은 `Action::moves_head()`(기본 참)가 거짓인 액션(`discard`, `clear-lock`, `take-out`, `recover-parked`)은 건너뛴다. 더해서 `commit`과 `stage`는 status에 충돌 항목(unmerged)이 있으면 `OperationInProgress`로 거절한다(리베이스·cherry-pick 중 홈에서 버전 저장을 누르면 충돌 표시가 든 파일이 저장되는 것을 막음). R1에서 D-011 상태를 "이행됨(반복 2)"으로 고친다.
- **R3b**: `RestoreTree` 되돌리기는 HEAD가 이미 올린 버전이면 R4 전까지 `UndoPushed`(D-042). `refs` 검사는 `refs/heads/`와 `refs/stash` 접두사만 허용(`is_valid`). 넣기·빼기 한 번마다 `capture`가 두 번 돈다(git 6회 안팎)는 점을 architecture에 적는다.
- **R4**: revert로 되돌린 뒤에는 경계 줄을 덧붙인다(HEAD가 새 버전이라 그 앞 줄은 어차피 `UndoStale`이 된다 → `NothingToUndo`로 깔끔하게). commit hook·gpg 서명 실패 뒤 `revert --abort`·`reset --merge` 정리 테스트를 더한다.
- **R5 로그(C10 고침)**: 로그 줄에는 시각·종류·액션 id·오류 코드·git 하위 명령 이름·종료 코드만 쓴다. git 원문(자세히 보기)은 쓰지 않는다(파일·브랜치 이름, hook 출력이 들어 있어 `redact`로 못 가림). 줄바꿈·제어 문자는 지운다(로그 위조 방지). 로거는 `blocking()`이 아니라 `Core::execute`·`undo`·`redo`에서 부른다.
- **R6(C9 고침)**: `read_blob`은 파일마다 프로세스를 띄우지 않고 `cat-file --batch` 한 프로세스로 읽는다. `ignore` 되돌리기는 `.gitignore`가 덧붙인 뒤 그대로이고 index 트리가 `after`와 같을 때만, `.gitignore`가 링크면 읽지도 쓰지도 않는다. D-040 영향 문장은 R1에서 "보완됨(D-0xx)"으로 표시한다.
- **R8**: `merge-file`의 `--diff3` 구분선 `=======`에는 이름표가 붙지 않는다. `--marker-size=40`을 쓰고, 세 입력 어디에든 길이 40 이상의 `<`·`|`·`=`·`>` 연속 줄이 있으면 오류(→ 파일 단위 선택)로 떨어뜨린다. 경로가 UTF-8이 아니거나 `U+FFFD`가 들어 있으면 그 파일은 `Whole`로 두고 `ResolveFile`을 받지 않는다. diff3의 "곳" 수가 터미널과 다를 수 있음을 위험에 적는다.
- **R9a(C1 고침)**: 합칠 대상은 `refs/heads/<branch>`로 한정해 넘긴다(같은 이름 태그가 먼저 풀리지 않게). 넣은 파일이 있으면 겹침 여부와 상관없이 `StagedChanges`. hook(`pre-merge-commit`·`commit-msg`)이 합치기 버전을 막으면 `MERGE_HEAD`와 충돌 0개 상태가 남는다 — 테스트하고, R14가 이 상태("충돌 0곳, 마치기만")를 연다. 태그·브랜치 이름이 같은 저장소 테스트. `merge_hint`는 `(HEAD 해시, 기본 브랜치 해시)`로 core 안에 결과를 기억하고, 겹침 검사가 1초를 넘으면 그 저장소에서는 겹침 표시를 뺀다.
- **R9b(C3 고침)**: "main에 합치기"는 갈아타지 않는다. 기본 브랜치가 지금 HEAD의 조상일 때만 `update_ref(refs/heads/<기본>, HEAD, Is(옛 값))`로 앞으로 감는다(지금 브랜치와 고친 파일은 그대로). 기본 브랜치가 다른 worktree에서 쓰이는 중이면 거절. 메타 `merge-into`/`merge`/Normal/`RestoreRef`, `tracked_refs = [refs/heads/<기본>]`이고 되돌리기는 R3b의 `refs` 복원만 한다(HEAD 비교는 같음 확인). 다시 하기는 같은 ref 쓰기.
- **R9c**: 동기화의 합칠 대상도 `refs/remotes/<원격>/<브랜치>`로 한정한다. 넣은 파일이 있으면 받을 것이 있을 때 `StagedChanges`.
- **R11b·R11c(C2 고침, D-034 4번)**: `conflict-finish`의 `checkout-index -f`와 `conflict-abort`의 `merge --abort`는 사용자가 외부 편집기로 고친 충돌 파일을 덮어쓴다. 덮어쓰기 전에 충돌 파일의 작업 폴더 내용을 **stash 모양 커밋**으로 만들어(임시 index에 `read-tree HEAD` → 그 파일들의 `hash-object -w` 블롭으로 `update-index` → `write-tree` → 부모 `HEAD`와 index 커밋 둘을 가진 `commit-tree`) 백업 ref(`conflict-files-<브랜치>`)에 남긴다. 새 `GitOp::SnapshotFiles { paths, message } -> oid`(R8에 넣는다). 작업 내역에서 "꺼내기"로 살릴 수 있다. `ResolveFile`이 중간에 실패한 뒤 다시 마치면 이미 해결된 경로는 입력에 있어도 건너뛰고, 남은 충돌 경로가 모두 덮이는지만 본다. 구간이 0개인 텍스트 충돌(모드만 다름 등)은 `Whole`로 둔다.
- **R13·R14**: 합치는 중에는 `sync`(Ctrl+Shift+S)·`branches`(Ctrl+B) 단축키를 끄고 테스트한다. 고른 것은 `localStorage`에 `저장소 경로 + MERGE_HEAD 해시` 키로 보관해 앱을 껐다 켜도 이어진다(try/catch, 없어도 동작). 충돌 화면의 두 색(내 브랜치 `--graph-2`, 합칠 브랜치 `--graph-1`)은 `branchColor` 규칙의 예외로 design.md에 적는다.
- **R15(C8 고침)**: `stash push -m`의 reflog 제목은 `On <브랜치>: <메시지>`, 메시지 없는 사용자 stash는 `WIP on …`이다. "higit parked/stash" 판별은 이 접두사를 벗기고 비교한다. `StashDropOid`는 위치를 찾은 뒤 drop 직전에 같은 oid인지 다시 본다.
- **R16(C15 고침)**: 앱이 시작한 꺼내기를 `<git 폴더>/higit/unpark.json`(브랜치, stash 해시. journal처럼 링크면 쓰지 않음)에 적고, 충돌 화면의 `Unpark` 맥락은 이 파일이 있을 때만 쓴다(터미널의 `stash pop` 충돌과 구분). 마치면 맡긴 ref를 지우지 않고 백업 ref(`parked-<브랜치>`)로 옮긴 뒤 지운다. 맡길 때 새 파일이었던 것이 이미 있으면(add/add) `stash apply`가 충돌 상태를 못 만드므로 지금처럼 `UnparkConflict`로 거절한다(경로 선택 UI 없음, 그 테스트로 바꿈).
- **R17(C6)**: `moves_head() = false`라 실행기가 HEAD 백업을 만들지 않는다(작업 내역 소음). 버리기 스냅샷 백업만 남는다.
- **R19b·R19c(C17 고침)**: 백업 ref의 종류는 이름표가 아니라 객체 모양으로 가른다: 부모가 둘 이상이고 둘째 부모 메시지가 `index on `으로 시작하면 `Changes`(stash 모양: `parked-*`, `discard-*`, `conflict-files-*`), 아니면 `Version`. 서버 쪽 검증: `restore { oid }`는 core가 `recovery` 목록을 다시 계산해 그 안에 있고 `Version`이며 커밋인 것만, `take-out Backup { refname }`은 `list_refs("refs/higit/backup/")` 결과에 있고 stash 모양인 것만 받는다. journal은 서명 여부를 함께 돌려주는 `Journal::read_all()`(형식·`is_valid` 통과 줄)로 읽고 서명이 맞지 않는 줄은 보이되 `restore: None`. 정리(C11)는 한 번 열 때 최대 20개씩 지우고 다음에 이어 간다.
- **R21~R24(C12 고침, D-066·D-041 보완)**: 로컬 쓰기는 시간 제한으로 끝내지 않는다(D-066 그대로). 다만 로컬 쓰기가 60초를 넘으면 진행 표시에 "응답 없음 · 강제로 멈추기"를 보이고, 누르면 확인(`Irreversible`과 같은 대화상자) 뒤 그 git 프로세스 트리를 끝낸다(gpg 핀 입력·느린 hook으로 잠금째 무기한 붙잡히는 것을 푼다). 그래서 R21의 감시·취소는 원격 작업뿐 아니라 실행 중인 모든 git 프로세스에 걸 수 있게 하고(로컬 명령도 `prepare_killable`), 자동 종료(10분·60초·90초 멈춤)는 원격에만, 강제 멈추기는 사람이 확인할 때만 로컬에도 쓴다. `op-progress`는 잠금을 잡은 모든 작업에 3초 뒤부터 보낸다. 멈춘 뒤 남은 잠금 파일은 R24a/b가 정리한다. R1에서 D-066·D-041에 "보완됨" 줄을 단다. 동기화가 받기 단계에서도 작업 폴더 잠금을 쥐는 것은 나누지 않고 취소 버튼으로 대신한다(D-068 영향에 적음).
- **R24b(C13·C14 고침)**: `clear-lock`은 `Irreversible`이지만 `journal_mode = Skip`(경계 줄을 남기면 앞선 Ctrl+Z 기록이 까닭 없이 막힘). `recover-parked`·`take-out`도 HEAD를 바꾸지 않아 `Skip`. `recover-parked`는 메시지만 믿지 않고 브랜치 이름 검사와 `parked_paths` 안전 검사(`..`, `.git`)를 거친다(받은 저장소가 가짜 항목을 꽂을 수 있음). 그 거절 테스트를 둔다.

### 이번 반복에서 하지 않는 것(검토에서 나온 것)
- 받은 저장소의 `remote.<n>.pushurl`·`url.*.pushInsteadOf`·`core.worktree`를 신뢰 확인에 넣는 일: 반복 5(배포) 전에 정한다. 대신 R12·R23에서 올릴 곳의 원격 이름을 보여 준다.
- journal 저장소 바인딩·줄 순번: 이월 유지.
