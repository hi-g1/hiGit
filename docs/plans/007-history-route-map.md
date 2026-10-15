# 007 버전 기록 노선도 (반복 2b)
- 관련 결정: D-017(확정, 이 반복의 근거), D-003, D-010, D-019, D-026, D-031, D-035, D-037, D-038, D-045, D-046, D-049, D-057, D-058, D-061, D-065, D-066, D-073, D-085. 새로 정할 것은 아래 "결정할 것" C1~C9이고, 구현 전에 Claude가 `docs/decision.md` 맨 아래에 D-087부터 "(자동 결정)"으로 올린다(M1).
- 관련 명세: spec.md "화면 구성"의 버전 기록·시작 화면 행, 원칙 4("그림으로 보여준다"). architecture.md "버전 기록 그리기 계층", "GitBackend", "Query", "프론트와 백엔드 계약", "성능 기준". design.md "홈 노선도", "시작 화면 색", 색 토큰 `--graph-1`~`--graph-6`.
- 참고 자료(결정 아님): `docs/research/git-visualization.md` "추천 조합"(① 노선도 + ⑧ 확대·축소 + ⑤ 선 굵기), `docs/research/img/flow4/r-log.png`·`mockups/home/r-log.html`(버전 기록 줄 배치와 역 메뉴), `r-home.png`(홈 두 줄), `img/start-normal.png`·`start-normal-dark.png`(시작 화면 큰·작은 노선도), `quick-actions.md`(역 메뉴 "더 보기"), 계획 003 "나중 단계" N1~N3, 계획 005 H17·H18.
- 앞선 계획: 006(반복 2, 끝남), 005(홈 두 줄 노선도 `layoutTwoLine`·`SvgRenderer`가 이 계획의 출발점), 003(시작 화면). 진행: 사람 확인 없이 Claude가 계획 → code-analyst 검토 → 아래 순서대로 구현, 한 단계 = 한 커밋.

## 목표
버전 기록 화면이 목록이 아니라 지하철 노선도가 된다. 브랜치는 노선, 버전은 역, 합치기는 큰 점(환승역), 지금 위치는 바깥 고리로 그리고, 지도처럼 확대·축소(모든 버전 / 주요 버전 / 태그만)하며, 선 굵기가 그 버전의 변경량을 보여준다. 역을 누르거나 키보드로 고르면 작은 메뉴가 뜬다. 시작 화면의 "마지막으로 하던 곳"과 프로젝트 목록에도 같은 그리기 코드로 큰·작은 노선도가 나온다. 반복 3이 역 메뉴에 기능을 채울 틀이 이 반복에서 끝난다.

## 범위
- 버전 기록 노선도: 레인 배치, 가상 스크롤, 확대·축소 3단계, 선 굵기, 브랜치 범위 전환, 키보드·스크린 리더, 역 메뉴 틀.
- 시작 화면: 큰 노선도(브랜치 색), 작은 노선도(회색)(계획 003 N1~N3).
- 홈 두 줄 노선도: 공용 장면(Scene)·Renderer 위로 옮기고, 반복 2 끝 점검에서 넘어온 "첫 역 이름이 왼쪽 끝에 붙음"을 고친다.
- 다크 모드 그래프 색(계획 003 N2가 "노선도 반복에서 정한다"고 넘긴 것)과 대비 테스트.

## 범위 밖
"이번 반복에서 하지 않는 것" 절에 모았다.

## 현재 상태
이 계획이 기대는 사실만 적는다.
- `src/screens/History/History.tsx`는 `useLog`(`HISTORY_LIMIT` 200, `skip: 0` 한 쪽)로 받은 `toHistoryRows(log)`를 `HistoryList`(단순 목록)로 그린다. Esc는 window keydown에서 `!e.defaultPrevented`면 홈으로 간다.
- `core::view::LogQuery { skip, limit }`, `LogPage { commits, has_more }`, `CommitRow`(oid, short_oid, parents, summary, author, 시각)가 있고 `Queries::log`는 `MAX_LOG_PAGE`(500)로 자르며 시작점은 늘 HEAD다. `git::LogQuery { skip, limit, rev: Option<String> }`의 `log`는 `--max-count`·`--skip`만 쓰고 `--topo-order`는 없다. 브랜치 끝점은 `GitBackend::branches()`(`Branch.oid`)가 이미 주고, 태그를 읽는 메서드는 없다. 줄 수(변경량)를 읽는 메서드도 없다.
- 홈 두 줄 노선도: `core::query::home_map` → `HomeMapView`(current·default_branch·two_lines·current_commits 최대 3·main_since·upstream·changed_files), `src/features/history/layout/twoLine.ts`의 `layoutTwoLine(view, width, labels) → MapScene { width, lines, stations, texts, tag, folded }`(픽셀 좌표, 한 줄 `MAP_HEIGHT` 104), `render/Renderer.ts`의 `Renderer = ({ scene, colors: { main, current }, label }) => ReactElement`, `SvgRenderer.tsx`(선 굵기 main 3·current 4, 역 r 6, "지금 여기" 점선 원 r 9, 글자는 `--text`·`--text-muted`). 배치는 `HomeMapView`에 묶여 있어 일반 그래프를 받지 못한다. 한 줄일 때 첫 역이 x=`LEFT`(24)에 놓이고 이름이 가운데 정렬이라(최대 14자, 한글 약 12px/자) 이름 왼쪽이 0 밖으로 나가 `.route { overflow: hidden }`에 잘린다.
- 색: `branchColor(name, defaultBranch)`(기본 브랜치 `--graph-1`, 나머지 이름 해시로 `--graph-2`~`6`). `--graph-*`는 라이트 값만 있고 다크 덮어쓰기가 없다. `contrast.test.ts`는 그래프 색을 확인하지 않는다.
- 시작 화면: `screens/Start/{Start, LastProject, ProjectRow}.tsx`에 노선도 자리가 없다. `Queries::project_summary`가 최근 목록에 있는 프로젝트를 열지 않고도 읽고(`listed` 확인) 신뢰 확인을 거친다. 같은 접근 규칙의 노선도 조회는 없다.
- 단축키 표(`lib/shortcuts.ts`)의 `scope`는 `global | field | fileList | conflict`. 웹뷰 예약 키에 Ctrl+=·Ctrl+−·Ctrl+0이 있다. 목록 안 방향키·Enter·Esc는 표 밖의 부품 조작이다(반복 1b 점검 결정).
- 새 의존성 없이 간다: 가상 스크롤은 직접 만들고(행 높이 고정), 메뉴는 이미 있는 `@radix-ui/react-dropdown-menu`, 쪽 넘기기는 `@tanstack/react-query` v5의 `useInfiniteQuery`.

## 결정할 것
구현 전(M1)에 Claude가 `docs/decision.md`에 D-087부터 "(자동 결정)"으로 올린다(상태 확정). 추천안을 적었고, 기존 확정 결정을 바꾸지 않는다(근거 열).

| 번호 | 결정(추천안) | 까닭 | 대안 | 근거·부딪힘 확인 |
| --- | --- | --- | --- | --- |
| C1 (D-087) | 확대·축소는 세 단계 "모든 버전 / 주요 버전 / 태그만". 주요 버전 = 합치기 버전·갈라진 곳(자식 둘 이상)·브랜치 끝·태그·지금 위치·첫 버전. 태그만 = 태그·브랜치 끝·지금 위치·첫 버전. 접힌 곳은 "버전 N개" 한 줄(역 아닌 작은 마름모)로 두고 그 줄을 누르면(Enter) 그 구간만 펼친다. 단계는 백엔드가 걸러 주지 않고, 받은 버전을 화면의 순수 함수가 접는다(`LogQuery`는 단계를 받지 않는다). "지금 여기"와 지금 위치 역은 모든 단계에서 남는다. architecture.md의 "확대·축소 단계별로 필요한 저장만 백엔드에서 걸러 받는다(`LogQuery`)"와 `LogQuery` 문장은 이 결정에 맞게 고친다. | 접으려면 부모·자식 관계가 필요해 어차피 그래프를 받아야 한다. git의 `--simplify-by-decoration`은 `%P`가 다시 쓰인 부모를 주는지가 버전마다 다르고 refs/higit·stash도 꾸밈으로 잡아 믿기 어렵다. 접힌 구간 펼치기는 비교 화면의 "변경 없는 12줄 · 펼치기"와 같은 감각이다. | 단계별 백엔드 조회(위 불확실성, 단계마다 쪽 넘기기 상태가 따로), 두 단계만(research ⑧은 세 단계) | D-017(단계 이름은 research ⑧), D-049(이름과 숫자만), D-003(기계용 출력만) |
| C2 (D-088) | 버전 기록의 기본 범위는 "모든 브랜치"(지금 위치 + 로컬 브랜치 끝 최대 200개, 최근에 고친 것 먼저), 전환하면 "지금 브랜치만"(지금 위치 한 곳에서의 조상). 원격에만 있는 브랜치는 그리지 않는다. 쪽 넘기기는 첫 쪽이 정한 끝점 목록(`tips`, 해시만)을 되돌려 보내 같은 모양으로 이어 받는다. 200개를 넘겨 못 그린 브랜치 수는 알려 준다. `--all`은 쓰지 않는다. | r-log 머리의 "모든 브랜치"가 이 전환이다. `--all`은 `refs/higit/backup`·`parked`·`refs/stash`의 버전까지 그려 노선도를 어지럽힌다. 끝점을 해시로만 넘기면 ref 이름이 명령줄에 닿지 않아 옵션 오해가 없다(D-035). 쪽 사이에 ref가 움직여도 건너뛰거나 겹치지 않는다. | `git log --branches --remotes`(다른 사람 브랜치 수백 개), 쪽마다 끝점을 다시 계산(쪽 사이 어긋남) | D-035, D-058 |
| C3 (D-089) | 선 굵기 = 그 역에서 바뀐 줄 수(추가+삭제, 이진은 0줄)를 로그 눈금으로 2~5px(`round(2 + log10(1+줄))`, 최대 5), 지금 브랜치의 선은 1px 더 굵게(최대 6). 합치기 버전과 아직 모르는 버전은 2px. 접힌 구간은 담은 버전들의 합. 줄 수는 `query_log_stats`로 보이는 곳만 늦게 받아 붙인다(받기 전에는 가늘게 그린다). 홈 두 줄 노선도는 굵기를 변경량에 묶지 않고 지금처럼 3·4로 고정한다. | research ⑤의 "굵기에 상한을 두고 로그 눈금". 줄 수는 버전마다 diff를 계산해 느리므로 첫 화면(300ms)에 얹지 않는다. design.md의 "지금 브랜치는 굵기로 구분"을 지킨다. | 파일 수로 굵기(변경 크기를 못 담음), 로그 쪽에 `--numstat`을 함께(첫 화면이 큰 커밋에 막힘) | D-017, D-026(현재 브랜치는 굵기로 구분) |
| C4 (D-090) | 홈 두 줄 노선도는 입력이 고정된 요약(`HomeMapView`)이라 자체 배치 `layoutTwoLine`을 유지하되, 장면 타입(`Scene`)·`Renderer`·`SvgRenderer`·경로·글자 도우미·색 키 규칙은 버전 기록·시작 화면과 하나를 쓴다. 전체 노선도(버전 기록, 시작 화면)는 새 배치 `layoutRoute` 하나가 세로·가로, 큰 것·작은 것을 모두 만든다. | 홈에는 해시로 이어진 부모 관계가 없고 "…"로 접기·점선 "지금" 역 같은 요약 전용 요소가 있어, 일반 그래프 배치에 억지로 넣으면 가짜 노드가 생기고 반복 1b에서 끝난 화면이 회귀한다. 그리기·클릭 판정이 하나라 Canvas 전환도 한 곳이다. 로드맵의 "넓혀 쓴다"를 장면·Renderer 수준에서 지킨다. | 홈도 `layoutRoute`로(가짜 노드, 회귀 위험), 따로 두기(그리기 코드 두 벌) | D-046, D-061, D-017 |
| C5 (D-091) | 노선도 그리기 규칙. 0번 레인은 기본 브랜치 끝에서 시작하는 선(없으면 지금 브랜치). 레인 색은 그 레인이 시작되는 끝의 브랜치 이름으로 `branchColor`, 이름 없는 레인(지운 브랜치)은 `--text-muted`. 역 = `--surface` 채움 + 레인 색 테두리, 합치기 버전은 더 큰 점(환승역), 지금 위치는 `--text` 바깥 고리(r-log). 선은 세로(또는 가로)와 45도 근처만, 꺾이는 곳은 한 줄 높이 안. 세로 노선도: 행 높이 48px, 레인 간격 24px(레인이 많으면 8px까지 줄이고 그래프 폭 최대 240px, 24레인을 넘으면 마지막 칸에 겹쳐 그림), 첫 레인 x=40. 글자는 모두 `--text`·`--text-muted`(D-031), 이름표 앞 작은 점만 브랜치 색. "안 올림"은 지금 브랜치의 안 올린 버전에만 붙인다. | r-log 그림의 값(레인 x 40·64·88, 역 r 5)을 4px 단위에 맞췄다(그림은 46px 행). 이름 있는 레인만 색을 주면 같은 브랜치는 어느 화면에서나 같은 색(design.md). | 레인 순서대로 색(스크롤·범위 전환마다 색이 바뀜), 원격 브랜치도 "안 올림" | D-031, D-026, design.md "브랜치마다 색" |
| C6 (D-092) | 역 메뉴 틀: 항목은 `features/history/menu/items.ts`의 표 하나(id, 이름표, 묶음 `main\|more`, `availability(info) → show\|hide\|disabled(까닭)`, `run(info, ctx)`)로 정하고, 고른 역의 정보는 `StationInfo`(해시·짧은 해시·이름·작성자·시각·부모·합치기 여부·첫 버전 여부·지금 위치 여부·지금 위치의 조상 여부·브랜치 이름들·태그들·안 올림 여부) 하나로 넘긴다. 2b에서 실제 항목은 "버전 번호 복사" 하나이고, 반복 3의 항목(되돌리기, 이 버전으로 돌아가기, 마지막 버전 고치기, 이 버전만 복사하기, 태그, 여기서 새 브랜치)은 표에 아직 없으므로 보이지 않는다(비활성으로 미리 보이지 않는다). "더 보기"는 그 묶음에 항목이 있을 때만 생긴다. r-log의 "이때 파일 보기"는 로드맵에 없어 반복 3 계획에서 넣을지 정한다. 반복 3의 액션은 `StationInfo.oid`를 `ActionInput`에 담아 보내고 core가 해시를 다시 확인한다(D-085와 같은 방식). 조작: 목록 하나가 포커스를 받고(`role="listbox"`, `aria-activedescendant`), ↑↓·Home·End·PageUp·PageDown으로 역 이동, Enter·Space·ContextMenu 키·Shift+F10으로 메뉴 열기, Esc는 메뉴만 닫는다. 선택은 하나(여러 개 선택은 기록 정리가 열릴 때 반복 3). | 가상 스크롤에서는 행이 사라지므로 포커스를 행이 아니라 목록에 두는 것이 접근성 표준 패턴이다. 누를 수 없는 항목을 미리 보이면 D-033(복제 숨김)과 어긋난다. 항목 하나라도 있어야 틀을 끝까지 시험하고 두 OS에서 메뉴·포커스를 확인할 수 있다. | 항목 0개로 메뉴 틀만(열면 빈 메뉴), 반복 3 항목을 꺼 두고 보이기 | D-033, D-043, D-085, D-049 |
| C7 (D-093) | 시작 화면 노선도 데이터는 새 조회 `query_route_preview(repo, limit)` 하나다(계획 003 N1을 이 이름으로). 끝점은 지금 위치와 기본 브랜치 끝, 최근 버전 `limit`개(큰 것 12, 작은 것 10)를 `--topo-order`로. 접근·신뢰 확인은 `project_summary`와 같은 규칙(최근 목록에 있으면 열지 않아도, 신뢰하지 않은 저장소는 git을 돌리지 않음). 줄은 보일 때만 불러오고 동시에 3개까지, 상태가 성공인 프로젝트만. | 사라진 폴더·저장소 아님·신뢰 필요 줄에서 git을 부르지 않는다. 20개 프로젝트의 git을 한꺼번에 띄우면 Windows에서 느리다. | 프로젝트마다 `home_map` 재사용(대표 버전 이름이 모자람), 한꺼번에 불러오기 | D-031, D-067, 계획 003 N1~N3 |
| C8 (D-094) | 다크 그래프 6색을 `#7fa7d6`, `#6bbf8a`, `#d9a54a`, `#b59ae0`, `#5fbcc4`, `#c9967a`(순서대로 `--graph-1`~`--graph-6`)로 하고 `contrast.test.ts`가 라이트·다크 모두 `--bg`·`--surface`에서 3:1 이상(그래픽 기준)인지 확인한다. 못 넘는 값은 명도를 올려 다시 고른다. 글자에는 계속 쓰지 않는다. | 계획 002 Q10과 003 N2가 "다크 그래프 값은 노선도 반복에서 정한다"고 넘겼다. `--accent`·`--added`·`--modified` 다크 값을 이어 받아 틀과 어울린다. | 라이트 값 그대로(다크 `--bg` 대비 3.1~3.5로 약함) | D-026, D-031 |
| C9 (D-095) | 확대·축소 조작: 화면 안에서 `+`·`-`(숫자패드 포함, 단축키 표에 `scope: "history"`로), Ctrl+휠·터치패드 핀치, 헤더의 "−"·"+" 단추. Ctrl+=·Ctrl+−는 웹뷰 예약이라 쓰지 않는다. 고른 단계·범위는 앱을 켜는 동안만 기억하고(`historyStore`), 설정 화면이 생기면 설정으로 옮긴다. | 예약 키를 피하고 지도 앱의 익숙한 조작을 따른다. Tauri 창은 기본으로 웹뷰 확대 단축키·휠 확대를 끄므로 앱이 Ctrl+휠을 받는다(두 OS에서 손으로 확인). | 단계를 저장(저장 자리가 아직 없음) | D-033, D-056 |

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04 git 2.34.1, Windows 11)에서 손으로, 가능한 것은 자동 테스트로 확인한다.
- 상단 바 "버전 기록"(Ctrl+H)을 열면 r-log처럼 왼쪽에 레인과 역, 오른쪽에 이름(안 올림·태그·브랜치 이름이 이름 옆에 옅게)·작성자·날짜가 있는 노선도가 나온다. 기본 브랜치가 가장 왼쪽 선이고 선 색은 브랜치마다 같다. 합치기 버전은 큰 점, 지금 위치는 바깥 고리, 지금 브랜치의 안 올린 버전에 "안 올림"이 있다. 다크·라이트 모두 선이 배경에서 구분된다.
- 헤더의 "−"·"+"(또는 `+`·`-`, Ctrl+휠)로 "모든 버전 → 주요 버전 → 태그만"으로 줄고 늘며, 접힌 구간은 "버전 N개" 한 줄이고 Enter로 펼쳐진다. 어느 단계에서도 지금 위치가 보이고 "지금 여기" 단추가 거기로 옮긴다.
- 변경이 큰 버전의 선이 굵다(2~5px, 지금 브랜치는 한 칸 더). 줄 수는 보이는 곳만 늦게 도착하고 그동안 화면이 멈추지 않는다.
- "모든 브랜치 / 지금 브랜치만" 전환이 된다. 쪽을 넘겨도(스크롤) 선이 끊기거나 같은 버전이 두 번 나오지 않고, 그 사이 새 버전이 생겨도 어긋나지 않는다(감시가 목록을 처음부터 다시 받는다).
- 버전 1만 개 저장소에서 첫 화면이 300ms 안에 나오고(첫 쪽 200개), 스크롤하는 동안 DOM의 행은 60개 이하이며, 배치 계산은 1만 행에 50ms 안팎이다.
- 키보드만으로: 목록에 Tab으로 들어가 ↑↓로 역을 옮기고, Enter로 메뉴를 열고 "버전 번호 복사"를 고르면 클립보드에 해시가 들어가고 토스트가 뜬다. Esc는 메뉴만 닫고(메뉴가 닫혀 있으면 홈으로). 스크린 리더가 줄마다 "이름, 작성자, 날짜, 브랜치, 태그, 지금 여기, 합치기, 안 올림, N번째"를 읽는다.
- 시작 화면: "마지막으로 하던 곳"에 start-normal.png처럼 큰 노선도(main·브랜치, 역 이름, 끝에 "지금 여기")가, "다른 프로젝트" 줄마다 회색 작은 노선도가 보이고, 줄이 화면에 들어올 때만 불러온다. 사라진 폴더·저장소 아님·신뢰 필요 줄은 노선도 없이 지금처럼 보인다.
- 홈 두 줄 노선도의 한 줄 보기(저장 몇 개뿐인 프로젝트 포함)에서 첫 역 이름이 왼쪽 끝에 붙거나 잘리지 않는다. 두 줄 보기는 이전과 같은 모양이다.
- 800px 너비와 다크 모드에서 세 화면(버전 기록·홈·시작) 모두 글자가 겹치거나 잘리지 않는다(긴 이름은 말줄임).
- `pnpm check:full` 통과, 두 OS CI 초록(`git-min` 포함). code-analyst 점검 위반 0건. 화면에서 git·`invoke` 직접 호출 없음, 색 값은 `tokens.css` 밖에 없음, 새 의존성 없음.

## 단계
각 단계 = 커밋 하나, 메시지는 `<영역>: <요약>`. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`. 새 Tauri 명령은 `src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣는다(`tests/security_config.rs`가 확인). `GitBackend`에 메서드를 더하면 가짜 구현 둘(`executor.rs` 테스트의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)도 같은 커밋에서 고친다. 새 git 명령은 `git-min`(Ubuntu 22.04, 2.34.1)에서 확인한다. 테스트 저장소는 `core.autocrlf false`·`user.name`·`user.email`을 저장소에 넣고 파일 이름에 Windows 금지 글자(`*?:<>|"`)를 쓰지 않는다. 용어 호버는 `<Term>`, 문장 속 용어는 `t(id)`(D-029). 계획 003의 N1은 M6, N2는 M14, N3는 M15다.

### A. 결정·토큰

M1. 문서·결정·다크 그래프 색 (`docs: add iteration 2b decisions and dark graph colors`)
- 목표: 이후 단계가 쓰는 결정을 먼저 올리고 다크 그래프 색과 그 대비 테스트를 넣는다.
- 바꿀 파일: `docs/decision.md`(C1~C9를 D-087~D-095 "(자동 결정)"으로, 상태 확정), `docs/architecture.md`("버전 기록 그리기 계층"의 "단계별로 백엔드에서 걸러 받는다" 문장과 "LogQuery 단계" 문장을 C1에 맞춤. 나머지는 각 단계에서 코드와 함께), `docs/design.md`(색 토큰 절에 다크 그래프 값, 새 부품 규칙 "버전 기록 노선도"·"시작 화면 노선도"는 M11·M14에서 코드와 함께 채움), `src/styles/tokens.css`(`:root:not([data-theme="light"])` 미디어 블록과 `:root[data-theme="dark"]` 둘 다에 `--graph-1`~`6` 다크 값), `src/styles/contrast.test.ts`(`--graph-1`~`6` / `--bg`·`--surface` 3:1, 라이트·다크).
- 내용: 다크 값은 C8의 후보. 테스트가 통과하지 못하는 값만 명도를 올린다. 충돌 화면의 `--conflict-*-bg` 위 글자 대비 테스트(기존)는 그대로 통과해야 한다(그래프 색은 글자가 아니다).
- 테스트: `src/styles/contrast.test.ts`(새 쌍 12개), `pnpm lint`(stylelint: tokens.css 밖 색 값 없음).
- 완료 기준: `pnpm test`, `pnpm lint`.
- Windows/Ubuntu: 차이 없음. 다크에서 두 웹뷰의 색 모양은 M16 손 확인.
- 보안: 해당 없음.

### B. 데이터 (git·core·app)

M2. git: 여러 끝점의 버전 기록과 태그 (`git: add graph log and tag list`)
- 목표: 노선도가 쓸 "여러 끝점에서 위상 순서로" 읽기와 태그 목록을 `GitBackend`에 둔다. 기존 `log`는 건드리지 않는다.
- 바꿀 파일: `crates/git/src/types.rs`(`GraphLogQuery { tips: Vec<Oid>, exclude: Vec<Oid>, skip: u32, limit: u32 }`, `TagRef { name: String, commit: Oid }`), `backend.rs`(`log_graph(&self, repo, &GraphLogQuery) -> Result<Vec<Commit>>`, `tags(&self, repo) -> Result<Vec<TagRef>>`), `cli/mod.rs`, `cli/log.rs`(`FORMAT`·`parse` 재사용), `cli/branches.rs`(태그 형식·파서), `lib.rs`(내보내기), 가짜 구현 둘, `docs/architecture.md` GitBackend 목록.
- 내용:
  - `log_graph`: `git log --topo-order --skip=N --max-count=M <FORMAT>` 뒤에 `push_revs`로 `tips`의 해시들과 `^<exclude 해시>`들을 넘긴다. 끝점·제외 모두 `Oid`(16진수만 받는 타입)라 옵션으로 읽힐 수 없다. `tips`가 비면 빈 목록(HEAD로 대신하지 않는다). 끝점이 201개를 넘으면 `GitError`가 아니라 호출 쪽(core)이 막는다.
  - `tags`: `for-each-ref --format=%(refname)%00%(objectname)%00%(*objectname) refs/tags`(주석 태그는 `*objectname`이 가리키는 버전). 커밋을 가리키지 않는 것은 못 가린다(화면에서 해당 해시 줄이 없으면 무시). 최대 1000개, 이름순.
- 테스트(실제 git): 위상 순서(갈라졌다 합친 기록에서 자식이 부모보다 먼저), `skip`·`limit` 이어 읽기가 겹치거나 빠지지 않음, 끝점 둘, `exclude`로 안 올린 버전만, 빈 끝점 → 빈 목록, 저장 없는 저장소, 한글 이름 태그·주석 태그가 가리키는 버전·가벼운 태그·1000개 한도. `git-min`에서 `^<해시>`가 `--end-of-options` 뒤에서 제외로 읽히는지.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`, `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음(출력 형식은 `LC_ALL=C`·`core.quotepath=off` 공통 설정). 큰 기록의 `--topo-order`는 전체를 훑으므로 느릴 수 있다(위험 절).
- 보안: 해시만 인자로(D-035). 셸 없음.

M3. git: 버전별 변경 줄 수 (`git: add commit stats`)
- 목표: 선 굵기에 쓸 버전별 변경 줄 수를 기계용 출력으로 읽는다.
- 바꿀 파일: `crates/git/src/types.rs`(`CommitStat { oid: Oid, files: u32, added: u32, removed: u32 }`), `backend.rs`(`commit_stats(&self, repo, oids: &[Oid]) -> Result<Vec<CommitStat>>`), `cli/mod.rs`, 신규 `cli/stats.rs`(파서), 필요하면 `cli/runner.rs`(stdin과 출력 상한을 함께 쓰는 변형), 가짜 구현 둘, architecture.md.
- 내용: 해시를 stdin으로 넘기고(`--stdin`) `log --no-walk=unsorted --numstat -z --no-renames --format=%x1e%H`로 버전마다 `추가\t삭제\t경로\0`를 읽는다(먼저 이 방식으로 시험하고 2.34.1에서 안 되면 `diff-tree --stdin --root --numstat -z --no-renames --pretty=format:%x1e%H`). 이진은 `-\t-`라 줄 0·파일 1. 합치기 버전은 줄 0(바뀐 줄을 재지 않음, C3). 출력은 2MB로 자르고(`run_capped` 방식) 잘리면 파싱된 앞부분만 돌려준다(뒤의 버전은 결과에 없다). 한 번에 최대 100개는 호출 쪽(core)이 지킨다.
- 테스트: 고정 출력 파서(경로에 탭·줄바꿈·한글, 파일 0개인 빈 버전, 이진), 실제 git으로 추가·수정·삭제·이진·첫 버전(`--root`)·합치기(0)·빈 버전, 100개 한 번, 잘린 출력, `git-min`.
- 완료 기준: `cargo test -p git` 두 OS와 `git-min`.
- Windows/Ubuntu: `-z` 출력이라 경로 구분자와 줄바꿈 차이가 없다.
- 보안: 해시는 `Oid`만. 파일 경로는 읽지 않고 개수만 센다(결과에 경로가 없다).

M4. core: 버전 기록 조회 확장 (`core: log with scope, tips snapshot and labels`)
- 목표: `query_log`가 모든 브랜치 범위·이름표·안 올림·끝점 스냅샷을 준다.
- 바꿀 파일: `crates/core/src/view/mod.rs`(`LogScope { All, Current }`(기본 All, camelCase), `LogQuery { skip, limit, #[serde(default)] scope, #[serde(default)] tips: Option<Vec<String>> }`, `LogPage`에 `labels: Vec<RefLabelView>`·`tips: Vec<String>`·`tips_cut: u32`·`head`·`current_branch`·`default_branch`·`unpushed: Vec<String>` 추가, `RefLabelView { oid, kind: Branch\|Tag, name }`), `crates/core/src/query.rs`(`log` 다시 씀, 기본 브랜치는 기존 `default_branch` 도우미), `src-tauri/src/commands.rs`(시그니처 그대로), 생성물(`src/lib/bindings.ts`), `src/lib/queries.ts`는 M12에서, `docs/architecture.md` Query 목록.
- 내용:
  - 첫 쪽(`tips: None`): HEAD 해시와(`All`이면) 로컬 브랜치 끝 해시를 모아 중복을 없애고 최근에 고친 순으로 200개까지(`tips_cut` = 못 넣은 수). `Current`면 HEAD 하나. 이 목록을 `LogPage.tips`로 돌려준다. 다음 쪽 요청이 `tips: Some`을 보내면 그대로 쓴다(각 항목은 `Oid::parse`, 201개 넘으면 `InvalidInput`).
  - 읽기: `log_graph(tips, exclude: [], skip, limit + 1)`으로 `has_more` 판정, 한도는 기존 `MAX_LOG_PAGE`.
  - `labels`: 이 쪽 버전에 붙은 로컬 브랜치 이름과 태그만(`branches()`의 `oid`, `tags()`). 한 버전에 이름이 여럿이면 모두.
  - `unpushed`: 지금 브랜치에 upstream이 있으면 `log_graph(tips: [HEAD], exclude: [upstream 해시], limit 501)`의 해시 중 이 쪽 버전에 든 것. upstream이 없으면 빈 목록. 저장이 없는 저장소는 빈 쪽.
  - 이전 화면(`{ skip, limit }`만 보냄)도 그대로 읽힌다.
- 테스트: 가짜 백엔드와 실제 git으로 `All`이 다른 브랜치 버전을 포함하고 `Current`는 조상만, 끝점 스냅샷을 되돌려 보내면 그 사이 새 버전이 생겨도 둘째 쪽이 첫 쪽과 겹치거나 빠지지 않음, 라벨은 그 쪽 버전만, 브랜치 201개에서 `tips_cut`, `unpushed`가 `rev-list upstream..HEAD`와 같음·upstream 없음·떨어진 HEAD(`current_branch: None`), 잘못된 `tips`(비 16진·201개 초과)는 `InvalidInput`, `limit`이 500에서 잘림, 열지 않은 저장소는 `NotOpened`.
- 완료 기준: `cargo test --workspace`, 바인딩 최신, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음.
- 보안: 화면이 보낸 끝점은 해시 형식과 개수만 믿고 명령줄에는 `Oid`로만 닿는다. 이름은 읽기 전용 라벨로만 돌려보낸다.

M5. core·app: 버전별 줄 수 조회 (`core: add log stats query`)
- 목표: 화면이 보이는 곳의 줄 수만 늦게 받는다.
- 바꿀 파일: `crates/core/src/view/mod.rs`(`CommitStatView { oid, files, added, removed }`), `query.rs`(`log_stats(repo, oids: Vec<String>) -> Result<Vec<CommitStatView>>`), `src-tauri/src/{commands.rs, bindings.rs}`(`query_log_stats`), `build.rs` `COMMANDS`, `capabilities/default.json`(`allow-query-log-stats`), 생성물, `src/lib/api.ts`(`logStats`), `docs/architecture.md`(명령 목록, Query 목록, 입력 상한: 줄 수 조회 한 번에 100개).
- 내용: 열린 저장소만(`opened_repo`). 각 해시는 `Oid::parse`, 100개 넘으면 `InvalidInput`. 결과에 없는 해시(합치기 아님인데 못 읽음·잘림)는 화면이 "모름"으로 두고 다시 묻지 않는다(응답에 없다는 사실만으로).
- 테스트: 열지 않은 저장소 거절, 101개 거절, 잘못된 해시 거절, 실제 git으로 값 확인, 보안 설정 테스트(`tests/security_config.rs`).
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 차이 없음.
- 보안: 읽기 전용, 경로·내용을 돌려주지 않고 숫자만. IPC 새 명령이므로 architecture.md 보안 표의 해당 줄 확인.

M6. core·app: 시작 화면 노선도 조회 (`core: add route preview query`) — 계획 003 N1
- 목표: 최근 프로젝트의 최근 버전 몇 개와 갈라진 브랜치 하나를 한 번에 준다.
- 바꿀 파일: `crates/core/src/view/route.rs`(신규 `RoutePreviewView { commits: Vec<RouteCommit { oid, parents, summary }>, labels: Vec<RefLabelView>, head: Option<String>, current_branch: Option<String>, default_branch: Option<String>, has_more: bool }`), `query.rs`(`route_preview(repo, limit) -> Result<RoutePreviewView>`, `project_summary`의 최근 목록 확인·신뢰 확인 부분을 `summary_root(repo)` 도우미로 뽑아 둘이 같이 쓴다), `src-tauri`(`query_route_preview` 네 곳), 생성물, `src/lib/api.ts`(`routePreview`), architecture.md.
- 내용: 끝점은 HEAD 해시와 기본 브랜치 끝(HEAD와 다르고 있을 때). `limit`은 20으로 자른다. `log_graph(tips, [], 0, limit + 1)`, 라벨은 M4와 같은 규칙(그 버전들에 붙은 브랜치·태그). git 호출은 status 1, branches 1, remotes·remote_head, log, tags 정도라 줄 수를 늘리지 않는다. 저장이 없으면 빈 값.
- 테스트: 가짜 백엔드로 신뢰하지 않은 저장소는 git을 부르지 않음(호출 횟수 0), 최근 목록에만 있고 열지 않은 저장소는 읽힘, 목록에도 열린 적도 없으면 `NotOpened`, 현재가 기본 브랜치면 끝점 하나, 갈라졌으면 둘, `limit` 20 상한, 저장 없는 저장소.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 차이 없음. 경로 비교는 기존 `platform::same_path`.
- 보안: `project_summary`와 같은 접근 규칙을 도우미 하나로 공유해 한쪽만 느슨해지지 않게 한다(D-067). 읽기 전용.

### C. 그리기 바탕 (화면의 순수 코드)

M7. 공용 장면·Renderer·도우미와 홈 옮기기, 첫 역 이름 고치기 (`features: shared route scene and home on it`)
- 목표: 홈·버전 기록·시작 화면이 같이 쓸 장면 타입과 그리기를 만들고, 홈을 그 위로 옮기며 첫 역 이름 문제를 고친다.
- 바꿀 파일: 신규 `src/features/history/layout/scene.ts`(`Scene { width, height, lines, stations, texts, tag?, folded }`, `Line { d, colorKey, width, dashed?, from?, to? }`(`from`·`to`는 시간축 범위, 보이는 구간 거르기용), `Station { id?, x, y, r, colorKey, kind: "commit"\|"merge"\|"fold"\|"open", ring? }`, `Text`, `Tag`), 신규 `layout/geometry.ts`(`textWidth(text, px)`: 한글·전각 1em, 영문·숫자 약 0.55em, `fitText(text, maxPx)`, 곡선·꺾임 경로 도우미, `edgeWidth(lines)`는 M10), `render/Renderer.ts`(`Renderer = ({ scene, palette: (colorKey) => string, label, viewport? }) => ReactElement`), `render/SvgRenderer.tsx`·`.module.css`, `layout/twoLine.ts`(새 장면 타입으로, 경로 도우미 사용), `screens/Workspace/HomeMap.tsx`(`palette`로 main·current 색 전달), `layout/twoLine.test.tsx`, 신규 `layout/geometry.test.ts`.
- 내용:
  - 장면은 픽셀 좌표만 갖고 방향을 모른다. `Renderer`는 장면과 `palette`(색 키 → `var(--graph-n)` 등)와 보이는 구간만 받는다. 클릭·호버 판정은 M10의 `hitTest`가 장면 좌표로 한다.
  - `layoutTwoLine`은 출력 모양을 유지한다(두 줄 보기의 경로 문자열은 바꾸지 않는다). 한 줄 보기·접힌 보기에서 첫 역 이름이 0 밖으로 나가지 않게 시작 위치를 `max(LEFT, textWidth(첫 이름)/2 + 8)`로 하고, 이웃 이름이 겹치지 않게 이름 최대 폭을 `fitText(이름, 간격 − 8)`로 줄인다(지금은 글자 수 14자만 본다).
  - 글자는 계속 `--text`·`--text-muted`이고 저장소에서 온 글자는 React 글자로만(D-031, D-037). SVG 글자에 `unicode-bidi: isolate`(이름표 속 방향 문자로 이웃 글자가 뒤집히지 않게).
- 테스트: 기존 두 줄·한 줄·접힘·upstream 표시 테스트가 같은 기대로 통과, 새 테스트: 한 줄 보기에서 14자 한글 첫 이름의 왼쪽 끝 ≥ 8(너비 420·800·1200), 접힌 보기도 같음, 이웃 이름 상자가 겹치지 않음, 두 줄 경로 문자열 스냅샷이 이전과 같음, `textWidth`·`fitText` 표, 글자에 `graph` 색 변수가 없음(기존).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: r-home.png와 같은 모양, 저장 1~2개짜리 프로젝트의 한 줄 보기에서 이름이 잘리지 않음.
- Windows/Ubuntu: Pretendard가 번들 글꼴이라 글자 폭 추정이 두 OS에서 같다. SVG 글자 기준선이 WebKitGTK와 WebView2에서 다를 수 있어 두 OS에서 글자 겹침을 본다.
- 보안: 저장소 글자는 `dangerouslySetInnerHTML` 없이 그린다(ESLint가 막음).

M8. 화면 모델과 레인 배치 (`features: graph model and lane layout`)
- 목표: 받은 버전들(부모 관계 포함)에서 레인과 선을 정하는 순수 함수. DOM 없음.
- 바꿀 파일: `src/features/history/model.ts`(`HistoryRow`에 `labels`, `unpushed`, `isHead` 추가, 신규 `RouteNode { id, parents, kind: "commit", labels, isHead, … }`, `toRouteNodes(pages: LogPage[])`는 쪽을 이어 붙이고 같은 해시는 한 번만), 신규 `layout/lanes.ts`·`lanes.test.ts`, `model.test.ts`.
- 내용(알고리즘 규칙, 구현은 자유):
  - 입력은 위상 순서(최신이 위)의 노드와 `trunkId`(0번 레인 끝점: 기본 브랜치 끝이 있으면 그 해시, 없으면 지금 위치). 레인 칸마다 "다음에 만날 노드 id"를 갖는다. 0번 칸은 `trunkId`가 나올 때까지 비워 두고 그리지 않는다(다른 끝점이 0번을 차지하지 않게).
  - 노드: 자기를 기다리는 칸이 있으면 가장 왼쪽 칸을 쓰고 나머지 칸의 선은 이 노드로 합류시킨다(갈라진 곳). 없으면 가장 왼쪽 빈 칸에서 새 레인을 연다(끝점). 첫 부모는 같은 칸이 이어받고, 다른 부모는 이미 기다리는 칸이 있으면 그 칸으로 잇고 없으면 빈 칸을 새로 연다. 부모가 없으면 칸을 비운다. 자식보다 부모가 먼저 나오는 입력(시계 어긋남)과 부모가 받은 범위 밖인 노드(쪽 끝)에도 깨지지 않는다. 범위 밖 부모는 아래 끝까지 이어지는 선으로 남긴다.
  - 선의 모양: 두 칸을 잇는 선은 오래된 칸을 따라 길게 가고, 꺾임(한 줄 높이 안, 45도 근처)은 새로 열린 쪽 끝에 둔다(r-log: 곁가지가 갈라진 역 바로 위에서 꺾여 main으로 들어감, 합친 곁가지는 합치기 바로 아래에서 갈라져 나옴).
  - 레인 색 키: 레인이 열리는 노드의 브랜치 이름(지금 위치 브랜치 > 기본 브랜치 > 이름순 첫째), 없으면 `null`(= `--text-muted`). 같은 레인의 아래 노드가 다른 이름을 가져도 레인 색은 안 바뀐다. 0번 레인 예약 칸의 이름은 기본 브랜치.
  - 결과: 노드마다 행 번호·레인·색 키·"이 노드가 합치기"·"끝까지 이어지는 선", 선마다 (위 노드, 아래 노드, 위·아래 레인, 꺾임 위치, 색 키, 이 선이 닿는 노드 id들).
  - 지금 위치의 조상 표시: 지금 위치에서 부모를 따라가 도달하는 받은 노드 집합(`onHeadLine`)도 함께 낸다.
- 테스트(표): 일직선, 갈라졌다 합침, 끝점 둘(main이 0번), 세 부모 합치기, 끝점이 기본 브랜치의 조상(같은 레인), 이름 없는 레인, 쪽 끝에서 부모가 범위 밖, 시계 어긋남, 같은 해시 쪽 겹침, 레인 재사용(한 번 닫힌 칸에 다음 끝점이 들어감), `trunkId`가 없을 때. 레인 수 상한과 그래프 폭 규칙은 M10.
- 완료 기준: `pnpm test`, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(순수 함수).
- 보안: 해당 없음.

M9. 확대·축소(접기) (`features: zoom levels by folding the graph`)
- 목표: 단계에 따라 받은 버전을 접어 "버전 N개" 한 줄로 바꾸는 순수 함수.
- 바꿀 파일: 신규 `layout/simplify.ts`·`simplify.test.ts`, `model.ts`(`RouteNode.kind`에 `"fold"` 추가: `{ id: "<위해시>><아래해시>", count, statOids: string[] }`).
- 내용:
  - `simplify(nodes, zoom: "all" | "key" | "tags", expanded: ReadonlySet<string>, headId) → RouteNode[]`. 남길 노드: `all`은 전부. `key`는 합치기·갈라진 곳(받은 노드 안 자식 수 ≥ 2)·라벨(브랜치·태그)이 있는 것·지금 위치·첫 버전. `tags`는 태그·브랜치 끝·지금 위치·첫 버전. 모든 단계에서 부모가 받은 범위 밖인 노드(쪽 끝)와 지금 위치는 남긴다(아래로 이어지는 선이 남은 역에서 끝나게).
  - 접기: 남기지 않는 노드들을 아래쪽 끝(부모 쪽)에서부터 가장 가까운 남긴 조상까지 따라가 한 줄 `fold`로 바꾼다. 남긴 노드의 부모 선은 `fold`를 거쳐 그 조상으로 이어진다. 합치기나 갈라진 곳이 접히는 `tags` 단계에서는 남긴 조상 후보가 여럿일 수 있으므로 아래에서 위로 한 번 훑으며 "가장 가까운 남긴 조상 집합"(최대 6개, 넘으면 가까운 순으로 자르고 그 사실을 표시)을 구해 `fold`마다 연다. `expanded`에 든 `fold` id는 접지 않는다.
  - 순서: `fold`는 자식(위 노드) 바로 뒤에 둔다. 위상 순서와 `fold`당 자식 하나·부모 하나가 유지된다.
  - 줄 수 합산용으로 `fold`가 덮는 버전 해시(최대 500개까지 기록, 넘으면 "이상"으로 표시)를 `statOids`에 담는다.
- 테스트(표): 단계 세 개에서 남는 노드 집합, 갈라진 곳·합치기·지금 위치가 `key`에서 남음, 태그 없는 저장소의 `tags` 단계(끝점·지금 위치·첫 버전만), 한 구간만 `expanded`, 쪽 끝 노드 보존, 선이 끊기지 않음(모든 남긴 노드의 부모가 남긴 노드나 `fold`), 접힌 합치기가 있는 `tags` 단계의 조상 집합, 1만 행 3단계 모두 합쳐 50ms 안팎.
- 완료 기준: `pnpm test`.
- Windows/Ubuntu: 차이 없음.
- 보안: 해당 없음.

M10. 장면 만들기: 좌표·굵기·팔레트·가로 보기·판정 (`features: route scene for vertical and horizontal maps`)
- 목표: 레인 배치를 픽셀 장면으로 바꾼다(세로 = 버전 기록, 가로 = 시작 화면). 굵기와 색, 판정 포함.
- 바꿀 파일: 신규 `layout/route.ts`·`route.test.ts`(`buildRoute(nodes, opts) → Route`, `routeScene(route, opts) → Scene`), `layout/geometry.ts`(`edgeWidth`, 레인 간격·폭 상한), 신규 `layout/hit.ts`·`hit.test.ts`(`hitTest(scene, x, y, slop) → Station id\|null`, 세로 목록용 `rowAt(y)`), `render/SvgRenderer.tsx`(보이는 구간 거르기, `fold`·`merge`·고리 그리기), `render/SvgRenderer.module.css`.
- 내용:
  - 두 단계: `buildRoute`(비싼 쪽: 단순화 → 레인 → 행·레인 좌표, 노드·선 배열)와 `routeScene`(싼 쪽: 굵기·색·팔레트·픽셀). 줄 수가 도착하면 `routeScene`만 다시 돈다.
  - 세로(버전 기록): 행 높이 48, 첫 레인 x=40, 레인 간격 24(그래프 폭 최대 240이 되도록 8까지 줄임, 24레인 밖은 마지막 칸), 역 r=5(선 2.4), 합치기 r=7, `fold` 작은 마름모, 지금 위치는 `--text` 바깥 고리(r 11, 1.4). 선은 전체를 한 줄에 그리지 않고 선(노드 쌍)마다 한 경로로, `from`·`to`로 보이는 구간만 그린다.
  - 굵기(C3): `edgeWidth(줄)` = `round(2 + log10(1 + 줄))` 최대 5, 0·모름 2, 지금 위치 브랜치의 선(양 끝 노드가 모두 `onHeadLine`에 든 선)은 +1. 선의 굵기가 따르는 줄은 "그 선이 이어 주는 아래 노드의 자식 버전" 규칙: 부모가 하나인 노드의 부모 선은 그 노드의 줄, 합치기 노드의 첫 부모 선은 2, 다른 부모 선은 그 부모 노드의 줄, `fold` 선은 `statOids` 합. 줄 수 모음(`ReadonlyMap<해시, number|null>`)은 `routeScene`의 입력.
  - 가로(시작 화면): 오래된 버전이 왼쪽. `density: "preview" | "mini"`. `preview`는 홈과 같은 굵기(기본 3·지금 4)와 간격(120px), 이름을 레인 바깥쪽에 `fitText`로, 끝에 "지금 여기", 너비가 모자라면 오래된 쪽을 "…"로 접는다(홈과 같은 방식). `mini`는 이름 없음, 선 2, 역 r 3.5, 고정 폭. `trunk: "top" | "bottom"`: 홈은 위, 시작 화면은 아래(start-normal.png)로 둔다. 레인은 최대 3개까지만 그리고 나머지는 그리지 않는다.
  - 팔레트: `branch`(색 키 → `branchColor`, `null` → `--text-muted`)와 `mono`(0번 레인 `--border-strong`, 나머지 `--text-muted`, 시작 화면 작은 노선도). 글자는 늘 `--text`·`--text-muted`.
  - `hitTest`는 장면 좌표로만(DOM 모름). 세로 목록은 `rowAt`으로 y에서 행을 구해 같은 판정을 쓴다.
- 테스트: r-log와 같은 모양의 표본 기록으로 레인 x = 40 + 24n, 행 y 중심 = 48i + 24, 꺾임 끝 좌표(곁가지 → main 합류가 한 행 안), 굵기 구간 경계(줄 0·9·10·99·100·999·1000, 지금 브랜치 +1, 합치기 2), 접힌 선의 합, 레인 40개에서 폭 ≤ 240, 가로 보기 좌표(trunk top/bottom 뒤집힘, 오래된 것이 왼쪽, 너비가 모자랄 때 접기), 팔레트 두 가지, 글자에 그래프 색 변수 없음, `hitTest` 경계, 1만 행 `buildRoute` 50ms 안팎·`routeScene` 5ms 안팎(느슨한 상한으로 확인하고 실제 값을 로그).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(순수 함수). SVG 가는 선(2px)이 두 OS에서 흐려 보이지 않는지는 M16 손 확인(정수 굵기·반 픽셀 정렬 `+0.5`).
- 보안: 해당 없음.

### D. 버전 기록 화면

M11. 목록·가상 스크롤·선택·키보드 (`features: virtual route list with listbox keyboard`)
- 목표: 장면을 가상 스크롤 목록으로 보여주고 키보드로 역을 옮긴다. 메뉴는 M13.
- 바꿀 파일: 신규 `src/features/history/RouteList.tsx`·`RouteList.module.css`·`RouteList.test.tsx`, `src/features/history/useWindow.ts`(고정 행 높이 창 계산), 신규 `src/state/historyStore.ts`(zustand: `selectedId`, `zoom`, `scope`, `expanded`), `docs/design.md`("버전 기록 노선도" 부품 규칙).
- 내용:
  - 화면 배치(r-log.png, 4px 단위로 맞춤): 바깥은 스크롤 영역 하나(`overflow: auto`, `--scrollbar-size`), 안에 행 수 × 48px 높이의 빈 틀, 그 위에 보이는 행 ± 10행만 절대 위치로. 그래프 폭만큼 왼쪽 칸에 SVG 한 장(보이는 구간만 `Renderer`에 `viewport`로), 오른쪽에 행: 이름(저장소 글자는 `<bdi>`로 방향 격리, 말줄임) 옆에 옅은 글자로 "안 올림"·태그 이름·브랜치 이름(앞에 브랜치 색 작은 점), 작성자 90px, 날짜 90px 오른쪽 정렬(`formatDate`), 줄 사이 `--border` 1px, 오른쪽 여백 32px. 선택 행은 `--surface-hover` 바탕과 왼쪽 2px `--accent` 막대(design.md 목록 행). `fold` 행은 "버전 N개 · 펼치기"(`--text-muted`).
  - 상호작용(D-017: 판정은 장면 좌표): 목록 컨테이너 하나가 `onPointerDown`에서 y로 `rowAt`을 구해 선택한다. 행에는 클릭 핸들러가 없다.
  - 접근성: 컨테이너 `role="listbox"`, 이름은 "버전 기록", 포커스는 컨테이너가 받고 `aria-activedescendant`가 선택 행을 가리킨다. 행 `role="option"`, `aria-selected`, `aria-posinset`·`aria-setsize`(더 있으면 -1), `aria-label` = 이름·작성자·날짜·브랜치들·태그들·"지금 여기"·"합치기"·"안 올림"을 쉼표로(messages, 저장소 글자는 200자까지). SVG는 `aria-hidden`.
  - 키: ↑↓(한 행), PageUp·PageDown(한 화면), Home·End, 선택이 화면 밖이면 스크롤. 한글 조합 중 무시(`isComposing`). Enter·Space·ContextMenu 키·Shift+F10은 메뉴 열기(M13), 선택이 `fold`면 펼치기. `+`·`-`·Ctrl+휠은 M12.
  - 가상화: 행 수가 1만이어도 DOM 행 60개 이하, 스크롤마다 배치를 다시 만들지 않고(`buildRoute` 메모) 창만 바꾼다. 스크롤 이벤트는 `requestAnimationFrame`으로 합친다.
  - 색은 `tokens.css` 변수만. 고대비(`forced-colors: active`)에서 선·점이 보이도록 `.station`·선에 `CanvasText`를 쓰는 규칙을 둔다.
- 테스트(Testing Library): 행 수 1만 입력에서 DOM 행 ≤ 60, ↓ 두 번 → `aria-activedescendant` 두 번째 행, Home·End, 선택이 화면 밖일 때 `scrollTop` 이동, 한글 조합 중 ↓ 무시, 저장소 글자가 `<bdi>` 안에 React 글자로(HTML 문자열이 있어도 태그로 안 그려짐), 각 행의 `aria-label` 내용(합치기·지금 여기·안 올림·태그), `fold` 행에서 Enter → 펼침, 가짜 `getBoundingClientRect`·`ResizeObserver` 사용.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: r-log.png와 배치 비교(라이트·다크).
- Windows/Ubuntu: 스크롤바는 CSS로 통일, WebKitGTK의 `scrollTo`·`overflow: auto` 동작과 휠 속도 차이를 본다. 800px 너비에서 이름 칸이 말줄임으로 줄고 그래프 폭이 240을 넘지 않는다.
- 보안: 이름·작성자·브랜치·태그는 모두 믿지 않는 글자(D-034). React 글자와 `<bdi>`로만, `title`·`aria-label`에 쓰는 글자도 길이를 자른다.

M12. 버전 기록 화면 연결 (`screens: history screen on the route map`)
- 목표: `History`가 실제 데이터로 노선도를 그린다. 범위 전환, 확대·축소, 쪽 넘기기, 줄 수 늦게 받기, 지금 여기.
- 바꿀 파일: `src/lib/api.ts`(이미 M5·M6에서), `src/lib/queries.ts`(`useLogPages(repo, scope)`: `useInfiniteQuery`, 쪽 크기 `HISTORY_LIMIT` 200, 다음 쪽은 첫 쪽의 `tips`와 `skip`을 보냄, `queryKeys.log(repo)` 아래에 `scope` 포함), 신규 `src/features/history/stats.ts`(`useCommitStats(repo, visibleOids)`: 보이는 행 중 모르는 해시를 150ms 뒤 100개씩 한 번에 하나씩 묻고 `Map`에 저장(해시별, 변하지 않으므로 무효화 없음, 최대 2만개 넘으면 오래된 것부터 버림), 못 받은 것은 "모름" 표시), `src/screens/History/History.tsx`·`History.module.css`(머리 줄: r-log처럼 왼쪽 "버전 기록"(`<Term id="log" />`)과 범위 전환 "모든 브랜치 · 지금 브랜치만"(밑줄 글자 단추 둘, `aria-pressed`), 오른쪽 "지금 여기" 단추와 확대·축소 묶음 "−  모든 버전  +"), `src/lib/shortcuts.ts`(`zoomIn`·`zoomOut`, `scope: "history"`, `inTextField: "app"`, 키 `Equal`·shift+`Equal`·`NumpadAdd` / `Minus`·`NumpadSubtract`), `src/lib/useRepoWatch.ts`(`.git` 변경에서 `log`를 이어붙이기 대신 처음부터 다시: `resetQueries`), `src/messages/{ko,en}.ts`(단계 이름 "모든 버전"·"주요 버전"·"태그만", "−"·"+"의 이름, "지금 여기", "안 올림", "버전 N개", "펼치기", "브랜치 200개까지 보여요" 같은 숫자 문구는 이름과 숫자만, D-049), `HistoryList.tsx` 삭제.
- 내용:
  - 데이터 흐름: 쪽들 → `toRouteNodes` → `simplify`(단계·펼침) → `buildRoute` → `routeScene`(줄 수 반영) → `RouteList`. 범위·단계·펼침이 바뀔 때만 앞 단계를 다시 계산한다.
  - 자동 이어 받기: 스크롤이 끝에서 `2 × 화면 높이` 안이면 다음 쪽을 받는다. 접힘으로 행이 화면보다 적게 남으면(`행 수 × 48 < 2 × 화면 높이`) 최대 10쪽까지 자동으로 이어 받고 그 뒤는 스크롤로. 받은 버전이 2만 개에 닿으면 멈추고 기존 `moreCommits` 문구("최근 N개만 보여요")를 보인다. `tips_cut`이 있으면 같은 줄에 "브랜치 200개까지 보여요".
  - "지금 여기": 지금 위치 행으로 선택·스크롤을 옮긴다. 아직 안 받았으면 최대 25쪽까지 받아 찾고, 못 찾으면 단추가 꺼진다. 머리에 지금 브랜치 이름과 짧은 해시가 늘 보인다(스크롤과 무관하게 "지금 여기"의 위치 안내).
  - 확대·축소: 단추·`+`·`-`·Ctrl+휠·핀치(`wheel`의 `ctrlKey`를 비수동적 리스너로 받아 `preventDefault`). 바꿀 때 선택한 역과 보이던 첫 행이 남도록 선택 해시 기준으로 스크롤을 맞춘다(없으면 지금 위치). 단계가 바뀌면 펼침은 지운다. 단계 이름은 `role="status"` 영역에 읽힌다("주요 버전 · 120개").
  - Esc: 메뉴가 열려 있으면 메뉴만 닫는다(M13이 `preventDefault`). 기존 `History`의 window keydown은 `!e.defaultPrevented`일 때만 홈으로.
  - 비어 있음·오류·불러오는 중은 지금처럼 `EmptyState`·`ErrorNotice`. 저장이 없으면 "아직 저장한 버전이 없어요".
  - 노선도 보기에서 합치는 중이면 이 화면은 안 보인다(기존 충돌 화면이 대신).
- 테스트: api 모킹으로 첫 쪽 → 스크롤 끝 → 둘째 쪽 요청에 첫 쪽의 `tips`가 담김, 범위 전환이 `scope`를 바꿔 처음부터, 단계 전환이 요청 없이 접힘을 바꿈, 접힘으로 행이 모자라면 자동 이어 받기가 10쪽에서 멈춤, 줄 수 조회가 보이는 해시만(100개 이하, 150ms 디바운스), 같은 해시는 다시 묻지 않음, "지금 여기"가 안 받은 위치를 이어 받아 찾음, Esc 순서(메뉴 → 홈), `+`·`-`가 입력칸이 아닌 곳에서만, 단축키 표 겹침 테스트(`shortcuts.test.ts`), `copy-rules.test.ts`·`terms-in-messages.test.ts`.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 합친 기록이 있는 실제 저장소(터미널로 브랜치 둘·합치기·태그를 만들어)에서 세 단계, 범위 전환, Ctrl+휠·핀치, Esc.
- Windows/Ubuntu: Ctrl+휠이 WebView2·WebKitGTK에서 페이지 확대가 아니라 앱에 오는지, 터치패드 핀치가 `ctrlKey` 휠로 오는지, `+`·`-`·숫자패드 키가 두 OS에서 오는지는 손으로 확인(`docs/progress.md` "사람이 할 일"에 적음). 한글 입력 상태에서 `code`로 판정.
- 보안: 새 명령은 M5·M6이 이미 열었다. 화면은 `api.ts`만 부른다. `tips`는 core가 준 값 그대로만 되돌려 보낸다.

M13. 역 메뉴 틀 (`features: station menu frame`)
- 목표: 역을 고르면 뜨는 작은 메뉴의 틀과 반복 3이 항목을 더하는 자리를 만든다. 2b의 실제 항목은 "버전 번호 복사" 하나(C6).
- 바꿀 파일: 신규 `src/features/history/menu/{info.ts, items.ts, StationMenu.tsx, StationMenu.module.css, StationMenu.test.tsx, copy.ts}`, `RouteList.tsx`(메뉴 열기 연결), `src/messages/{ko,en}.ts`("버전 번호 복사", "복사했어요"), `docs/design.md`(메뉴 규칙), `docs/architecture.md`("역 메뉴" 한 문단: 항목 표·`StationInfo`·반복 3이 하는 일).
- 내용:
  - `StationInfo`(`info.ts`): `{ oid, shortOid, summary, author, time, parents, isMerge, isRoot, isHead, onHeadLine, branches: string[], tags: string[], unpushed: boolean }`. `RouteList`가 고른 행과 `RouteNode`에서 만든다(`onHeadLine`은 M8). `fold` 행은 메뉴가 없다.
  - 항목 표(`items.ts`): `StationMenuItem { id, group: "main" | "more", label: ReactNode(용어는 `<Term>`), availability(info): "show" | "hide" | { disabled: 까닭 메시지 키 }, run(info, ctx: { repo, toast, close }) }`. 액션이 되는 항목은 `actionId`를 가질 수 있고(반복 3), 있으면 `action-meta.json`에 있는 id인지 테스트가 확인한다(`registry-glossary.test.ts` 방식). 2b에는 `copy-id` 하나. 반복 3은 이 표에 항목을 더하고 `run`에서 `api.executeAction({ id, input: { …, oid: info.oid } })`를 부르면 된다(위험도·되돌리기는 `action-meta.json`과 실행기가 알아서, 화면이 git을 직접 부르지 않음).
  - 표시: `disabled`는 글자 `--text-muted`로 두고 툴팁에 까닭(비교 칸 아래 "꺼내기"와 같은 방식, design.md), `hide`는 아예 없음. "더 보기"는 `more` 묶음에 보이는 항목이 하나 이상일 때만 선 아래에 생기고 하위 메뉴로 연다. 2b는 `more`가 비어 보이지 않는다.
  - 열기: 선택된 행에서. Radix `DropdownMenu`를 `open`·`onOpenChange` 제어로 쓰고, 위치용 보이지 않는 `Trigger`(`asChild`, 선택 행의 그래프 점 오른쪽 위치)를 둔다. 닫힐 때 포커스를 목록으로 돌려준다(`onCloseAutoFocus`에서 목록에 포커스). Esc는 `preventDefault`해 History의 홈 이동을 막는다. 메뉴가 열린 채 목록이 스크롤되거나 선택 행이 사라지면 닫는다. Radix 부품이 가상 스크롤 위에서 어긋나면 `Popover`로 바꾸되 `role="menu"` 의미는 유지(구현 때 두 OS에서 확인).
  - `copy-id`: `navigator.clipboard.writeText(전체 해시)`, 실패하면 숨은 글상자와 `document.execCommand("copy")`로 대신하고 둘 다 실패하면 토스트 "복사하지 못했어요"(오류 문구 규칙). 성공은 토스트 "복사했어요"(되돌리기 없음). 해시는 16진수 40·64자라 클립보드에 넣어도 위험이 없다.
  - 마우스: 역이나 행을 누르면 선택하고 바로 메뉴를 연다(r-log). 같은 행을 다시 누르면 닫는다.
- 테스트: 가짜 항목 표로 `show`·`hide`·`disabled`(툴팁 까닭)·"더 보기"가 생기고 사라지는 규칙, 2b 실제 표에는 `copy-id`만 있고 "더 보기" 없음, Enter·Space·Shift+F10·ContextMenu 키로 열림, Esc가 메뉴만 닫고 History는 그대로·한 번 더 누르면 홈, 닫으면 목록에 포커스, `fold` 행은 메뉴 없음, `copy-id`가 `navigator.clipboard`와 대체 경로에서 해시를 넣고 토스트, `actionId`가 있는 항목은 `action-meta.json`에 있어야 한다는 규칙 테스트, `StationInfo`가 이름 여러 개·합치기·지금 위치·안 올림·`onHeadLine`을 맞게 채움.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 마우스·키보드로 열고 복사, 800px, 다크.
- Windows/Ubuntu: `navigator.clipboard`가 두 웹뷰(`tauri://localhost`, WebKitGTK 포함)에서 쓰이는지(안 되면 대체 경로), Radix 메뉴의 포커스 이동·Esc가 WebKitGTK에서 되는지, 고대비에서 메뉴 테두리가 보이는지(반복 1b 프로젝트 메뉴와 같은 확인).
- 보안: 복사하는 글자는 해시뿐. 메뉴 항목 `run`은 화면이 git을 직접 부르지 않고 `api.ts`·core 액션만 쓰게 한다(D-010). 반복 3의 액션 입력은 core가 해시를 다시 확인해야 한다는 점을 architecture.md "역 메뉴" 문단에 적는다.

### E. 시작 화면

M14. 시작 화면 큰 노선도 (`features: route preview on start screen`) — 계획 003 N2
- 목표: "마지막으로 하던 곳"에 큰 노선도(브랜치 색)를 그린다.
- 바꿀 파일: 신규 `src/features/history/RoutePreview.tsx`·`.module.css`·`RoutePreview.test.tsx`(`<RoutePreview data palette density />`: 데이터는 `RoutePreviewView`를 `RouteNode`로 바꾼 것, `buildRoute`·`routeScene`·`SvgRenderer`만 쓰고 배치를 따로 갖지 않음), `src/lib/queries.ts`(`useRoutePreview(repo, limit, enabled)`), `src/screens/Start/LastProject.tsx`·`Start.module.css`, `docs/design.md`("시작 화면 색"에 큰 노선도 규칙은 이미 있음, 높이·간격 보강), `src/messages`.
- 내용: 상태 문장과 "이어서 하기" 사이에 놓는다(start-normal.png). 높이 104, 너비는 칸 너비(최대 1100). 선·점만 `--graph-*`(기본 브랜치 `--graph-1`, 나머지 이름 해시), 역 이름·"지금 여기"·브랜치 이름("main")은 `--text`·`--text-muted`. trunk는 아래, 갈라진 브랜치가 위. 조회는 상태 요약이 성공일 때만(`enabled`). 아직 안 왔을 때는 같은 높이의 빈 자리(아래가 밀리지 않게). 오류·저장 없음이면 자리를 접는다(높이 0이 아니라 상태 문장과 간격만). 800px에서는 오래된 역이 "…"로 접힌다. 시작 화면의 상태 요약 오류(`folderNotFound` 등)에서는 조회하지 않는다. 노선도는 `aria-hidden`이 아니라 `role="img"`에 "버전 기록" 이름과 요약(브랜치, 버전 N개)을 달고, 문장이 이미 있어 추가 읽기가 길지 않게 한다.
- 테스트: `Start.test.tsx`에 노선도가 상태 성공 뒤에만 불러짐, 폴더 없음·신뢰 필요에서는 부르지 않음, 불러오는 동안 높이 유지, 이름이 `--text`·`--text-muted`(글자에 `graph` 변수 없음), 주 버튼이 여전히 하나.
- 완료 기준: `pnpm test`, `pnpm lint`. 손 확인: start-normal.png(라이트)·start-normal-dark.png와 비교, 800px.
- Windows/Ubuntu: 한글 경로 프로젝트·긴 이름, SVG 글자 기준선 두 OS 비교.
- 보안: 읽기 전용 조회, 저장소 글자는 React 글자로만.

M15. 시작 화면 작은 노선도 (`features: mini route in project list`) — 계획 003 N3
- 목표: "다른 프로젝트" 줄마다 회색 작은 노선도를 둔다. 보일 때만 불러온다.
- 바꿀 파일: `src/screens/Start/ProjectRow.tsx`·`Start.module.css`, 신규 `src/lib/useInView.ts`(IntersectionObserver, 없으면 항상 보임으로 간주 — jsdom), `src/lib/queries.ts`(`useRoutePreview`가 모듈 안 큐로 동시에 3개까지만 요청), `RoutePreview.tsx`(`density="mini"`, `palette="mono"`), 테스트.
- 내용: 줄 가운데 칸(이름 200px과 상태 사이)에 폭 240px·높이 28px. 선·점 `--border-strong`, 갈라진 브랜치만 `--text-muted`(start-normal.png). 줄이 화면에 들어올 때 처음 불러오고(`useInView`), 동시에 3개까지 큐로. 사라진 줄·저장소 아님·신뢰 필요 줄은 노선도 칸이 비고(그 줄의 지금 모양 유지), 오류에서 조용히 비운다. 줄 높이는 지금 그대로(`calc(var(--row-height) * 1.5)`), 노선도는 장식이라 `aria-hidden`(상태 글자가 같은 정보를 준다).
- 테스트: 보이기 전에는 조회하지 않음, 보이면 조회, 20개 프로젝트에서 동시에 3개 이하, 사라진 줄은 조회 안 함·칸 비움, `IntersectionObserver` 없는 환경에서 동작, 색 규칙.
- 완료 기준: `pnpm test`, `pnpm lint`. 손 확인: 프로젝트 20개 근처로 스크롤 시 한꺼번에 느려지지 않는지(Windows), 800px에서 상태 글자가 줄임표로 줄 뿐 노선도가 겹치지 않는지, 다크.
- Windows/Ubuntu: Windows는 프로세스 시작이 느려 처음 몇 줄이 차례로 채워지는 것을 확인. 
- 보안: 신뢰하지 않은 저장소에서는 git을 안 돌린다(M6 규칙).

### F. 마무리

M16. 문서·성능·점검 (`docs: wrap up iteration 2b`)
- 목표: 문서를 구현에 맞추고 성능·규칙·접근성을 점검한다.
- 바꿀 파일: `docs/architecture.md`(GitBackend 목록에 `log_graph`·`tags`·`commit_stats`와 "반복 2b까지", Query 목록에 `log`(확장)·`log_stats`·`route_preview`, 명령 목록에 `query_log_stats`·`query_route_preview`, "버전 기록 그리기 계층"을 실제 층(model → simplify → lanes → route → Scene → Renderer)으로, 입력 상한(줄 수 조회 100개, 끝점 201개, 노선도 미리보기 20개), 성능 기준에 이번 측정값), `docs/spec.md`(화면 구성의 버전 기록 행: 노선도·확대와 축소·선 굵기·범위 전환·역 메뉴 틀, "역 메뉴의 되돌리기·이 버전만 복사하기·태그는 반복 3"; 시작 화면 행의 "노선도는 반복 2b"를 현재형으로), `docs/design.md`(버전 기록 노선도·역 메뉴·시작 화면 노선도 부품 규칙 정리), `docs/roadmap.md`(반복 2b 완료 표시와 반복 3 메모: 역 메뉴 항목 표에 항목을 더한다), `docs/progress.md`(반복 2b 완료, 사람이 할 일), `.claude/agents/code-analyst.md`(점검 줄: 화면의 노선도 코드에서 git·invoke 없음, `layout/`이 DOM을 모름, 새 명령 둘이 네 곳에).
- 성능 점검: 1만 개 버전(`git fast-import`로 만든 임시 저장소, 갈라짐·합치기·태그 포함)에서 `query_log` 첫 쪽 시간을 재 `#[ignore]` 성능 테스트와 `docs/progress.md`에 값 기록, 프론트 배치 시간 측정값 기록. 목표를 넘으면 `--topo-order` 없는 대안과 쪽 크기를 위험 절 기준으로 다시 정한다.
- code-analyst 점검: 계층 규칙(`rg "invoke" src --glob '!src/lib/**'`), 용어·문구 규칙, 색 값(`tokens.css` 밖), `rg "dangerouslySetInnerHTML|innerHTML" src`, 새 명령 둘이 `commands.rs`·`bindings.rs`·`build.rs`·`capabilities`에 모두, `rg '"--end-of-options"' src-tauri/crates/git/src`가 `args.rs`에만, `Command::new`가 runner·platform에만, `layout/`에서 `document`·`window` 사용 없음, 단축키가 `shortcuts.ts` 밖에 없음, `unsafe`·`unwrap` 없음, `cargo deny check`(새 의존성 없음).
- 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full` 통과, "완료 기준(반복 전체)"을 두 OS에서 손으로 확인(못 하는 것은 `docs/progress.md` "사람이 할 일").

## 성능 (긴 기록)
- 쪽 단위: 한 번에 200개(`HISTORY_LIMIT`), 상한 500(`MAX_LOG_PAGE`). 끝점 스냅샷으로 쪽 사이가 어긋나지 않는다. 받은 버전은 2만 개까지만 쥔다.
- 첫 화면 300ms: 첫 쪽은 `log_graph` 한 번 + `branches`·`tags`·`status` 정도의 git 호출이고 줄 수는 얹지 않는다. 느릴 수 있는 곳은 `--topo-order`(커밋 그래프 파일이 없으면 전체를 훑음)이므로 M16에서 1만 개로 재고, 목표를 넘으면 쪽 크기를 줄이거나(쪽 100) 첫 쪽만 `--date-order`로 시험해 본다.
- 배치: `buildRoute`는 O(행 × 레인)이고 쪽이 붙을 때만 다시 만든다(이어 계산이 아니라 처음부터, 1만 행 50ms 안팎이라 충분). 줄 수가 도착하면 `routeScene`만 다시 돈다(5ms 안팎). 스크롤은 배치를 다시 만들지 않는다.
- 그리기: DOM 행 60개 이하, SVG는 보이는 구간의 선·점만. 줄 수 조회는 보이는 해시만 100개씩 150ms 디바운스로 하나씩(큰 커밋의 diff 계산이 오래 걸릴 수 있어 출력 2MB 상한).
- 접힘(`simplify`)은 O(행 × 부모 수)이고 `tags` 단계의 조상 집합은 6개로 자른다.
- Canvas 전환은 하지 않는다. SVG가 느려지면(보이는 선이 수천 개) `Renderer` 한 곳을 바꾸면 되게 `hitTest`·`routeScene`은 이미 DOM을 모른다.

## 접근성
- 목록 하나가 포커스를 받고(`role="listbox"`, `aria-activedescendant`) 행은 이름·작성자·날짜·브랜치·태그·"지금 여기"·"합치기"·"안 올림"·위치(N번째)를 한 문장으로 읽는다. 선·색은 정보가 아니라 장식이므로 SVG는 `aria-hidden`이고, 색으로만 알리지 않는다(브랜치 이름 글자, "안 올림" 글자, 합치기·지금 위치는 `aria-label`과 모양).
- 확대·축소 단계와 행 수는 `role="status"`로 알린다. 모든 조작이 키보드로 된다(역 이동, 메뉴, 펼치기, 확대·축소 `+`·`-`와 단추, "지금 여기" 단추, 범위 전환 단추). 포커스 표시는 `--accent` 2px(design.md).
- 저장소에서 온 글자는 `<bdi>`로 방향을 격리한다. 한글 조합 중 키는 무시한다.
- 움직임 없음(단계 전환은 즉시). 고대비(`forced-colors`)에서 선·점이 보이게 한다.

## 800px 너비와 다크 모드
- 800px: 그래프 폭 최대 240px이라 이름 칸이 약 300px 이상 남는다. 작성자·날짜 칸은 고정 90px, 이름은 말줄임. 헤더는 한 줄에 "버전 기록 · 범위 전환 · … · 지금 여기 · −모든 버전+"가 들어가고 모자라면 범위 전환 글자부터 줄인다. 시작 화면 큰 노선도는 너비에 맞춰 오래된 역을 "…"로 접고, 작은 노선도는 고정 폭 240px.
- 다크: 그래프 6색을 `tokens.css`에서 바꾸고(M1) 노선도 코드는 변수만 읽으므로 같다. 선택·고리·역 채움은 `--accent`·`--text`·`--surface`.

## 용어 변경
- spec.md 용어표·`src/glossary`: 없음. 쓰는 용어(`log`·`branch`·`merge`·`tag`)는 모두 이미 있다. 노선도·확대·축소·"모든 버전"·"주요 버전"·"태그만"·"지금 여기"·"안 올림"·"버전 번호 복사"는 Git 용어가 아닌 화면 문구라 `src/messages`(ko·en)에 둔다. 문장 속 "버전"·"브랜치"는 `t(id)`로 꺼낸다(D-029). spec.md는 화면 구성 행의 설명만 고친다(M16).

## 이번 반복에서 하지 않는 것
| 항목 | 가는 곳 | 까닭 |
| --- | --- | --- |
| 역 메뉴의 되돌리기·이 버전으로 돌아가기·마지막 버전 고치기·이 버전만 복사하기·태그·여기서 새 브랜치 기능 | 반복 3 | 틀과 항목 표만 이번 반복(C6) |
| 역 메뉴 "이때 파일 보기"(고른 버전의 바뀐 파일 비교) | 반복 3 계획이 넣을지 정함 | 로드맵에 없고 버전 단위 비교 조회가 새로 필요 |
| 기록 정리(카드 지도), 여러 역 선택·범위 선택 | 반복 3 | 선택은 하나만(`historyStore.selectedId`), 여러 개는 기록 정리가 열릴 때 |
| "실제 시간으로 보기", "사람별로 보기", 재생 헤드 시점 미리보기 | 나중 후보 | research ②③④. 같은 장면에 배치만 바꾸는 것이라 이번 계층이 막지 않는다 |
| Canvas Renderer | 필요해질 때 | SVG 창 그리기로 충분 |
| 노선도로 끌어놓아 합치기, 폴더 끌어놓기 열기(003 N5) | 나중 | Windows 끌어놓기 정책과 함께 정함(Q7) |
| 원격에만 있는 브랜치 그리기, 원격 브랜치 "안 올림" | 나중 | C2·C5 |
| 버전 찾기(검색)·작성자·파일 필터 | 나중 | 요청 범위 밖 |
| 홈 노선도 조회를 `route_preview`로 합치기 | 하지 않음 | C4 |
| 확대 단계·범위 기억(설정), 설정 화면 | 설정 화면이 생길 때 | C9 |
| 시작 화면 복제·처음 실행 이름·이메일·OS별 Git 설치 안내(003 N4·N6·N7) | 반복 4 이후 | 이 반복과 무관 |

## 위험과 열린 질문
- `git log --topo-order`는 커밋 그래프 파일이 없는 큰 저장소에서 전체를 훑어 첫 쪽이 느릴 수 있다. M16에서 1만 개로 재고, 넘으면 쪽 크기 100·`--date-order`를 시험한다(배치는 시계 어긋남에 견디게 만든다). 수십만 개 저장소는 이번 반복 목표 밖이다.
- 줄 수(`log --numstat`)는 큰 커밋(수만 파일, 첫 가져오기)에서 느리다. 출력 2MB 상한과 보이는 곳만 묻기, 못 받으면 가늘게 그리는 것으로 버틴다. 줄 수를 재는 동안 같은 저장소의 다른 읽기가 늦어질 수 있어 동시에 하나만 보낸다.
- 접힌 "태그만" 단계에서 합치기가 접히면 선의 연결이 실제 진행과 다르게 보일 수 있다(남긴 가장 가까운 조상으로 잇는 단순화). 조상 집합 6개 상한을 넘으면 일부 선이 빠진다. 툴팁 "버전 N개"로 접힘임을 알리고, 정확함이 필요한 사람은 "모든 버전"으로 본다.
- 이름 없는 레인(지운 브랜치)은 `--text-muted`라 구분이 약하다. r-log의 보라 곁가지처럼 색이 더 있으면 좋지만 같은 브랜치 = 같은 색(design.md) 규칙을 지키려고 이름 있는 레인만 색을 준다. 사용하다 모자라면 이름 없는 레인에 `--graph-*`를 해시로 줄지 다시 정한다.
- 레인이 24개를 넘거나 그래프 폭이 240으로 줄면 점이 겹친다. 브랜치가 아주 많은 저장소는 "지금 브랜치만"이 더 읽기 쉽다(범위 전환 단추).
- `tips` 스냅샷은 쪽 사이의 일관성을 주지만 오래 열어 두면 낡는다. `.git` 변경 알림이 오면 처음부터 다시 받아 맨 위로 돌아간다(선택한 해시는 남기고 보이는 위치는 지금 위치 또는 선택 해시 기준으로 맞춤).
- 클립보드: `navigator.clipboard`가 `tauri://localhost`에서 두 웹뷰 모두 되는지 확신할 수 없어 `execCommand` 대체 경로를 두었다. 둘 다 안 되면 `tauri-plugin-clipboard-manager`(새 의존성·capability)가 필요한데 그 결정은 이 반복에서 하지 않고 "사람이 할 일"에 적는다.
- Ctrl+휠·핀치·`+`·`-`가 웹뷰에서 앱에 오는지는 손으로만 확정된다(Tauri 창의 확대 단축키 기본 설정 포함). 안 오면 단추와 `+`·`-`만 남기고 표만 고친다.
- Radix `DropdownMenu`의 위치용 보이지 않는 `Trigger` 방식이 가상 스크롤 위에서 어긋나면 `Popover`로 바꾼다(M13).
- 주의 순서: M2~M6(데이터)와 M7(홈 이전)은 서로 독립이라 병렬로 시작해도 되지만 M10은 M7·M8·M9에, M12는 M4·M5·M10·M11에, M14는 M6·M10에 달려 있다. 한 단계가 끝나면 CI 확인 뒤 다음으로.
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md`에 "(자동 결정)"으로 올린 뒤 이어 간다.

## 검토 반영 (code-analyst, 구현 전)
구현은 아래를 위 단계 설명보다 우선한다. 결정 번호는 C1~C10 → D-087~D-096.

### 바뀐 결정
- **C4(D-090) 바꿈: 홈도 같은 배치 함수로 그린다.** D-017(확정, 사용자)의 영향 문장 "홈의 두 줄 노선도와 같은 배치 계산·그리기 코드를 쓴다"와 로드맵 2b를 그대로 지킨다. 홈은 프론트 어댑터 `homeToRouteNodes(view: HomeMapView)`가 요약을 `RouteNode`(갈라진 곳, main 끝, 지금 줄 역, 접힌 구간은 `fold`, 고친 파일은 `open`)로 바꾸고, 버전 기록과 같은 `buildRoute`(옵션: 0번 칸을 기본 브랜치로 고정, `maxLanes: 2`)와 가로 투영(곡선 꺾임, 폭에 맞춘 간격과 앞쪽 "…" 접기, 고정 굵기 trunk 3·지금 4, `annotations`로 trunk 이름·"지금 여기"·"고친 파일 N개"·upstream 칩)으로 그린다. 홈 특화 요소는 `annotations`로만 넘기고 레인·투영 코어는 모른다. 옛 `layoutTwoLine`은 표본 `HomeMapView` 10여 개(한 줄/두 줄/접힘/`main_since` 0/고친 파일 0/upstream/긴 한글 이름)에서 새 경로와 역 좌표·글자·접힘 수가 ±1px 안인지 비교하는 테스트를 통과한 뒤 지운다(첫 역 이름 고침만 의도한 차이로 기대값을 바꾼다). core는 이 반복에서 `HomeMapView`를 바꾸지 않는다(병합이 섞인 `base..HEAD`는 지금처럼 일직선).
- **C7(D-093) 바꿈: 시작 화면 데이터는 홈과 같은 요약이다.** `query_route_preview(repo, limit)`은 `HomeMapView`를 돌려준다. core의 `home_map`을 `limit`을 받는 함수(홈 3, 큰 노선도 12, 작은 노선도 6, 상한 20)로 일반화하고, 접근은 `project_summary`와 같은 `summary_root` 도우미(discover → 신뢰 확인 순서 그대로, 최근 목록에 있거나 연 저장소)로 한다. 갈라진 곳은 홈처럼 `merge_base`로 구하므로 오래 갈라진 브랜치도 main과 갈라진 곳이 보인다. `RoutePreviewView`·`RouteCommit`은 만들지 않는다. 시작 화면도 `homeToRouteNodes` → `buildRoute` → 가로 투영(`density: "preview" | "mini"`)으로 그린다.
- **C5(D-091) 보탬: 이름 없는 레인(지운 브랜치)은 레인을 여는 버전 해시로 `--graph-2`~`--graph-6` 중 하나를 고른다**(같은 기록이면 늘 같은 색). r-log의 색 있는 곁가지와 맞추고, "같은 브랜치 = 같은 색"은 이름 있는 레인에만 적용된다. `--text-muted` 레인은 쓰지 않는다.
- **C10(D-096) 새로: 줄 수 조회(읽기 전용)는 10초가 지나면 끝낸다.** 큰 커밋의 `--numstat`은 출력 없이 오래 걸릴 수 있어 출력 상한만으로는 못 끊는다. 읽기만 하므로 끝내도 반쯤 바뀐 상태가 없다(D-066의 "로컬 쓰기는 끝내지 않는다"와 부딪히지 않음, D-066 보완 줄을 단다). 못 받은 해시는 화면이 "모름"(가는 선)으로 둔다.

### 바뀐 순서와 나눈 단계
M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8 → M9 → M10a → M10b → M11 → M12a → M12b → M13 → M14 → M15 → M16.
- M7: 장면 타입(`Scene`)·`Renderer`(`palette`)·`geometry`(`textWidth`·`fitText`·경로 도우미)만 만들고 `twoLine.ts`는 새 타입으로 옮기기만 한다(모양 그대로). 첫 역 이름 고침은 M10b.
- M10a: 세로 `buildRoute`/`routeScene`/굵기/팔레트/`hitTest`.
- M10b: 가로 투영(`preview`·`mini`·홈), `homeToRouteNodes`, 홈을 새 배치로 옮기기, 옛 배치와 비교 테스트 뒤 `twoLine.ts` 삭제, 첫 역 이름이 왼쪽 끝에 붙는 문제 고침(시작 위치 `max(LEFT, textWidth(첫 이름)/2 + 8)`, 이웃 이름 `fitText`).
- M12a: `useLogPages`(`useInfiniteQuery`), `stats.ts`, 기록 무효화 도우미, `useRepoWatch`.
- M12b: 화면 머리·범위 전환·확대·축소·`shortcuts.ts`·`messages`·`HistoryList.tsx` 삭제.

### 단계별 고침
- **M1**: `docs/architecture.md`의 "노선도 보기는 아직 제안이다"(345줄 근처)와 359~360줄 근처 두 문장도 고친다. D-066에 C10 보완 줄. `contrast.test.ts`는 다크 블록 두 곳(미디어 쿼리, `data-theme="dark"`)을 각각 읽어 시험한다(지금 `colors()`는 첫 `}`까지만 읽는다).
- **M2**: `log_graph`는 리비전 뒤에 `--`를 붙이고 `--no-show-signature`를 준다(`log.showSignature=true` 개인 설정이 형식 출력에 섞이지 않게). 태그 형식 `%(*objectname)`은 한 단계만 벗기므로 태그를 가리키는 태그는 라벨에서 빠진다(architecture.md에 한 줄).
- **M3**: 출력 형식은 실제 git(먼저 `git-min` 2.34.1)으로 확인한 뒤 고정 출력 파서 테스트를 쓴다. `--no-show-signature`, 첫 버전은 `--root`를 명시한다. 러너는 stdin + 출력 2MB 상한 + 시간 제한 10초(C10)를 함께 쓰는 변형을 둔다.
- **M4**: `unpushed`는 첫 쪽(`skip == 0`)에서만 한 번 계산해(최대 500개) 돌려주고 화면이 합쳐 쓴다. 다음 쪽은 빈 목록. `tips` 상한 201개(해시 약 8KB, Windows 명령줄 한도 약 32K 안)의 근거를 architecture.md 입력 상한에 적는다.
- **M6**: C7대로 `home_map`을 일반화한다. 테스트의 "신뢰하지 않은 저장소는 git을 부르지 않음"은 "status·branches·log 호출 0"으로 정의한다(discover·신뢰 확인은 돈다). M16 성능 측정에 Windows에서 프로젝트 20개를 3개씩 불러오는 총 시간을 넣는다.
- **M12a**: `["log", repo]` 무효화가 8곳(`useRepoWatch`, `NameBox`, `BranchList`, `useSync`, `useMergeAction`, `Conflict`, `Activity` 등)이다. `useInfiniteQuery`는 무효화하면 받은 쪽을 모두 다시 받으므로, `src/lib/queries.ts`에 `invalidateLog(queryClient, repo)`(무효화 전에 `setQueryData`로 `pages`를 첫 쪽 하나로 자름)를 두고 8곳을 이 도우미로 바꾼다. 테스트: 쪽 3개를 받은 뒤 무효화하면 요청 1번.
- **M12b**: `zoomIn`·`zoomOut`은 `inTextField: "native"`, `Shortcut.scope` 타입에 `"history"`를 더하고 화면이 열릴 때만 켠다(`conflict`와 같은 방식). Ctrl+휠·핀치 판정(`wheel`의 `ctrlKey`)은 `src/lib/wheelZoom.ts`에 둔다(D-056 점검 정규식). 주 조작은 머리의 "−"·"+" 단추다(자판마다 `+` 위치가 다름, spec에 적음). 버튼·상태 글자 속 용어는 `<Term>`(예: "모든 `<Term id="branch"/>`"), 문장 속 용어만 `t(id)`. "브랜치 200개까지 보여요"는 이름과 숫자만으로 "`<Term id="branch"/>` 200개까지"로 줄인다.
- **M13**: Radix `DropdownMenu`는 `modal={false}`(열린 채 스크롤되면 닫기). Esc는 Radix가 닫으며 스스로 `preventDefault`하므로 `onEscapeKeyDown`에서 막지 않는다(막으면 안 닫힘). History의 `!e.defaultPrevented` 검사로 충분하고 테스트는 실제 Radix로 한다. 메뉴가 열린 채 목록을 누르면 바깥 누름 닫기와 "같은 행 다시 누르면 닫기"가 겹쳐 다시 열리지 않게 하고 테스트한다. 선택 행은 창 밖이어도 늘 그려 `aria-activedescendant`가 사라진 요소를 가리키지 않게 한다(테스트: 선택 행이 창 밖일 때). 반복 3 틀: `availability(info, env)`(`env`: 고친 파일 유무, 지금 브랜치, 합치는 중 여부 등 화면 밖 상태), `run(info, ctx)`의 `ctx`에 `queryClient`·`onDone`(실행 뒤 기록 다시 읽기, 선택 유지)을 넣는다.
- **M15**: 동시 3개 큐에 테스트용 초기화 함수를 둔다.
- **M16**: 성능 측정에 1만 개 저장소의 깊은 쪽(`skip=9800`)과 "지금 여기" 25쪽 이어 받기 시간을 더한다.
- "이번 반복에서 하지 않는 것" 표의 "홈 노선도 조회를 `route_preview`로 합치기 — 하지 않음"은 지운다(C7이 반대로 정함).
