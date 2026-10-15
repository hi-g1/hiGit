# 제품 명세 (Spec)

## 한 줄 요약
Git을 모르는 사람도 프로젝트 폴더를 열고 버튼 몇 번으로 저장·갈래·합치기·되돌리기·검토 요청을 할 수 있는 데스크톱 앱. Windows와 Ubuntu에서 동작한다.

## 사용자
- **주 사용자**: 코드를 다루지만 Git이 낯선 사람(비전공 개발자, 디자이너, 기획자, 학생).
- **보조 사용자**: Git을 알지만 명령어 대신 빠르고 안전한 화면을 원하는 개발자.

## 원칙
1. **쉬운 말이 먼저, 원래 이름은 호버로.** 모든 Git 용어는 쉬운 이름으로 보이고, 마우스를 올리면 원래 이름·명령·설명이 뜬다(D-006).
2. **망쳐도 돌아올 수 있다.** 위험한 작업은 미리보기 → 백업 → 실행 → "방금 작업 취소" 순서(D-007).
3. **지금 상태가 항상 보인다.** 현재 갈래, 저장 안 한 변경 수, 올리지 않은 저장 수, 받을 저장 수를 상단에 늘 표시.
4. **그림으로 보여준다.** 갈래와 합치기는 선과 점으로 그린다. 끌어놓기로 합치기·가져오기·순서 바꾸기가 된다.
5. **담백하게.** 장식 없는 화면, 회색 바탕에 포인트 색 하나(`docs/design.md`).

## 화면 구성
| 화면 | 들어가는 방법 | 하는 일 |
| --- | --- | --- |
| 홈 | 앱 실행 | 최근 프로젝트, "폴더 열기", "프로젝트 가져오기(clone)", "새로 시작(init)" |
| 작업 공간 | 프로젝트 열기, 또는 폴더 우클릭 "Git으로 관리" | 왼쪽: 갈래 목록. 가운데: 바뀐 파일과 비교. 오른쪽: 저장 상자(메시지 + 저장 버튼) |
| 저장 기록 | 상단 탭 | 갈래 그래프 타임라인. 저장 우클릭으로 되돌리기·가져오기·버전 표시 ([추천 조합](research/git-visualization.md#추천-조합), D-017 제안) |
| 기록 정리 | 저장 기록에서 여러 개 선택 | 순서 바꾸기, 묶기, 메시지 고치기, 빼기(interactive rebase) ([추천 조합](research/git-visualization.md#추천-조합), D-017 제안) |
| 합치기 / 충돌 | 갈래 끌어놓기 | 미리보기, 충돌 시 줄 단위로 내 것/상대 것/둘 다 선택 |
| 검토 요청 | 상단 "검토 요청" | GitHub PR 목록과 상태, 새 요청 만들기 |
| 안전망 | 상단 시계 아이콘 | 최근 작업 목록(reflog + 앱 백업)에서 원하는 시점으로 복구 ([추천 조합](research/git-visualization.md#추천-조합), D-017 제안) |
| 설정 | 톱니 아이콘 | 이름·이메일, GitHub 연결, "Git 원래 이름으로 보기", 테마, 언어 |

## 진입점
- 앱 안의 "폴더 열기".
- 폴더 우클릭 메뉴 "Git으로 관리": Windows 탐색기, Ubuntu Nautilus.
- 명령줄: `higit <폴더경로>`. 이미 실행 중이면 그 창에서 연다(단일 인스턴스).
- 폴더가 Git 저장소가 아니면 "이 폴더를 Git으로 관리할까요?" 확인 후 init.

## 기능 목록과 용어표
이 표가 용어의 **원본**이다. 코드의 `glossary` 데이터는 이 표와 일치해야 한다.

| id | 쉬운 이름 | 원래 이름 · 명령 | 호버 설명 | 위험도 | 빈도 |
| --- | --- | --- | --- | --- | --- |
| status | 바뀐 내용 | status · `git status` | 마지막 저장 이후 바뀐 파일 목록 | safe | 매일 |
| diff | 비교 보기 | diff · `git diff` | 파일이 어떻게 바뀌었는지 줄 단위로 비교 | safe | 매일 |
| stage | 저장에 넣기 | stage · `git add` | 다음 저장에 포함할 파일 고르기 | safe | 매일 |
| commit | 저장하기 | commit · `git commit` | 지금 상태를 설명과 함께 기록으로 남기기 | safe | 매일 |
| fetch | 새 소식 확인 | fetch · `git fetch` | 원격 저장소의 변경을 내려받되 아직 합치지 않음 | safe | 매일 |
| pull | 최신으로 받기 | pull · `git pull` | 원격의 변경을 받아 내 갈래에 합치기 | normal | 매일 |
| push | 올리기 | push · `git push` | 내 저장들을 원격 저장소로 보내기 | normal | 매일 |
| branch | 작업 갈래 | branch · `git branch` | 원래 작업에 영향 없이 따로 작업하는 길 | safe | 매일 |
| switch | 갈래 바꾸기 | switch · `git switch` | 다른 작업 갈래로 이동 | normal | 매일 |
| log | 저장 기록 | log · `git log` | 지금까지의 저장 목록 | safe | 매일 |
| merge | 합치기 | merge · `git merge` | 다른 갈래의 작업을 지금 갈래에 합치기 | normal | 자주 |
| conflict | 겹친 부분 고르기 | merge conflict | 두 갈래가 같은 곳을 다르게 고쳤을 때 고르기 | normal | 자주 |
| stash | 잠시 치워두기 | stash · `git stash` | 하던 작업을 서랍에 넣었다가 나중에 꺼내기 | safe | 자주 |
| discard | 변경 취소 | restore · `git restore` | 저장 안 한 변경을 마지막 저장 상태로 되돌리기 | caution | 자주 |
| pr | 검토 요청 | Pull Request | 내 갈래를 합쳐 달라고 팀에 요청 | safe | 자주 |
| clone | 프로젝트 가져오기 | clone · `git clone` | 원격 저장소를 내 컴퓨터로 복사 | safe | 자주 |
| init | 관리 시작 | init · `git init` | 이 폴더를 Git으로 관리하기 시작 | safe | 자주 |
| amend | 방금 저장 고치기 | amend · `git commit --amend` | 마지막 저장의 설명이나 내용 고치기 | caution | 가끔 |
| revert | 기록 남기고 취소 | revert · `git revert` | 취소했다는 새 저장을 만들어 안전하게 되돌리기 | normal | 가끔 |
| reset | 기록째 되돌리기 | reset · `git reset` | 특정 시점으로 갈래를 옮기기. 이후 저장은 기록에서 사라짐 | danger | 가끔 |
| rebase-i | 기록 정리하기 | interactive rebase · `git rebase -i` | 저장 순서 바꾸기, 묶기, 메시지 고치기 | danger | 가끔 |
| rebase | 갈래 옮겨 심기 | rebase · `git rebase` | 내 갈래를 최신 갈래 끝으로 옮기기 | caution | 가끔 |
| cherry-pick | 저장 하나만 가져오기 | cherry-pick · `git cherry-pick` | 다른 갈래의 저장 하나를 복사해 오기 | normal | 가끔 |
| tag | 버전 표시 | tag · `git tag` | 저장에 "v1.0" 같은 이름표 달기 | safe | 가끔 |
| blame | 누가 언제 바꿨나 | blame · `git blame` | 각 줄을 마지막으로 바꾼 사람과 시점 | safe | 가끔 |
| reflog | 안전망 | reflog · `git reflog` | 최근 작업 이력. 지운 것도 여기서 복구 | safe | 가끔 |
| force-push | 강제로 올리기 | force push · `git push --force-with-lease` | 원격 기록을 내 기록으로 덮어쓰기 | danger | 가끔 |
| remote | 연결된 저장소 | remote · `git remote` | 이 프로젝트가 연결된 원격 저장소 목록 | normal | 드묾 |
| ignore | 저장 안 할 파일 | .gitignore | Git이 무시할 파일 규칙 | safe | 드묾 |

위험도:
- `safe`: 바로 실행.
- `normal`: 바로 실행, 실패 시 쉬운 말로 이유와 해결 버튼.
- `caution`: 미리보기 후 실행, 실행 뒤 "방금 작업 취소".
- `danger`: 미리보기 + 백업 + 한 번 더 확인. 원격에 올린 기록이 바뀌면 추가 경고.

## 오류 메시지 원칙
- Git 원문 오류를 숨기지 않되 접어둔다("자세히 보기").
- 먼저 쉬운 말로 무슨 일인지, 그다음 할 수 있는 버튼 1~2개.
  - 예: "올리기 전에 최신 내용을 먼저 받아야 해요. [최신으로 받기]"

## 범위 밖 (지금은 하지 않음)
- submodule, LFS 관리 화면, bisect, worktree 관리.
- GitHub 외 호스팅(GitLab 등)은 구조만 열어두고 구현은 나중에.
- 코드 편집기 기능.

## 열린 질문
- 앱 이름 `hiGit`이 이미 쓰이는 제품과 겹치는지 공개 전에 확인(D-009).
- GitHub 로그인 방식: OAuth device flow(추천) vs 개인 토큰 입력.
