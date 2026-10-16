# UX 조사

- 날짜: 2026-10-16
- 바탕: 다른 Git 앱의 문서와 사용자 평, GitHub Desktop 이슈 반응 수, Git 사용성 연구, 비개발자용 버전 기록 제품, Windows Fluent와 GNOME HIG, WCAG 2.2, 토스 UX 라이팅. 그리고 지금 `src/`와 `docs/research/img/polish/` 점검.
- 출처는 맨 아래. "확인 못 함"이라고 쓴 숫자는 결정에 올리기 전에 다시 확인한다.

## 한눈에

1. 연구가 가장 일관되게 말하는 것은 Git이 어려운 까닭이 화면이 아니라 개념이라는 점이다. 스테이징, 임시 보관, 분리된 HEAD가 혼란의 대부분을 만든다. hiGit은 이미 이 셋을 화면에서 감추는 쪽으로 가고 있고, 이 방향이 맞다.
2. 다만 감춘 일은 반드시 끝난 뒤 쉬운 말로 보여 줘야 한다. GitButler를 그만 쓴 사람의 이유가 "내 변경이 어디 갔는지 모르겠다"였다. 감추기만 하면 신뢰를 잃는다.
3. 전문가도 가장 많이 헤매는 곳은 되돌리기와 복구다. Stack Overflow에서 가장 많이 본 Git 질문이 revert와 reflog다. hiGit의 작업 내역과 백업 ref가 가장 큰 강점이 될 수 있다.
4. 지금 코드에서는 위험한 확인 창 네 곳에서 단축키가 새어 나가는 문제, 첫 저장이 이름·이메일 때문에 막히는 문제, 오류 토스트가 성공 토스트와 같은 모양인 문제가 가장 급하다.

## 1. 지금 앱에서 고칠 것

### 높음

| 문제 | 위치 | 고칠 방향 |
|---|---|---|
| 확인 창이 떠 있어도 전역 단축키가 동작한다. "고친 것 버리기" 확인 중 Ctrl+Z를 누르면 직전 작업이 조용히 되돌려진다. Ctrl+Shift+S, Ctrl+H, Ctrl+O도 같다. | `CompareFooter.tsx:131`, `StuckNotice.tsx:55`, `ClearLockButton.tsx:63`, `TrustDialog.tsx` 근처 38줄. 전역 리스너는 `useShortcut.ts:16-22` | 네 곳을 `DialogFrame`의 확인 변형으로 합친다. 대화상자가 열려 있으면 `useShortcut`이 아무것도 하지 않게 한 번 더 막는다. |
| 처음 쓰는 사람이 버전 저장을 누르면 "이름과 이메일이 설정되어 있지 않아요"만 나오고 고칠 곳이 없다. | `NameBox.tsx:179`, 설정 `GeneralSection.tsx` | 오류 자리에 이름·이메일 칸을 바로 띄운다. 정식으로는 D-126 처음 실행 화면. 설정에도 칸을 둔다. |
| 오류가 토스트로 나갈 때 성공과 같은 `--primary` 모양이고, 8초 뒤 사라지며, "자세히 보기"와 해결 버튼이 없다. 13곳. | `Toast.module.css:18-27`, `BranchList.tsx:115`, `Activity.tsx:68` 등 | 오류용 토스트를 따로 둔다. `role="alert"`, 닫을 때까지 남김, "자세히 보기"와 해결 버튼 하나. |
| 오류 문구가 "가져오기부터 해 주세요"라고 하는데 화면에는 "동기화" 버튼만 있다. | `ko.ts`의 `pushRejected`, `notFastForward` | 문구 속 행동 이름을 실제 버튼 이름과 맞춘다. 문구 속 버튼 이름이 실제 버튼에 있는지 테스트로 막는다. |

### 중간

- 불러오는 표시가 바로 뜬다. 비교 칸은 파일을 ↑↓로 넘길 때마다 "불러오는 중…"이 번쩍인다. `DiffView.tsx:27`, `Workspace.tsx:96`, `Activity.tsx:83`, `Rewrite.tsx:85`, `History.tsx:311`. 지연 표시 훅 하나로 모으고 이전 내용을 유지한다. 기준은 4장 표.
- 꺼진 버튼이 `opacity: 0.5`라서 대비가 낮고, 까닭을 말하지 못한다. `Button.module.css:22`. 역 메뉴처럼 `aria-disabled`와 툴팁을 쓰는 `disabledReason` 속성으로 일반화한다.
- 원격 확인 실패가 6px 색점과 `title` 하나뿐이다. `SyncButton.tsx:85-91`. 키보드로 닿지 않고 색으로만 알린다. 알림 줄에 "원격 확인 실패 · 3분 전"을 한 번 쓴다.
- 브랜치 목록이 ↑↓ 위치를 화면 읽기 프로그램에 알리지 않는다. `BranchList.tsx:328-345`. 버전 기록 목록처럼 `aria-activedescendant`를 쓴다.
- 프로젝트 메뉴에 항목이 하나뿐이고, 버튼 안에 `h1`이 있다. `ProjectMenu.tsx:15-23`. 최근 프로젝트, 폴더 열기, 복제, 폴더 위치 열기를 넣는다.
- 단축키가 눈에 보이는 곳이 거의 없다. 상단 바 버튼 툴팁에 단축키를 붙이고, 단축키 표를 둔다. `SHORTCUTS` 표가 이미 한 곳에 있다.
- 파일 줄은 한 줄당 Tab이 두 번이다. 파일 100개면 Tab 200번. 목록 하나를 Tab 한 번으로 하고 안에서는 방향키로 움직인다.
- `outline: none`이 12곳이다. 모두 대신할 포커스 표시가 있는지 확인하고, 포커스 테두리를 전역 `:focus-visible` 하나로 모은다.

### 낮음

- 토스트가 한 번에 하나라서 새 토스트가 "되돌리기" 단추를 지운다. `toastStore.ts:30`. 되돌리기 단추가 있는 토스트는 다음 토스트가 오면 작업 내역 안내로 바뀌게 한다.
- 역을 누르기만 해도 메뉴가 열린다. `RouteList.tsx:197`. 선택과 메뉴 열기를 나눈다.
- 클립보드 실패가 "예상하지 못한 문제"로 뭉개진다. `StationMenu.tsx:41`. 이미 있는 `copyFailed`를 쓴다.
- `Term`이 모두 Tab 정지라 Tab 횟수가 늘어난다. `Term.tsx:68-75`. 문장 속 Term만 포커스되게 한다.

### 잘 되어 있는 것

쉬운 말과 용어 한 경로, 괄호 금지 테스트, 되돌릴 수 없는 일만 묻는 원칙, 한글 입력 중 단축키 처리, 버전 기록 목록의 접근성 구조, 동기화가 오래 걸릴 때의 단계별 표시, 대비 테스트. 이 틀은 유지한다.

## 2. 다른 Git 앱에서 배울 것

| 장면 | 본받을 것 | 피할 것 |
|---|---|---|
| 처음 실행·로그인 | GitHub Desktop은 브라우저 로그인 뒤 토큰을 OS 키체인에 둔다. "자격 증명이 틀림"이면 "다시 로그인" 버튼 하나. | GitKraken, Sourcetree는 로컬 폴더를 열 때도 계정을 요구해 평이 나쁘다. |
| 복제 | GitKraken처럼 주소 입력 대신 "내 GitHub 프로젝트" 목록에서 고른다. hiGit은 D-133으로 이미 한다. | 주소만 받는 칸 |
| 버전 저장 | GitHub Desktop은 바뀐 파일이 모두 체크된 채 시작한다. 이걸 끄는 옵션 요청은 반응 40개에 그쳤다. 대부분은 기본값에 만족한다. | 스테이징을 먼저 배우게 하는 흐름 |
| 비교 | Fork는 이미지를 나란히, 밀어 보기, 겹쳐 보기로 비교한다. Anchorpoint는 기록에 썸네일을 보인다. GitHub Desktop의 이미지 미리 보기 요청은 아직 열려 있다. | Sublime Merge처럼 긴 줄을 자르는 비교. 글 쓰는 사람에게는 쓸 수 없다. |
| 버전 기록 | Fork는 합치기 역을 눌러 갈래를 접는다. 브랜치 이름표 색이 선 색과 같다. VS Code는 받을 버전과 올릴 버전을 기록에 미리 표시한다. | GitHub Desktop은 그래프가 초보를 헷갈리게 한다며 그래프 요청을 닫았다. 반응 178개. 단순한 기본 보기와 펼쳐 보기를 함께 두는 것이 답이다. |
| 브랜치 바꾸기 | GitHub Desktop은 고친 것이 있으면 "지금 브랜치에 두기" "새 브랜치로 가져가기" 두 가지만 묻는다. 임시 보관이라는 말은 나오지 않는다. Fork는 Ctrl+P로 이름을 쳐서 바꾼다. | 그냥 막고 오류만 보이기 |
| 충돌 | JetBrains는 왼쪽 내 것, 오른쪽 상대 것, 가운데 결과. 파일 전체 "내 것 쓰기" "상대 것 쓰기", 겹치지 않는 변경 한 번에 넣기, 처음부터 다시. | GitHub Desktop은 충돌을 외부 편집기로 보내서 "가장 혼란스럽다"는 이슈가 있다. 학생들은 충돌이 나면 다시 복제해 버린다. 빠져나갈 길이 안 보여서다. |
| 되돌리기 | GitButler는 큰 작업마다 스냅숏을 남기고, 목록에서 "이 작업 전으로"를 누른다. Tower는 브랜치 지우기, 합치기, 리베이스, 임시 보관까지 Ctrl+Z로 되돌린다. Tower는 작업 폴더를 패치로 저장해서 밖에서 바꾼 파일을 덮어쓰지 않고 안전하게 실패한다. | GitKraken은 "되돌릴 수 있을 때만" 되돌려서 될지 안 될지 예측할 수 없다. |
| 동기화 | 버튼 하나가 할 일을 말한다. "2개 올리기 · 1개 받기". Fork는 실행한 Git 명령과 출력을 작업 목록에 남긴다. | 원문 오류만 보이고 다음 할 일이 없는 경우 |
| 큰 파일 | Anchorpoint는 큰 파일이 생기면 LFS를 알아서 설정한다. 바이너리는 합치지 않고 잠근다. | GitHub 100MB 제한에 올릴 때가 되어서야 실패 |
| 속도 | Fork와 Sublime Merge가 가장 칭찬받는 점은 빠르다는 것이다. GitKraken, Sourcetree의 가장 큰 불만은 느리고 무겁다는 것이다. | |
| Linux | GitHub Desktop의 "Linux판" 요청이 반응 4,853개로 저장소에서 가장 많다. Ubuntu 지원이 잘 되어 있으면 그 자체로 차별점이다. | |

## 3. 비개발자 제품과 연구에서 배울 것

### 연구

- MIT의 Git 개념 분석과 Gitless: 스테이징, 임시 보관, 브랜치마다 하나뿐인 작업 폴더가 혼란의 근원이다. Gitless는 스테이징과 임시 보관을 없애고 브랜치마다 고친 것을 따로 두었고, 사용자들은 "복잡함이 크게 줄었다"고 했다. 실험 규모는 11명이라 강한 증거는 아니다.
- 학생 연구: 브랜치를 폴더로 이해해서 `cd`로 들어가려 한다. 브랜치는 폴더가 아니라 기록이 갈라진 선이라고 그림으로 보여 줘야 한다. 충돌은 풀지 않고 다시 복제해서 피한다.
- 연구자·인문학자 조사: 머릿속 그림이 없는 것이 가장 큰 어려움이다. GUI는 "뒤에서 무슨 일이 일어나는지 숨겨서 믿기 어렵다". 쓸 때마다 다시 배운다.
- 법조계는 Word의 "비교"와 변경 내용 추적으로 버전을 다룬다. 두 파일을 빨간 줄로 비교하는 것이 익숙한 모양이다.

### 제품

| 제품 | 저장 지점 | 되돌리기 | 갈래와 합치기 |
|---|---|---|---|
| Google 문서 | 자동 버전 + "이 버전 이름 지정". "이름 있는 버전만 보기" 전환 | "이 버전 복원". 복원도 새 버전이 되고 기록은 남는다. "사본 만들기" | 없음 |
| Figma | 30분마다 자동, "버전 기록에 저장"에 제목과 설명 | "이 버전 복원". 스스로 "되돌릴 수 있는 일"이라고 쓴다. | 브랜치 → main에서 업데이트 → 검토하고 합치기 → 합친 브랜치는 보관 |
| Overleaf | 자동 기록 + 라벨. "현재 상태" 표시 | 파일 하나 또는 프로젝트 전체 복원, zip 내려받기 | 없음 |
| Dropbox | 저장할 때마다 | 버전 복원 | 동시에 고치면 "충돌된 사본" 파일을 옆에 만든다 |
| Abstract | 시각적 커밋 | 복원 | 브랜치와 합치기를 쉬운 말로, 올리기·받기는 자동 |
| Kactus | Git 용어 그대로 | | 디자이너에게 가장 어려웠다는 평. 용어를 바꾸지 않은 결과 |

비개발자가 쉽게 받아들이는 것은 다음과 같다.

- 되돌리기도 새 기록이 된다. 지운 것이 없으니 안심한다.
- 저장 지점은 두 층이다. 자주 생기는 보통 버전, 그리고 이름 붙인 중요한 버전.
- "이 옛 버전의 사본 만들기"가 브랜치를 이해하는 디딤돌이다.
- 비교는 빨강과 초록으로 바뀐 곳을 문장 안에서 보여 주는 것.
- 충돌은 피하거나 고르게 한다. 표시 문자 `<<<<<<<`를 직접 고치게 하지 않는다.
- 동기화는 버튼 하나이거나 보이지 않는다.
- 기록 맨 위에는 늘 "지금"이 있다.

## 4. 데스크톱 기준

### 기다림 표시

| 걸리는 시간 | 보일 것 |
|---|---|
| 1초 미만 | 아무것도 보이지 않는다. 이전 내용을 그대로 둔다. |
| 1초 넘음 | 이때부터 표시한다. 한 번 보이면 0.5초는 유지해 깜빡이지 않게 한다. |
| 2~10초 | 모양을 아는 목록과 비교 칸은 빈 줄 모양, 작업 버튼은 버튼 안 표시 |
| 10초 넘음 | 진행 막대와 Git이 주는 개수. "파일 7/200개 받는 중" |
| 1분 넘음 | 창이 뒤에 있을 때 끝나면 OS 알림 |

hiGit은 D-159에서 300ms로 정했다. Nielsen은 1초를 권한다. 정할 것에 올린다.

### 창과 웹뷰

- Windows 기준 본문 14px, 최소 12px, 강조는 Bold 대신 Semibold. GNOME 기본 글자는 약 15px. hiGit은 D-138로 이미 맞췄다.
- GNOME은 모든 데스크톱 앱이 1024×600에서 쓸 수 있어야 한다고 한다. 지금 최소 800×560이면 들어간다. 창 크기와 위치 기억은 아직 없다. `tauri-plugin-window-state`.
- 제목 표시줄은 OS 것을 쓴다. 직접 만들면 Windows 11의 스냅 레이아웃과 GNOME의 버튼 배치를 잃는다. 지금 OS 것을 쓰고 있으니 유지한다.
- 웹페이지처럼 보이는 것들을 끈다.
  - 브라우저 기본 오른쪽 클릭 메뉴. 입력칸은 잘라내기·복사·붙여넣기만 남긴다.
  - 브라우저 단축키 F5, Ctrl+R, Ctrl+P, Ctrl+F, F12. Windows는 WebView2 설정 `AreBrowserAcceleratorKeysEnabled`, Ubuntu는 keydown에서 막는다.
  - 화면 끝에서 튕기는 스크롤. `overscroll-behavior: none`.
  - 버튼과 목록의 글자 선택. 비교 칸, 오류 원문, 버전 번호, 경로는 선택되게 둔다.
  - 버튼 위 손가락 커서. 데스크톱 앱은 화살표, 링크만 손가락.
  - 창이 처음 뜰 때 흰 화면. 테마 바탕색으로 창을 만들고 첫 그림 뒤에 보인다.
  - 이미지와 링크 끌기.
- Ubuntu 배율 125%, 150%에서 WebKitGTK 글자가 흐린 버그가 있다. 100, 125, 150, 200%에서 확인한다. 0.5px 선은 쓰지 않는다.
- Ubuntu는 `prefers-color-scheme`이 GTK 테마 설정에 따라 맞지 않을 수 있다. D-147처럼 Rust에서 읽어 `data-theme`으로 넣는 방식이 맞다.

### 접근성

- WCAG 2.2 AA: 글자 대비 4.5:1, 테두리와 포커스 3:1, 누를 곳 최소 24×24px, 포커스가 토스트나 고정 머리에 가려지지 않기, 끌기로 하는 일은 클릭이나 키보드로도.
- 목록은 Tab 한 번으로 들어가고 안에서는 방향키. 오른쪽 클릭 메뉴는 메뉴 키와 Shift+F10으로도 연다.
- 화면 읽기 프로그램: Windows는 NVDA와 내레이터, Ubuntu는 Orca로 폴더 열기, 버전 저장, 동기화를 한 번씩 해 본다. Orca와 WebKitGTK는 긴 목록에서 느리므로 목록 가상화를 유지한다.

## 5. 한국어 문구

- 토스 UX 라이팅 원칙: 다음에 일어날 일을 예고, 군더더기 빼기, 빈 문장 빼기, 핵심 하나, 말하듯 쉽게, 강요하지 않기, 누구나 아는 말, 읽는 사람의 마음.
- 말투는 해요체. "~하시겠습니까?"는 쓰지 않는다. 확인 창 제목은 물음 대신 결과를 쓴다. "이 변경은 되돌릴 수 없어요".
- 두 버튼 중 물러나는 쪽은 상황에 따라 "닫기"나 "다음에". 위험 버튼은 결과를 이름으로 쓴다. "고친 것 버리기", "덮어쓰며 올리기". 위험 버튼은 Enter 기본값이 아니다.
- 글자 배치: 본문에 `word-break: keep-all`. 없으면 한글 낱말이 중간에서 끊긴다. 지금 코드에 없다. 경로, 버전 번호, 브랜치 이름은 `overflow-wrap: anywhere`. 개수와 시각은 `tabular-nums`. 본문 줄 높이 1.6 안팎, 화면 문구 1.45 안팎.
- 비교 칸 고정폭 글꼴은 한글이 섞인 줄에서 폭이 맞도록 JetBrains Mono 뒤에 D2Coding을 둔다.

## 6. hiGit에 할 일

순서는 효과와 크기를 함께 본 것이다.

1. 높음 4건을 바로 고친다. 1장.
2. 브랜치 바꾸기에서 고친 것이 있을 때 "여기 두기"와 "가져가기" 두 가지를 묻는다. 지금은 Git이 거절하면 오류다. 임시 보관이라는 말은 나오지 않게 한다. 바꾼 뒤에는 "고친 파일 3개를 '디자인-수정'에 두고 왔어요"처럼 무엇을 했는지 토스트로 말한다.
3. 오류 문구 틀을 정한다. 무슨 일이 있었는지, 무엇을 하면 되는지 한두 문장, 해결 버튼 하나, 원문은 "자세히 보기"와 복사 버튼. 가장 먼저 다룰 것은 로그인 만료, 100MB 넘는 파일, 다른 프로그램이 쓰는 중, 올리기 거절.
4. 작업 내역을 GitButler처럼 "이 작업 전으로 되돌리기" 목록으로 보인다. 줄마다 되돌릴 수 있는지 표시해서 Ctrl+Z가 될지 안 될지 미리 알 수 있게 한다. 홈에서 작업 내역으로 가는 길이 보이게 한다.
5. 고친 것 버리기로 버린 파일 내용은 백업 ref나 OS 휴지통에 남긴다. 지금은 되돌릴 수 없는 일로 묻고 있다. 남길 수 있다면 확인 창 대신 되돌리기 토스트로 바꿀 수 있다.
6. 이름 붙인 버전을 노선도에서 큰 역으로 그리고, "이름 붙인 버전만 보기"를 둔다. 태그를 쓴다.
7. 처음 하는 일마다 한 번만 뜨는 짧은 안내. 첫 브랜치, 첫 충돌, 첫 동기화, 첫 되돌리기. 예: "브랜치는 폴더가 아니라 기록이 갈라진 선이에요. 파일은 그대로 이 폴더에 있어요." 닫으면 다시 뜨지 않고, 설정에서 다시 켤 수 있다.
8. 이미지 비교. png, jpg, svg를 나란히, 겹쳐 보기, 밀어 보기. 버전 기록의 이때 파일 보기에서 썸네일.
9. 동기화 버튼 옆에 마지막 동기화 시각, 버튼 툴팁에 할 일 "2개 올리기 · 1개 받기". 노선도에 받을 버전을 흐린 역으로 미리 보인다.
10. 큰 파일: 50MB 넘는 파일을 버전에 넣으려 하면 한 번 알리고 LFS 설정을 권한다. 100MB 넘으면 GitHub이 거절한다는 것을 미리 말한다.
11. 명령 팔레트 Ctrl+K. `action-meta.json`의 모든 액션, 브랜치 바꾸기, 최근 프로젝트를 찾는다. 한글 초성 검색 "ㅂㄹㅊ"도 된다. 이름 옆에 단축키를 보인다.
12. 웹페이지처럼 보이는 것 끄기. 4장 목록. `crates/platform`과 앱 시작 코드에서.
13. 창 크기와 위치 기억, 패널 폭 조절. 패널 폭은 따로 보낸 패널-보내기와 같은 일이다.
14. 자주 쓰는 프로젝트와 브랜치 고정. GitHub Desktop 요청 반응 61개, 30개.
15. 화면 읽기 프로그램과 Ubuntu 배율 확인을 수동 점검 목록에 더한다.

## 정할 것

- 브랜치를 바꿀 때 고친 것이 있으면 "여기 두기"와 "가져가기"를 묻는다. 기본은 가져가기.
- 오류는 전용 토스트나 알림 줄로 보이고, 해결 버튼 하나와 "자세히 보기"를 둔다. 닫을 때까지 남는다.
- 불러오는 표시 지연은 지금 300ms를 유지한다. 한 번 보이면 0.5초는 유지한다.
- 처음 하는 일 안내는 한 번만, 설정에서 다시 켤 수 있다.
- 버린 파일 내용은 백업 ref로 남겨 되돌리기 토스트를 준다. 남기지 못하는 경우만 확인 창.
- 명령 팔레트 Ctrl+K, 초성 검색 포함.
- 브라우저 기본 메뉴와 단축키는 끈다. 버튼 커서는 화살표.

## 출처

- Perez De Rosso, Jackson, What's Wrong with Git?: https://dspace.mit.edu/bitstream/handle/1721.1/105233/onward009-perez_de_rosso.pdf
- 같은 저자, Purposes, Concepts, Misfits, and a Redesign of Git: https://blog.acolyer.org/2016/10/25/purposes-concepts-misfits-and-a-redesign-of-git/
- Isomöttönen, Cochez, Challenges and Confusions in Learning Version Control with Git: https://jyx.jyu.fi/bitstream/123456789/44859/1/dvcsmisconceptionsfinal%203.pdf
- Yang 외, Do Developers Really Know How to Use Git Commands?: https://neverworkintheory.org/2022/09/19/developer-git-knowledge.html
- Rampin, Nguyen, A Behavioral Approach to Understanding the Git Experience: https://scholarspace.manoa.hawaii.edu/items/4c69fe96-2659-4046-9127-471028865e6c/full
- Chris Coyier, GitButler 사용기: https://chriscoyier.net/2024/04/05/11263/
- Diversion 창작자 조사: https://www.diversion.dev/blog/survey-results-2024-version-control-for-game-dev-creative-industries---virtual-production-archviz-xr
- GitHub Desktop 이슈: 그래프 https://github.com/desktop/desktop/issues/1634 , 충돌 https://github.com/desktop/desktop/issues/5400 , Linux https://github.com/desktop/desktop/issues/1525 , 즐겨찾기 https://github.com/desktop/desktop/issues/8183 , 브랜치 고정 https://github.com/desktop/desktop/issues/15767 , 이미지 미리 보기 https://github.com/desktop/desktop/issues/2981
- GitHub Desktop 브랜치 바꾸기: https://github.com/github/docs/blob/main/content/desktop/making-changes-in-a-branch/managing-branches-in-github-desktop.md
- Fork: https://fork.dev/blog , https://fork.dev/blog/posts/quick-launch/
- Tower 되돌리기: https://www.git-tower.com/features/undo , https://www.git-tower.com/blog/how-we-built-undo
- GitKraken 되돌리기: https://help.gitkraken.com/gitkraken-client/undo-and-redo
- GitButler 작업 기록: https://docs.gitbutler.com/features/timeline
- JetBrains 충돌: https://jetbrains.com/help/idea/resolve-conflicts.html
- VS Code 소스 제어: https://code.visualstudio.com/docs/sourcecontrol/overview
- Anchorpoint: https://www.anchorpoint.app/features/git-for-artists
- Google 문서 버전 기록: https://support.google.com/docs/answer/190843
- Figma 버전 기록과 브랜치: https://help.figma.com/hc/en-us/articles/360038006754 , https://help.figma.com/hc/en-us/articles/360063144053
- Overleaf 기록: https://docs.overleaf.com/writing-and-editing/history-and-versioning
- Abstract, Kactus, Plant 비교: https://blog.prototypr.io/abstract-vs-kactus-vs-plant-a-guide-of-version-control-solutions-for-sketch-7da0a8ab5105
- Windows 글자, 간격, 제목 표시줄, 포커스: https://learn.microsoft.com/en-us/windows/apps/design/style/typography , https://learn.microsoft.com/en-us/windows/apps/design/style/spacing , https://learn.microsoft.com/en-us/WINDOWS/APPS/design/basics/titlebar-design
- WebView2 설정: https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/winrt/microsoft_web_webview2_core/corewebview2settings
- GNOME HIG: https://developer.gnome.org/hig/guidelines/adaptive.html , https://developer.gnome.org/hig/guidelines/keyboard.html , https://developer.gnome.org/hig/patterns/feedback/toasts.html , https://developer.gnome.org/hig/patterns/feedback/dialogs.html
- WebKitGTK 배율 버그: https://bugs.webkit.org/show_bug.cgi?id=271714
- WCAG 2.5.8: https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html
- Nielsen 기다림 표시: https://www.uxtigers.com/post/progress-indicators
- 토스 UX 라이팅: https://toss.im/tossfeed/article/uxwriter-interview
- 한글 줄바꿈: https://daleseo.com/css-text-wrap/
