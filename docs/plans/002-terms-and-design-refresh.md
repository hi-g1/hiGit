# 002 용어·디자인 개편
- 관련 결정: D-024(용어), D-025(2단 배치), D-026(먹색 모노 색), D-027(문구 규칙), D-028(충돌 화면 1안) — 모두 확정(2026-10-15). 기대는 확정 결정: D-001, D-004, D-006, D-007, D-010, D-014, D-015, D-020. D-017(제안)은 건드리지 않는다.
- 관련 명세: spec.md "한 줄 요약", "원칙" 3·5, "화면 구성"(작업 공간, 저장 기록, 합치기 / 충돌, 안전망), "기능 목록과 용어표", "오류 메시지 원칙"의 예. design.md 전체. plans/001의 15~22단계(1a 남은 단계)와 19·20단계(1b).
- 참고 자료(결정 아님): `docs/research/terminology.md`, `docs/research/design-direction.md`, `docs/research/git-visualization.md`.

## 목표
화면의 Git 용어가 밖에서 배우는 말과 이어지고("브랜치", "버전 저장", "변경 사항"), 작업 공간이 상단 바 + 2단으로 단순해지며, 틀은 무채색이고 색은 뜻이 있을 때만 쓰인다. 반복 1a의 남은 화면(15~18, 21단계)은 처음부터 새 용어·새 색·2단 배치로 만든다.

## 완료 기준
- `pnpm test`가 통과한다: spec.md 용어표·`ko.ts`·`en.ts` 동기화(spec-sync), 액션 위험도·glossaryId(registry-glossary), 새 문구 규칙 테스트(괄호·옛 용어 없음), 새 대비 테스트(tokens.css의 글자 쌍 4.5:1 이상, 포커스·주 버튼 테두리 3:1 이상, 라이트·다크 둘 다).
- `rg "갈래|저장하기|저장에 넣기|새 소식|안전망|최신으로 받기|관리 시작" src --glob '!*.test.*'`가 주석 말고는 비어 있다.
- `action-meta.json`과 `bindings.ts`는 이 개편의 1~7단계에서 바뀌지 않는다(`gen-bindings` 후 `git diff`가 빔).
- Ubuntu 22.04와 Windows 11에서 `pnpm tauri dev`로 저장소를 열면 상단 바와 2단(왼쪽 "변경 사항 | 버전 기록" 탭, 오른쪽 비교)이 보이고, 주 버튼은 흑연색, 파랑은 포커스·선택에만 보인다. 라이트·다크 모두.
- spec.md "합치기 / 충돌"에 1안 원칙과 색 규칙이 있고 2안은 "나중 후보"로만 적혀 있다.

## 용어 id를 바꾸나
바꾸지 않는다(D-024). 쉬운 이름과 호버 설명만 바꾼다.
- 이유: 화면의 액션 이름표는 `action-meta.json`에 들어 있지 않고 `glossaryId`로 glossary에서 가져온다(`ActionButton` → `<Term id={meta.glossaryId}>`). 그래서 쉬운 이름만 바꾸면 Rust·바인딩·액션 태그는 그대로다.
- 바꾸면 생기는 영향(참고, 하지 않음): `TERM_IDS`와 spec 표 id 열, `ActionInput` enum 태그(`#[serde(rename_all = "kebab-case")]`라 변형 이름 변경 → TS `{ id: "…" }` 변경), `crates/core/src/actions/*.rs`의 `ActionMeta.id`·`glossary_id`, `action_input_tags_match_registry` 테스트, `action-meta.json`·`bindings.ts` 재생성, `ActionButton` 호출부(`Home.tsx`의 `{ id: "init" }`), `App.test.tsx`의 `executeAction` 인자, 1b 계획의 액션 id(`branch-create`, `switch` …). id는 Git 원래 이름이라 사용자에게 보이지 않으므로 이 비용을 쓸 이유가 없다.

## 새 용어표 (3단계에서 spec.md 표를 이 내용으로 바꾼다)
id 순서·원래 이름·명령·위험도·빈도는 그대로. 바뀌는 칸은 쉬운 이름과 호버 설명이다. 설명에도 "저장", "갈래"를 쓰지 않는다. 호버 설명은 동작이면 "~하기", 대상이면 명사로 끝내고 메모체("~않음·~함·~사라짐")를 쓰지 않는다. 주의할 내용은 둘째 문장에 "~요"(D-024, 2026-10-15 사용자 확정).

| id | 쉬운 이름 | 원래 이름 · 명령 | 호버 설명 | 위험도 | 빈도 |
| --- | --- | --- | --- | --- | --- |
| status | 변경 사항 | status · `git status` | 마지막 버전 이후 바뀐 파일 목록 | safe | 매일 |
| diff | 비교 보기 | diff · `git diff` | 파일이 어떻게 바뀌었는지 줄 단위로 비교 | safe | 매일 |
| stage | 이번 버전에 넣기 | stage · `git add` | 이번 버전에 포함할 파일 고르기 | safe | 매일 |
| unstage | 이번 버전에서 빼기 | unstage · `git restore --staged` | 이번 버전에 넣었던 파일을 다시 빼기 | safe | 매일 |
| commit | 버전 저장 | commit · `git commit` | 지금 상태를 설명과 함께 버전으로 남기기 | safe | 매일 |
| fetch | 새로 고침 | fetch · `git fetch` | 다른 사람이 올린 버전을 확인하기. 내 파일은 아직 바뀌지 않아요. | safe | 매일 |
| pull | 가져오기 | pull · `git pull` | 다른 사람이 올린 버전을 받아 지금 브랜치에 합치기 | normal | 매일 |
| push | 올리기 | push · `git push` | 내 버전들을 원격 저장소로 보내기 | normal | 매일 |
| branch | 브랜치 | branch · `git branch` | 원래 작업에 영향 없이 따로 작업하는 공간 | safe | 매일 |
| switch | 브랜치 바꾸기 | switch · `git switch` | 다른 브랜치로 옮겨 가기 | normal | 매일 |
| log | 버전 기록 | log · `git log` | 지금까지 저장한 버전 목록 | safe | 매일 |
| merge | 합치기 | merge · `git merge` | 다른 브랜치의 작업을 지금 브랜치에 합치기. 병합이라고도 해요. | normal | 자주 |
| conflict | 충돌 | merge conflict | 두 브랜치가 같은 곳을 다르게 고쳐서 어느 쪽을 남길지 정해야 하는 상태 | normal | 자주 |
| stash | 임시 보관 | stash · `git stash` | 하던 작업을 잠시 보관했다가 나중에 꺼내기 | safe | 자주 |
| discard | 변경 취소 | restore · `git restore` | 아직 버전으로 저장하지 않은 변경을 버리고 마지막 버전으로 돌아가기 | caution | 자주 |
| pr | 검토 요청 | Pull Request | 내 브랜치를 합쳐 달라고 팀에 요청 | safe | 자주 |
| clone | 복제 | clone · `git clone` | 원격 저장소를 내 컴퓨터로 복사 | safe | 자주 |
| init | 버전 관리 시작 | init · `git init` | 이 폴더를 Git으로 버전 관리하기 시작 | safe | 자주 |
| amend | 마지막 버전 고치기 | amend · `git commit --amend` | 마지막 버전의 설명이나 내용 고치기 | caution | 가끔 |
| revert | 되돌리기 | revert · `git revert` | 고른 버전의 변경을 취소하는 새 버전 만들기. 기록은 그대로 남아요. | normal | 가끔 |
| reset | 이 버전으로 돌아가기 | reset · `git reset` | 브랜치를 고른 버전으로 옮기기. 그 뒤의 버전은 기록에서 빠져요. | danger | 가끔 |
| rebase-i | 기록 정리 | interactive rebase · `git rebase -i` | 버전 순서 바꾸기, 묶기, 메시지 고치기 | danger | 가끔 |
| rebase | 리베이스 | rebase · `git rebase` | 내 브랜치의 버전들을 다른 브랜치의 최신 버전 뒤로 옮기기 | caution | 가끔 |
| cherry-pick | 이 버전만 복사하기 | cherry-pick · `git cherry-pick` | 다른 브랜치의 버전 하나를 지금 브랜치로 복사하기 | normal | 가끔 |
| tag | 태그 | tag · `git tag` | 이 버전에 v1.0 같은 이름 붙이기 | safe | 가끔 |
| blame | 줄별 기록 | blame · `git blame` | 각 줄을 마지막으로 바꾼 사람과 날짜 | safe | 가끔 |
| reflog | 복구 | reflog · `git reflog` | 최근 작업을 되짚어 지운 버전이나 브랜치를 되살리기 | safe | 가끔 |
| force-push | 덮어쓰며 올리기 | force push · `git push --force-with-lease` | 원격 저장소의 기록을 내 기록으로 덮어쓰기. 다른 사람의 작업이 사라질 수 있어요. | danger | 가끔 |
| remote | 원격 저장소 | remote · `git remote` | 이 프로젝트가 연결된 원격 저장소 목록 | normal | 드묾 |
| ignore | 제외할 파일 | .gitignore | 버전에 넣지 않을 파일 규칙 | safe | 드묾 |

- 표에 없던 것: `unstage`·`switch`는 같은 기준으로 맞춘 제안(사람 확인 Q4). `pull + push`의 "동기화"는 등록할 액션이 1b 범위라 이번에 행을 넣지 않는다(Q5).
- 영어 `en.ts`(확정, 설명은 위 한국어 설명과 같은 기준·같은 뜻으로 3단계에서 쓴다): status Changes, diff Compare, stage Include in version, unstage Leave out of version, commit Save version, fetch Refresh, pull Get updates, push Upload, branch Branch, switch Switch branch, log Version history, merge Combine, conflict Conflict, stash Set aside, discard Discard changes, pr Review request, clone Clone, init Start version control, amend Fix last version, revert Undo, reset Go back to this version, rebase-i Tidy history, rebase Rebase, cherry-pick Copy this version, tag Tag, blame Line history, reflog Recover, force-push Upload and overwrite, remote Remote repository, ignore Excluded files.

## 색 토큰 (5단계에서 design.md, 7단계에서 tokens.css)
라이트·다크 값은 design-direction.md 먹색 모노를 따르되, 대비를 다시 계산해 미달인 두 값만 고쳤다(★, 사람 확인 Q8·Q9). 대비는 WCAG 2.x 상대 휘도 공식으로 계산했다.

| 토큰 | 라이트 | 다크 | 쓰임 |
| --- | --- | --- | --- |
| `--bg` | `#FAFAF9` | `#161718` | 앱 바탕, 상단 바 |
| `--surface` | `#FFFFFF` | `#1C1D1F` | 목록·비교 칸 |
| `--surface-sunken` (새) | `#F2F2F0` | `#121314` | 입력칸 안, 코드 바탕, 접힌 줄 |
| `--surface-hover` | `#F2F2F0` | `#232528` | 행 호버 |
| `--border` | `#E5E5E2` | `#2B2D30` | 칸 사이 1px 선 (장식선) |
| `--border-strong` (새) | `#8C9096` | `#6E7379` | 입력칸·체크 상자 테두리 |
| `--text` | `#24292F` (Q7) | `#ECECEA` | 본문 |
| `--text-muted` | ★`#6A6F75` (조사안 `#6E7379`) | `#9EA3A8` | 보조 글, 시각, 해시 |
| `--primary` (새) | `#4A5059` | `#C9CBCE` | 주 버튼 채움 |
| `--primary-hover` (새) | `#3F454D` | `#DDDFE2` | 주 버튼 호버 |
| `--primary-text` (새) | `#FFFFFF` | `#161718` | 주 버튼 글자 |
| `--accent` | `#3A6EA5` | `#7FA7D6` | 선택 막대, 포커스 테두리, 링크만 |
| `--added` | ★`#2D7A4D` (지금 `#2F7D4F`) | `#6BBF8A` | 추가된 파일·줄 |
| `--modified` | `#9A6110` | `#D9A54A` | 수정된 파일 |
| `--deleted` | `#B4423A` | `#E07A70` | 삭제, 위험 작업 |
| `--diff-add-bg` | `#EAF5EE` | `#1E2E24` | 추가 줄 배경 |
| `--diff-del-bg` | `#FBECEA` | `#33201F` | 삭제 줄 배경 |

- 없애는 토큰: `--accent-text`(주 버튼 글자는 `--primary-text`). `--graph-1`~`--graph-6`, 글꼴·간격·모양 토큰은 그대로.
- 대비 계산 결과(글자는 4.5 이상, 테두리·포커스는 3 이상이 기준):

| 쌍 | 라이트 | 다크 |
| --- | --- | --- |
| `--text` / bg · surface · sunken | 14.03 · 14.65 · 13.07 | 15.18 · 14.26 · 15.73 |
| `--text-muted` / bg · surface · sunken·hover | 4.85 · 5.07 · 4.52 (조사안 `#6E7379`는 4.58 · 4.78 · **4.27 미달**) | 7.06 · 6.63 · 7.31 (hover 6.04) |
| `--primary-text` / `--primary` | 8.13 | 11.04 |
| `--primary-text` / `--primary-hover` | 9.68 | 13.44 |
| `--primary` / `--bg` (버튼 경계) | 7.78 | 11.04 |
| `--accent` / bg · surface · sunken | 5.09 · 5.31 · 4.74 | 7.18 · 6.75 · 7.44 |
| `--added` / bg · surface · hover | 5.0 · 5.25 · 4.68 (지금 `#2F7D4F`는 hover에서 **4.496 미달**) | 8.07 · 7.59 · 6.91 |
| `--modified` / bg · surface · hover | 4.91 · 5.13 · 4.58 | 8.06 · 7.58 · 6.90 |
| `--deleted` / bg · surface · hover | 5.32 · 5.55 · 4.95 | 6.14 · 5.77 · 5.26 |
| `--added` / `--diff-add-bg` | 4.70 | 6.42 |
| `--deleted` / `--diff-del-bg` | 4.84 | 5.26 |
| `--text` / diff 배경 둘 | 13.1 · 12.8 | 12.1 · 13.0 |
| `--border-strong` / surface · bg | 3.21 · 3.07 | 3.53 · 3.75 |
| `--border` / surface | 1.26 (장식선이라 기준 밖) | 1.22 (같음) |

- 주의: 라이트 `--border-strong`은 `--surface-sunken` 위에서 2.86이라 sunken 바탕 위 입력칸 테두리로 쓰지 않는다(design.md에 적음).

## 문구 점검 결과 (6단계에서 고침)
D-027(괄호 금지, 버튼은 동사 하나, 보충은 툴팁·둘째 줄)과 D-024(새 용어), CLAUDE.md(존댓말, 짧게)로 지금 화면 문구를 점검했다. 화면 코드(`Home`, `Workspace`, `ChangedFiles`, `HistoryList`, `ErrorNotice`, `ActionButton`)에는 하드코딩한 한국어가 없다. 모든 문구는 `messages`와 `glossary`에 있다. 괄호를 쓴 문구도 없다. 어긋나는 것은 아래와 같다.

| 키 (`src/messages/ko.ts`) | 지금 | 문제 | 고칠 문구 | en |
| --- | --- | --- | --- | --- |
| `noChanges` | 마지막 저장 이후 바뀐 파일이 없어요. | 옛 용어 | 마지막 버전 이후 바뀐 파일이 없어요. | No files changed since the last version. |
| `noCommits` | 아직 저장한 기록이 없어요. | 옛 용어 | 아직 저장한 버전이 없어요. | No versions saved yet. |
| `errors.watchUnavailable` | …창으로 돌아오면 새로 고칩니다. | 합쇼체 섞임, "새로 고침"이 fetch 이름과 겹침 | 파일 변경을 자동으로 알아채지 못해요. 창으로 돌아오면 다시 읽어요. | …The view reloads when you return to the window. |
| `errors.nothingToCommit` | 저장할 변경이 없어요. 먼저 파일을 저장에 넣어 주세요. | 옛 용어, "저장"이 파일 저장과 헷갈림 | 이번 버전에 넣은 파일이 없어요. 먼저 파일을 골라 주세요. | No files are included in this version. Pick files first. |
| `errors.identityMissing` | 저장에 쓸 이름과 이메일이… | 옛 용어 | 버전에 남길 이름과 이메일이 설정되어 있지 않아요. | Your name and email for versions are not set. |
| `errors.invalidBranchName` | 갈래 이름으로… | 옛 용어 | 브랜치 이름으로 쓸 수 없는 글자가 있어요. | (그대로) |
| `errors.branchExists` | 같은 이름의 갈래가… | 옛 용어 | 같은 이름의 브랜치가 이미 있어요. | (그대로) |
| `errors.wouldOverwrite` | 저장하지 않은 변경이 덮어써져요. 먼저 저장해 주세요. | "저장"이 파일 저장과 헷갈림 | 버전으로 저장하지 않은 변경이 덮어써져요. 먼저 버전을 저장해 주세요. | Changes not saved as a version would be overwritten. Save a version first. |
| `errors.notFastForward` | …합치기는 다음 버전에서 지원해요. | "버전"이 이제 commit 뜻이라 "앱의 다음 버전"과 헷갈림 | 내 버전과 원격 저장소의 버전이 갈라져 있어요. 합치기는 아직 준비 중이에요. | Your versions and the remote have diverged. Combining is not ready yet. |
| `errors.pushRejected` | 올리기 전에 최신 내용을 먼저 받아야 해요. | 옛 용어(받기) | 올리기 전에 원격 저장소의 새 버전을 먼저 가져와야 해요. | Get the new versions from the remote before uploading. |
| `errors.noUpstream` | 아직 원격에 올린 적이 없는 갈래예요. | 옛 용어 | 아직 원격 저장소에 올린 적이 없는 브랜치예요. | (그대로) |
| `changeKind.conflicted` | 겹침 | 새 용어는 "충돌" | 충돌 | Conflict (그대로) |
| `refresh` | 새로 고침 | fetch 쉬운 이름과 같은 낱말, 하는 일은 화면 다시 읽기(기준 4) | 키를 지우고 `reload: "다시 읽기"`를 둔다. 상단 바 버튼은 없애고, 파일 감시를 쓸 수 없을 때 안내 줄(`errors.watchUnavailable`)에만 이 버튼을 보여준다(Q3) | Reload |

- 지켜지고 있어 그대로 둘 것: `openFolder` "폴더 열기", `openOtherFolder` "다른 폴더 열기", `showDetail` "자세히 보기"(버튼 규칙 맞음), 나머지 오류 문구.
- 길지만 이번에 두는 것: `errors.sshAuthFailed`(두 문장). 둘째 문장을 둘째 줄로 내리려면 `ErrorNotice`에 보충 줄이 필요하다. 1b 20단계에서 오류별 해결 버튼을 붙일 때 함께 정한다.
- 테스트 문자열: `src/ui/Term.test.tsx`("바뀐 내용", "저장 기록", "저장하기", "관리 시작", en "Save"), `src/App.test.tsx`("바뀐 내용", "저장 기록", "관리 시작", "마지막 저장 이후…", "아직 저장한 기록이 없어요.")는 해당 단계에서 함께 고친다.
- spec.md 안의 화면 이름 괄호("프로젝트 가져오기(clone)", "새로 시작(init)", "(interactive rebase)", "(reflog + 앱 백업)")는 4단계에서 괄호 없이 고친다.

## 반복 1에 넣을 것과 미룰 것
| 항목 | 언제 | 비고 |
| --- | --- | --- |
| 결정 D-024~D-028 확정 | 1a, 15단계 전 | 1단계 |
| 001 계획을 2단·새 용어로 개정 | 1a, 15단계 전 | 2단계 |
| 새 용어표, glossary ko/en, 테스트 문자열 | 1a, 15단계 전 | 3단계 |
| spec.md 화면 구성·원칙·충돌 화면 설명·오류 예시 | 1a, 15단계 전 | 4단계 |
| design.md 색·대비·문구 규칙·배치 | 1a, 15단계 전 | 5단계 |
| 화면 문구(messages) 정리와 문구 규칙 테스트 | 1a, 15단계 전 | 6단계 |
| tokens.css, 주 버튼, 대비 테스트 | 1a, 15단계 전 | 7단계 |
| 2단 + 상단 바, 저장 상자 위치, 체크 상자 모양 | 1a 15~18단계 안에서 | 001 개정본대로 |
| 문구 규칙·옛 용어 점검 | 1a 22단계 | code-analyst 점검 항목에 추가 |
| 브랜치 목록을 상단 바 드롭다운으로 | 1b 19단계 | 001 개정본대로 |
| 상단 바 오른쪽 새로 고침(fetch)·가져오기·올리기, 동기화 여부 | 1b 20단계 | Q5 |
| 충돌 해결 화면 1안, `ours`·`theirs` 용어 행 | 반복 2 | D-028 |
| 설정 "브랜치 목록 항상 보기"(3단 사이드바) | 설정 화면이 생기는 반복 | D-025 |
| 비교 보기의 나란히 보기, 단어 강조, 변경 없는 줄 접기, 깜빡이 비교, 그림 전후 막대 | 나중 | 17단계는 한 줄로 보기만 |
| 충돌 화면 2안(제안부터), 문서처럼 보기 | 나중 후보 | D-028 |
| 처음 쓰는 사람 안내 모드(C안) | 미정 | 결정 없음 |
| 모서리·그림자 세부 규칙(창 안 패널 0, 떠 있는 것 8px) | 사람 확인 후 | Q12 |
| 다크 모드의 그래프 6색, 현재 브랜치 표시 방식 | 노선도 반복 | Q10 |
| Rust 주석·생성 바인딩 주석의 "갈래" | 그 파일을 고칠 때 | 사용자에게 안 보임 |

## 단계
각 단계 = 커밋 하나. 1~7단계는 1a의 15단계 전에 한다. 1~2·4~5단계는 문서만 바뀐다(4단계는 spec.md를 건드리므로 pre-commit이 프론트 테스트를 돈다). 3·6·7단계는 프론트만 바뀌고 Rust·바인딩은 그대로다.

1. 결정 확정 (`docs: confirm terms and design decisions`)
   - 사람이 이 계획과 아래 "사람 확인 필요"에 답한 뒤, `docs/decision.md`의 D-024~D-028 상태를 `확정`으로 바꾸고 답에 따라 결정 문구를 고친다(예: Q8에서 `#6E7379` 유지를 고르면 D-026의 값 표 참조를 고침). 답이 "보류"인 결정은 제안으로 둔다.
   - 변경 파일: `docs/decision.md`.
   - 완료 기준: D-024~D-027이 확정이다(D-028은 반복 2 전까지 제안으로 둬도 됨).
   - 위험: 결정이 확정되지 않으면 3단계 이후를 시작하지 않는다(CLAUDE.md "결정이 바뀌면 decision.md를 먼저").

2. 001 계획 개정 (`docs: revise plan 001 for two-pane layout and new terms`)
   - 변경 파일: `docs/plans/001-iteration1-daily.md`.
   - 내용:
     - 머리에 한 줄: "2026-10-15 개정: D-024~D-026으로 남은 단계(6, 8, 9, 12, 13, 15~22)를 새 용어와 2단 배치로 고쳤다. 끝난 단계 본문은 기록이라 옛 용어 그대로 둔다. 계획 002 참고."
     - "관련 명세", "목표", "범위"의 "작업 공간 3단 배치" → "작업 공간 2단 + 상단 바", "저장 기록 탭 분리" → "왼쪽 탭에 변경 사항 | 버전 기록".
     - 완료 기준의 용어(저장에 넣기 → 이번 버전에 넣기, 저장하기 → 버전 저장, 저장 기록 → 버전 기록, 갈래 → 브랜치, 새 소식 확인 → 새로 고침, 최신으로 받기 → 가져오기, "[최신으로 받기]" → "[가져오기]", "아직 올린 적 없음"은 유지).
     - 15단계 다시 쓰기: 상단 바 + 2단. 파일은 `Workspace.tsx`·`Workspace.module.css`, `TopBar.tsx` + CSS(신규, 옛 계획의 `StatusBar` 대신), `ChangesTab`·`HistoryTab`(신규, 왼쪽 탭 내용), `src/state/repoStore.ts`(`leftTab: "changes" | "history"`, `selectedPath`), `src/ui/Panel.*`(작업 공간에서는 둥근 상자 대신 1px 선 칸, 필요하면 `variant="pane"`), `messages`, 테스트. 상단 바 왼쪽: 프로젝트 이름(경로는 툴팁), 브랜치 단추 "● main"(`<Term id="branch" />` 툴팁, 1a는 누를 수 없는 표시, 1b에서 드롭다운), 브랜치가 없으면 "특정 버전을 보는 중", upstream이 있으면 `<Term id="push" />` N · `<Term id="pull" />` N 글자, 없으면 "아직 올린 적 없음". 오른쪽: 다른 폴더 열기. 1a에는 화면 다시 읽기 버튼을 두지 않는다(Q3, 1b에서 이 자리에 fetch "새로 고침"). 컨트롤 5개 이하. 왼쪽 칸(너비 320px, 최소 280px): 탭 "`<Term id="status" />` N | `<Term id="log" />`"(role=tablist, 좌우 화살표로 이동, 고른 탭 밑줄 `--accent`), 변경 사항 탭 아래쪽에 버전 저장 상자 자리. 오른쪽 칸: 변경 사항 탭에서는 비교(17단계 전에는 빈 상태 "파일을 고르면 비교가 보여요."), 버전 기록 탭에서는 목록을 창 전체 폭으로(Q13). 왼쪽 갈래 목록 자리와 "최소 너비에서 왼쪽 패널 접힘"은 지운다. 테스트: 탭 전환, 상단 바 문구 네 경우(upstream 있음/없음/브랜치 없음/버전 없음), 세 번째 칸이 없음.
     - 16단계: 체크 상자는 왼쪽 탭 목록의 행 앞. 체크 상자에 보이는 글자는 없고, 접근성 이름과 툴팁은 glossary에서(`stage`/`unstage` 쉬운 이름). 이를 위해 `src/glossary/index.ts`에 `useTermLabel(id)`(설정 "Git 원래 이름으로 보기"를 따르는 문자열)를 추가한다. 목록 머리에 "모두" 체크 상자 하나(세 상태). 체크 상자는 OS 기본 모양 대신 `appearance: none` + `--border-strong` 테두리 + 켜짐 `--primary` 채움(Q15). "모두 넣기/모두 빼기" 버튼은 지운다.
     - 17단계: 오른쪽 칸. 맨 위 한 줄 "마지막 버전과 지금 파일을 비교해요 · N줄 추가, M줄 삭제"(Q14, 숫자는 `model.ts`가 hunk에서 셈). 한 줄로 보기만. 줄 배경은 `--diff-*-bg`, 글자 색 + `+`/`−` 기호 함께, 코드 바탕은 `--surface`.
     - 18단계: 버전 저장 상자는 왼쪽 칸 아래(오른쪽 칸이 아님). 입력칸 안내 문구 "무엇을 바꿨나요?", 주 버튼 `<Term id="commit" />`(= "버전 저장", `--primary`, 작업 공간의 유일한 주 버튼). 버튼이 꺼진 이유는 버튼 아래 둘째 줄로("이번 버전에 넣을 파일을 골라 주세요." / "무엇을 바꿨는지 적어 주세요."). 성공 문구 "버전을 저장했어요 · <짧은 해시>".
     - 19단계(1b): 브랜치 목록은 왼쪽 칸이 아니라 상단 바 브랜치 단추의 드롭다운(`@radix-ui/react-popover`, D-004). 현재 브랜치 표시는 `--accent` 막대 대신 체크 표시 + 굵게(Q10). "새 `<Term id="branch" />`" → 대화상자.
     - 20단계(1b): 상단 바 오른쪽에 `<Term id="fetch" />`(새로 고침, 원격이 없으면 화면만 다시 읽음), `<Term id="pull" />` ↓N, `<Term id="push" />` ↑N. 동기화 단추는 Q5 결과에 따름. `pushRejected` 해결 버튼은 [가져오기].
     - 21단계: 감시가 켜지지 않거나 꺼지면(`watchUnavailable`, `watch-status`의 `active: false`) 작업 공간 위에 안내 줄 "파일 변경을 자동으로 알아채지 못해요. 창으로 돌아오면 다시 읽어요."와 "다시 읽기" 버튼을 보여준다. 버튼은 status·log·diff 쿼리를 무효화한다. 감시가 켜져 있으면 안내 줄도 버튼도 없다.
     - 22단계 점검 항목에 추가: 문구 규칙 테스트 통과, 옛 용어 `rg`(이 계획 완료 기준의 명령), `--accent`를 주 버튼에 쓰는 곳 없음(`rg "background: var\(--accent\)" src`가 비어 있음).
     - 남은 1b 단계(6, 8, 9, 12, 13) 본문의 "갈래"·"새 소식 확인"·"최신으로 받기"를 새 용어로. 위험 절의 "작업 공간 3단 배치가 800px에서 좁다"는 "2단으로 해결(D-025)"로.
     - "용어 변경" 절에 "D-024로 쉬운 이름 전체 개편(계획 002 3단계)" 한 줄.
   - 완료 기준: `rg "3단" docs/plans/001-iteration1-daily.md`가 개정 안내 줄과 기록 말고는 없다. 15~22단계를 읽고 구현자가 002를 보지 않아도 따라 할 수 있다.
   - 위험: 끝난 단계 본문까지 고치면 기록이 흐려진다 → 남은 단계만.

3. 용어표와 glossary 개편 (`glossary: adopt familiar terms`)
   - 변경 파일: `docs/spec.md`("기능 목록과 용어표"의 표만, 위 "새 용어표"로 교체), `src/glossary/ko.ts`(쉬운 이름·설명), `src/glossary/en.ts`(위 영어 제안과 같은 뜻의 설명), `src/ui/Term.test.tsx`(기대 문자열: 변경 사항, 버전 기록, 버전 저장, 버전 관리 시작, en "Save version", 툴팁 설명 "마지막 버전 이후 바뀐 파일 목록"), `src/App.test.tsx`(제목 Term "변경 사항"·"버전 기록", 버튼 이름 "버전 관리 시작").
   - `src/glossary/types.ts`(`TERM_IDS`)는 바뀌지 않는다. 표와 `ko.ts`가 한 커밋에서 같이 바뀌어야 spec-sync가 통과한다.
   - 테스트: `pnpm test`(spec-sync 3개, en 3개, registry-glossary 3개, Term, App).
   - 완료 기준: 위 테스트 통과. `cargo run -p app --bin gen-bindings` 후 `git diff --exit-code -- src/lib`가 빈다(Rust·바인딩 무변경 확인). 수동: 두 OS에서 "버전 관리 시작" 버튼에 호버하면 "init · git init · 이 폴더를 Git으로 버전 관리하기 시작".
   - Windows/Ubuntu: 차이 없음. 글자 폭이 늘어난 이름("이번 버전에서 빼기")이 버튼·툴팁에서 줄바꿈되는지 두 OS 글꼴 렌더링으로 확인(Pretendard 번들이라 같아야 함).
   - 위험: 표 칸에 `|`나 줄 앞뒤 공백이 섞이면 `readSpecTable`이 깨진다. 설명 칸의 큰따옴표(`tag`)는 지금처럼 그대로 둔다.

4. spec.md 화면·원칙·충돌 화면 (`docs: update spec screens for new terms and layout`)
   - 변경 파일: `docs/spec.md`(용어표 밖), `docs/roadmap.md`, `docs/architecture.md`.
   - spec.md:
     - 한 줄 요약: "저장·갈래·합치기·되돌리기·검토 요청" → "버전 저장·브랜치·합치기·되돌리기·검토 요청".
     - 원칙 3: "현재 브랜치, 버전으로 저장하지 않은 변경 수, 올리지 않은 버전 수, 가져올 버전 수를 상단 바와 탭에 늘 표시." 원칙 4의 "갈래" → "브랜치", "가져오기" → "이 버전만 복사하기". 원칙 5: "무채색 바탕, 색은 브랜치 선·파일 상태·위험 작업처럼 뜻이 있을 때만(`docs/design.md`)."
     - 화면 구성 표:
       - 홈: "최근 프로젝트, 폴더 열기, 복제, 버전 관리 시작".
       - 작업 공간: "상단 바: 프로젝트 이름, 브랜치 단추(누르면 브랜치 목록), 새로 고침, 가져오기·올리기. 아래 2단: 왼쪽은 변경 사항 | 버전 기록 탭과 버전 저장 상자, 오른쪽은 고른 파일의 비교 보기. 설정 '브랜치 목록 항상 보기'를 켜면 왼쪽에 브랜치 사이드바가 생긴다(D-025)."
       - 버전 기록: 들어가는 방법 "작업 공간 왼쪽 탭", 하는 일 "브랜치 그래프 타임라인. 버전 우클릭으로 되돌리기·이 버전만 복사하기·태그 …".
       - 기록 정리: "버전 기록에서 여러 개 선택", "순서 바꾸기, 묶기, 메시지 고치기, 빼기".
       - 합치기 / 충돌: 들어가는 방법 "브랜치 끌어놓기", 하는 일 "미리보기. 충돌이 나면 한 곳씩 내 브랜치·합칠 브랜치·둘 다 중에서 고르기. 아래 '충돌 해결 화면'."
       - 안전망 → 복구: "상단 바 메뉴", "최근 작업 목록에서 원하는 시점으로 복구. reflog와 앱 백업을 함께 보여줌".
       - 설정: "브랜치 목록 항상 보기" 추가.
     - 새 절 "### 충돌 해결 화면 (반복 2, D-028)"을 화면 구성 표 아래에 추가. 내용:
       - 화면이 묻는 질문 하나: "두 브랜치가 같은 줄을 다르게 고쳤어요. 어느 쪽을 남길까요?"
       - 원칙 7개: 위아래 카드 두 장에 이름표("내 브랜치 · 사람 · 날짜 · 메시지", "합칠 브랜치 · …"), 한 번에 한 곳("3곳 중 2곳 남음", 이전·다음, 파일 목록에 ✓/●), 고르면 "저장될 모습"이 바로 바뀜, `<<<<<<<`·`=======`·`>>>>>>>`는 어디에도 보이지 않음("직접 고치기"도 깨끗한 편집 칸), "합치기 그만두기"가 늘 보임(`git merge --abort`, 위험도는 반복 2에서 정함), 원래 코드는 "고치기 전 원래 코드 보기"로 접음(툴팁에 base), 모든 곳을 고르면 주 버튼이 "합치기 마치기"로 바뀜.
       - 색 규칙: 이 화면에서 색은 브랜치를 뜻할 때만 쓴다. 내 브랜치는 `--graph-2`(초록), 합칠 브랜치는 `--graph-1`(파랑)(조사 문서의 `--l2`·`--l1`, Q11). 카드 왼쪽 띠·코드 바탕·저장될 모습의 줄 바탕이 고른 쪽 색을 따른다. 고른 카드의 "이걸로" 버튼은 `--primary` 채움, 안 고른 카드는 흐리게. 진행 표시 ✓·●와 선택 막대는 먹색(`--text`). `--accent`는 main 브랜치 색과 겹치므로 이 화면에서 쓰지 않는다.
       - 용어: "내 브랜치"(ours), "합칠 브랜치"(theirs), 호버하면 원래 이름. 리베이스 중에는 Git의 ours/theirs가 뒤바뀌므로 매핑을 뒤집어 화면의 뜻을 유지. `ours`·`theirs` 행은 반복 2에서 용어표에 추가.
       - 나중 후보: 둘 다 살린 제안부터 보여주기(2안), 문서처럼 보기. 구현하지 않는다.
     - 오류 메시지 원칙의 예: "올리기 전에 원격 저장소의 새 버전을 먼저 가져와야 해요. [가져오기]".
   - roadmap.md: 반복 1b "작업 갈래 만들기·바꾸기, 받기·올리기, 새 소식 확인" → "브랜치 만들기·바꾸기, 새로 고침·가져오기·올리기". 반복 2 "잠시 치워두기" → "임시 보관", "안전망 화면" → "복구 화면". 반복 3 이름들을 새 용어로(마지막 버전 고치기, 되돌리기, 이 버전으로 돌아가기, 기록 정리, 이 버전만 복사하기, 태그). 반복 4 "프로젝트 가져오기" → "복제". 1a에 "용어·디자인 개편(계획 002)" 한 줄.
   - architecture.md: "안전망" → "복구"(계층 그림의 core 설명, UndoJournal 절의 화면 이름). "저장 기록 그리기 계층" 절 제목과 본문의 "저장 기록"은 D-017(제안)이 그 이름을 쓰므로 바꾸되 D-017은 고치지 않는다(Q16). 화면 목록의 "저장 기록, 충돌" → "버전 기록, 충돌 해결".
   - 테스트: `pnpm test`(spec.md를 건드렸으므로 spec-sync 재확인, 표는 3단계에서 이미 바뀜).
   - 완료 기준: `rg "갈래|저장 기록|안전망|프로젝트 가져오기|잠시 치워두기" docs/spec.md docs/roadmap.md docs/architecture.md`가 비어 있다. spec.md에 "나중 후보"가 있고 2안은 거기에만 있다.
   - Windows/Ubuntu: 문서라 차이 없음.
   - 위험: architecture.md의 "저장 기록 그리기 계층"으로 링크하는 곳(spec.md 표의 앵커 `research/git-visualization.md#추천-조합`은 다른 파일이라 무관)을 `rg "그리기 계층"`으로 함께 고친다.

5. design.md 색·대비·문구 규칙·배치 (`docs: monochrome palette, copy rules and two-pane layout`)
   - 변경 파일: `docs/design.md`.
   - 내용:
     - 머리 방향 문장: "회색 계열 바탕, 포인트 색은 차분한 파란색 하나" → "무채색 틀, 주 버튼은 흑연색 채움. 파랑은 선택·포커스·링크에만, 그 밖의 색은 뜻이 있을 때만(D-026)".
     - 색 토큰 표를 위 "색 토큰" 표로 교체하고, 대비 표를 함께 둔다(문서의 "모두 WCAG AA 이상으로 계산해 두었다"를 "`src/styles/contrast.test.ts`가 확인한다"로).
     - 그래프 색 문단: "현재 갈래만 `--accent`" 문장을 지우고 "현재 브랜치는 색이 아니라 '지금 여기' 표시와 굵기로 구분한다(Q10)". "갈래" → "브랜치".
     - 새 절 "배치": 작업 공간은 상단 바 + 2단(D-025). 상단 바 왼쪽 프로젝트·브랜치, 오른쪽 새로 고침·가져오기·올리기, 컨트롤 5개 이하, 아이콘만 있는 버튼엔 툴팁. 왼쪽 칸 320px(최소 280px), 오른쪽 칸은 나머지. 칸 사이는 간격 없이 `--border` 1px 선, 작업 공간 칸에는 둥근 모서리·그림자 없음.
     - 부품 규칙: 버튼 "주 버튼(`--primary` 채움, 글자 `--primary-text`)은 화면마다 하나", 글자 버튼은 `--text`(파랑 아님), 위험 버튼은 그대로. 목록 행 "선택 시 왼쪽 2px `--accent` 막대"는 그대로. 체크 상자·입력칸 테두리는 `--border-strong`, 체크 켜짐은 `--primary`(Q15). `--border-strong`은 `--surface-sunken` 위에 쓰지 않는다. 상태 표시는 색 + 글자 함께.
     - 새 절 "문구"(D-027): 괄호로 덧붙이는 설명 금지. 버튼은 한 가지 동작을 동사로 끝나는 짧은 말(목적어 허용). 보충은 Term 툴팁·일반 툴팁·둘째 줄. 예 "합치기 그만두기 (원래대로)" → "합치기 그만두기". 존댓말 해요체로 통일(합쇼체 섞지 않음). 한 낱말은 한 기능에만(D-024 기준 4). 빈 상태는 한 줄 설명 + 버튼 하나, 왼쪽 정렬.
   - 완료 기준: design.md의 색 표 값이 7단계 tokens.css와 같다(7단계 테스트가 design.md 표를 읽지는 않으므로 사람이 대조, 또는 Q 결과에 따라 7단계에서 표 대조 테스트를 추가).
   - Windows/Ubuntu: "확인 사항"에 한 줄 추가: Windows 고대비(WebView2의 `forced-colors: active`)에서 주 버튼·체크 상자가 보이는지 확인. WebKitGTK는 GTK 고대비 테마를 웹 내용에 적용하지 않는다.
   - 위험: 없음(문서).

6. 화면 문구 정리와 문구 규칙 테스트 (`messages: align wording with new terms and copy rules`)
   - 변경 파일: `src/messages/ko.ts`, `src/messages/en.ts`(위 "문구 점검 결과" 표대로), `src/App.test.tsx`(빈 상태 문구 기대값 "마지막 버전 이후 바뀐 파일이 없어요.", "아직 저장한 버전이 없어요."), `src/messages/copy-rules.test.ts`(신규, node 환경).
   - `copy-rules.test.ts`가 확인하는 것: ① `ko`·`en` 메시지(함수형 값은 예시 인자로 호출해 문자열로)와 `ko.ts`·`en.ts` glossary의 `friendly`·`description`에 `(`·`)`·`（`·`）`가 없다. ② 한국어 메시지와 glossary 쉬운 이름·설명에 옛 용어 목록(`갈래`, `저장하기`, `저장에 넣`, `저장에서 빼`, `저장 기록`, `새 소식`, `최신으로 받기`, `안전망`, `잠시 치워두기`, `겹친 부분`, `관리 시작`이되 `버전 관리 시작`은 제외)이 없다. ③ 한국어 메시지 끝이 `습니다.`·`니다.`로 끝나지 않는다(해요체). 목록은 테스트 파일 맨 위 상수로 두고 주석에 D-024·D-027.
   - 테스트: `pnpm test`.
   - 완료 기준: 위 테스트 통과, 일부러 `"(원래대로)"`를 넣으면 실패.
   - Windows/Ubuntu: 차이 없음.
   - 위험: 옛 용어 검사가 정상 문구를 잡을 수 있다(예: "버전 관리 시작"). 제외 규칙을 테스트 안에 명시하고, 목록은 짧게 유지한다.

7. 먹색 모노 토큰과 주 버튼 (`styles: monochrome tokens and graphite primary button`)
   - 변경 파일: `src/styles/tokens.css`(라이트, `prefers-color-scheme: dark` 블록, `[data-theme="dark"]` 블록 세 곳을 위 표로. `--accent-text` 삭제, 새 토큰 추가), `src/ui/Button.module.css`(`.primary`는 `--primary`/`--primary-text`, 호버 `--primary-hover`. `.text`는 `color: var(--text)`), `src/ui/Button.tsx`(주석 `--accent 채움` → `--primary 채움`), `src/styles/contrast.test.ts`(신규, node 환경).
   - `contrast.test.ts`: tokens.css를 읽어 라이트 `:root`와 `:root[data-theme="dark"]` 블록의 색 변수를 뽑고, WCAG 상대 휘도로 대비를 계산한다. 위 "대비 계산 결과" 표의 쌍마다 기준(글자 4.5, `--primary`/`--bg`·`--border-strong`/`--surface`·`--accent`/`--bg` 3.0)을 넘는지 확인. 다크 미디어 쿼리 블록과 `[data-theme="dark"]` 블록의 값이 같은지도 확인(두 곳 복사본이 어긋나지 않게).
   - 테스트: `pnpm test`, `pnpm lint`(stylelint 포함. tokens.css 밖에 색 값이 없는지는 기존 규칙대로).
   - 완료 기준: 대비 테스트 통과. `rg "accent-text" src --glob '*.css' --glob '*.tsx'`가 비어 있다(`contrast.test.ts`는 그 토큰이 없는지 확인하느라 글자를 쓴다). 수동: 두 OS에서 홈의 "폴더 열기" 버튼이 흑연색, Tab 포커스 테두리는 파랑, 라이트/다크 전환.
   - Windows/Ubuntu: Windows 고대비 모드에서 버튼 테두리(지금 `transparent`)가 시스템 색으로 그려지는지 확인. 다크에서 `color-scheme: dark`로 WebView2·WebKitGTK의 스크롤바·입력칸 기본색이 바뀌는지 확인.
   - 위험: 지금 `--accent`를 글자 버튼 색으로 쓰는 곳(`.text`)이 본문색으로 바뀌어 "새로 고침", "다른 폴더 열기"가 덜 눈에 띈다. 의도한 변화다(Q10). 그래프 색 `--graph-1`이 다크 `--surface` 위에서 3.18로 낮지만 노선도 반복에서 다룬다.

8. 이후: 1a의 15·16·17·18·21·22단계를 2단계에서 개정한 001대로 진행한다. 15단계 전에 1~7단계가 끝나 있어야 새 화면 테스트를 새 문구로 한 번만 쓴다.
   - 순서 제안: 1 → 2 → 3 → 4 → 5 → 6 → 7 → (001) 15 → 16 → 17 → 18 → 21 → 22. 2·4·5단계(문서)는 사람이 결정을 확정하는 동안 먼저 해도 되지만, 3·6·7단계는 1단계 뒤에.

## 용어 변경
- spec.md 용어표: 30개 행의 쉬운 이름과 설명을 위 "새 용어표"로(3단계). id·원래 이름·명령·위험도·빈도는 그대로.
- `src/glossary/ko.ts`·`en.ts`: 같은 내용. `types.ts`는 그대로.
- 새로 넣지 않는 것: `sync`(동기화, 1b에서 Q5에 따라), `ours`·`theirs`(반복 2).

## 위험과 열린 질문
- "버전"이 commit 뜻이 되면서 앱 버전·릴리스 버전("v1.0" 태그)과 같은 낱말을 쓴다. 앱 업데이트를 말하는 문구에는 "버전"을 쓰지 않는다(6단계의 `notFastForward`처럼). 태그 설명 "버전에 v1.0 같은 이름표"는 헷갈릴 수 있다(Q21).
- 3단계와 6단계 사이에는 glossary는 새 용어, messages는 옛 용어인 상태가 한 커밋 동안 있다. 화면이 섞여 보일 뿐 테스트는 통과한다. 싫으면 두 단계를 한 커밋으로 합친다.
- 7단계에서 글자 버튼이 파랑에서 본문색으로 바뀌어 기존 화면의 링크처럼 보이던 버튼이 덜 드러난다.
- 대비는 손으로 계산한 값이며 7단계 테스트가 최종 확인이다. 테스트가 다른 값을 내면 테스트를 따른다.

## 결정 결과 (2026-10-15)
- D-024~D-028 확정.
- Q1 cherry-pick "이 버전만 복사하기". Q2 reset "이 버전으로 돌아가기"(revert "되돌리기"와 동사를 나눔).
- Q3 1a에는 화면 다시 읽기 버튼을 두지 않는다. 파일 감시를 쓸 수 없을 때만 안내 줄에 "다시 읽기" 버튼. 상단 바 "새로 고침"은 1b의 fetch.
- Q4 사용자가 고친 호버 설명 17줄 반영(위 표). 원칙: 동작은 "~하기", 대상은 명사, 메모체 금지, 주의는 둘째 문장 "~요".
- Q5 추천안대로(1b 20단계에서 정함). Q6 영어 이름 확정(cherry-pick "Copy this version", reset "Go back to this version"), 설명은 같은 기준.
- Q7 본문 `#24292F`. Q8 `#6A6F75`. Q9 `#2D7A4D`. Q10 글자 버튼은 본문색, 현재 브랜치는 표시와 굵기. Q11 충돌 화면에서만 내 브랜치 초록·합칠 브랜치 파랑 의미색, 노선도 색은 D-017 확정 때 맞춘다.
- Q12 이번에는 작업 공간 칸 모서리만 없앤다. Q13 버전 기록 탭은 목록을 창 전체 폭. Q14 한 줄 설명 넣음. Q15 체크 상자 켜짐은 `--primary`.
- Q16·Q17 D-017·D-006 본문은 고치지 않는다. Q18·Q19·Q20 추천안대로(이번 범위 밖).
- Q21 "태그"는 그대로, 설명 "이 버전에 v1.0 같은 이름 붙이기". "버전"은 commit 한 뜻으로 유지.
- "다음 버전에서 지원해요" 류 문구는 "아직 준비 중이에요"로.
