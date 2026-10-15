# 008 기록 다루기 (반복 3)
- 관련 결정: D-017(역 메뉴 기능은 노선도 위에서, 기록 정리는 카드 지도), D-003, D-010, D-019, D-020, D-021, D-024, D-028, D-033, D-035, D-042, D-043, D-049, D-054, D-056, D-057, D-063, D-064, D-065, D-066, D-069, D-070, D-072, D-073, D-080, D-082, D-083, D-084, D-085, D-086, D-091, D-092, D-095. 새로 정할 것은 아래 "결정할 것" C1~C12이고, 구현 전에 Claude가 `docs/decision.md` 맨 아래에 D-097부터 "(자동 결정)"으로 올린다(E1).
- 관련 명세: spec.md "화면 구성"의 버전 기록·기록 정리·합치기/충돌·작업 내역 행, "충돌 해결 화면"(특히 용어 절의 ours/theirs 뒤바뀜), 용어표(`amend`·`revert`·`reset`·`rebase-i`·`cherry-pick`·`tag`와 위험도 설명), 원칙 4. architecture.md "GitBackend", "Action", "되돌리기와 UndoJournal", "끝나지 않은 작업", "버전 기록 그리기 계층"의 역 메뉴 문단, "보안", "새 액션 추가 절차". design.md "역 메뉴", "충돌 해결 화면", "확인 대화상자".
- 참고 자료(결정 아님): `docs/research/git-visualization.md` ⑦ 이야기 분기 카드 지도(끌어서 순서 바꾸기, 겹쳐서 묶기, 밖으로 빼서 버리기, 카드에서 메시지 고치기, "한 화면 20장 정도가 한계"), `docs/research/quick-actions.md`("가끔 이 버전으로 되돌리기: 역을 누르면 작은 메뉴", "드물게 이 뒤 버전 지우고 돌아가기·이 버전만 복사·태그: 같은 메뉴의 더 보기"), `docs/research/img/flow4/r-log.png`(메뉴 항목 "이때 파일 보기 · 이때로 되돌리기 · 여기서 새 브랜치 · 더 보기"). 기록 정리에는 그림이 없어 같은 느낌(`--surface` 카드, 얇은 선, 먹색 모노)으로 만든다.
- 앞선 계획: 006(합치기·복구, 끝남), 007(노선도·역 메뉴 틀, 끝남). 진행: 사람 확인 없이 Claude가 계획 → code-analyst 검토 → 아래 순서대로 구현, 한 단계 = 한 커밋.

## 목표
버전 기록 노선도에서 역을 눌러 그 버전을 되돌리고(변경을 취소하는 새 버전), 이 버전만 복사하고, 이 버전으로 돌아가고, 태그를 달고, 여기서 새 브랜치를 만들고, 그 버전의 바뀐 파일을 볼 수 있다. 맨 위 버전은 메시지와 내용을 고치고(마지막 버전 고치기), 지금 브랜치 선 위의 버전 여러 개를 골라 카드 지도에서 순서 바꾸기·묶기·메시지 고치기·빼기를 한다(기록 정리). 복사·되돌리기·기록 정리가 충돌을 남기면 기존 충돌 해결 화면으로 푼다. 브랜치 목록에서 브랜치 이름을 바꾸고 지운다. 이미 올린 버전의 기록은 바꾸지 않고 "되돌리기"를 권한다.

## 범위
- 역 메뉴 항목 표(`features/history/menu/items.ts`)에 항목을 더하고 `run`에서 `api.executeAction`으로 core 액션을 부른다. 액션 입력의 해시는 core가 다시 확인한다(D-085와 같은 방식, C3).
- 새 액션: `amend`, `revert`, `cherry-pick`, `reset`, `rebase-i`(기록 정리), `tag`, `tag-delete`, `branch-delete`, `branch-rename`, `branch-create`의 `from`(여기서 새 브랜치).
- 앱이 시작한 cherry-pick·revert·기록 정리의 충돌을 기존 충돌 해결 화면(D-069, D-070, D-086)으로 푼다. 리베이스 중 ours/theirs 뜻을 뒤집어 화면의 뜻을 유지한다(C5).
- 기록 정리 카드 지도 화면과 여러 역 선택(지금 브랜치 선 위의 연속 구간).
- "이때 파일 보기"(C9), 반복 2b·2에서 넘어온 것 중 아래 "넘어온 것 처리" 표의 "넣음".

## 범위 밖
"이번 반복에서 하지 않는 것" 절에 모았다.

## 현재 상태
이 계획이 기대는 사실만 적는다(코드로 확인).
- 역 메뉴 틀(2b): `items.ts`의 `STATION_ITEMS`에 `copy-id` 하나, `StationMenuItem { id, group, label: (m) => string, actionId?, availability(info, env), run(info, ctx) }`, `MenuEnv { repo, currentBranch, changedFiles, merging }`(`merging`은 늘 `false`), `MenuCtx { repo, toast, queryClient, onDone, m }`. `StationMenu.tsx`는 Radix `DropdownMenu`(`modal={false}`)이고 흐린 항목은 `disabled`와 네이티브 `title`, `run`이 실패하면 `errors.internal` 토스트. `StationInfo`에 `isAncestorOfHead`·`onHeadLine`·`unpushed`(지금 브랜치의 upstream 기준, upstream이 없으면 모두 false)가 있다. 목록은 `role="listbox"`이고 선택은 하나(`historyStore.selectedId`), `listKey`는 Shift 조합을 받지 않는다(Shift+F10만).
- 버전 기록 화면은 합치는 중이면 보이지 않는다(`Workspace`가 충돌 화면을 대신 연다). 다른 도구의 리베이스·cherry-pick·revert 중에는 보이고 `conflict.otherOperation`이 `Workspace`에 있다.
- core 액션 21개(`actions::register_all`)에 이번 반복 액션이 없다. `GitOp::Revert { oid, mainline }`는 반복 2의 "올린 뒤 Ctrl+Z"(`undo/run.rs` `revert_pushed`)가 쓰고 실패하면 `revert --abort`로 정리한다. `GitOp::ResetKeep`·`ResetSoft`는 있다. `BranchCreateInput { name }`은 지금 위치에서만 만든다. 태그를 읽는 `tags()`는 있고 쓰기·이름 검사는 없다(`check_branch_name`만).
- 되돌리기: `UndoState.refs`는 `refs/heads/*`·`refs/stash`만 `is_valid`가 받는다. `RestoreRef`는 HEAD가 그대로면 ref만 되돌리고(`merge-into`), `RestoreTree`는 `reset --keep`이다. `Executor`는 `risk >= Caution && moves_head()`면 HEAD를 `refs/higit/backup/<시각>-<순번>/<액션 id>`로 남긴다. 되돌릴 수 없는(`Irreversible`) 액션은 이번 반복에도 없다.
- 충돌: `conflict::context`는 `Merge`(`MERGE_HEAD`)와 `Unpark`(`<git 폴더>/higit/unpark.json` 표식, 서명 없음)만 안다. 리베이스·cherry-pick·revert는 `other_operation`으로만 알리고 `conflict-finish`·`conflict-abort`는 그 중에 `OperationInProgress`로 거절한다. `ConflictContext`는 `Merge | Unpark`. `conflict_paths`는 stage 2를 내 쪽, 3을 합칠 쪽으로 읽고 `ResolveSide { ours }`는 `checkout --ours/--theirs`다.
- `undo()`는 합치기·리베이스·cherry-pick·revert 중에는 `refuse_during_operation`으로 거절한다. `Core.execute`는 새 액션에도 작업 폴더 잠금과 진행·취소 표시(`tracked`)를 자동으로 건다. `run_local_watched`(진행 알림, 사람이 확인한 강제 멈춤)는 `Commit`·`Merge`·`Revert`가 쓰고 환경 변수 인자를 받는 `run_watched`가 러너에 있다.
- 작업 내역(`recovery::collect`): 서명이 맞지 않는 journal 줄도 `journal_marks`로 쓰여 같은 순간의 reflog 줄을 숨기고, 백업 ref마다 `kind_of`(git log 한 번)를 부른 뒤에 200개로 자른다. `take-out`은 `recover-parked`와 달리 `safe_paths` 검사를 거치지 않는다.
- Git 최소 버전 2.30, 이 PC·`git-min`(Ubuntu 22.04 컨테이너)은 2.34.1. 새로 쓰는 git 명령은 `git-min`에서 확인한다.

## 결정할 것
구현 전(E1)에 Claude가 `docs/decision.md`에 D-097부터 "(자동 결정)"으로 올린다(상태 확정). 추천안을 적었고 기존 확정 결정을 바꾸지 않는다(근거 열). D-043의 "올린 버전을 지우는 일도 묻는다" 문장은 C1이 좁힌다(강제 올리기가 생길 때 적용).

| 번호 | 결정(추천안) | 까닭 | 대안 | 근거·부딪힘 확인 |
| --- | --- | --- | --- | --- |
| C1 (D-097) | 올린 버전의 기록은 바꾸지 않는다. 마지막 버전 고치기·이 버전으로 돌아가기·기록 정리는 바뀌거나 빠지는 버전이 모두 "안 올린 것"(어떤 원격 추적 브랜치 `refs/remotes/*`에도 닿지 않음)일 때만 한다. 하나라도 올렸으면 core가 `PublishedHistory`로 거절하고 화면은 "이미 올린 버전이 있어요"와 되돌리기(revert)를 권한다. **강제 올리기(D-043)와 원격 브랜치 지우기는 이번 반복에 넣지 않는다**(설정 화면이 생길 때). | D-042 "원격에 올린 뒤에는 기록을 바꾸는 방법을 쓰지 않는다". 올린 기록을 로컬에서만 고치면 동기화(가져오기→합치기)가 원래 버전을 다시 들여와 같은 변경이 둘이 되거나 충돌한다. 강제 올리기는 "설정에서 켜야 보임"으로 정해졌는데 설정 화면이 아직 없어 켜는 길이 없다. 거절하면 D-043의 "올린 버전 확인"도 이번 반복에는 필요 없다. | 올린 것도 허용하고 강제 올리기를 함께(설정 화면 없이 켜는 길이 없음), 경고만 하고 허용(동기화가 되돌려 놓는 혼란) | D-042, D-043, D-073, spec 위험도 danger 설명(E1이 "올린 기록을 바꾸는 일은 거절한다"로 고침) |
| C2 (D-098) | 역 메뉴 항목. 주 묶음: 버전 번호 복사(기존), 이때 파일 보기, 되돌리기(`revert`), 여기서 새 브랜치, 마지막 버전 고치기(지금 위치 역만). 더 보기: 이 버전만 복사하기, 이 버전으로 돌아가기, 태그, 여기부터 기록 정리. 보이는 조건과 흐린 까닭은 E21~E25 표. **되돌리기(revert)와 반복 2의 올린 뒤 Ctrl+Z의 관계**: 둘 다 `git revert`로 "변경을 취소하는 새 버전"을 만들고 기록은 그대로 남긴다. Ctrl+Z 쪽은 직전 작업을 되돌리다 자동으로 일어나고(실패하면 정리해 시작 전으로) 메뉴 쪽은 사용자가 고른 임의의 버전에 쓴다(올렸든 안 올렸든 됨, 충돌이 나면 멈춰 충돌 화면으로). 메뉴의 `revert` 자체는 Ctrl+Z로 되돌린다(`RestoreTree`, 올린 뒤에는 다시 revert). | r-log·quick-actions의 메뉴 구성을 따른다. 되돌리기는 기록을 바꾸지 않아 C1과 부딪히지 않는 유일한 "버전 취소"라 주 묶음에 둔다. 위험한 것(돌아가기, 정리)은 더 보기. | 메뉴에 모든 항목을 한 줄로(항목이 8개를 넘음), `revert`를 Ctrl+Z로만(고른 옛 버전은 못 취소) | D-017, D-042, D-073, D-092 |
| C3 (D-099) | 기록을 바꾸는 액션은 화면이 준 해시·이름을 core가 다시 확인한다. 해시는 `Oid::parse`, 명령줄에는 `Oid`로만 닿는다. ① `revert { oid }`: 버전이고 지금 위치의 조상. ② `cherry-pick { oid }`: 버전, 합치기 버전 아님, 지금 위치의 조상이 아님, 로컬 브랜치 끝에서 닿음. ③ `reset { oid }`: 지금 위치의 조상이고 지금 위치가 아님, 빠지는 버전이 모두 안 올림(C1). ④ `amend { expect_head, message? }`: `expect_head`가 지금 위치와 같고 지금 위치가 안 올림. ⑤ `rebase-i { oldest, steps[] }`: 지금 브랜치 첫 부모 선 위의 연속 구간과 정확히 같고 합치기·첫 버전 없음, 안 올림, 50개 이하(C4). ⑥ `tag { oid, name }`: ②의 해시 조건이거나 작업 내역에 있음, `tag-delete { name, oid }`: 그 태그가 지금 그 해시를 가리킴. ⑦ `branch-create { name, from? }`: `from`은 ②의 조건이거나 작업 내역(reflog·버전 모양 백업 ref)에 있음. ⑧ `branch-delete { name }`·`branch-rename { from, to }`: 있는 로컬 브랜치, 이름 검사. | 화면 상태는 낡거나 조작될 수 있다(D-034). 같은 규칙을 한 모듈(`core::guard`)에 모으면 액션마다 검사가 달라지지 않는다. | 화면을 믿기(해시가 아무 객체일 수 있음) | D-034, D-035, D-085 |
| C4 (D-100) | 기록 정리는 `git rebase -i`로 한다. 할 일 목록은 편집기 대신 상수 명령 `GIT_SEQUENCE_EDITOR='printf %s "$HIGIT_TODO" >'`과 환경 변수 `HIGIT_TODO`로 넘긴다. 목록은 검증한 16진수 해시와 상수 단어(`pick`·`fixup`·`drop`)만 담는다. 메시지를 고친 카드는 그룹 끝에 `exec "$HIGIT_GIT" commit --amend --no-verify -m "$HIGIT_MSG_<n>"`(상수 줄, 메시지는 환경 변수라 셸이 다시 읽지 않음)을 붙인다. 묶기는 `fixup`(아래 카드에 합침, 그룹 메시지는 맨 아래 카드 것, 고치면 위 `exec`). 합치기 버전·첫 버전·올린 버전이 구간에 있거나 고친 파일이 있으면 거절하고 최대 50개. 되돌리기는 `RestoreTree`(Ctrl+Z로 원래 기록). 먼저 두 OS와 `git-min`에서 이 방식이 도는지 시험한다(E6, 안 되면 위험 절의 대안). | `rebase -i`는 hook·서명·reflog·충돌 정지·`--abort`를 git이 이미 갖고 있고 앱을 껐다 켜도 상태가 저장소에 있다. 편집기 명령에는 사용자 데이터를 넣지 않아 D-035의 "셸에 이름을 넣지 않는다"를 지킨다(git이 상수 명령을 셸로 부르는 것이 유일한 셸). | 해시마다 plumbing(`commit-tree`+3-way 병합)으로 재생(2.38 미만 병합 구현이 필요), 할 일 파일을 임시 폴더에 쓰기(경로 인용 문제), 메시지를 `reword`+편집기로(메시지마다 편집기 값이 달라야 함) | D-003, D-035, D-043, D-052 |
| C5 (D-101) | 앱이 시작한 cherry-pick·revert·기록 정리의 충돌은 기존 충돌 해결 화면이 푼다. 앱이 시작했다는 표식(`<git 폴더>/higit/operation.json`, C6)이 있고 git 상태(`CHERRY_PICK_HEAD`·`REVERT_HEAD`·`rebase-merge/`)가 표식과 맞을 때만 `Operation` 맥락으로 읽고, 그 밖의 작업은 지금처럼 안내 줄만 보인다(D-082). 화면의 두 쪽: cherry-pick은 내 브랜치=지금 위치(stage 2)·합칠 쪽="복사할 버전"(stage 3), revert는 내 브랜치(stage 2)·합칠 쪽="되돌린 모습"(stage 3), **기록 정리는 매핑을 뒤집는다**: 화면의 내 쪽="옮기는 버전"(git stage 3, `REBASE_HEAD`)·합칠 쪽="앞서 정리한 기록"(git stage 2, 지금 HEAD). 뒤집기는 `conflict_paths`에서 두 항목을 바꿔 한 곳에서 하고 해결 쓰기(`ResolveSide`의 `--ours` 여부)도 같은 곳에서 되돌린다. 마치기는 맥락에 맞는 `--continue`, 그만두기는 `--abort`(충돌 파일과 고친 파일을 먼저 백업 ref로). 기록 정리는 여러 번 멈출 수 있어 마치기가 다음 충돌을 다시 열 수 있다. | spec "충돌 해결 화면" 용어 절이 "리베이스에서는 매핑을 뒤집어 화면의 뜻(내 쪽 = 내가 작업한 쪽)을 유지한다"고 정했다. 내 브랜치/합칠 브랜치 용어(ours·theirs)의 설명은 합치기 기준이라 다른 맥락에서는 일반 이름표 글자를 쓴다. | 충돌이 나면 자동으로 그만두기(사용자가 풀 길이 없음), 다른 도구가 시작한 작업까지 이어받기(받은 저장소가 `rebase-merge/`에 `exec` 줄을 심을 수 있어 위험) | D-028, D-069, D-070, D-082, D-086 |
| C6 (D-102) | 앱이 시작한 작업의 표식(`operation.json`)과 기존 `unpark.json`은 앱 데이터의 키로 HMAC 서명하고(저장소 git 폴더 실제 경로와 내용을 묶음) 서명이 없거나 틀리면 표식이 없는 것으로 본다. 기록 정리의 `--continue` 전에는 `rebase-merge/`의 `head-name`·`orig-head`·`onto`와 `git-rebase-todo`의 남은 줄이 표식과 맞는지 다시 본다(해시는 표식의 해시 접두사, `exec` 줄은 상수와 같음). 서명 없는 옛 `unpark.json`은 이 반복 뒤 처음 한 번 "다른 도구의 작업" 안내로 떨어진다(감수). **journal 줄의 저장소·줄 순번 묶기는 이번에 하지 않는다**(아래 처리표). | 받은 zip의 `rebase-merge/`에는 `exec` 줄(명령 실행)을 쓸 수 있어, 앱이 `--continue`를 부르는 순간 그 명령이 돈다. 표식에 키 서명이 있으면 남이 앱 몰래 그 길을 열지 못한다. unpark 표식 서명은 계획 006에서 넘어온 것이다. | 표식 서명 없이 파일 존재만 보기(위 위협), journal 형식까지 지금 바꾸기(이 반복은 journal 형식을 바꾸지 않고 끝낼 수 있다) | D-034, D-063, D-082, D-083 |
| C7 (D-103) | 브랜치 지우기·이름 바꾸기(D-084 이행). **지우기**(`branch-delete`, 새 용어 `branch-delete` "브랜치 지우기" `caution`): 지금 브랜치·기본 브랜치·다른 작업 폴더가 쓰는 브랜치는 못 지운다. 지우기 전에 끝 버전을 백업 ref(`branch-delete-<이름>`)로 남기고 맡긴 ref(`refs/higit/parked/<이름>`)는 백업 ref로 옮긴 뒤 지운다(D-054). 설정 구역은 `git branch -D`가 지운다. 원격의 같은 이름 브랜치는 건드리지 않는다(목록에 원격 브랜치로 남아 다시 만들 수 있다). 확인 창 없음(D-043, 되돌릴 수 있음). Ctrl+Z는 ref를 되살리고 같은 이름의 원격 추적 브랜치가 하나뿐이면 upstream을 다시 잇는다. **이름 바꾸기**(`branch-rename`, 새 용어 "브랜치 이름 바꾸기" `normal`): 로컬만(`branch -m`, 대소문자만 다르면 `-M`), 연결된 upstream은 끊는다(안 끊으면 `push_target`이 옛 이름으로 올린다. 다음 동기화가 새 이름으로 올리며 연결). 맡긴 ref를 새 이름으로 옮긴다. 원격의 옛 브랜치는 그대로. 합치는 중(앱·다른 도구)에는 둘 다 거절. | D-054가 "앱에서 이름을 바꾸면 ref도 옮기고 지우면 백업 ref로"를 정했다. 원격 브랜치를 바꾸는 일은 네트워크·인증·D-043 확인이 따르고 C1이 원격 기록 변경을 미뤘다. | 원격 브랜치까지 함께 지우기·바꾸기(C1, 반복 4), 지우기를 `branch` 용어(`safe`)로(백업이 필요한 위험도가 안 맞음) | D-043, D-054, D-084, D-064 |
| C8 (D-104) | 태그는 가벼운 태그만(`git tag <이름> <해시>`) 만들고 지운다. 이름은 `check-ref-format refs/tags/…`로 검사하고 `-`로 시작하면 막고 255바이트 이하, 같은 이름이 있으면 `TagExists`. 되돌리기는 `refs/tags/<이름>`을 journal의 추적 ref에 넣어 `RestoreRef`로(`UndoState::is_valid`가 `refs/tags/`도 받게 넓힘). 주석 태그·태그 올리기는 하지 않는다(반복 4). | 이름 붙이기(용어표 `tag`)만 쓰고 서명·메시지(`tag.gpgSign`, 사용자 이름)를 피한다. 올리기는 원격 작업이라 C1과 같은 까닭. | 주석 태그(사용자 이름 필요, 서명 설정이 끼어듦) | D-035, D-072 |
| C9 (D-105) | "이때 파일 보기"를 넣는다. 역 메뉴 주 묶음 첫째 항목이 대화상자를 열어 그 버전의 바뀐 파일 목록(경로·상태, 합치기 버전은 첫 부모와 비교)과 고른 파일의 비교(기존 `DiffView`)를 보여준다. 읽기 전용 조회 `query_commit_detail`과 `DiffRequest.commit`을 더한다. 같은 조회가 마지막 버전 고치기 대화상자의 전체 메시지도 준다. | r-log 첫 항목이고 반복 2b가 넘겼다. 되돌리기·돌아가기를 고르기 전에 무엇이 바뀌는지 보는 것이 이 반복의 안전망이다. | 나중으로 미루기(되돌리기 전에 내용을 못 봄) | D-017, D-092 |
| C10 (D-106) | 여러 역 선택은 지금 브랜치 선(첫 부모 선) 위의 연속 구간만이다. Shift+↑↓·Shift+클릭으로 넓히고(범위 모드에서 ↑↓는 그 선의 이웃 역으로 건너뜀) 선택이 둘 이상이면 머리에 "N개 선택 · 기록 정리" 단추, 한 역에서는 메뉴 "여기부터 기록 정리"(그 역부터 지금 위치까지, 50개 이하)로 연다. 모든 버전 보기에서만(접힌 구간이 끼면 못 고름). 기록 정리 화면은 카드 지도: 새 버전 먼저(노선도와 같은 방향) 한 열의 카드, 카드마다 짧은 해시·이름(메시지 첫 줄)·작성자·날짜·상태 글자("옮김"·"묶임"·"빠짐")와 단추("위로"·"아래로"·"아래 카드와 묶기"·"빼기"·"메시지 고치기"). 끌기는 순서 바꾸기만(포인터 이벤트, 새 의존성 없음), 묶기·빼기는 단추와 키(Alt+↑↓ 이동, Delete 빼기, Enter 메시지). 하단 "정리 후 N개 · 빠지는 버전 M개" 줄과 주 버튼 "정리하기", 글자 버튼 "그만두기". 확인 창 없음(되돌릴 수 있음). | 선택이 한 선 위의 연속 구간이어야 `rebase -i`의 할 일 목록과 1:1이 된다. research ⑦은 20장 정도가 한계라고 했지만 스크롤로 50장까지 허용한다. 끌어서 묶기·빼기는 두 OS 포인터 동작을 더 확인해야 해 나중으로 둔다. | 아무 역이나 고르기(여러 선 사이 순서가 뜻이 없음), 끌어서 묶기·빼기까지(범위가 큼) | D-017, D-056(목록 안 키는 부품 조작), D-092 |
| C11 (D-107) | 역 메뉴 틀 보완: ① 마우스 오른쪽 클릭(`contextmenu`)도 그 행을 골라 메뉴를 연다(웹뷰 기본 메뉴는 `preventDefault`). ② 흐린 항목의 까닭은 네이티브 `title` 대신 `Tooltip`으로, 키보드로도 닿게 Radix `disabled` 대신 `aria-disabled`와 선택 막기를 쓴다. ③ `MenuEnv`는 `merging: boolean`을 `operation: "none" \| "other"`(다른 도구의 작업 중)로 바꾸고 `workspace`가 실제 상태를 이어 준다(앱이 시작한 합치기·복사·정리 중에는 이 화면이 아예 보이지 않는다). `stagedFiles`·`upstream`·`defaultBranch`·`zoom`도 더한다. ④ `ctx.ask(dialog)`로 입력이 필요한 항목(태그, 새 브랜치, 마지막 버전 고치기, 이때 파일 보기)이 `History`의 대화상자를 연다. ⑤ **되돌릴 수 없는 액션(D-043)의 확인 창 틀은 만들지 않는다**: 이번 반복의 항목에는 `Irreversible`이 없다(C1·C7이 그렇게 정함). 대신 표의 모든 `actionId`가 `Irreversible`이 아님을 확인하는 가드 테스트를 두어, 그런 항목이 생기면 그때 확인 창 틀(`preview` 토큰 흐름은 `CompareFooter`가 이미 쓴다)을 같이 만들게 한다. ⑥ 접힌 구간을 펼치면 펼친 첫 버전을 고른다. | 2b 점검이 넘긴 것. 쓰는 항목이 없는 확인 창을 미리 만들면 시험되지 않은 코드가 된다. | 확인 창 틀을 지금 만들기(쓰는 곳이 없음) | D-033, D-043, D-092 |
| C12 (D-108) | 이월한 것의 처리(아래 "넘어온 것 처리" 표). **넣음**: unpark 표식 서명(C6), 작업 내역의 서명 없는 journal 줄 신뢰 경계와 성능(서명 안 된 줄은 reflog 줄을 숨기지 않고 보이는 수를 50개로 제한, 200개로 자른 뒤에 `git log`·`parked_paths`를 부름), `take-out`의 `safe_paths` 검사. **미룸**: journal 줄을 저장소·줄 순번에 묶기와 키를 못 쓸 때 알리기(반복 5 전 배포 보안 묶음, `pushurl` 등과 함께), 충돌 선택의 `localStorage` 평문, 취소 표시가 저장소 단위인 문제. | 넣은 것은 이번에 새로 만드는 표식·작업 내역 줄과 같은 파일을 건드린다. 미룬 것은 journal 형식을 바꿔야 하거나(옛 줄 호환이 필요) 같은 사용자 폴더에 이미 평문으로 있는 내용과 같은 신뢰 경계라 위험이 낮다. | 모두 넣기(journal 형식 변경은 옛 줄 호환 설계가 따로 필요) | D-063, D-072, D-079 |

## 넘어온 것 처리
| 출처 | 항목 | 이번 반복 | 단계 |
| --- | --- | --- | --- |
| 2b 점검 | `MenuEnv.merging` 잇기 | 넣음(`operation`으로 바꿔 `Workspace`가 잇는다) | E21 |
| 2b 점검 | 되돌릴 수 없는 액션의 확인 창(D-043)을 `ctx`에 | 넣지 않음(항목이 없음). 가드 테스트로 대신 | E22 |
| 2b 점검 | 흐린 항목 까닭을 Tooltip으로 | 넣음 | E21 |
| 2b 점검 | 마우스 오른쪽 클릭으로 열기 | 넣음 | E21 |
| 2b 점검 | 접힌 구간을 펼친 뒤 선택 | 넣음 | E21 |
| 2b 점검 | 시작 화면 상태 요약을 동시 실행 큐에 | 미룸(이번 반복과 무관, 20개 프로젝트에서 느리다고 확인되면) | — |
| 2b 점검 | `hitTest` 미사용 | 그대로(노선도 위에서 직접 끄는 기능은 나중) | — |
| 2b 점검 | "이때 파일 보기" | 넣음(C9) | E19, E25 |
| 2 점검 | journal 서명을 저장소·줄 순번에 묶기, 키를 못 쓸 때 알리기 | 미룸(C12) | — |
| 2 점검 | `unpark.json` 표식 서명 | 넣음(C6) | E8 |
| 2 점검 | 서명 안 된 journal 줄의 신뢰 경계·성능 | 넣음(C12) | E20 |
| 2 점검 | `take-out`의 `safe_paths` | 넣음(C12) | E20 |
| 2 점검 | 충돌 선택 `localStorage` 평문 | 미룸(C12) | — |
| 2 점검 | 취소 표시가 저장소 단위 | 미룸(C12) | — |
| 2 점검 | 진행 알림 1초부터(계획은 3초) | 그대로 | — |
| 006 | 리베이스·cherry-pick·revert 중 충돌 해결 | 넣음(C5) | E9, E12, E15, E26 |
| 006·D-084 | 브랜치 지우기·이름 바꾸기(맡긴 ref 옮기기·백업, 원격·upstream) | 넣음(C7) | E17, E30 |
| 006·D-085 | 지운 브랜치를 새 브랜치로 되살리기 | 넣음(작업 내역 카드의 "새 브랜치로") | E18, E31 |
| 006 | 임시 보관 이름 붙이기·지우기 | 미룸(로드맵 반복 3 목록에 없음) | — |
| 006 | `pushurl`·`pushInsteadOf`·`core.worktree` 신뢰 확인 | 미룸(반복 5 전) | — |

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04 git 2.34.1, Windows 11)에서 손으로, 가능한 것은 자동 테스트로 확인한다.
- 버전 기록에서 역을 누르거나 오른쪽 클릭·Enter로 메뉴가 열리고(r-log), 지금 위치 역에는 "마지막 버전 고치기"가 있다. 흐린 항목은 까닭이 툴팁(키보드 포커스에도)으로 뜬다.
- "되돌리기": 고른 버전의 변경을 취소하는 새 버전이 생기고 토스트 "되돌리는 새 버전을 만들었어요 · 되돌리기"가 뜬다. 올린 버전에도 된다. 겹치면 충돌 해결 화면이 열려 한 곳씩 고르고 "마치기"로 새 버전이 생긴다. "그만두기"는 시작 전으로 돌린다.
- "이 버전만 복사하기": 다른 브랜치의 버전 하나가 지금 브랜치에 새 버전으로 복사되고 Ctrl+Z로 사라진다. 겹치면 위와 같다.
- "이 버전으로 돌아가기": 안 올린 버전만 뒤로 빠지고 고친 파일은 남는다. 올린 버전이 끼면 "이미 올린 버전이 있어요"로 거절한다. Ctrl+Z로 돌아온다. 빠진 버전은 작업 내역에 있다.
- "마지막 버전 고치기": 메시지(와 이번 버전에 넣은 파일)를 고쳐 새 해시가 되고 Ctrl+Z로 이전 버전이 돌아온다. 올린 버전이면 거절한다.
- 태그: 이름을 쓰면 그 역에 태그가 붙고(노선도 이름 옆에 보임), 지울 수 있고, 각각 Ctrl+Z로 돌아온다. "여기서 새 브랜치": 그 버전에서 새 브랜치를 만들고 갈아탄다(고친 파일이 있으면 흐림).
- "이때 파일 보기": 그 버전에서 바뀐 파일 목록과 고른 파일의 비교가 대화상자로 보인다.
- 기록 정리: 지금 브랜치 선 위의 두 역 이상을 Shift+↑↓로 골라(또는 메뉴 "여기부터 기록 정리") 카드 지도에서 순서를 바꾸고, 아래 카드와 묶고, 메시지를 고치고, 빼서 "정리하기"를 누르면 그대로 새 기록이 되고 Ctrl+Z로 원래 기록이 돌아온다. 합치기 버전·첫 버전·올린 버전·고친 파일이 있으면 시작하지 못하고 까닭이 보인다. 순서를 바꿔 충돌이 나면 충돌 해결 화면(내 쪽 = 옮기는 버전, 합칠 쪽 = 앞서 정리한 기록)에서 풀고 마치면 다음 충돌로 이어지거나 끝난다. "그만두기"는 원래 기록으로 돌린다.
- 앱을 껐다 켜도 앱이 시작한 복사·되돌리기·기록 정리 충돌은 충돌 화면으로 다시 열린다. 다른 도구가 시작한 것과 서명이 없는 표식은 안내 줄만 보이고 앱이 `--continue`·`--abort`를 부르지 않는다.
- 브랜치 목록에서 로컬 브랜치 이름을 바꾸고(upstream은 끊김, 맡긴 파일이 따라옴) 지울 수 있다(지금·기본 브랜치는 흐림). 지우면 토스트의 되돌리기와 작업 내역(백업)에서 되살릴 수 있고 원격 브랜치는 그대로다.
- 작업 내역의 버전 카드에 "새 브랜치로"가 있고 백업 이름(`reset`·`amend`·`rebase-i`·`branch-delete-…`)이 용어로 보인다. 서명 없는 journal 줄이 reflog 줄을 숨기지 않는다.
- 새 액션 9개(`amend`·`revert`·`cherry-pick`·`reset`·`rebase-i`·`tag`·`tag-delete`·`branch-delete`·`branch-rename`)의 위험도가 용어표 위험도와 같고(`registry-glossary` 테스트) `Irreversible`은 없다.
- 800px 너비와 다크 모드에서 기록 정리·대화상자·메뉴 글자가 겹치거나 잘리지 않는다(긴 이름은 말줄임).
- `pnpm check:full` 통과, 두 OS CI 초록(`git-min` 포함). code-analyst 점검 위반 0건. 화면에서 git·`invoke` 직접 호출 없음, 색 값은 `tokens.css` 밖에 없음, 새 의존성 없음.

## 단계
각 단계 = 커밋 하나, 메시지는 `<영역>: <요약>`. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령·액션 목록이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`. 새 Tauri 명령은 `src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣는다(`tests/security_config.rs`가 확인). 새 액션은 architecture.md "새 액션 추가 절차"대로: spec 용어표·glossary(E1에서 끝남) → `GitOp`(git 크레이트 단계) → `ActionInput`·`actions/<id>.rs`·`register_all`·`Outcome`/`ActionResult` → 화면 → 테스트(임시 저장소에서 실행 → 결과 → 되돌리기). `GitBackend`에 메서드를 더하면 가짜 구현 둘(`executor.rs` 테스트의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)도 같은 커밋에서 고친다. 새 git 명령은 `args::Cmd` 변형(리비전 앞 `--end-of-options`가 2.34.1에서 되는지)을 정하고 `git-min`에서 확인한다(`reset`·`checkout`은 넣지 않는 기존 규칙, `Oid`는 16진수만이라 표시를 못 넣어도 옵션으로 읽히지 않음). 테스트 저장소는 `core.autocrlf false`·`user.name`·`user.email`·`commit.gpgsign false`를 저장소에 넣고 파일 이름에 Windows 금지 글자(`*?:<>|"`)를 쓰지 않는다. 용어 호버는 `<Term>`, 문장 속 용어는 `t(id)`(D-029). 화면 문구는 `src/messages` ko·en(괄호·옛 용어·합쇼체 없음, `copy-rules`·`terms-in-messages` 테스트)에 두고 이름과 숫자만(D-049, 토스트·오류 문장은 D-057). 색은 `tokens.css` 변수만. 새 ErrorCode는 `messages`의 `errors`에 ko·en 문구를 함께(완전성 테스트). 모든 새 화면 입력은 한글 조합 중(`isComposing`) Enter를 무시한다.

### A. 결정·용어

E1. 문서·결정·용어 (`docs: add iteration 3 decisions and terms`)
- 목표: 이후 단계가 쓰는 결정과 새 용어를 먼저 확정한다(D-015: spec 표, glossary ko·en 함께).
- 바꿀 파일: `docs/decision.md`(C1~C12를 D-097~D-108 "(자동 결정)", 상태 확정. D-043 영향에 "올린 버전 확인은 강제 올리기가 생길 때(D-097)", D-084 "이행됨(반복 3)", D-086의 "리베이스·cherry-pick·revert 충돌은 안내 줄만"에 "반복 3에서 앱이 시작한 것은 D-101"), `docs/spec.md`(용어표에 `branch-delete`·`branch-rename` 행, 위험도 절 `danger` 줄을 "올린 기록을 바꾸는 일은 거절한다(D-097)"로, 화면 구성의 버전 기록·기록 정리·합치기/충돌 행과 "충돌 해결 화면" 용어 절에 새 맥락·매핑, "나중 후보"에 변동 없음), `src/glossary/{types,ko,en}.ts`(`TERM_IDS`는 spec 표 순서), `docs/roadmap.md`(반복 3 상세는 계획 008, 결정 반영), `docs/architecture.md`는 각 단계가 코드와 함께 고친다(이 단계에서는 "역 메뉴" 문장의 "반복 3" 표현만).
- 내용: 새 용어 `| branch-delete | 브랜치 지우기 | branch -D · git branch -D | 브랜치 이름표를 지우기. 버전은 작업 내역에 남아요. | caution | 드묾 |`, `| branch-rename | 브랜치 이름 바꾸기 | branch -m · git branch -m | 브랜치 이름을 바꾸기. 원격의 같은 이름은 그대로예요. | normal | 드묾 |`(표에서 `branch` 다음). en: "Delete branch", "Rename branch". 다른 이번 반복 용어(`amend`·`revert`·`reset`·`rebase-i`·`cherry-pick`·`tag`)는 이미 있고 위험도는 그대로다. `revert` 설명의 "기록은 그대로 남아요", `reset` 설명의 "그 뒤의 버전은 기록에서 빠져요"는 바꾸지 않는다. 화면 문구("N개 선택", "정리하기", "이때 파일 보기" 등)는 Git 용어가 아니라 `messages`에 둔다.
- 테스트: `src/glossary/spec-sync.test.ts`, `TERM_IDS` 순서, `copy-rules.test.ts`, `terms-in-messages.test.ts`, `registry-glossary.test.ts`(아직 액션 없음).
- 완료 기준: `pnpm test`, `pnpm lint`.
- Windows/Ubuntu: 차이 없음.
- 보안: 해당 없음.

### B. git 크레이트(조회·쓰기)

E2. git: 올린 버전·도달 판정과 메시지 읽기 (`git: add unpublished count, branch reachability and commit messages`)
- 목표: C1·C3의 검사와 마지막 버전 고치기·기록 정리의 전체 메시지를 읽는 조각을 `GitBackend`에 둔다.
- 바꿀 파일: `crates/git/src/types.rs`(`CommitMessage { oid: Oid, message: String }`), `backend.rs`, `cli/mod.rs`, 신규 `cli/history.rs`(+`history_tests.rs`), `lib.rs`, 가짜 구현 둘, `docs/architecture.md` GitBackend 목록.
- 내용: ① `count_unpublished(repo, tip: &Oid, base: Option<&Oid>) -> u32`: 어떤 원격 추적 브랜치에도 닿지 않는 `tip`의 조상 수. `for-each-ref --format=%(objectname) refs/remotes/`로 끝점을 모아(1000개까지) `rev-list --count --stdin`에 `tip`, `^base`, `^<원격 끝점>…`을 줄로 넘긴다(명령줄 길이 한도를 피하고 `--not --remotes` 위치 문제를 피함). 원격이 없으면 `tip`의 조상 수 전부. ② `reachable_from_branches(repo, oid) -> bool`: `for-each-ref --contains=<oid> --count=1 --format=%(refname) refs/heads/`가 한 줄이라도 내면 참. ③ `commit_messages(repo, &[Oid]) -> Vec<CommitMessage>`: 해시를 stdin(`--stdin`)으로 넘기는 `log --no-walk=unsorted --no-show-signature --format=%H%x00%B%x1e`, 출력 1MB 상한(`run_capped`), 100개 한 번, 상한을 넘으면 앞부분만.
- 테스트(실제 git): `count_unpublished`: 원격 없음(전부), bare 원격에 올린 뒤 새 버전 둘(2), `base` 지정, 갈라진 브랜치가 올라간 경우, 원격 끝점 1000개 넘는 저장소의 잘림 표시, 해시가 많은 stdin. `reachable_from_branches`: 브랜치 끝·조상·어느 브랜치에도 없는 버전(백업 ref만 가리킴). `commit_messages`: 한글·여러 줄·빈 줄·끝 줄바꿈 없는 메시지, 코멘트 문자(`#`)로 시작하는 줄, `NUL` 없음, 100개, 잘린 출력. `git-min`에서 세 명령.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`, `cargo test --workspace`.
- Windows/Ubuntu: `LC_ALL=C`·`core.quotepath=off` 공통. stdin 줄 끝은 `\n`.
- 보안: 해시는 `Oid`만, 이름은 읽지 않고 개수만 쓴다.

E3. git: 버전의 바뀐 파일과 그 버전의 비교 (`git: add commit file list and commit diff`)
- 목표: "이때 파일 보기"(C9)가 쓸 조회.
- 바꿀 파일: `types.rs`(`CommitFile { path, orig_path: Option<String>, change: Change }`, `DiffKind::Commit { oid: Oid }`), `backend.rs`(`commit_files(repo, &Oid) -> Vec<CommitFile>`), `cli/history.rs`, `cli/mod.rs`의 `diff`(새 종류), 가짜 구현 둘, architecture.md.
- 내용: `commit_files`: `diff-tree -r -z --root --name-status -M --no-commit-id --first-parent -m <oid>`(합치기 버전은 첫 부모와 비교, 루트 버전은 빈 트리와). 파일 500개까지 읽고 넘으면 개수만 알린다(`more`). `diff`의 `Commit`: 같은 방식으로 한 파일의 `-p`(`--no-ext-diff --no-textconv`, 경로는 `--` 뒤 `--literal-pathspecs`)를 기존 `diff` 파서·큰 파일 상한·이진·링크 처리와 함께 쓴다. 링크는 내용을 읽지 않고 대상만(D-052).
- 테스트: 추가·수정·삭제·이름 바꿈·이진·모드 변경·빈 버전·루트 버전·합치기 버전(첫 부모 기준)·한글 경로·공백 경로·파일 501개. 비교: 루트 버전·삭제 파일·큰 파일(상한)·링크. `git-min`.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`.
- Windows/Ubuntu: `-z` 출력이라 경로 구분자·줄바꿈 차이가 없다. CRLF 파일의 줄 끝 표시는 기존 비교와 같다.
- 보안: 읽기 전용, `Oid`만 인자, 경로는 `--`와 `--literal-pathspecs`.

E4. git: 마지막 버전 고치기·복사·되돌리기의 시작·이어가기·그만두기 (`git: add amend, cherry-pick and revert operations`)
- 목표: 기록을 바꾸는 단일 작업의 `GitOp`를 둔다. 충돌이 나면 멈춘 채 남기고(앱이 충돌 화면으로 푼다), 이어가기·그만두기도 `GitOp`로 한다.
- 바꿀 파일: `types.rs`(`GitOp::Amend { message: Option<String> }`, `CherryPickStart { oid }`, `RevertStart { oid, mainline: Option<u32> }`, `OperationContinue { kind: OperationKind }`, `OperationAbort { kind }`, `enum OperationKind { CherryPick, Revert, Rebase }`), `cli/mod.rs`(`run`에 매핑, 진행 알림과 강제 멈춤이 필요하니 `run_local_watched`로), 신규 `cli/operation.rs`, `cli/args.rs`(`Cmd` 변형), `failure.rs`(+테스트: 비어 버린 복사·되돌리기 "now empty"·"nothing to commit"를 `NothingToCommit`이 아닌 새 `Empty`로, 나머지는 기존 `MergeConflict`·`WouldOverwrite`·`IdentityMissing`), 가짜 구현 둘 변화 없음(`run`만 쓰임), architecture.md `GitOp` 목록.
- 내용: `Amend`: 메시지가 있으면 `commit --amend -F -`(stdin), 없으면 `--amend --no-edit`. 이번 버전에 넣은 파일은 git이 함께 넣고 사용자 hook·서명을 따른다(`Commit`과 같음). `CherryPickStart`: `cherry-pick <oid>`(`-x`·`--allow-empty` 없음). `RevertStart`: `revert --no-edit [-m N] <oid>`. 두 시작 작업은 **충돌이나 hook 거부로 멈추면 정리하지 않는다**(기존 `Revert`와 다름. 기존 `GitOp::Revert`는 올린 뒤 Ctrl+Z가 계속 쓴다). 호출한 core가 상태(`CHERRY_PICK_HEAD`·`REVERT_HEAD`와 충돌 항목)를 보고 정한다. `OperationContinue`: `cherry-pick --continue`·`revert --continue`·`rebase --continue`(`GIT_EDITOR=:`라 편집기가 안 뜸). `OperationAbort`: `cherry-pick --abort`·`revert --abort`·`rebase --abort`.
- 테스트(실제 git): `Amend`: 메시지만, 메시지+넣은 파일, 한글·여러 줄, 루트 버전, 합치기 버전, hook 거부 후 상태. `CherryPickStart`: 깨끗이 됨, 충돌로 멈춰 `CHERRY_PICK_HEAD`와 `index_entries` stage 1·2·3(stage 2 = 지금 위치, 3 = 복사할 버전을 확인), 이미 같은 내용이라 비어 버림(`Empty`), 합치기 버전은 `-m` 없이 거절. `RevertStart`: 깨끗이 됨, 충돌로 멈춰 `REVERT_HEAD`(stage 3은 되돌릴 버전의 부모 쪽 내용), 합치기 버전 `-m 1`, 비어 버림. `Continue`: 해결 뒤 새 버전, 해결 안 한 채면 실패하고 상태 유지. `Abort`: 시작 전으로. `GIT_EDITOR` 설정과 `commit.gpgsign=false`·이름 없음. `git-min`에서 `cherry-pick --continue`·`revert --continue`·`--abort`.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`, `cargo test --workspace`.
- Windows/Ubuntu: 편집기·편집 중 파일이 잠긴 Windows에서는 `cherry-pick`이 실패할 수 있다(오류로 알림). 메시지는 stdin이라 명령줄 인코딩 문제가 없다.
- 보안: 해시는 `Oid`만. `--continue`·`--abort`는 core가 표식을 확인한 뒤에만 부른다(E12·E15). 메시지는 stdin.

E5. git: 태그·브랜치 쓰기 (`git: add tag, branch delete/rename and create-at`)
- 목표: 태그·브랜치 액션이 쓸 `GitOp`와 태그 이름 검사.
- 바꿀 파일: `backend.rs`(`check_tag_name(name) -> bool`), `types.rs`(`GitOp::TagCreate { name, oid }`, `TagDelete { name }`, `BranchDelete { name }`, `BranchRename { from, to, force: bool }`, `UnsetUpstream { branch }`, `SetUpstream { branch, upstream }`, `CreateAndSwitchAt { name, oid }`), `cli/mod.rs`, 신규 `cli/branch_ops.rs`(+테스트), `cli/args.rs`, `branches.rs`의 `obviously_bad_branch_name` 재사용(`-` 시작·`HEAD`·공백), 가짜 구현 둘(`check_tag_name`).
- 내용: `check_tag_name`은 `check-ref-format refs/tags/<이름>`(`check_branch_name`과 같은 방식). `TagCreate`: `tag <이름> <oid>`(가벼운 태그, 주석·서명 없음, `-c tag.gpgSign=false`). `TagDelete`: `tag -d <이름>`. `BranchDelete`: `branch -D <이름>`(설정 구역도 지움). `BranchRename`: `branch -m <from> <to>`(`force`면 `-M`, 대소문자만 다를 때). `UnsetUpstream`·`SetUpstream`: `branch --unset-upstream <b>`(upstream이 없어 실패하면 성공으로 봄)·`branch --set-upstream-to=<remote/branch> <b>`. `CreateAndSwitchAt`: `switch --no-guess -c <name> <oid>`.
- 테스트: 이름 검사 표(공백·`..`·`~`·`:`·`-x`·`.lock`·한글·유니코드·255바이트), 만들기·지우기·이미 있음, 한글 태그·브랜치, 지울 때 `branch.<이름>.*` 설정 구역이 사라짐, 이름 바꿀 때 설정 구역이 따라가고 upstream을 끊으면 `@{upstream}`이 없음, 대소문자만 바꾸는 이름 바꾸기(Windows CI에서 실제로 대소문자 무시 파일시스템), `CreateAndSwitchAt`이 작업 폴더를 그 버전으로. `git-min`에서 `branch -m/-D/--unset-upstream`·`tag -d`·`switch -c … <oid>`.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`.
- Windows/Ubuntu: Windows에서 `aux`·`con` 같은 예약 이름은 git이 파일을 못 만들어 실패하므로 `InvalidRefName`이나 `GitFailed`로 알린다(테스트는 Windows에서만 기대를 다르게). 대소문자 이름 바꾸기는 위 `-M`.
- 보안: 이름은 `check_*_name`과 core `validate_ref_like`를 거치고 `-`로 시작하면 막는다. 해시는 `Oid`.

E6. git: 기록 정리 (`git: add history rewrite with a todo from the environment`)
- 목표: `git rebase -i`를 할 일 목록 환경 변수와 상수 편집기 명령으로 돌린다(C4). **본 구현 전에 이 방식이 두 OS와 `git-min`에서 도는지 시험을 먼저 쓴다.**
- 바꿀 파일: `types.rs`(`GitOp::RewriteHistory { base: Oid, steps: Vec<RewriteStep>, git_exe: String }`, `struct RewriteStep { oid: Oid, kind: RewriteKind, message: Option<String> }`, `enum RewriteKind { Pick, Fixup, Drop }`), 신규 `cli/rewrite.rs`·`rewrite_tests.rs`, `cli/mod.rs`(`run_local_watched`가 환경 변수 인자를 받게), `failure.rs`(`rebase`의 "cannot rebase: You have unstaged changes"는 `WouldOverwrite`가 아니라 새 `DirtyTree`), architecture.md(실행 규칙에 "편집기 상수 명령" 한 문단).
- 내용: ① 목록 만들기(순수 함수): `steps`를 그대로 줄로(`pick|fixup|drop <전체 해시>`), 메시지가 있는 단계는 그 묶음의 마지막 줄 뒤에 `exec "$HIGIT_GIT" commit --amend --no-verify -m "$HIGIT_MSG_<번호>"`. `Fixup`이 첫 줄이거나 `Drop` 바로 뒤면 오류. 메시지는 `Pick`에만. ② 실행: `-c rebase.autoSquash=false -c rebase.missingCommitsCheck=ignore -c rebase.updateRefs=false -c rebase.autoStash=false rebase -i <base 해시>`와 환경 변수 `GIT_SEQUENCE_EDITOR=printf %s "$HIGIT_TODO" >`(상수), `HIGIT_TODO`(목록 글자), `HIGIT_GIT`(앱이 찾은 git 절대 경로), `HIGIT_MSG_<n>`(메시지). 환경 변수 이름은 앱이 만든 것뿐이고 값은 셸이 다시 읽지 않는다. ③ 끝나면 성공. 실패하면 `rebase-merge/`가 있는지 보고(있으면 충돌이나 hook 거부로 멈춘 것이라 오류가 아니라 "멈춤"으로 돌려 core가 충돌 화면으로 간다), 없으면 시작하지 못한 오류(고친 파일·hook 등)로 돌려준다. 메시지 길이는 하나 16KB, 환경 변수 전체 24KB를 넘으면 시작 전에 거절한다(Windows 환경 블록 한도 아래).
- 테스트(실제 git, 두 OS와 `git-min`): 순서 바꾸기(두 버전 서로 겹치지 않음), `fixup` 묶기와 묶음 메시지(`exec`), 한글·여러 줄·따옴표·`$`·백틱·`;`·줄바꿈이 든 메시지가 그대로 들어가고 셸이 해석하지 않음, 빼기(`drop`), 변화 없는 목록, 겹쳐서 `rebase-merge/`가 남고 stage 2·3이 `REBASE_HEAD`·HEAD와 맞음(C5 매핑 확인), hook(`pre-commit`이 있어도 `--no-verify`라 안 돎, `post-rewrite`는 돎), 고친 파일 있음(`DirtyTree`), 목록이 잘못된 입력 거절, `core.editor`·`sequence.editor`·`GIT_SEQUENCE_EDITOR` 환경이 이미 있어도 이 환경 변수가 이김, 메시지 한도. **안 되면 대안**(위험 절): `exec` 대신 `edit`로 멈추고 앱이 `commit --amend`(배열 인자)를 부른 뒤 `--continue`를 반복.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`(Windows CI의 Git for Windows `sh`·환경 변수 줄바꿈 포함).
- Windows/Ubuntu: Git for Windows는 편집기·`exec`를 자기 `sh`로 돈다. MSYS가 환경 변수 값을 경로로 바꿀 수 있어(`HIGIT_GIT`) 인용해 쓴다. 목록 값에는 `/`·`:`가 없다.
- 보안: 셸 명령은 상수이고 사용자 데이터(해시 외)는 환경 변수 값뿐이다. 해시는 `Oid`. 목록에 사용자 글자가 들어가지 않는다. 진행 알림·강제 멈춤은 `run_local_watched`.

### C. core

E7. core: 공통 검사·오류·journal 범위 (`core: add history guards, errors and tag refs in undo state`)
- 목표: 기록을 바꾸는 액션이 같이 쓰는 검사와 오류, journal 되돌리기 범위를 먼저 둔다.
- 바꿀 파일: 신규 `crates/core/src/guard.rs`(+테스트), `error.rs`·`view/mod.rs`(`CoreError::PublishedHistory`·`HasChanges`·`TagExists`·`BranchProtected`·`NoChange`와 `ErrorCode`), `src/messages/{ko,en}.ts`(`errors` 문구), `undo/mod.rs`(`UndoState::is_valid`의 `ref_ok`에 `refs/tags/<이름>`), `action.rs`(`Action::journal_before(&self, ctx, input, captured: UndoState) -> UndoState` 기본은 그대로, E15가 씀), `executor.rs`(캡처한 `before`를 이 훅에 통과시킴), 생성물, architecture.md(오류 코드, journal 추적 ref).
- 내용: `guard`: `verified_commit(git, repo, oid, Scope)`(커밋인지, `Scope::HeadAncestor`/`BranchReachable`/`BranchOrActivity`), `refuse_published(git, repo, tip, base)`(`count_unpublished`가 `count_between`의 앞서 있는 수와 다르면 `PublishedHistory`), `refuse_dirty(git, repo)`(추적 중인 파일이 바뀌었거나 넣은 파일이 있으면 `HasChanges`, 새 파일(추적 안 함)은 허용), `refuse_staged`는 기존 `actions::merge::refuse_staged`를 이쪽으로 옮겨 둘이 같이 쓴다, `require_branch(git, repo) -> String`(특정 버전을 보는 중이면 거절). 모두 `refuse_during_operation`과 함께 쓰는 순서를 문서로 적는다. 새 오류 문구(예): `publishedHistory` "이미 올린 버전이 있어서 기록을 바꿀 수 없어요. 되돌리기를 써 보세요.", `hasChanges` "고친 파일을 먼저 저장하거나 임시 보관해 주세요.", `tagExists`, `branchProtected` "지금 브랜치와 기본 브랜치는 지울 수 없어요.", `noChange` "이미 같은 내용이에요."(문구는 D-057 오류 문장 규칙, `NothingToCommit`와 별개).
- 테스트: `guard` 표(가짜 `GitBackend`로 호출 횟수, 올린 것·안 올린 것·원격 없음), `is_valid` 표(`refs/tags/v1.0`·`refs/tags/-x`·`refs/tags/a..b`·`refs/remotes/…` 거절), 옛 journal 줄이 그대로 읽힘(고정 JSON·서명), `journal_before` 기본이 항등, 메시지 완전성(`errors` 키 = `ErrorCode`).
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음.
- 보안: journal은 믿지 않는 입력이라 `is_valid` 범위를 태그 이름 규칙까지만 넓힌다. 검사 함수가 화면 값을 명령줄에 닿게 하지 않는다.

E8. core: 서명 표식과 키 잇기 (`core: sign operation markers`)
- 목표: 앱이 시작한 작업의 표식을 키로 서명하는 바탕과 unpark 표식 이전(C6).
- 바꿀 파일: 신규 `crates/core/src/marker.rs`(+테스트), `parked.rs`(`write_marker`·`read_marker`·`clear_marker`를 `marker`로 옮김), `bootstrap.rs`(앱 데이터 키를 `Queries`에도 줌), `query.rs`(`Queries`에 키), `conflict.rs`(`context(git, repo, key)`), `actions/conflict.rs`·`actions/switch.rs`·`actions/stash.rs`의 표식 호출, 테스트 공통(`crates/core/tests/common`)의 키, architecture.md("끝나지 않은 작업" 표식 문단).
- 내용: `marker::write<T>(git_dir, name, key, &T)`·`read<T>(git_dir, name, key) -> Option<T>`·`clear`. 파일은 `{ "v": 1, "dir": "<git 폴더 실제 경로>", "body": {…}, "sig": … }`이고 서명은 `v`·`name`·`dir`·`body`의 글자에 건다. 링크 검사·`higit` 폴더 위치 검사는 기존 `place_ok`와 같다. 서명이 없거나 틀리거나 `dir`이 지금 실제 경로와 다르면 `None`. 저장소 폴더를 옮기면 진행 중 표식이 무효가 되어 "다른 도구의 작업" 안내로 떨어진다(감수). unpark는 이 모듈로 옮기고 서명 없는 옛 표식은 읽지 않는다.
- 테스트: 쓰고 읽기, 내용 한 글자 바꿈·다른 키·다른 저장소 폴더로 복사·링크인 `higit` 폴더·서명 없는 옛 형식 거절, 기존 unpark 충돌 흐름 테스트가 그대로 통과, 키가 임시(`ephemeral`)여도 동작.
- 완료 기준: `cargo test --workspace`.
- Windows/Ubuntu: 실제 경로는 `canonicalize`(Windows `\\?\` 접두사 차이를 같은 함수로 맞춤, `platform::same_path`).
- 보안: 표식이 `rebase --continue`(할 일 목록의 `exec`)의 문이므로 서명이 핵심이다. 키는 `Debug`로 나오지 않는다(기존 `JournalKey`).

E9. core: 작업 맥락과 충돌 조회의 새 맥락 (`core: operation context and flipped sides for conflicts`)
- 목표: 충돌 조회가 복사·되돌리기·기록 정리 맥락을 읽고 기록 정리에서 두 쪽 뜻을 뒤집는다(C5).
- 바꿀 파일: 신규 `crates/core/src/operation.rs`(`OpMarker { kind, branch: Option<String>, head_before: String, base: Option<String>, oids: Vec<String>, started_at }`, `verify_state(git, repo, marker) -> bool`), `conflict.rs`(`Context::Operation(OpMarker)`, `other_operation`이 앱 표식이 맞는 작업은 제외, `conflict_paths(entries, flip)`), `query.rs`(`conflict_state`·`conflict_file`이 맥락별 이름표), `view/conflict.rs`(`ConflictContext::{CherryPick, Revert, Rebase}`, `ConflictSide`는 그대로), `pending.rs`(끝나지 않은 작업이 앱 표식 작업을 "다른 도구"로 알리지 않게), 생성물, architecture.md.
- 내용: `verify_state`: `CherryPick`은 `CHERRY_PICK_HEAD`가 `oids[0]`, `Revert`는 `REVERT_HEAD`가 `oids[0]`, `Rebase`는 `rebase-merge/`가 있고 `head-name` = `refs/heads/<branch>`·`orig-head` = `head_before`·`onto` = `base`. 맞지 않으면 맥락 없음(다른 도구로 알림). 표식이 있는데 git 상태가 없으면 표식을 지운다. 이름표: `CherryPick`은 내 쪽 = 지금 위치(브랜치 이름), 합칠 쪽 = `CHERRY_PICK_HEAD` 버전(이름·사람·날짜), `Revert`는 내 쪽 = 지금 위치, 합칠 쪽 = `REVERT_HEAD` 버전, `Rebase`는 내 쪽 = `REBASE_HEAD` 버전, 합칠 쪽 = 지금 HEAD(라벨은 표식의 브랜치 이름). `Rebase`면 `conflict_paths`가 stage 2·3을 바꿔 담는다(그러면 `load_file`·`merge_segments`·`hunks`·조립이 그대로 "내 쪽 = 화면의 내 쪽"이다). `ConflictFileView.ours_side`·`theirs_side`도 같은 쪽에서 온다.
- 테스트(실제 git): E4·E6의 저장소로 각 맥락의 `conflict_state`(맥락·이름표·파일 수)와 `conflict_file`(구간의 `ours`·`theirs` 줄이 화면 뜻과 맞음 — 기록 정리에서는 `ours`가 옮기는 버전의 줄), 표식 없는 `CHERRY_PICK_HEAD`는 `other_operation`, 표식은 있는데 `orig-head`가 다르면 맥락 없음, 서명 틀림, 합치기·unpark 기존 테스트 그대로. 이름 바꿈·삭제·이진 충돌이 `Whole`로 뒤집힌 쪽에서도 맞음.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(블롭만 읽음).
- 보안: 표식 서명(E8)과 git 상태 일치가 둘 다 맞아야 앱이 다룬다. 새 IPC 명령은 없다(기존 두 조회 확장).

E10. core: `amend` (`core: add amend action`)
- 목표: 지금 위치 버전의 메시지와 이번 버전에 넣은 파일을 고쳐 다시 만든다.
- 바꿀 파일: `actions/amend.rs`, `actions/mod.rs`, `view/input.rs`(`Amend(AmendInput { expect_head, message: Option<String> })`, `action_id`, `journal_copy`는 그대로), `action.rs`(`Outcome::Amended { short_oid, summary }`), `view/mod.rs`(`ActionResult`), 생성물, `src/lib/api.ts`는 바인딩이 바뀜, architecture.md 액션 목록.
- 내용: `amend`(`amend`/Caution/`RestoreRef`, `moves_head` 기본). 순서: `require_branch` → `refuse_during_operation` → `expect_head` = 지금 위치 → `refuse_published(HEAD)` → 충돌 항목 없음 → 메시지가 없고 넣은 파일도 없으면(`commit_messages`로 비교) `NoChange`. 메시지는 1MB 이하. 실행기가 HEAD를 `amend` 백업 ref로 남긴 뒤 `GitOp::Amend`. 되돌리기는 기존 `undo_commit`(백업 뒤 `reset --soft`)로 이전 버전이 돌아오고 amend에 넣은 파일이 다시 "넣은 파일"로 남는다(버전 저장 되돌리기와 같다).
- 테스트(실제 git, 실행기 거침): 메시지만 고침(새 해시, 부모·작성자·트리 그대로), 넣은 파일 포함, 둘 다, 한글, 루트 버전, 합치기 버전(부모 둘 유지), 올린 버전은 `PublishedHistory`이고 아무것도 안 바뀜, `expect_head`가 다르면 거절, 바뀐 것 없음 `NoChange`, 합치는 중 `OperationInProgress`, hook 거부 후 상태, 되돌리기·다시 하기와 백업 ref 존재, 작업 내역 줄.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 `amend`(위험도 `caution`이 용어표와 같음).
- Windows/Ubuntu: 서명 설정(gpg)이 있으면 오래 걸려 진행 알림·강제 멈춤이 쓰인다(`run_local_watched`).
- 보안: 해시는 `expect_head`로 다시 확인, 메시지는 stdin, 입력 상한 1MB.

E11. core: `revert`·`cherry-pick` 시작 (`core: add revert and cherry-pick actions`)
- 목표: 고른 버전을 되돌리는 새 버전·다른 브랜치의 버전 복사를 시작한다. 충돌이면 표식을 남기고 멈춘다.
- 바꿀 파일: `actions/pick.rs`(두 액션), `actions/mod.rs`, `view/input.rs`(`Revert(OidInput)`·`CherryPick(OidInput)`, `OidInput { oid }`), `action.rs`(`Outcome::Reverted`·`Picked`·`OperationConflicts { kind, paths }`, 충돌 결과는 `journal_mode = Skip`), `view/mod.rs`, 생성물, architecture.md.
- 내용: `revert`(`revert`/Normal/`RestoreTree`), `cherry-pick`(`cherry-pick`/Normal/`RestoreTree`). 공통: `require_branch` → `refuse_during_operation` → `refuse_staged`(넣은 파일이 있으면 합친 버전에 섞이므로 `StagedChanges`). `revert`: `verified_commit(HeadAncestor)`, 합치기 버전이면 `-m 1`. `cherry-pick`: `verified_commit(BranchReachable)`, 합치기 버전·지금 위치의 조상은 `InvalidInput`. 시작 전에 표식(`operation.json`: 종류·`oids[0]`·지금 브랜치·`head_before`)을 쓴다. `RevertStart`·`CherryPickStart`가 성공하면 표식을 지우고 `Reverted`·`Picked`. 실패하면 상태를 본다: 충돌 항목이 있거나 `*_HEAD`가 남았으면 표식을 두고 `OperationConflicts { paths }`(충돌 0곳, hook 거부도 같음), 상태가 없으면 표식을 지우고 오류(`Empty`는 `NoChange`, `WouldOverwrite`는 그대로). 되돌리기는 기존 `undo_tree`: 안 올린 새 버전은 `reset --keep`으로, 그사이 올라갔으면 다시 revert(기존 `revert_pushed`).
- 테스트(실제 git, 실행기 거침): `revert`: 안 올린·올린 버전, 합치기 버전, 루트 버전, 이미 취소된 변경 `NoChange`, 넣은 파일 `StagedChanges`, 고친 파일과 겹치면 `WouldOverwrite`이고 작업 폴더 그대로, 지금 위치가 아닌 해시·조상이 아닌 해시 거절, 충돌 → 표식 + `OperationConflicts`, 되돌리기(Ctrl+Z)·다시 하기, 반복 2의 올린 뒤 Ctrl+Z 테스트가 그대로 통과. `cherry-pick`: 다른 브랜치의 버전 복사·새 해시·메시지 그대로, 합치기 버전 거절, 이미 조상 거절, 브랜치 끝에서 안 닿는 해시(백업 ref만 가리킴) 거절, 비어 버림 `NoChange`, 충돌 → 표식, Ctrl+Z.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션(둘 다 `normal`).
- Windows/Ubuntu: Windows에서 편집기가 파일을 잡으면 실패할 수 있다(`WouldOverwrite`/`GitFailed`로 알림, 표식은 정리). 줄바꿈은 git 설정(autocrlf)을 따른다.
- 보안: 해시는 core가 다시 확인하고 `Oid`로만 닿는다. 표식은 서명(E8).

E12. core: 복사·되돌리기 충돌 마치기·그만두기 (`core: finish and abort cherry-pick and revert conflicts`)
- 목표: 기존 `conflict-finish`·`conflict-abort`가 새 맥락 중 cherry-pick·revert를 처리한다.
- 바꿀 파일: `actions/conflict.rs`(맥락 분기, `refuse_other_operation`이 앱 표식 작업은 통과), `action.rs`·`view/mod.rs`(`Outcome::ConflictFinished`를 그대로, 되돌리기 종류 `RestoreTree`), 생성물, architecture.md.
- 내용: `Context::Operation(CherryPick|Revert)`면 `write_resolutions`(기존, 남은 충돌 경로를 한 번씩 덮는지 확인)로 쓴 뒤 `OperationContinue { kind }`로 새 버전을 만들고 표식을 지운다. hook이 막아도 해결한 것은 남아 다시 마치면 이어진다(합치기와 같음). 그만두기는 충돌 파일을 `back_up_conflicts`로 백업 ref에 남긴 뒤 `OperationAbort { kind }`와 표식 지우기. 합칠 쪽·내 쪽 뜻은 뒤집지 않는다. `Continue` 전에 `CHERRY_PICK_HEAD`/`REVERT_HEAD`가 표식의 해시인지 한 번 더 본다(`verify_state`).
- 테스트(실제 git): 충돌 파일 구간 3개를 내 쪽·합칠 쪽·둘 다·직접으로 마쳐 새 버전의 내용·메시지·`git status` 깨끗, 이진·삭제 충돌 `Whole`, 그만두기 뒤 시작 전 상태와 백업 ref, 다 고르지 않으면 `InvalidInput`, 표식이 없으면 거절(`OperationInProgress`), 앱을 껐다 켠 것처럼 새 `Core`로 이어가기, Ctrl+Z가 마친 새 버전을 되돌림(`RestoreTree`).
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: `checkout-index` 실패(편집기가 잡은 파일) 뒤 다시 마치면 이어진다(기존 합치기와 같음).
- 보안: `--continue`·`--abort`는 표식 서명과 상태 일치(E9)가 모두 맞을 때만 부른다.

E13. core: `reset` (`core: add reset action`)
- 목표: 지금 브랜치를 고른 옛 버전으로 옮긴다(안 올린 버전만 빠짐).
- 바꿀 파일: `actions/reset.rs`, `actions/mod.rs`, `view/input.rs`(`Reset(OidInput)`), `action.rs`(`Outcome::Reset { dropped }`), `view/mod.rs`, 생성물, architecture.md.
- 내용: `reset`(`reset`/Danger/`RestoreTree`). 순서: `require_branch` → `refuse_during_operation` → `verified_commit(HeadAncestor)`(지금 위치와 같으면 `InvalidInput`) → `refuse_published(HEAD, base = oid)` → 실행기가 HEAD를 `reset` 백업 ref로 남김 → `GitOp::ResetKeep { oid }`. 고친 파일이 겹치면 git이 아무것도 안 바꾸고 `WouldOverwrite`. `restore`(작업 내역)와 같은 `ResetKeep`이지만 후보 버전의 출처가 다르다(여기는 노선도 위 조상). 되돌리기는 기존 `undo_tree`가 앞으로 옮긴다(`moving_back` 거짓).
- 테스트: 안 올린 버전 2개 빠짐·고친 파일 그대로·백업 ref에 옛 끝, 올린 버전이 빠지면 `PublishedHistory`(올린 버전 위 안 올린 버전만 빠지는 경우는 허용), 조상이 아닌 해시·지금 위치 거절, 겹치는 고친 파일 `WouldOverwrite`, 되돌리기·다시 하기, 합치기 버전이 빠지는 범위, 원격이 없는 저장소.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 `reset`(`danger`).
- Windows/Ubuntu: Windows에서 파일이 열려 있으면 `reset --keep`이 실패할 수 있다(오류로 알림, 상태 그대로).
- 보안: 해시 재확인, 빠지는 버전 모두 안 올림 확인.

E14. core: 기록 정리 시작과 계획 조회 (`core: add history rewrite action and plan query`)
- 목표: 카드 지도가 쓸 조회와 `rebase-i` 액션(충돌 없는 경로).
- 바꿀 파일: `actions/rewrite.rs`(+테스트), `actions/mod.rs`, `view/input.rs`(`Rewrite(RewriteInput { oldest, steps })`, `RewriteStepInput { oid, kind: RewriteKindInput(pick|squash|drop), message: Option<String> }`, `journal_copy`는 메시지를 빈 글자로), `view/history.rs`(신규: `RewritePlanView { cards: Vec<RewriteCardView { oid, short_oid, summary, message, author, time }>, blocked: Option<RewriteBlocked> }`, `RewriteBlocked`: `Detached`·`NotOnBranchLine`·`HasMerge`·`IncludesFirstVersion`·`Published`·`TooMany`·`HasChanges`·`OtherOperation`), `query.rs`(`rewrite_plan(repo, oids)`), `src-tauri`(`query_rewrite_plan` 네 곳), `src/lib/api.ts`(`rewritePlan`), `action.rs`(`Outcome::Rewritten { kept, dropped, joined }`), 생성물, architecture.md(명령·입력 상한: 기록 정리 50개, 메시지 16KB·전체 24KB).
- 내용: ① 공통 계산 `guard::rewrite_block(git, repo, oids) -> Result<Block|Ok(chain)>`: 지금 브랜치의 첫 부모 선을 지금 위치부터 따라가 `oids`가 그 선 위의 **연속 구간**인지, 구간과 그 아래 기준(구간 가장 오래된 버전의 부모)까지 합치기 버전이 없는지, 첫 버전(부모 없음)이 아닌지, 50개 이하인지, `refuse_published`(구간 맨 아래의 부모 ~ 지금 위치), 고친 파일 없음(`refuse_dirty`), 다른 작업 없음을 차례로 본다. 조회는 막힌 까닭을 `blocked`로 알리고 액션은 같은 검사를 오류로 낸다(조회 결과를 믿지 않고 실행 때 다시 계산). ② `rewrite_plan`은 `commit_messages` 한 번으로 카드의 전체 메시지를 채운다(구간 밖 해시는 `NotOnBranchLine`). ③ `rebase-i`(`rebase-i`/Danger/`RestoreTree`, `moves_head`): 입력 검사(첫 단계가 묶기가 아님, 묶기는 빼기 뒤가 아님, 메시지는 묶음의 맨 아래 카드에만, `steps`의 해시 집합 = 구간과 정확히 같음, 한도) → 바뀐 것이 없으면(순서·메시지 그대로) `NoChange` → 구간 위(새 쪽)의 버전은 `Pick`으로 이어 붙임 → 표식(`operation.json`: `Rebase`, 브랜치, `head_before`, 기준 해시 `base`, `oids` 전체) 쓰기 → 실행기가 HEAD를 `rebase-i` 백업 ref로 남김(먼저) → `GitOp::RewriteHistory`. 성공이면 표식을 지우고 `Rewritten`. 멈춤(`rebase-merge/` 있음)이면 표식을 두고 `OperationConflicts { Rebase, paths }`(Skip). 시작하지 못한 오류면 표식을 지우고 오류. ④ 되돌리기는 `undo_tree`(`reset --keep`으로 원래 끝, 올라가지 않았으므로 revert 경로 아님).
- 테스트(실제 git, 실행기 거침): 순서 바꾸기·묶기(그룹 메시지)·메시지 고치기·빼기와 조합, 구간 위에 안 고른 버전이 더 있어도 이어짐, 결과 기록의 해시·메시지·내용·작성자 시각 보존, 백업 ref에 옛 끝, Ctrl+Z가 원래 기록과 같은 해시를 되돌림(백업 ref 경유), 고른 구간이 연속이 아님·다른 브랜치 버전 섞임·합치기 버전·첫 버전·올린 버전·고친 파일 각각 거절과 `blocked` 값, 50개 초과, 메시지 한도, 바뀐 것 없음 `NoChange`, 겹쳐서 멈춤 → 표식과 `OperationConflicts`, 입력의 `steps`가 구간과 다르면 `InvalidInput`.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신, `action-meta.json`에 `rebase-i`(`danger`).
- Windows/Ubuntu: E6의 환경 변수 방식이 두 OS 모두 돌아야 한다(E6 시험). 긴 기록 정리는 진행 알림이 돈다.
- 보안: 해시 재확인(C3 ⑤), 메시지는 환경 변수 값, 입력 상한, 새 IPC 명령 `query_rewrite_plan`은 열린 저장소만(`opened_repo`)이고 읽기 전용, 결과에 파일 내용 없음(메시지·이름·시각만).

E15. core: 기록 정리 충돌 마치기·그만두기 (`core: finish and abort rewrite conflicts`)
- 목표: 기록 정리가 충돌로 멈췄을 때 충돌 화면이 마치고, 다음 충돌로 이어지고, 그만둘 수 있게 한다.
- 바꿀 파일: `actions/conflict.rs`(`Operation(Rebase)` 분기, `Action::journal_before`로 되돌리기 기준을 표식의 `head_before`로), `action.rs`·`view/mod.rs`(`Outcome::ConflictContinued`), `operation.rs`(`verify_todo`), 생성물, architecture.md.
- 내용: 마치기: `write_resolutions`(뒤집힌 쪽 기준, `ResolveSide`의 `ours`는 화면 값의 반대) → `verify_state` + `verify_todo`(`git-rebase-todo`의 남은 줄이 모두 `pick|fixup|drop <표식 해시의 접두사>`이거나 상수 `exec` 줄(번호 ≤ 메시지 수)·빈 줄·`#` 줄, 아니면 거절 `OperationInProgress`) → `OperationContinue { Rebase }`. 비어 버린 커밋으로 멈추면("now empty") `rebase --skip`로 그 커밋을 빼고 이어간다(결과 글에 "비어서 빠짐" 수를 남김). 끝나면 표식 지우고 `ConflictFinished`, 다시 충돌로 멈추면 표식을 두고 `ConflictContinued`(화면이 다음 충돌을 연다). journal은 마치는 때 한 줄: `before`는 `journal_before`가 표식의 `head_before`·브랜치로 바꾼다(캡처한 값은 정리 중 떨어진 HEAD라 쓸 수 없음), `after`는 끝난 뒤 브랜치 끝. 그만두기: 충돌 파일과 고친 파일(추적 중인 모든 변경, 1000개까지)을 `back_up_conflicts` 확장으로 한 번에 백업 ref에 남긴 뒤 `OperationAbort { Rebase }`(git이 원래 끝으로 `reset --hard`하므로 충돌 파일뿐 아니라 그 사이 바꾼 파일도 백업) → 표식 지우기.
- 테스트(실제 git): 충돌 둘이 이어지는 정리(두 번 마치기), 마치기 사이에 앱을 껐다 켜기(새 `Core`), 정리 중 그 사이에 바꾼 다른 파일이 그만두기 뒤 백업 ref에 있음, 그만두기가 원래 해시로, `git-rebase-todo`에 낯선 `exec`(손으로 심음)이 있으면 마치기 거절·git은 안 부름, `orig-head`가 다르면 거절, 표식 없이 `rebase-merge/`만 있으면 거절, 마친 뒤 Ctrl+Z가 정리 전 기록으로(백업 ref 경유, `RestoreTree`), 비어 버린 커밋.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 같은 파일을 편집기가 잡은 Windows에서 `rebase --continue`가 실패하면 상태가 남아 다시 마치면 이어진다.
- 보안: 이 단계가 이번 반복의 가장 큰 보안 확인점이다. `exec` 줄 검증과 표식 서명이 없으면 `--continue`를 부르지 않는다(받은 저장소 위협, C6).

E16. core: `tag`·`tag-delete` (`core: add tag actions`)
- 목표: 가벼운 태그를 만들고 지운다(C8).
- 바꿀 파일: `actions/tag.rs`, `actions/mod.rs`, `view/input.rs`(`Tag(TagInput { oid, name })`, `TagDelete(TagDeleteInput { name, oid })`), `action.rs`(`Outcome::Tagged { name }`·`TagDeleted { name }`), `view/mod.rs`, 생성물, architecture.md.
- 내용: 둘 다 `tag`/Safe/`RestoreRef`, `moves_head` 거짓, `tracked_refs = [refs/tags/<name>]`. `tag`: 이름 `trim`·255바이트·`validate_ref_like`·`check_tag_name`·`-` 시작 금지 → 같은 이름이 있으면 `TagExists` → `verified_commit(BranchOrActivity)` → `TagCreate`. `tag-delete`: 태그가 `oid`를 가리키는지(`resolve_ref`) → `TagDelete`. 되돌리기는 기존 `RestoreRef`(HEAD 그대로면 추적 ref만) 그대로.
- 테스트: 만들기·지우기·한글 이름·이미 있음·잘못된 이름(`..`, 공백, `-x`, 255바이트 초과)·옛 위치를 가리키는 태그를 지우려 하면 거절, Ctrl+Z·다시 하기(만든 태그가 사라지고 돌아옴), 되돌릴 때 사이에 태그가 옮겨졌으면 실패, 작업 내역 줄, `tags()`로 노선도 이름표에 보임, 지금 위치가 아닌 버전에도 달림.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션(`safe`).
- Windows/Ubuntu: 태그 이름의 대소문자 구분은 파일시스템을 따른다(Windows는 `V1`·`v1`이 같은 파일이라 `TagExists`로 막음, 이름 검사에서 소문자 비교).
- 보안: 이름 검사, 해시 재확인. 주석 태그·서명 없음.

E17. core: `branch-delete`·`branch-rename` (`core: add branch delete and rename actions`)
- 목표: D-084 이행(C7).
- 바꿀 파일: `actions/branch_edit.rs`, `actions/mod.rs`, `parked.rs`(`move_ref(old, new)`, `retire_to_backup(branch)`: `refs/higit/`를 쓰는 코드는 여기에 둔다), `undo/run.rs`(`undo_branch_rename`, `RestoreRef` 되돌리기 뒤 `branch-delete`면 upstream 다시 잇기), `view/input.rs`(`BranchDelete(BranchNameInput)`, `BranchRename(BranchRenameInput { from, to })`), `action.rs`(`Outcome::BranchDeleted { name }`·`BranchRenamed { from, to }`), `view/mod.rs`, 생성물, architecture.md.
- 내용: `branch-delete`(`branch-delete`/Caution/`RestoreRef`, `moves_head` 거짓, `tracked_refs = [refs/heads/<name>]`). 순서: `validate_ref_like` → 있는 로컬 브랜치 → 지금 브랜치·기본 브랜치·다른 작업 폴더가 쓰는 브랜치(`checked_out_elsewhere`)면 `BranchProtected` → `refuse_during_operation`(앱 밖 작업 포함) → 끝 버전을 `backup_oid(branch-delete-<이름>)`(못 남기면 지우지 않음) → 맡긴 ref가 있으면 `retire_to_backup`(백업 ref `parked-<이름>`로 옮기고 지움) → `BranchDelete`. 원격 브랜치는 건드리지 않는다. Ctrl+Z: ref를 되살리고(기존 `restore_refs`) 같은 이름의 원격 추적 브랜치가 하나뿐이면 `SetUpstream`으로 다시 연결한다(맡긴 파일은 되살리지 않고 작업 내역 백업에 있다). `branch-rename`(`branch-rename`/Normal/`Inverse`, `tracked_refs = [옛, 새]`): 이름 검사·새 이름이 다른 브랜치와 대소문자까지 겹치면 `BranchExists`·`refuse_during_operation`(unpark 표식이 있어도 거절) → `BranchRename`(대소문자만 다르면 `-M`) → upstream이 있었으면 `UnsetUpstream` → 맡긴 ref가 있으면 `move_ref(옛, 새)`. 되돌리기(`undo_branch_rename`): `BranchRename { 새 → 옛 }`과 맡긴 ref 되돌리기, upstream은 되살리지 않는다(끊긴 채, 토스트에 이름과 숫자만).
- 테스트(실제 git, 실행기): 지우기(끝 버전 백업 ref·지금 브랜치 거절·기본 브랜치 거절·다른 worktree가 쓰는 브랜치 거절·합치는 중 거절·올린 브랜치(원격 브랜치는 그대로)·맡긴 파일이 백업으로 옮겨짐·`branch.<이름>.*` 설정 구역 삭제), Ctrl+Z(ref와 upstream 복원, 원격 추적이 둘이면 복원 안 함), 이름 바꾸기(지금 브랜치·다른 브랜치·맡긴 ref 이동·upstream 끊김·이미 있는 이름·대소문자만 다른 이름), 이름 바꾼 뒤 동기화가 새 이름으로 올림(로컬 bare 원격), Ctrl+Z로 이름 복원, 작업 내역에 백업 줄.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `action-meta.json`에 두 액션(`caution`·`normal`이 새 용어표와 같음).
- Windows/Ubuntu: 대소문자만 다른 브랜치 이름 바꾸기(Windows·`-M`), 맡긴 ref의 `%2F` 인코딩(D-064)을 그대로 쓴다.
- 보안: 이름 검사, 백업 먼저(실패하면 지우지 않음), 원격 변경 없음.

E18. core: `branch-create`의 `from` (`core: create a branch from a chosen version`)
- 목표: "여기서 새 브랜치"(역 메뉴)와 작업 내역 "새 브랜치로"(E31)가 같은 액션을 쓴다.
- 바꿀 파일: `view/input.rs`(`BranchCreateInput { name, #[serde(default)] from: Option<String> }`), `actions/branch_create.rs`, `undo/run.rs`(`redo_create`가 `from`으로), 생성물.
- 내용: `from`이 없으면 지금과 같다. 있고 지금 위치와 다르면 `verified_commit(BranchOrActivity)` → `refuse_dirty`(추적 중인 고친 파일이 따라가면 새 브랜치의 내용과 안 맞음) → `CreateAndSwitchAt { name, oid }`. 같으면 기존 `CreateAndSwitch`(고친 파일이 따라감). 되돌리기는 기존 `undo_create`(이전 브랜치로 돌아가 새 브랜치 지움). 이름 규칙·`BranchExists`는 그대로.
- 테스트: 옛 버전에서 만들기(작업 폴더 내용이 그 버전)·새 브랜치 끝 해시·고친 파일이 있으면 `HasChanges`(새 파일(추적 안 함)만이면 허용)·작업 내역 백업 해시에서 만들기·그 밖의 해시 거절·Ctrl+Z·다시 하기가 같은 해시에서·`from` 없는 옛 입력 JSON이 그대로 읽힘.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음.
- 보안: `from` 재확인, 이름 검사.

E19. core·app: 버전 상세 조회와 버전별 비교 (`core: add commit detail query and commit diff`)
- 목표: "이때 파일 보기"·마지막 버전 고치기가 쓸 읽기 전용 조회(C9).
- 바꿀 파일: `view/history.rs`(`CommitDetailView { oid, short_oid, message, author, time, parents, files: Vec<CommitFileView { path, orig_path, change }>, more_files }`), `view/diff.rs`(`DiffRequest { …, #[serde(default)] commit: Option<String> }`), `query.rs`(`commit_detail(repo, oid)`, `diff`가 `commit`이면 `verified_commit(BranchOrActivity)` 뒤 `DiffKind::Commit`), `src-tauri`(`query_commit_detail` 네 곳), `src/lib/{api,queries}.ts`(`commitDetail`, `useCommitDetail(repo, oid)`, `useDiff`의 키에 `commit`), 생성물, architecture.md(명령 목록, Query 목록).
- 내용: 메시지는 64KB까지 자르고 파일은 500개까지(`more_files`). 해시는 `Oid::parse`와 `verified_commit` 통과만. 비교 경로는 기존 `paths::validate`. 화면 타입에 `crates/git` 타입을 노출하지 않는다(D-019).
- 테스트: 열지 않은 저장소 `NotOpened`, 잘못된 해시·브랜치에서 안 닿는 해시 거절, 합치기·루트 버전, 한글 경로, 파일 501개 `more_files`, 큰 메시지 잘림, 비교(`commit`)가 지금 작업 폴더와 무관한 값, 보안 설정 테스트(`tests/security_config.rs`).
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음.
- 보안: 읽기 전용, 열린 저장소만, 새 IPC 명령 한 개를 `architecture.md` 보안 표에서 확인. 저장소 글자는 데이터로만.

E20. core: 작업 내역·꺼내기 신뢰 경계 (`core: harden activity list and take-out`)
- 목표: 2 점검에서 넘어온 것 중 넣기로 한 것(C12).
- 바꿀 파일: `recovery.rs`(`collect`), `actions/stash.rs`(`take-out`), `parked.rs`(`safe_paths`를 `pub(crate)`로), `crates/core/tests/recovery.rs`(없으면 새로), architecture.md "작업 내역".
- 내용: ① 서명이 맞지 않는 journal 줄은 `journal_marks`에 넣지 않아 같은 순간의 reflog 줄을 숨기지 않고 보이는 수를 50개로 제한. ② 모든 원천을 가벼운 값으로 모아 시각순으로 자른 뒤(200개) 살아남은 백업 ref에만 `kind_of`(git log)를 부르고 맡긴 항목의 파일 경로 읽기(`parked_paths`)도 자른 뒤에 한다. ③ `take-out`(임시 보관·백업 ref·맡긴 ref 모두)이 `stash`에 든 경로를 읽어 `safe_paths`(상대 경로, `.git` 아님)를 통과하지 못하면 `InvalidInput`으로 꺼내지 않는다(지금은 git의 `.git` 경로 거부에 기댐).
- 테스트: 서명 없는 줄(다른 키)이 같은 해시·시각의 reflog 줄을 가리지 못함, 서명 없는 줄 80개 중 50개만, 백업 ref 500개에서 `git log` 호출 수가 200 이하(가짜 백엔드 호출 횟수), 경로가 `../x`·`.git/config`인 stash는 꺼내지 않음.
- 완료 기준: `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음(`safe_relative`는 Windows 예약 이름·`.git` 변형 포함).
- 보안: 받은 zip의 journal·stash가 앱 동작을 숨기거나 `.git`을 덮어쓰지 못하게 한다(D-063, D-082).

### D. 화면

E21. 역 메뉴 틀 보완 (`features: station menu env, tooltips and context menu`)
- 목표: 반복 3 항목이 올라탈 틀을 끝낸다(C11).
- 바꿀 파일: `src/features/history/menu/{items.ts, StationMenu.tsx, StationMenu.module.css, StationMenu.test.tsx, info.ts, run.ts(신규), ask.ts(신규)}`, `src/features/history/RouteList.tsx`(`onContextMenu`, `onExpand` 뒤 선택), `src/screens/History/History.tsx`, `src/screens/Workspace/Workspace.tsx`(`otherOperation` 전달), `src/messages/{ko,en}.ts`, `docs/design.md`("역 메뉴"에 툴팁·오른쪽 클릭), `docs/architecture.md`("역 메뉴" 문단).
- 내용: ① `StationMenuItem.label: (m) => ReactNode`(용어는 `<Term focusable={false}>`). ② `MenuEnv`: `merging` 대신 `operation: "none" | "other"`, 더해서 `stagedFiles`·`upstream`(지금 브랜치 upstream 이름)·`defaultBranch`·`zoom`. `Workspace`가 `conflict.otherOperation`을 `History` 속성으로 잇고 `History`가 `env`를 만든다. ③ `ctx.ask(dialog: StationDialog)`와 `StationDialog` 합집합(`tag | newBranch | amend | files`)을 `ask.ts`에 두고 `History`가 상태로 쥐어 `StationDialogs`(E23~E25가 채움)를 그린다. ④ `run.ts`의 `runAction(ctx, input, { done, conflict })`: `api.executeAction` 호출 → 성공이면 `ctx.onDone(문구)`·기록·상태·브랜치·노선도·작업 내역·합치기 제안·기록 정리 계획 쿼리 무효화(선택은 유지) → `…Conflicts` 결과면 충돌 상태 쿼리를 무효화해 `Workspace`가 충돌 화면으로 바꾸게 함 → 실패는 `ApiError` 코드의 `m.errors[code]` 토스트(`PublishedHistory`·`UndoPushed`처럼 작업 내역 안내가 필요한 것은 `{ label: 작업 내역 }`). `StationMenu`의 `run` 실패 처리를 이 헬퍼로 바꾼다. ⑤ 흐린 항목: Radix `disabled` 대신 `aria-disabled="true"`와 `onSelect={(e) => e.preventDefault()}`, 항목 전체를 `Tooltip`으로 감싼다(키보드로 하이라이트하면 까닭이 뜸, `--text-muted`). ⑥ 오른쪽 클릭: 목록 `onContextMenu`에서 `preventDefault`, 눌린 행을 고르고 연다(같은 행에서 열려 있으면 그대로). ⑦ 접힌 구간 펼치기(`onActivate`의 `fold`)는 펼친 뒤 `fold.statOids[0]`(가장 새 버전)을 고른다.
- 테스트(Testing Library, 실제 Radix): 오른쪽 클릭이 행을 고르고 메뉴를 열며 기본 메뉴는 막힘, 흐린 항목이 포커스를 받고 툴팁 까닭이 읽힘·선택해도 닫히지 않고 실행 안 됨, `ctx.ask`가 상태를 바꿈, `runAction`의 성공·충돌·오류 코드별 토스트, `operation: "other"`에서 편집 항목이 흐림(E22가 항목을 더한 뒤 표로 검증), 펼친 구간의 첫 버전이 선택됨, `StationMenu.test.tsx`의 기존 Esc·포커스·바깥 누름 테스트가 그대로 통과.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 마우스 오른쪽 클릭에 웹뷰 기본 메뉴가 안 뜨는지, 툴팁이 두 웹뷰에서 뜨는지.
- Windows/Ubuntu: WebView2·WebKitGTK의 `contextmenu` 이벤트와 터치패드 두 손가락 클릭, Radix 툴팁 위치(가상 스크롤 위)를 확인(`docs/progress.md` "사람이 할 일").
- 보안: 해당 없음(저장소 글자는 React 글자로만).

E22. 역 메뉴 항목: 되돌리기·이 버전만 복사하기·이 버전으로 돌아가기 (`features: revert, cherry-pick and reset in the station menu`)
- 목표: 확인 창 없이 바로 하는 세 항목과 가드 테스트.
- 바꿀 파일: `items.ts`, `src/features/history/menu/items.test.ts`, `src/messages/{ko,en}.ts`(토스트 "되돌리는 새 버전을 만들었어요"(기존 `reverted`), "버전을 복사했어요", "N개 버전이 기록에서 빠졌어요", 흐린 까닭들), `src/lib/shortcuts.ts`는 변경 없음.
- 내용: 항목 표(보이는 조건 / 흐린 까닭, 문구는 이름만):
  - `revert` "되돌리기"(`<Term id="revert">`, 주 묶음): 지금 위치의 조상일 때 보임(그 밖은 숨김). 흐림: `operation === "other"`("다른 도구의 작업 중이에요"), `stagedFiles > 0`("이번 버전에 넣은 파일이 있어요"), 지금 브랜치 없음.
  - `cherry-pick`(더 보기): `!isAncestorOfHead`일 때 보임. 흐림: 합치기 버전("합치기 버전은 복사할 수 없어요"), `stagedFiles > 0`, `operation === "other"`.
  - `reset`(더 보기): `isAncestorOfHead && !isHead`일 때 보임. 흐림: `operation === "other"`, 지금 브랜치 없음.
  `run`은 `runAction`으로 `api.executeAction(repo, { id, input: { oid: info.oid } })`. 완료 문구 뒤 토스트의 "되돌리기"는 `onDone`이 붙인다(되돌릴 수 없는 항목이 없으므로 확인 창은 없다).
- 테스트: 표(조건별 보임·숨김·흐림과 까닭)를 `StationInfo`·`MenuEnv` 조합으로, 실행이 올바른 입력을 보냄(가짜 `api`), 충돌 결과가 충돌 쿼리를 무효화, `PublishedHistory` 오류가 작업 내역 안내가 붙은 토스트, **가드: `items`의 모든 `actionId`가 `action-meta.json`에 있고 `undo`가 `irreversible`이 아니다**(그런 항목이 생기면 확인 창 틀(`preview` 토큰, `CompareFooter` 방식)을 같이 만들도록 테스트 설명에 적음), 위험도 표시가 용어표와 같은 액션 메타를 씀.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 실제 저장소에서 세 항목과 Ctrl+Z, 합치기 버전 되돌리기.
- Windows/Ubuntu: 차이 없음.
- 보안: 화면은 `api.ts`만 부른다(D-010), 입력 해시는 core가 다시 확인(C3).

E23. 태그 대화상자와 여기서 새 브랜치 (`features: tag and new branch dialogs for a station`)
- 목표: 이름을 입력받는 두 항목.
- 바꿀 파일: 신규 `src/features/history/menu/dialogs/{StationDialogs.tsx, TagDialog.tsx, NewBranchDialog.tsx, Dialogs.module.css, TagDialog.test.tsx, NewBranchDialog.test.tsx}`, `items.ts`, `src/lib/queries.ts`(이름 검사 재사용: `useBranchNameCheck`), `src/messages/{ko,en}.ts`, `docs/design.md`("확인 대화상자" 옆에 "이름 대화상자").
- 내용: 공통 틀은 Radix `Dialog`(폭 440px, `CompareFooter`의 모양). `TagDialog`: 위에 입력칸 하나(placeholder "v1.0", Enter로 만들기, 한글 조합 중 Enter 무시), 아래에 이 역에 이미 붙은 태그 목록(`info.tags`)과 줄마다 글자 버튼 "지우기". 만들기·지우기는 각각 `runAction`이 `tag`·`tag-delete`를 부르고 토스트 "태그를 달았어요 · 되돌리기"·"태그를 지웠어요 · 되돌리기". `NewBranchDialog`: 입력칸 하나, 기존 `useBranchNameCheck`로 이름 규칙·같은 이름 안내(`m.badBranchName`·`m.sameBranchName`), Enter로 `branch-create { name, from: info.oid }`, 성공이면 노선도의 "지금 여기"가 옮겨 간다(기록 무효화). 항목: `tag`(더 보기, "태그", 숨김 없음, 흐림: `operation === "other"`), `new-branch`("여기서 새 브랜치", 주 묶음, 흐림: `!info.isHead && changedFiles > 0`("고친 파일이 있어요"), `operation === "other"`).
- 테스트: 입력·검사·Enter·Esc, 한글 조합 중 Enter 무시, 만들기가 `tag` 입력을 보냄·목록의 "지우기"가 `tag-delete`를 보냄, 오류 코드(`tagExists`·`invalidBranchName`)가 대화상자 안에 보임, 새 브랜치가 `from`을 보냄, 포커스가 입력칸으로 가고 닫으면 목록으로 돌아옴, 저장소 글자(태그 이름)는 React 글자로.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 한글 IME 태그 이름, 800px·다크.
- Windows/Ubuntu: 한글 IME 두 OS(`isComposing`·WebKitGTK `keyCode 229`), 포커스 되돌림.
- 보안: 이름은 core가 검사(C3·C8), 화면은 힌트만. `dangerouslySetInnerHTML` 없음.

E24. 마지막 버전 고치기 대화상자 (`features: amend dialog`)
- 목표: 지금 위치 역의 메시지와 넣은 파일을 고친다.
- 바꿀 파일: 신규 `src/features/history/menu/dialogs/AmendDialog.tsx`·`AmendDialog.test.tsx`, `items.ts`, `src/lib/queries.ts`(`useCommitDetail`), `src/messages`.
- 내용: `info.isHead`인 역에만 보이는 주 묶음 항목 "마지막 버전 고치기"(`<Term id="amend">`). 흐림: `operation === "other"`, 지금 브랜치 없음, `env.upstream`이 있는데 `!info.unpushed`(이미 올린 버전, "이미 올린 버전이에요"; upstream이 없으면 core가 판단하므로 흐리지 않음). 대화상자: `useCommitDetail`로 전체 메시지를 받아 한 글상자에 채운다(첫 줄이 이름, 나머지가 설명; `NameBox`와 같은 감각의 문서 제목 글꼴), 아래 `--text-muted` 한 줄 "넣은 파일 N개 함께 들어가요"(`stagedFiles`가 0이면 숨김), 주 버튼 "고치기"(Ctrl+Enter 도 같음, `commit` 단축키 `field` 범위를 재사용하지 않고 이 글상자가 직접 처리하며 한글 조합 중 무시), 글자 버튼 "닫기". 바꾼 것이 없으면 주 버튼이 꺼진다. `amend { expectHead: info.oid, message }`. 성공 토스트 "고쳤어요 · 되돌리기".
- 테스트: 메시지 채움·바뀜 감지·저장 호출과 `expectHead`, 넣은 파일 안내, 올린 버전 흐림, `publishedHistory` 오류가 대화상자 안에 보임, 한글 조합 중 Ctrl+Enter 무시, 긴 메시지(1MB 상한 안내 없이 core 오류로).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`.
- Windows/Ubuntu: 글상자 IME, `Ctrl+Enter`가 두 웹뷰에서 오는지는 기존 `commit` 확인과 같이 본다.
- 보안: 메시지는 사용자 글자(저장소에서 온 것은 `textarea` 값으로만), core가 길이·해시를 다시 확인.

E25. 이때 파일 보기 (`features: view files of a version`)
- 목표: C9의 대화상자.
- 바꿀 파일: 신규 `src/features/history/menu/dialogs/FilesDialog.tsx`·`FilesDialog.module.css`·`FilesDialog.test.tsx`, `src/features/diff/DiffView.tsx`(`request.commit` 전달), `items.ts`, `src/lib/queries.ts`, `src/messages`, `docs/design.md`.
- 내용: 주 묶음 첫째 항목 "이때 파일 보기"(항상 보임). 대화상자는 위에 이름·짧은 해시·작성자·날짜(`--text-muted`), 왼쪽 파일 목록(경로·상태 글자 `+`·`−`·`~`·이름 바꿈, 합치기 버전이면 머리에 "첫 부모와 비교" 한 줄)과 오른쪽 `DiffView`(`request = { path, origPath, untracked: false, commit: info.oid }`), 파일이 500개를 넘으면 "파일 N개 더" 숫자만. ↑↓로 파일을 옮기고 Esc로 닫는다(`fileList` 범위 단축키를 쓰지 않고 목록 안 부품 조작). 폭 960px까지, 800px에서는 목록 위 비교 아래로 접힌다.
- 테스트: 목록·상태 글자·비교 표시, 합치기 버전 머리 줄, 파일 0개(빈 버전) 안내, 500개 초과 숫자, 키보드 이동, 링크·이진·큰 파일의 기존 `DiffView` 표시 그대로, 저장소 글자(경로)는 React 글자로.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 합친 기록의 합치기 버전·루트 버전, 800px·다크.
- Windows/Ubuntu: 한글·공백 경로, 긴 경로 말줄임.
- 보안: 읽기 전용, 파일 내용은 비교 보기가 이미 하던 방식(링크 내용 안 읽음).

E26. 충돌 해결 화면의 새 맥락 (`screens: conflict screen for copy, revert and rewrite`)
- 목표: C5의 화면 쪽. 맥락마다 이름표와 단추 문구가 다르고 기록 정리는 다음 충돌로 이어진다.
- 바꿀 파일: `src/screens/Conflict/Conflict.tsx`·`Conflict.test.tsx`, `src/features/conflict/{HunkPanel.tsx, conflictStore.ts}`, `src/features/pending/PendingNotice.tsx`(앱 표식 작업은 "다른 도구" 줄에서 빠짐), `src/messages/{ko,en}.ts`, `docs/design.md`("충돌 해결 화면"에 맥락 이름표), `docs/spec.md`는 E1에서.
- 내용: `ConflictContext`가 `Merge`·`Unpark`·`CherryPick`·`Revert`·`Rebase`. 윗줄 "● 합칠 쪽 → ● 내 쪽"의 글자와 카드 이름표를 맥락별 `messages`로 고른다: 합치기·꺼내기는 지금과 같다(`<Term ours/>`·`<Term theirs/>`). 복사: 내 쪽 "내 브랜치"(Term), 합칠 쪽 "복사할 버전". 되돌리기: 내 쪽 "내 브랜치", 합칠 쪽 "되돌린 모습". 기록 정리: 내 쪽 "옮기는 버전", 합칠 쪽 "앞서 정리한 기록"(Term 없음, 호버 설명이 합치기 기준이라). 색(`--graph-2`·`--graph-1`)은 쪽을 뜻하므로 그대로. 단추: 마치기·그만두기는 맥락별 문구("복사 마치기"·"복사 그만두기", `<Term revert>` 마치기·그만두기, "정리 마치기"·"정리 그만두기"). `ActionResult::ConflictContinued`이면 토스트 없이 상태를 다시 읽어 다음 충돌을 연다(`conflictStore.open` 키가 `repo|theirs.shortOid`라 새 충돌이면 선택이 새로 시작). 마친 뒤 토스트: 복사 "복사했어요", 되돌리기 "되돌리는 새 버전을 만들었어요", 정리 "정리했어요"(모두 "되돌리기" 단추 포함). 충돌 0곳·마치기만 남은 상태(hook 거부)도 이미 있는 "모두 골랐어요" 문구로. `PendingNotice`: 앱이 시작한 작업(`state.otherOperation === false`)은 줄을 띄우지 않고 다른 도구의 작업만 줄을 띄운다(기존).
- 테스트: 맥락 다섯 개의 이름표·단추 글자(표), 기록 정리에서 뒤집힌 쪽의 카드 내용이 core 값 그대로 쓰임(화면이 다시 뒤집지 않음), `ConflictContinued` 뒤 새 충돌로 선택이 비워짐, 마치기가 `locale`·선택을 보냄, 그만두기 토스트, 기존 합치기·꺼내기 테스트가 그대로 통과, 용어(`copy-rules`·`terms-in-messages`).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 복사·되돌리기·정리가 실제로 겹치는 저장소에서 끝까지(800px·다크).
- Windows/Ubuntu: 기존 충돌 화면 확인(1·2·3, Ctrl+↑↓, IME 직접 고치기)을 새 맥락에서도 한 번씩.
- 보안: 표시 문자 없음(기존 조회가 걸러 줌), 이름표의 사람·메시지는 React 글자로.

E27. 여러 역 선택 (`features: select a range of versions on the head line`)
- 목표: 기록 정리를 열 선택 구간(C10).
- 바꿀 파일: `src/lib/listKeys.ts`·`listKeys.test.ts`(Shift+↑↓가 `extendNext`·`extendPrev`, Shift+F10 열기는 그대로), 신규 `src/features/history/range.ts`·`range.test.ts`, `src/state/historyStore.ts`(`anchorId`, `rangeIds`), `src/features/history/RouteList.tsx`(Shift+클릭·범위 표시·`aria-selected`), `src/screens/History/History.tsx`(머리의 "N개 선택 · 기록 정리" 단추), `src/features/history/menu/items.ts`("여기부터 기록 정리"), `src/messages`, `docs/design.md`("선택 구간" 행 모양), `src/state/repoStore.ts`(`WorkspaceView`에 `"rewrite"`와 `rewriteOids`).
- 내용: `range.ts`(순수): `headLine(nodes)`(지금 위치에서 첫 부모를 따라간 노드 id 순서), `extend(nodes, headLine, anchor, active, dir)`(범위 모드에서는 지금 브랜치 선 위의 이웃으로 건너뜀), `eligible(range, zoom)`(모두 지금 브랜치 선 위·합치기 아님·연속·2개 이상 50개 이하·모든 버전 보기). 일반 클릭·화살표는 범위를 지우고 하나만 고른다. 범위 행은 `--surface-hover` 바탕과 왼쪽 2px 막대, 활성 행은 지금처럼. 접힌 구간이 끼면 범위 모드가 켜지지 않는다. Esc는 열린 메뉴 → 범위 지우기 → 홈 순서(기존 `defaultPrevented` 규칙). 머리의 단추는 `eligible`일 때만 보이고 누르면 `rewriteOids = rangeIds`(새 버전 먼저)로 `setView("rewrite")`. 메뉴 "여기부터 기록 정리"(더 보기): `onHeadLine && !isMerge && zoom === "all"`이고 그 역부터 지금 위치까지 개수가 50 이하일 때 보이고, 흐림은 `zoom !== "all"`("모든 버전 보기에서 할 수 있어요")·`operation === "other"`·`changedFiles > 0`. 누르면 그 역부터 지금 위치까지를 범위로 열고 같은 화면으로 간다.
- 테스트: `extend` 표(곁가지가 끼어 있는 목록에서 건너뜀·끝 도달·한 칸 줄이기), `eligible` 표(합치기 포함·곁가지 선택·접힘·50개 경계·1개), `listKey` Shift 조합(Shift+↑↓는 확장, Shift+F10은 열기, 한글 조합 중 무시), Shift+클릭, `aria-selected`가 범위 행에, Esc 순서, 단추가 `rewriteOids`를 채움.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 합친 기록에서 Shift+↑↓, 800px.
- Windows/Ubuntu: Shift+클릭 선택이 텍스트 선택으로 번지지 않게 `user-select: none`(두 웹뷰), 터치패드.
- 보안: 해당 없음(core가 구간을 다시 계산한다, C3 ⑤).

E28. 기록 정리 카드 모델 (`features: rewrite card model`)
- 목표: 카드 지도의 계산을 DOM 없이 만들고 시험한다.
- 바꿀 파일: 신규 `src/features/rewrite/{model.ts, model.test.ts, keys.ts, keys.test.ts}`.
- 내용: `Card { oid, shortOid, summary, author, time, message, newMessage: string | null, state: "keep" | "drop", joinBelow: boolean }`(화면 순서는 새 버전이 위). 순수 함수: `fromPlan(plan)`, `move(cards, index, delta)`(경계에서 멈춤), `joinWithBelow(cards, index)`(아래 카드가 빠짐 상태거나 없으면 불가), `split(cards, index)`, `toggleDrop(cards, index)`(빼면 묶임을 풂), `setMessage(cards, index, text)`(묶음의 맨 아래 카드에만), `groups(cards)`(맨 아래 카드가 묶음의 머리), `summary(cards)`(정리 후 개수·빠지는 개수·합쳐지는 개수), `changed(cards, original)`, `toInput(cards, oldest)`(맨 아래 카드부터 `pick|squash|drop`과 메시지, 바뀐 메시지만), `status(card, original)`(옮김·묶임·빠짐). `keys.ts`: 목록 안 조작 판정(`Alt+ArrowUp/Down` 카드 이동, `Delete`/`Backspace` 빼기 토글, `Enter` 메시지 고치기 열기, ↑↓ 선택 이동, 한글 조합 중 무시)을 `src/lib`의 `listKey` 방식으로(표 밖 부품 조작, D-056).
- 규칙: 묶음(머리 카드와 그 위로 `joinBelow`가 이어진 카드들)은 한 덩어리로 움직인다("위로"·"아래로"·끌기는 묶음 전체를 옮김). 묶인 카드 하나만 옮기려면 먼저 `split`("묶기 풀기")한다. 빠진 카드는 묶을 수 없고, 묶인 카드를 빼면 그 카드만 묶음에서 빠진다. 머리 카드를 빼면 그 위 카드들이 아래 카드로 이어 붙지 않고 묶음이 풀린다(위 카드가 새 머리).
- 테스트(표): 이동·경계, 묶기·풀기 규칙(위 규칙대로 묶음 이동·빠진 카드 아래로 묶기 불가·머리를 뺄 때), 메시지는 머리에만, `toInput` 결과가 E14 입력 형식과 일치(타입)·원래 상태면 "바뀐 것 없음", 50장, `status`.
- 완료 기준: `pnpm test`, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(순수 함수).
- 보안: 해당 없음.

E29. 기록 정리 화면(카드 지도) (`screens: rewrite card map`)
- 목표: C10의 화면. 순서 바꾸기(끌기·키)·묶기·빼기·메시지 고치기와 "정리하기".
- 바꿀 파일: 신규 `src/screens/Rewrite/{Rewrite.tsx, Rewrite.module.css, Rewrite.test.tsx}`, `src/features/rewrite/{CardMap.tsx, Card.tsx, CardMap.module.css, useDrag.ts, useRewritePlan.ts}`, `src/screens/Workspace/Workspace.tsx`(`view === "rewrite"`), `src/screens/Workspace/TopBar.tsx`(합치는 중 외에는 그대로), `src/lib/queries.ts`(`useRewritePlan`), `src/lib/useRepoWatch.ts`(`.git` 변경에서 `rewritePlan` 무효화), `src/messages`, `docs/design.md`("기록 정리 카드 지도" 부품 규칙), `docs/spec.md`(E1에서 이미).
- 내용(그림 없음, 노선도와 같은 느낌): 윗줄 왼쪽 `<Term id="rebase-i">`(제목)과 "← 버전 기록"(글자 버튼, Esc도), 오른쪽에 글자 버튼 "원래대로"와 주 버튼 "정리하기"(바뀐 것이 없거나 막혔으면 꺼짐). 본문은 한 열의 카드 목록(폭 560px까지, 가운데 정렬 아님·왼쪽 정렬), 왼쪽에 얇은 선(`--border-strong`)이 카드를 꿰고 카드는 `--surface` 채움·`--border` 테두리·행 높이 72px 이상. 카드: 짧은 해시(코드 글꼴 12px `--text-muted`) · 이름(첫 줄, 말줄임, `<bdi>`) · 작성자·날짜(`--text-muted` 12px), 오른쪽 위에 상태 글자("옮김"·"묶임"·"빠짐", `--text-muted`), 아래 줄에 글자 버튼 "위로"·"아래로"·"아래 카드와 묶기"(묶인 카드에서는 "묶기 풀기")·"빼기"(빠진 카드에서는 "넣기")·"메시지 고치기". 빠진 카드는 이름에 취소선과 `--text-muted`, 묶인 카드는 아래 카드와 간격 없이 붙어 한 덩어리로 보이고 머리에 "N개 묶음". 메시지 고치기는 카드 안 글상자(첫 줄 이름·나머지 설명, Esc로 닫기, 한글 조합 중 무시). 선택·포커스는 `--accent` 막대 2px. 목록은 `role="listbox"`·카드 `role="option"`(`aria-grabbed` 대신 "N번째" 읽기 이름에 상태). 끌기: 포인터 이벤트로 카드를 잡고(`setPointerCapture`, 손잡이는 카드 왼쪽 점) 위아래로 끌면 자리 표시선이 보이고 놓으면 순서가 바뀐다(끌기 중 스크롤은 가장자리에서 저절로, 키보드 `Alt+↑↓`가 같은 동작, 끌기는 묶기·빼기를 하지 않음). 아래 줄 `--surface-sunken` 바탕: "정리 후 N개 · 빠지는 버전 M개 · 합쳐지는 버전 K개". 막힌 까닭(`blocked`)이 있으면 위에 오류 줄(`ErrorNotice` 꼴, "이미 올린 버전이 있어요" 등, 올린 것은 "되돌리기를 써 보세요"와 버전 기록으로 돌아가는 단추), 카드는 읽기만. "정리하기": `runAction`으로 `rebase-i` → 성공이면 버전 기록 화면으로 돌아가고 토스트 "정리했어요 · 되돌리기", 충돌 결과면 충돌 화면(E26), 실패는 `errors`. 정리하는 동안 모든 단추가 꺼지고 진행 표시는 기존 `features/progress`가 알린다.
- 테스트: 계획 조회 → 카드, `blocked`별 오류 줄과 읽기 전용, 이동·묶기·빼기·메시지가 모델과 같은 결과, `정리하기`가 `toInput` 결과를 보냄, 바뀐 것 없으면 꺼짐, 키(Alt+↑↓·Delete·Enter)와 한글 조합 중 무시, 끌기(포인터 이벤트 합성으로 순서 바뀜, 취소 시 원래), 50장에서 DOM·스크롤, 충돌 결과 후 `Workspace`가 충돌 화면으로, Esc가 편집 중이면 편집만 닫고 아니면 버전 기록으로, 저장소 글자 `bdi`.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 실제 저장소(안 올린 버전 6~10개)에서 순서 바꾸기·묶기·메시지 고치기·빼기·정리하기와 Ctrl+Z, 충돌이 나는 순서 바꾸기, 800px·다크, 끌기가 두 웹뷰에서 되는지.
- Windows/Ubuntu: 포인터 캡처와 끌기 중 텍스트 선택(`user-select: none`·`touch-action: none`), WebKitGTK의 끌기 이벤트 순서, 고DPI Windows에서 자리 표시선 위치를 확인(`docs/progress.md` "사람이 할 일").
- 보안: 해당 없음(화면은 core가 준 카드만 다룬다. 입력은 E14가 다시 확인).

E30. 브랜치 목록: 이름 바꾸기·지우기 (`features: rename and delete branches in the list`)
- 목표: C7의 화면.
- 바꿀 파일: `src/features/branches/{BranchList.tsx, BranchList.module.css, BranchRowMenu.tsx(신규), BranchList.test.tsx}`, `src/features/branches/model.ts`(행별 지울 수 있는지·까닭), `src/messages`, `docs/design.md`("브랜치 목록 팝오버"에 줄 오른쪽 `…`).
- 내용: 로컬 행(원격·새로 만들기·맡긴 파일만 남음 줄 제외) 오른쪽에 호버·포커스·활성일 때 보이는 작은 `…` 단추(행 `<button>` 밖의 형제, Tab으로 닿음, Radix `DropdownMenu`). 항목 "이름 바꾸기"(`<Term branch-rename>`), "지우기"(`<Term branch-delete>`). 지우기 흐림: 지금 브랜치·기본 브랜치("지금 브랜치와 기본 브랜치는 지울 수 없어요", Tooltip), 합치는 중. 이름 바꾸기: 그 행의 이름이 입력칸으로 바뀌고(Enter로 확정, Esc로 취소, 한글 조합 중 Enter 무시) 기존 `useBranchNameCheck`로 안내. 확정하면 `branch-rename`, 성공이면 토스트 "이름을 바꿨어요 · 되돌리기"(upstream이 끊기므로 그 줄의 ↑↓ 숫자가 사라진다. 설명 문장은 목록에 두지 않는다, D-049). 지우기는 확인 창 없이 `branch-delete`, 토스트 "브랜치를 지웠어요 · 되돌리기"(맡긴 파일이 있었으면 "작업 내역" 단추). 목록과 상단 바 브랜치 이름은 무효화로 갱신.
- 테스트: 행별 `…` 보임 규칙, 흐린 항목과 툴팁, 이름 바꾸기 입력(검사·Enter·Esc·IME)·입력이 `branch-rename { from, to }`, 지우기가 `branch-delete { name }`, 오류 코드 토스트(`branchProtected`·`branchExists`), 열린 목록이 갱신됨, 키보드(Tab으로 `…`·메뉴), 기존 갈아타기·새 브랜치 테스트 그대로.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 맡긴 파일이 있는 브랜치의 이름 바꾸기·지우기·Ctrl+Z, 800px·다크.
- Windows/Ubuntu: 대소문자만 다른 이름 바꾸기(Windows), 한글 IME.
- 보안: 이름은 core가 검사, 화면은 React 글자로.

E31. 작업 내역: 백업 이름과 "새 브랜치로" (`screens: activity labels and branch from a past version`)
- 목표: 새 백업 이름을 용어로 보이고 지운 브랜치·옛 버전을 새 브랜치로 되살린다(D-085 이월).
- 바꿀 파일: `src/screens/Activity/Activity.tsx`·`Activity.test.tsx`, `src/features/history/menu/dialogs/NewBranchDialog.tsx`(재사용), `src/messages`, `docs/design.md`("작업 내역" 카드에 버튼 한 줄).
- 내용: 백업 줄의 이름(`item.label`)이 액션 id(`reset`·`amend`·`rebase-i`·`cherry-pick`·`revert`)이면 `<Term>`으로, 접두사 `restore-`·`undo-`·`discard-`·`parked-`·`conflict-files-`·`branch-delete-<이름>`이면 해당 용어(`reflog`·`reflog`·`discard`·`park`·`conflict`·`branch-delete`)와 이름으로 보여 준다(원문 코드는 호버 `title` 대신 Tooltip에). "이 시점으로 되돌리기"가 있는 `Version` 카드에 글자 버튼 "새 브랜치로"를 더해 `NewBranchDialog`(`from = item.oid`)를 연다(`branch-create`의 `from`이 작업 내역에 있는 해시를 core가 다시 확인, E18). 성공하면 기록·브랜치 쿼리를 무효화하고 토스트 "브랜치를 만들었어요 · 되돌리기".
- 테스트: 라벨별 표시, 알 수 없는 라벨은 코드 그대로, "새 브랜치로"가 `from`을 보냄·오류 안내, 서명 안 된 줄은 되살리기와 같이 "새 브랜치로"도 없음(기존 `restore: None` 규칙).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음.
- 보안: 서명 안 된 줄의 해시는 되살리기에 쓰지 않는 규칙(D-085)을 새 버튼에도 적용.

### E. 마무리

E32. 문서·점검 (`docs: wrap up iteration 3`)
- 목표: 문서를 구현에 맞추고 규칙·보안·접근성을 점검한다.
- 바꿀 파일: `docs/architecture.md`(GitBackend에 `count_unpublished`·`reachable_from_branches`·`commit_messages`·`commit_files`·`check_tag_name`, `GitOp` 목록(`Amend`·`CherryPickStart`·`RevertStart`·`OperationContinue`·`OperationAbort`·`TagCreate`·`TagDelete`·`BranchDelete`·`BranchRename`·`UnsetUpstream`·`SetUpstream`·`CreateAndSwitchAt`·`RewriteHistory`), 등록 액션 30개, 실행 규칙에 "편집기 상수 명령과 `HIGIT_*` 환경 변수", 시간 제한 표에 `Amend`·`CherryPickStart`·`RevertStart`·`RewriteHistory`가 `run_local_watched`, `UndoKind` 사용(태그 `RestoreRef`, 이름 바꾸기 `Inverse`), `UndoState.refs`가 `refs/tags/`도 받음, 표식 서명과 `operation.json`, 입력 상한(기록 정리 50개·메시지 16KB·24KB, 태그 이름 255바이트), 명령 목록 `query_commit_detail`·`query_rewrite_plan`, Query 목록, "끝나지 않은 작업"(앱이 시작한 작업과 다른 도구의 작업), "버전 기록 그리기 계층"의 역 메뉴 문단(항목·`ctx.ask`·`runAction`·가드), 보안 표에 "받은 저장소의 `rebase-merge/` 할 일 목록(`exec`)" 줄), `docs/spec.md`(화면 구성 행 최종), `docs/design.md`(기록 정리 카드 지도·이름 대화상자·이때 파일 보기·충돌 맥락·브랜치 목록 `…`·작업 내역 버튼), `docs/roadmap.md`(반복 3 완료 표시, 미룬 것 이월), `docs/progress.md`(반복 3 완료·사람이 할 일), `.claude/agents/code-analyst.md`(점검 줄 추가: 새 액션의 해시를 core가 다시 확인하는지(`guard`), 올린 버전 변경 거절, `HIGIT_*` 환경 변수 외에 셸 명령 없음과 `exec` 줄이 상수인지, `--continue`·`--abort`가 표식 서명·`verify_state`·`verify_todo` 뒤에만 불리는지, `refs/higit` 쓰기가 `undo`·`parked`·`actions/switch`·`actions/stash`·`actions/conflict`·`recover_parked`에만, 역 메뉴 항목의 `actionId`가 `Irreversible`이 아닌지, `rg "invoke" src --glob '!src/lib/**'`, 새 명령 둘이 네 곳에, 색 값 `tokens.css` 밖 없음).
- code-analyst 점검: 계층 규칙(화면에서 `invoke`·git 없음), 용어·문구 규칙, 위험도 = 용어표 위험도(새 액션 9개), `Command::new`가 runner·platform에만, `rg '"--end-of-options"' src-tauri/crates/git/src`가 `args.rs`에만, `unsafe`·`unwrap` 없음, `cargo deny check`(새 의존성 없음), `dangerouslySetInnerHTML` 없음, 단축키가 `shortcuts.ts` 밖에 없음(목록 안 키는 `src/lib`의 판정 함수), 새 오류 코드 문구 완전성, 접근성(메뉴·대화상자 포커스 되돌림, 카드 지도 키보드·읽기 이름, 흐린 항목 툴팁).
- 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full` 통과, "완료 기준(반복 전체)"을 두 OS에서 손으로 확인(못 하는 것은 `docs/progress.md` "사람이 할 일": Windows `sh`·환경 변수 방식의 기록 정리, 오른쪽 클릭, 끌기, 한글 IME, 파일이 열려 있을 때 `reset --keep`).

## 용어 변경
- spec.md 용어표와 `src/glossary`(`ko.ts`·`en.ts`, `TERM_IDS`는 표 순서)에 E1에서 추가한다.
  - 추가(`branch` 다음): `| branch-delete | 브랜치 지우기 | branch -D · git branch -D | 브랜치 이름표를 지우기. 버전은 작업 내역에 남아요. | caution | 드묾 |`, `| branch-rename | 브랜치 이름 바꾸기 | branch -m · git branch -m | 브랜치 이름을 바꾸기. 원격의 같은 이름은 그대로예요. | normal | 드묾 |`. en: "Delete branch", "Rename branch"(설명도 같은 뜻).
  - 그대로: `amend`(caution)·`revert`(normal)·`reset`(danger)·`rebase-i`(danger)·`cherry-pick`(normal)·`tag`(safe)·`branch`(safe). 새 액션의 위험도가 이 값과 같다(`registry-glossary` 테스트). `tag-delete`는 용어 `tag`를 쓴다.
  - 위험도 절 `danger` 줄: "`caution`과 같고, 원격에 올린 기록이 바뀌면 추가 경고" → "`caution`과 같다. 올린 기록이 바뀌는 일은 하지 않고 거절한다(D-097)". `force-push` 행(danger)은 그대로 두고 쓰이지 않는다.
  - `ours`·`theirs` 설명은 합치기 기준 그대로(다른 맥락은 화면 글자로).
- 용어표에 넣지 않는 화면 문구(`src/messages`): "이때 파일 보기", "여기서 새 브랜치", "여기부터 기록 정리", "N개 선택", "정리하기", "원래대로", "옮김·묶임·빠짐", "복사할 버전", "되돌린 모습", "옮기는 버전", "앞서 정리한 기록", "새 브랜치로". 문장 속 "버전"·"브랜치"·"태그"는 `t(id)`(D-029).

## 이번 반복에서 하지 않는 것
| 항목 | 가는 곳 | 까닭 |
| --- | --- | --- |
| 올린 버전의 기록 바꾸기(고치기·돌아가기·정리), 강제 올리기 | 설정 화면이 생길 때 | C1(켜는 길이 없음, D-043) |
| 원격 브랜치 지우기·이름 바꾸기 | 반복 4(GitHub)와 함께 | C7(네트워크·인증·확인 창) |
| 태그 올리기·주석 태그 | 반복 4 | C8 |
| 리베이스(브랜치를 다른 브랜치 위로 옮기기) | 후속 | 용어 `rebase`는 있으나 사용자 지시 범위 밖. 기록 정리와 같은 충돌 맥락(`Rebase`)을 쓸 수 있게 두었다 |
| 기록 정리의 합치기 버전·첫 버전 포함, `edit`·`reword`·`exec` 단어, 끌어서 묶기·빼기, 여러 선 위의 선택 | 나중 | C4·C10 |
| 여러 버전 한 번에 복사·되돌리기 | 나중 | 선택은 기록 정리용 연속 구간 |
| 다른 도구가 시작한 리베이스·cherry-pick·revert 이어받기 | 하지 않음(안내 줄만) | 받은 저장소의 `exec` 줄 위험(C5, C6) |
| 되돌릴 수 없는 액션의 확인 창 틀 | `Irreversible` 항목이 생길 때 | 쓰는 곳이 없다. 가드 테스트가 알려 줌(C11) |
| 임시 보관 이름 붙이기·지우기 | 다음 반복 후보 | 로드맵 반복 3 목록에 없음 |
| journal 줄의 저장소·줄 순번 묶기, 키를 못 쓸 때 알리기 | 반복 5 전(배포 보안 묶음) | C12(journal 형식 변경) |
| 충돌 선택 `localStorage` 평문, 취소 표시 저장소 단위, 시작 화면 요약 큐 | 반복 5 전·필요할 때 | C12 |
| `pushurl`·`pushInsteadOf`·`core.worktree` 신뢰 확인 | 반복 5 전 | 반복 2 점검 그대로 |
| 노선도 위에서 역을 직접 끌어 합치기·복사, `hitTest` 사용 | 나중 | D-017 영향의 끌어놓기는 별도 결정 |
| 줄별 기록(blame), 버전 찾기(검색) | 나중 | 요청 범위 밖 |

## 위험과 열린 질문
- **`git rebase -i`의 할 일 목록을 환경 변수로 넘기는 방식**(E6)은 Git for Windows의 `sh`·MSYS 환경 변수 변환·줄바꿈 처리에서 어긋날 수 있다. E6 시험이 Windows CI에서 실패하면 대안 순서: ① `exec` 대신 `edit`로 멈추고 앱이 배열 인자로 `commit --amend -F -`를 부른 뒤 `rebase --continue`를 반복(충돌을 만나도 표식에 단계 정보를 남기면 이어짐), ② 할 일 목록을 앱 데이터의 임시 파일에 쓰고 `GIT_SEQUENCE_EDITOR='cat "$HIGIT_TODO_FILE" >'`(경로 인용 위험을 문서화), ③ 메시지 고치기를 빼고 순서·묶기·빼기만. 메시지 고치기가 막히면 C10의 단추만 숨긴다.
- `rebase -i`는 기록을 다시 만들므로 서명(`commit.gpgsign`)이 있으면 커밋마다 서명 창이 뜰 수 있다(진행 알림·60초 뒤 강제 멈춤 D-080). `post-rewrite`·`reference-transaction` hook은 돈다(신뢰 확인 대상 D-067). 정리 중 한 번 멈춘 뒤 앱을 끄면 `rebase-merge/`가 남아 다음 실행에서 충돌 화면으로 이어지지만, 표식이 서명 검증을 통과하지 못하면(저장소 폴더를 옮김 등) 다른 도구의 작업으로 안내되고 사용자가 터미널에서 `git rebase --abort`를 해야 한다.
- 올린 버전 판정은 마지막으로 원격 확인(fetch)한 때의 원격 추적 브랜치로 한다. 원격 확인이 오래됐으면 "안 올림"이 실제로는 올라가 있을 수 있다(다른 PC에서 올린 같은 해시는 해시가 같을 때만 닿는다). 되돌릴 수 있고(백업·Ctrl+Z) 동기화가 합치므로 데이터는 잃지 않는다.
- `reset --keep`·`rebase`·`cherry-pick`·`revert`는 Windows에서 편집기가 연 파일 때문에 실패할 수 있다(상태가 남으면 충돌 화면이 아니라 오류가 보일 수 있어 core가 상태를 보고 표식을 정리한다).
- 브랜치 지우기의 되돌리기는 upstream을 같은 이름 원격 추적 브랜치가 하나뿐일 때만 다시 잇는다(휴리스틱). 맡긴 파일은 되살리지 않고 작업 내역 백업에서 꺼낸다. 이름 바꾸기는 upstream을 끊어 다음 동기화가 새 이름으로 새 원격 브랜치를 만든다(옛 원격 브랜치가 남음, 사용자가 안내를 못 볼 수 있어 토스트에 이름과 숫자만 둔다).
- 기록 정리 카드 지도는 그림이 없다. 처음 쓴 모양이 어색하면 `docs/design.md`만 고치고 데이터 모델(E28)은 그대로 둔다. 끌기는 두 OS 웹뷰의 포인터 동작을 손으로 봐야 하고, 안 되면 키보드와 "위로"·"아래로" 단추만 남긴다.
- 범위 선택은 목록을 모든 버전 보기로 제한한다. 접힌 보기에서 기록 정리를 열려면 먼저 "모든 버전"으로 돌아가야 해 한 번 더 눌러야 한다(흐린 까닭 툴팁이 안내).
- 새 액션 9개가 한꺼번에 들어와 작업 내역의 백업 줄이 늘어난다(30일 뒤 정리). `reset`·`amend`·`rebase-i`·`branch-delete`마다 한 줄이라 200개 상한 안에서 보인다.
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md`에 "(자동 결정)"으로 올린 뒤 이어 간다.
- code-analyst 검토 때 먼저 볼 곳: E15의 `exec`·표식·상태 검증(받은 저장소 위협), E6의 환경 변수 방식, E7의 `guard` 재확인이 모든 새 액션에 쓰였는지, E9의 매핑 뒤집기가 한 곳에서만 일어나는지, E21의 `aria-disabled` 항목이 키보드로 닿는지.
- 질문(사람 확인이 필요하면): ① 올린 버전의 기록 바꾸기를 이번 반복에 거절하고 강제 올리기를 설정 화면 뒤로 미루는 것(C1)에 동의하는지. ② 원격 브랜치 지우기·이름 바꾸기를 반복 4로 미루는 것(C7). ③ 기록 정리의 메시지 고치기가 환경 변수·`exec` 방식에 기대는 것(C4, 안 되면 단추를 숨김). ④ "이때 파일 보기"를 이번 반복에 넣는 것(C9). 사람 확인 없이 진행하는 지시이므로 위 추천안으로 구현하고 `docs/decision.md`에 "(자동 결정)"으로 남긴다.

## 검토 반영 (code-analyst, 구현 전)
구현은 아래를 위 단계 설명보다 우선한다. 결정 번호는 그대로 C1~C12 → D-097~D-108.

### 바뀐 결정
- **C4(D-100) 바꿈: 기록 정리의 메시지 고치기는 `exec`을 쓰지 않는다.** `exec "$HIGIT_GIT" … -m "$HIGIT_MSG_n"`은 앱을 껐다 켠 뒤 `--continue`에서 환경 변수가 없어 실패한다(git 2.34.1 실험). 메시지를 고친 그룹은 할 일 목록에서 `edit`(맨 아래 카드, 묶음이면 묶음 끝 `fixup` 뒤에 `edit`이 오도록 그룹의 첫 줄을 `edit`으로 두고 나머지는 `fixup`)로 멈추게 하고, 멈추면 앱이 배열 인자와 stdin으로 `commit --amend --no-verify --cleanup=whitespace -F -`를 부른 뒤 `--continue`한다. 고친 메시지는 서명한 표식(`operation.json`)에 담는다(전체 64KB 이하)라서 앱을 껐다 켜도 이어진다. 할 일 목록에는 해시와 `pick`·`fixup`·`edit`·`drop`만 있고 **`exec` 줄은 하나도 없어야 한다**(`verify_todo`가 `exec`·`x`·`break`·`label`·`reset`·`merge`·`update-ref` 줄이 있으면 거절). 편집기 상수 명령(`GIT_SEQUENCE_EDITOR='printf %s "$HIGIT_TODO" >'`)은 시작할 때 한 번만 쓰고 그 값에는 해시와 상수 단어만 든다(D-035의 "셸에 이름을 넣지 않는다"를 지킨다. 셸은 git이 상수 명령을 부를 때뿐이라는 예외를 D-100에 적고 D-035 영향에 한 줄). 시작과 이어가기 모두 `-c rebase.abbreviateCommands=false -c rebase.autoSquash=false`를 준다. "now empty"는 `--empty=drop`으로 예측되게 한다(2.26+).
- **C1(D-097) 보탬**: 다시 하기(Ctrl+Shift+Z)도 올린 버전을 기록에서 빼지 않는다(`reset`·`amend`·`rebase-i`의 다시 하기 앞에 `refuse_published`). 올림 판정은 모든 `refs/remotes/*`(원격 끝점이 1000개를 넘으면 "모름"으로 보고 거절, fail-closed)이고, 반복 2의 되돌리기 쪽 `is_pushed`(upstream만)도 같은 판정으로 맞춘다(더 보수적으로).
- **C6(D-102) 보탬**: 조회(`conflict_state`·`conflict_file`)는 표식을 지우지 않는다(읽기 전용). 표식 정리는 액션 안이나 `pending_work`에서 `repo_busy`가 거짓일 때만 한다. 표식 서명 입력에 도메인 접두사(`higit-marker`)를 두고 `JournalKey`의 상수 시간 검증을 다시 쓴다. 키가 임시(앱 데이터 폴더를 못 씀)면 멈출 수 있는 작업(복사·되돌리기·기록 정리)이 충돌로 멈출 때 바로 `--abort`로 시작 전으로 돌리고 알린다(껐다 켜면 표식이 무효라 터미널로 풀어야 하는 상황을 만들지 않는다).
- **C7(D-103) 보탬**: `branch-delete`는 끝 버전 백업 → `branch -D` → 맡긴 ref를 백업으로 옮김 순서(지우기가 실패하면 맡긴 파일은 제자리). `branch-rename` 되돌리기는 `undo()`의 `restore_refs`에서 `stash`처럼 뺀다. 다시 하기는 하지 않는다(`NothingToUndo`, 의도).
- **C8(D-104) 보탬**: 주석 태그(태그 객체)는 지우지 않는다(`AnnotatedTag`로 거절, 지우면 객체가 gc로 사라져 Ctrl+Z가 늦게 실패할 수 있다).

### 바뀐 순서와 나눈 단계
E1 → E2 → E3 → E4 → E5 → E6 → E7 → E8 → E9 → E10 → E11 → E12 → E13 → E14a → E14b → E15 → E16 → E17 → E18 → E19 → E20 → E21 → E26 → E22 → E23 → E24 → E25 → E27 → E28 → E29a → E29b → E30 → E31 → E32.
- E26(충돌 화면의 새 맥락)을 E22 앞으로 옮긴다: 메뉴의 되돌리기·복사가 충돌을 내기 전에 그 맥락을 푸는 화면이 있어야 한다(한 단계 = 동작하는 앱).
- E14a: `query_rewrite_plan`과 `rewrite_block`(검사). E14b: `rebase-i` 액션과 `edit` 멈춤 처리.
- E29a: 카드 목록·키·메시지 고치기. E29b: 끌어서 순서 바꾸기(두 웹뷰에서 안 되면 키·단추만 남긴다).

### 단계별 고침
- **E1**: D-043 상태 줄에 "(D-097로 보완)", spec 위험도 `danger` 줄과 `force-push` 행 설명을 같은 커밋에서. roadmap·progress에 반복 4 이후 "설정 화면 + 강제 올리기 + 올린 기록 바꾸기"를 이월 항목으로 남긴다.
- **E2**: 원격 끝점이 잘리면 "모름"(`PublishedHistory`로 거절)을 돌려준다. 테스트로 명시.
- **E4**: `Amend`는 `--cleanup=whitespace`(`commit.cleanup=strip` 설정이 `#` 줄을 지우지 않게)를 준다. 기존 `Commit`(`-F -`)도 같은 문제라 별도 커밋으로 같이 고친다. cherry-pick·revert 마치기에서 해결 결과가 비면(`nothing to commit`) `--skip`으로 정리하고 표식을 지운다(결과 `NoChange`).
- **E6**: 시작 전에 신원 확인(`git var GIT_COMMITTER_IDENT` 실패 → `IdentityMissing`, 시작하지 않음). 시작이 충돌이 아닌 까닭(신원·서명·hook)으로 멈추면 바로 `--abort`하고 그 오류를 돌려준다. 충돌 화면에는 충돌 파일이 있을 때만 간다. Windows CI에 한글·`/`·`C:\`로 시작하는 메시지(stdin이라 경로 바꿈이 없음을 확인)를 넣는다.
- **E9**: 맥락 판정은 앱 작업이 도는 동안(`repo_busy`) "아직 모름"으로 둔다. 모드가 다른 충돌의 뒤집힌 쪽 테스트 한 건 더.
- **E10**: `Amend`의 `journal_copy`는 메시지를 비운다(1MB 메시지가 journal 줄에 들어가지 않게).
- **E12**: 위 E4의 빈 결과 처리를 쓰고 테스트.
- **E14a**: `rewrite_block`은 `rev-list --merges --count base..HEAD`가 0이고 `rev-list --count base..HEAD`가 첫 부모 선 길이와 같을 때만 허용한다(구간 위에 합치기 버전이나 합쳐 들어온 곁가지가 있으면 `HasMerge`). 구간 안 버전을 다른 로컬 브랜치·태그가 가리키면 계획 조회에 그 수를 담아 카드 지도가 "옛 버전을 가리키는 이름표 N개"를 알린다(막지는 않는다).
- **E15**: 이어가기 전 검증(`verify_state`·`verify_todo`)은 위 C4 고침대로. `edit` 멈춤은 표식의 메시지로 amend 뒤 `--continue`, 충돌 멈춤은 충돌 화면. 앱을 껐다 켠 뒤 `edit` 멈춤에서 이어가는 테스트를 꼭 넣는다.
- **E16**: 주석 태그는 `AnnotatedTag`로 거절.
- **E17**: 위 C7 고침의 순서와 되돌리기 규칙, 각각 테스트.
- **E21~E25**: 대화상자가 열려 있는 동안 전역 단축키(Ctrl+Z·Ctrl+H·Ctrl+Shift+S·Ctrl+B)가 뒤에서 돌지 않는지 테스트한다(대화상자가 열리면 `useShortcut`을 끄거나 대화상자가 키를 막는다). 흐린 항목의 Tooltip이 비모달 메뉴를 닫지 않는지 Testing Library로 확인한다.
- **E24**: 마지막 버전 고치기 글상자의 Ctrl+Enter는 `matches("commit", e)`(한글 조합 처리 포함)를 쓴다.
- **E27**: 여러 역 선택 중인 목록은 `aria-multiselectable`. Shift+↑↓ 판정은 `src/lib/listKeys.ts`에.
- **E28·E29a**: 카드 키(Alt+↑↓, Delete, Enter) 판정은 `src/lib/cardKeys.ts`(D-056). 카드 목록은 `role="list"`와 항목별 roving tabindex(옵션 안에 단추를 두지 않는다), 이동 결과는 `aria-live`로 알린다. `.git` 변경으로 계획을 다시 읽어도 고친 카드 상태는 해시 기준으로 남기고 `blocked`만 갱신한다.
