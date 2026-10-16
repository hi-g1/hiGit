# 012 UX 다듬기 (웹 느낌 끄기·오류 안내·브랜치 바꾸기·키보드·명령 팔레트·이미지 비교)
- 관련 결정: D-006, D-010, D-015, D-020, D-024, D-029, D-033, D-037, D-042, D-043, D-045, D-047, D-049, D-054, D-056, D-057, D-064, D-072, D-074, D-076, D-080, D-085, D-087, D-092, D-107, D-136, D-138~D-164(다듬기 011 결과), **사용자 결정 D-165~D-171**(뒤집지 않는다). 새로 정할 것은 아래 "결정 후보" C1~C21이고, 첫 단계(U1)가 `docs/decision.md` 맨 아래에 D-172~D-192를 "(자동 결정)"으로 올린다. 사용자 결정 D-165~D-171에는 "(자동 결정)"을 붙이지 않는다. 사람에게 묻지 않고 추천안으로 진행하되, 사용자 결정과 어긋나는 구현이 필요해 보이면 멈추고 progress "사람이 할 일"에 적는다.
- 관련 명세: spec.md "화면 구성"의 작업 공간·버전 기록·작업 내역·설정 행, "진입점"(단축키), 용어표(`park`, `discard`)와 "위험도" 문단·"오류 메시지 원칙". design.md "부품 규칙"(버튼·토스트·알림 줄·브랜치 목록 팝오버·역 메뉴·확인 대화상자·불러오는 표시)·"문구"·"글꼴". architecture.md "되돌리기와 UndoJournal"(`UndoKind`)·"프론트와 백엔드 계약"(명령 목록)·"보안"·"OS별 처리"·"새 액션 추가 절차".
- 근거 자료(결정 아님): `docs/research/ux.md`(다른 Git 앱·연구·WCAG/Fluent/GNOME 기준과 코드 점검). 그 문서의 "정할 것"은 D-165~D-171이 이미 정했다. 조사 내용을 근거로 한 구현은 모두 이 계획의 결정 후보를 거쳐 decision.md에 오른다.
- 앞선 계획: 010(반복 4b), 011(다듬기). **이 계획은 다듬기(011)·반복 4b 나머지(010-H3부터)·반복 4c가 끝난 코드 위에서 시작하고 반복 5 앞에 들어간다.** 011의 P19(로딩 지연·말줄임·포커스 통일·Tab 순서)·P17b(파일 줄 오른쪽 메뉴)·P18(압축·실행 파일 알림), 010의 H5(처음 실행 패널)·H10(고급 설정)이 끝났다고 보고 쓴다. 구현을 시작할 때 아래 "현재 상태"를 `rg`로 한 번 다시 확인한다(줄 번호는 011-P11 시점).
- 단계 이름은 U1~U31이고 `docs/progress.md`와 커밋 본문에는 "012-U3"처럼 계획 번호를 붙여 적는다(005·010·011의 단계 이름과 겹치지 않게). 진행: 사람 확인 없이 Claude가 계획 → code-analyst 검토 → 아래 순서대로 구현, 한 단계 = 한 커밋(메시지 `<영역>: <요약>`).

## 목표
앱이 웹페이지가 아니라 데스크톱 프로그램처럼 굴고(브라우저 메뉴·단축키·손가락 커서·글자 끌림·흰 화면이 사라짐), 오류는 "무슨 일인지·무엇을 하면 되는지·버튼 하나"로 말하고, 브랜치를 바꿀 때 고친 파일을 "가져가기/여기 두기"로 고르고, 버린 파일도 되돌릴 수 있다. 키보드와 화면 읽기 프로그램으로 목록을 다루고, Ctrl+K 명령 팔레트(한글 초성 검색)·단축키 표·이미지 비교·프로젝트와 브랜치 고정·처음 하는 일 안내가 생긴다.

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04, Windows 11)에서 손으로, 가능한 것은 자동 테스트로 확인한다. 손 확인은 `docs/progress.md` "사람이 할 일"에 모은다(U31).
- 웹 느낌: 릴리스 빌드에서 마우스 오른쪽 클릭에 브라우저 메뉴가 뜨지 않고(입력칸은 잘라내기·복사·붙여넣기·모두 선택만, 선택한 글자는 복사만), F5·Ctrl+R·Ctrl+P·Ctrl+F·F12·Alt+←가 아무 일도 하지 않는다. 버튼·목록 위 커서는 화살표(링크만 손가락), 버튼·목록 글자가 끌려 선택되지 않고 비교 칸·오류 원문·버전 번호·경로는 선택된다. 화면 끝에서 튕기지 않고 이미지·링크가 끌려가지 않는다. 다크 모드로 켜도 흰 화면이 보이지 않는다. 본문은 `word-break: keep-all`이고 개수·시각 숫자는 같은 폭이다.
- 버튼·토스트·기다림: 꺼진 버튼은 투명도 대신 읽히는 색이고 까닭이 툴팁과 `aria-describedby`로 나온다. 누르는 곳은 24×24px 이상(WCAG 2.5.8). 되돌리기 토스트는 다음 토스트가 와도 자기 시간(8초)을 채우고(최대 셋), 토스트가 포커스를 가리지 않는다. 불러오기는 300ms가 넘을 때만 보이고 한 번 보이면 0.5초 유지, 그동안 이전 내용이 그대로이며 처음 로드는 빈 줄 모양, 작업 버튼은 버튼 안에 진행 표시가 있다.
- 오류: 모든 오류 코드가 `ERROR_FIX` 표에 해결 버튼 하나(또는 없음)를 갖고, 문구 속 행동 이름은 실제 버튼 이름과 같다(시험). 오류의 "자세히 보기"에 복사 버튼이 있다. 로그인 만료·100MB 넘는 파일 거절·다른 프로그램이 파일을 쓰는 중·올리기 거절이 각각 쉬운 문구와 버튼으로 나온다. 50MB 넘는 파일은 저장 전에 알리고 100MB 넘으면 "GitHub가 올리기를 거절해요"를 미리 말한다.
- 되돌릴 수 있는 일: 고친 파일이 있는 채로 브랜치를 바꾸면 "가져가기"(기본)·"여기 두기"를 묻고, 겹쳐서 가져갈 수 없으면 까닭과 함께 "여기 두기"만 쓸 수 있다. 바꾼 뒤 토스트가 한 일을 말하고 Ctrl+Z가 둘 다 되돌린다. 고친 것 버리기는 확인 창 없이 되고 토스트의 "되돌리기"/Ctrl+Z가 파일을 되살린다(백업을 남기지 못해 버리지 않은 경우뿐). 작업 내역의 줄마다 Ctrl+Z로 되돌릴 수 있는지가 보이고 "이 작업 전으로 되돌리기"가 있다.
- 길 찾기: 원격 확인 실패가 알림 줄("원격 확인 못 함 · 3분 전")과 해결 버튼으로 나오고(6px 색점 없음), 동기화 버튼 툴팁이 "2개 올리기 · 1개 가져오기"를 말하며 마지막 확인 시각이 옆에 있다. 상단 바 버튼 툴팁에 단축키가 있고 Ctrl+/가 단축키 표를 연다. Ctrl+K가 명령 팔레트를 열어 액션·브랜치 바꾸기·최근 프로젝트·화면 이동을 찾고 "ㅂㄹㅊ"으로 "브랜치"를 찾는다.
- 키보드·화면 읽기: 파일 목록은 Tab 한 번에 들어가 ↑↓·Space·메뉴 키로 쓰고, 버튼 안의 `<Term>`은 Tab 정지가 아니다. 브랜치 목록은 `aria-activedescendant`로 고른 줄을 알리고 키보드로 줄 메뉴를 연다. 노선도에서 역을 누르면 고르기만 하고 메뉴는 두 번째 누름·`⋯` 단추·Enter·오른쪽 클릭·메뉴 키로 연다. `outline: none` 12곳 모두 대신하는 포커스 표시가 있다(시험).
- 고정·안내: 시작 화면·프로젝트 메뉴·브랜치 목록에서 프로젝트(10개)와 브랜치(프로젝트당 10개)를 고정하면 맨 위에 남는다. 첫 브랜치·첫 충돌·첫 동기화·첫 되돌리기에 짧은 안내가 한 번만 뜨고, 닫으면 다시 뜨지 않으며 설정 "일반"에서 끄고 다시 켤 수 있다.
- 이미지: png·jpg·gif·webp·svg 파일의 비교에서 나란히·겹쳐 보기·밀어 보기가 된다(한쪽 3MB 이하). 노선도에서 태그 버전은 큰 역이다. (선택) 받을 버전이 흐린 역으로 미리 보인다. 비교 칸·충돌 코드의 한글이 섞인 줄이 칸이 맞는다(D2Coding).
- `pnpm check:full`, 두 OS CI(`git-min` 포함)·`cargo deny check` 통과. code-analyst 점검 위반 0건. 화면 읽기 프로그램·Ubuntu/Windows 배율 확인은 "사람이 할 일"에 있다.

## 범위
- **이번에 한다**: ux.md 1장 "중간"·"낮음" 전부, 4장(기다림 표시·창과 웹뷰·접근성), 5장(한국어 문구·글자 배치), 6장 2~15 중 아래 "뺀다"와 "이미 계획에 있음"이 아닌 것, 그리고 D-165~D-171 구현.
- **뺀다**(까닭과 갈 곳):
  - 10초 넘는 작업의 진행 막대·"파일 7/200개 받는 중": Git의 `--progress` 줄을 해석해야 해서 D-003·D-080("출력이 오는지만 본다")과 부딪힌다. 지금의 "동기화 중 · N초"·"응답 없음"·"취소"(D-080)를 유지한다. 사람에게 묻는다.
  - 1분 넘는 작업이 창 뒤에서 끝났을 때 OS 알림: 알림 플러그인(새 의존성·capability·cargo-deny)이 필요하다 → 반복 5 후보, 사람에게 묻는다.
  - 버전 기록 "이때 파일 보기" 목록의 이미지 썸네일: 이미지를 줄이는 코드가 없어 목록마다 원본을 보내야 한다. 고른 파일의 큰 비교만 한다(C18).
  - 프로젝트 메뉴의 "폴더 위치 열기"·창 크기·위치 기억: 반복 5(D-161).
  - Windows WebView2 전용 `AreBrowserAcceleratorKeysEnabled` 설정: OS 분기가 platform 밖에 생기거나 `webview2-com`·unsafe가 든다. 먼저 JS로 막고(U3) 두 OS 손 확인에서 새는 키가 있으면 반복 5의 platform 함수로 옮긴다(사람에게 묻는다).
  - 자동 접근성 검사 도구(axe): C21.
- **이미 계획에 있음**(중복 단계를 만들지 않는다):

| ux.md 항목 | 계획·결정 | 이 계획이 더 하는 것 |
| --- | --- | --- |
| 불러오는 표시 300ms 지연(`DiffView`·`Workspace`·`Activity`·`Rewrite`·`History`) | 011-P19 ②, D-159 | D-167: 한 번 보이면 0.5초 유지, 이전 내용 유지, 빈 줄 모양, 버튼 안 표시 → **U7이 011-P19 ②를 넓힌다** |
| 포커스 테두리 전역 하나·`outline` 개별 선언 정리 | 011-P19 ① | `outline: none` 12곳에 대신하는 표시가 있는지 확인하는 시험 → U5 |
| Tab 순서(상단 바→노선도→이름 칸→파일 목록→비교 칸)·긴 이름 말줄임 | 011-P19 ③④ | 파일 줄당 Tab 둘·`<Term>` Tab 정지 → U19 |
| 파일 줄 오른쪽 메뉴(경로 복사·버리기), 메뉴 키·Shift+F10 | 011-P17b | "버리기"의 확인 창을 되돌리기 토스트로 바꾼다 → U13 |
| 창 크기·위치 기억, "폴더에서 보기"·"폴더 위치 열기" | 반복 5, D-161 | 없음 |
| 패널 폭 조절 | D-164(구현됨, `ui/Splitter`) | 없음 |
| 처음 실행 패널, 설정 "일반"의 이름·이메일 | 010-H5, D-126(이름·이메일 칸은 앞선 처리로 일부 먼저) | 없음 |
| 비밀·압축·실행 파일 알림 | 011-P18, D-158 | 50MB·100MB 두 단계 문구와 제목 → U10 |
| 글자 크기·언어·화면 모드·설정 "고급" | 011-P5~P11·P19, D-138~D-145 | 없음 |
| 클립보드 실패가 "예상하지 못한 문제"로 뭉개짐 | **이미 처리됨**: `features/history/menu/items.tsx:87`이 `copyText` 결과로 `copied`/`copyFailed`를 띄우고 `StationMenu.test.tsx`가 확인한다 | 없음(`copyText` 쓰는 곳은 둘뿐) |
| 본문 14px·OS 제목 표시줄 유지·Ubuntu 다크 따라가기·목록 가상화(Orca 대비) | D-138, 현재 코드, D-151, `RouteList` | 없음(유지) |

- **앞선 처리**(011에 메인 세션이 먼저 넣는다. 이 계획의 단계가 아니다. 구현 상태는 시작할 때 확인한다):
  1. 확인 창 중 전역 단축키 새는 문제 → 공용 `ui/ConfirmDialog`(010-H6을 앞당김)와 `useShortcut`의 `modalOpen()` 차단.
  2. 처음 버전 저장이 이름·이메일 때문에 막히는 문제 → 010-H3·H4 앞당김, 버전 저장 오류 자리의 이름·이메일 칸과 설정 "일반"의 이름·이메일 줄. 시작 화면 처음 실행 패널은 010-H5에 남음.
  3. 오류 토스트가 성공과 같은 모양 → 오류 전용 토스트(`toastStore.showError`, `Toast.module.css .errorToast`, D-166).
  4. 오류 문구 속 행동 이름이 실제 버튼과 다름 → 문구 정리와 시험.
- 새 Tauri 명령 5개(`app_ready`, `recent_pin`, `recent_pin_branch`, `query_switch_plan`, `query_image_pair`). 새 액션 없음(`switch` 입력·`discard` 되돌리기 종류 변경). 바뀌는 `core::view`: `ActionMetaView.needsInput`, `SwitchInput.dirty`, `SwitchPlanView`(신규), 갈아타기 결과(`files`·`carried`), `RecentProject.pinned/pinnedBranches`, `RecoveryItem.undo/beforeOid`, `AppSettingsView/Patch`(`tipsOn`·`tipsSeen`), `ImagePairView`(신규), `HomeMapView.incoming`(U29, 선택), `SecretRule::HugeFile`, `ErrorCode` 2개. 새 의존성: `base64`(이미 트리에 있음, core에 직접 추가: C18)와 코드 글꼴 npm 패키지 1개(C20, 크기 확인 뒤).

## 현재 상태 (코드로 확인한 사실, 011-P11 시점)
- 웹 느낌: `src/styles/base.css`에 `user-select`는 칸 나눔선을 끄는 동안(`:root[data-resizing]`)뿐이고 `overscroll-behavior`·`word-break`·`font-variant-numeric`·`-webkit-user-drag`가 없다. `cursor: pointer`가 22개 CSS 파일 28곳(예 `ui/Button.module.css:11`), `outline: none`이 12곳(`RouteList`·`StationMenu`·`Dialogs`·`BranchList:214`·`CreatePrDialog`×2·`NameBox:45`·`CardMap:33`·`CloneDialog:23`·`Toast:13`·`ProjectMenu:52`·`ChangedFiles:56`). 글자 선택을 끈 곳은 `RouteList`·`DiffView`(줄 번호)·`CardMap`뿐이다. 마우스 오른쪽 클릭은 `RouteList`(`RouteList.tsx:203-211`)만 직접 처리하고(`preventDefault`) 나머지는 웹뷰 기본 메뉴(새로 고침·검사 등)가 뜬다. `lib/shortcuts.ts`의 `RESERVED`(`F5`·`F12`·`Ctrl+F/G/P/R/U/W/Q`·`Ctrl+Shift+I/J/C`·`Alt+←/→`)는 "앱 단축키로 쓰지 않는 키"의 표일 뿐 그 키를 막지는 않는다. `PLANNED`에 `Ctrl+K`.
- 창: `tauri.conf.json`의 창에 `visible`·`backgroundColor`가 없고 `zoomHotkeysEnabled: false`만 있다(Tauri 2.12). 앱은 `lib.rs`의 `setup`에서 진행 알림 싱크만 건다. CSP `img-src 'self' data:`는 이미 data URL 이미지를 허용한다.
- 버튼: `ui/Button.module.css:20-23`이 꺼진 버튼을 `opacity: 0.5`로 그린다. 까닭을 말하는 꺼짐은 역 메뉴(`StationMenu.tsx`의 `aria-disabled`+`Tooltip`)뿐이고 `NameBox`(버전 저장)·`CompareFooter`(임시 보관)·`Rewrite`(정리하기)는 `disabled`만 쓴다. `Checkbox.module.css`의 상자는 1rem(16px)이고 눌림 영역이 같다.
- 토스트: `ui/toastStore.ts`는 한 번에 하나(`toast: ToastMessage | null`)라 새 토스트가 "되돌리기" 단추를 지운다. 오류는 `showError`(앞선 처리)로 `Toast.tsx`가 `ErrorNotice` 판을 닫을 때까지 남긴다. 성공 토스트는 8초(`TOAST_DURATION_MS`).
- 기다림: "불러오는 중…"(`m.loading`)이 `Workspace.tsx:102`·`DiffView.tsx:27`·`History.tsx:313`·`Activity.tsx:84`·`Rewrite.tsx:85` 다섯 곳에서 `isPending`이면 바로 뜬다(011-P19 ② 전). `Workspace.tsx:127`의 `<DiffView key={selected.path}>`는 파일을 바꿀 때마다 새로 만들어져 이전 내용이 남지 않는다. `useDiff`(`lib/queries.ts:265`)는 `placeholderData`가 없다. 작업 버튼의 진행 표시는 동기화("동기화 중 · N초")뿐이다.
- 오류 문구: `messages/ko.ts:421-497`의 `errors`(약 90개 코드). 코드마다 해결 버튼은 화면이 제각각 붙인다(`SyncNotice`의 `ClearLockButton`, `useUndo`의 "작업 내역" 등). `ErrorNotice`는 `children`으로 단추를 받고 "자세히 보기"(`<details>`) 안에 원문만 있다(복사 없음). `crates/git/src/failure.rs`의 `RULES`에는 100MB 거절("GH001: Large files detected")·파일 사용 중("unable to unlink"·"Unlink of file")이 없어 둘 다 `Other`→`gitFailed`("Git 작업이 실패했어요")로 떨어진다.
- 큰 파일: `core/src/secrets.rs:8`의 `LARGE_BYTES = 50MB`가 `SecretRule::LargeFile`("50MB 넘는 파일")을 만든다(`query.rs:617`). 100MB 규칙과 "GitHub가 거절해요" 문구가 없다. 알림 제목은 "키나 비밀번호로 보이는 파일 N개"(`ko.ts:112`, 011-P18이 `heavyTitle`을 더한다).
- 브랜치 바꾸기: `core/src/actions/switch.rs:93-99`가 고친 파일이 있으면 늘 떠나는 브랜치의 `refs/higit/parked/<브랜치>`에 맡기고(`SwitchInput { branch, remote }`에 고르는 칸이 없다) 도착한 브랜치의 맡긴 것이 있으면 겹침 검사 뒤 꺼낸다. 되돌리기는 `undo/run.rs:512`의 `undo_switch`가 같은 흐름으로 이전 브랜치로 간다. `BranchList.tsx:127-151`의 `run`은 묻지 않고 `executeAction`한다.
- 버리기: `actions/discard.rs`는 `Caution`/`Irreversible`이고 `stash push`로 버리며 스냅샷을 `discard-<브랜치>` 백업 ref에 남긴다(백업을 못 쓰면 stash가 목록에 남고 `BackupFailed`, 내용은 잃지 않는다). `CompareFooter.tsx:74-98`이 `previewAction`→토큰→`ConfirmDialog`→`executeAction`을 하고 토스트는 "작업 내역" 단추다.
- 작업 내역: `RecoveryItem`(`view/recovery.rs`)에 `undone`·`restore`·`oid`(= journal 줄의 `after.head`)만 있다. 어느 줄을 Ctrl+Z가 되돌릴지는 `undo/run.rs:43`의 `stacks`(비공개)만 안다. 화면(`Activity.tsx:113-130`)의 단추는 "이 시점으로 되돌리기".
- 동기화: 원격 확인 실패는 `syncStore.fetchFailedAt`(메모리)로 `SyncButton.tsx:85-92`의 6px `role="img"` 색점과 `title`뿐이다(키보드 포커스 불가, 색으로만 알림). 마지막 확인 시각은 저장하지 않는다. 버튼 툴팁이 없다.
- 단축키: `TopBar.tsx`의 버튼은 `aria-keyshortcuts`만 있고 눈에 보이는 단축키는 `Kbd` 몇 곳(시작 화면·버전 이름 칸·토스트)뿐이다. 단축키 표 화면이 없다.
- 키보드: `ChangedFiles.tsx`는 줄마다 체크 상자와 이름 단추 두 Tab 정지이고 머리에도 둘(체크 상자·`tabIndex=0` 글자)이다(011-P16~P17b가 줄 모양을 바꾼다). `ui/Term.tsx:63-73`은 `focusable`이 기본 참이라 호출부 약 40곳 중 `focusable={false}`를 빼먹은 곳이 Tab 정지가 된다(`HunkPanel`·`PullRequests`·`History`·`PendingNotice`·`Activity`). `BranchList.tsx:326-370`은 `<ul>`+줄마다 `<button>`이고 ↑↓로 고른 줄은 `aria-current`로만 알린다. `RouteList.tsx:191-201`은 `pointerdown`에서 `onSelect`와 `onActivate`(메뉴 열기)를 함께 부른다(`History.tsx:163-185`).
- 프로젝트: `ProjectMenu.tsx`는 `Menu.Trigger` 안에 `<h1>`이 있고 항목이 "다른 프로젝트 열기" 하나다. 최근 목록은 `core/src/recent.rs`(`RecentEntry { path, name, last_opened_at }`, 20개, 형식 버전 1)이고 고정이 없다.
- 설정: `core/src/settings.rs`의 `AppSettings`는 `Copy`이고 칸마다 `#[serde(default)]`라 칸을 더해도 옛 파일이 읽힌다. 안내 표시 칸이 없다.
- 비교: `view/diff.rs`의 `FileDiffView.binary`가 참이면 `DiffView.tsx:32`가 "이진 파일이에요"만 보인다. 이미지를 읽는 길이 없다.
- 노선도: 세로 목록(`layout/route.ts`)은 태그를 이름 옆 글자로만 보이고 역 크기는 같다. 확대·축소 "태그만"(`ZOOMS`)이 이미 있다(`ko.ts:181`). 홈 요약(`HomeMapView`)에는 받을 버전의 목록이 없다(`count_between`의 수뿐).
- 글꼴: `tokens.css:42`의 `--font-mono`는 JetBrains Mono(latin) 뒤에 Pretendard를 이어 받는다(D-014). 한글이 섞인 코드 줄은 비례폭이라 칸이 어긋난다.

## 결정 후보 (U1이 D-172부터 "(자동 결정)"으로 올린다)
추천안을 적었다. 사용자 결정 D-165~D-171은 그대로 구현하는 쪽이고 아래는 그 세부다.

| 번호 | 결정(추천안) | 까닭 | 대안 |
| --- | --- | --- | --- |
| C1 (D-172) | **웹 느낌 끄기 세부(D-171).** ① 커서: 모든 컨트롤 `default`, 링크 모양(`linkStyle.module.css`)만 `pointer`, 끄는 칸·잡는 카드·입력칸은 각자 `col-resize`·`grab`·`text`. stylelint가 `cursor: pointer`를 `linkStyle`밖에서 막는다. ② 글자 선택: 버튼·목록 줄·메뉴 항목·`summary`·`label`·`kbd`는 `user-select: none`(`:where`로 낮은 우선순위), 선택되게 둘 곳은 `[data-selectable]` 속성(비교 칸 코드, 오류 원문, 버전 번호·경로, 설정의 Git 위치)과 `pre`·`input`·`textarea`. ③ `overscroll-behavior: none`(html·body와 스크롤 칸), `img, svg, a`의 `-webkit-user-drag: none`과 `dragstart`·`dragover`·`drop` 막기. ④ 오른쪽 클릭: 전역 `contextmenu`를 막고(이미 직접 처리한 곳은 그대로), 입력칸 안에서만 앱 메뉴(잘라내기·복사·붙여넣기·모두 선택)를, 선택한 글자 위에서만 "복사"를 띄운다. 메뉴는 `@radix-ui/react-dropdown-menu`를 보이지 않는 기준 위치에 열어 만들고(역 메뉴 방식) 새 의존성이 없다. 붙여넣기를 웹뷰가 막으면 항목을 꺼 두고 "Ctrl+V를 눌러 주세요" 까닭을 보인다. ⑤ 브라우저 키: `RESERVED` 표(앱 단축키 표 한 곳)의 키를 릴리스 빌드에서 `keydown` 캡처로 `preventDefault`한다(개발 빌드는 F12·Ctrl+Shift+I/J·F5·Ctrl+R을 열어 둔다). ⑥ 본문 `word-break: keep-all`+`overflow-wrap: break-word`, 경로·브랜치·해시·코드는 `overflow-wrap: anywhere`+`word-break: normal`, 숫자는 전체에 `font-variant-numeric: tabular-nums`(Pretendard가 `tnum`을 가진다), 줄 높이는 지금 1.5를 유지한다(ux.md의 1.6/1.45 사이). ⑦ 0.5px 같은 1px 미만 선을 쓰지 않는다(stylelint, WebKitGTK 125·150% 흐림). | D-171을 코드로 옮기는 한 곳(`lib/webFeel.ts`+`base.css`)을 만들고, 이미 있는 `RESERVED`·`isTextField`를 재사용한다. 입력칸 메뉴를 직접 만드는 까닭은 WebView2·WebKitGTK의 기본 메뉴를 잘라낼 방법이 JS에는 이것뿐이어서다. | 입력칸에서는 기본 메뉴를 두기(맞춤법·"검사" 같은 항목이 남음), 전역 `user-select: none`+선택 허용 목록(빠뜨리면 복사를 못 함), Windows 전용 WebView2 설정(OS 분기가 platform 밖) |
| C2 (D-173) | **첫 그림 전에 흰 화면을 보이지 않는다.** `tauri.conf.json` 창을 `visible: false`로 만들고, 화면이 첫 렌더를 커밋한 뒤 새 명령 `app_ready`(인자 없음)가 창을 보이고 포커스를 준다. JS가 안 떠도 3초 뒤 Rust가 스스로 보인다(`lib.rs` `setup`의 스레드 하나). 스키마가 지원하면 창 `backgroundColor`를 라이트 `--bg` 값으로 둔다. 확인은 `requestAnimationFrame`이 아니라 `useEffect`(숨긴 창에서 rAF가 멈출 수 있어서). | 직접 만든 창 덮개(스플래시)나 테마 읽기 없이 깜빡임이 없다. 앱 크기와 OS 분기가 늘지 않는다(`Window::show`는 공통 API). | `backgroundColor`만(다크 사용자에게는 여전히 흰색), 스플래시 창(창 둘·복잡) |
| C3 (D-174) | **꺼진 버튼은 `aria-disabled`+까닭, 누를 곳은 24px 이상.** `Button`에 `disabledReason`(문자열)을 더해 `disabled` 속성 대신 `aria-disabled="true"`+`Tooltip`+`aria-describedby`를 쓰고(키보드 포커스에도 까닭이 뜬다) 클릭은 무시한다. 모양은 투명도 대신 `--text-muted` 글자와 `--surface-sunken` 바탕(대비 시험 추가). 진행 중(`busy`)은 `disabled`가 아니라 `aria-busy`. 토큰 `--target-min: 1.5rem`. 체크 상자는 보이는 크기 16px을 두고 `::after`로 눌림 영역만 24px로 넓힌다. 까닭이 있는 곳만 쓴다(버전 저장, 임시 보관, 정리하기, 시작 화면의 복제·폴더 열기). | ux.md 1장 중간("역 메뉴처럼 `aria-disabled`와 툴팁"). 투명도로 흐리면 대비가 낮고 까닭이 없다. | 모든 `disabled`를 바꾸기(진행 중 꺼짐까지 툴팁이 뜨면 소란) |
| C4 (D-175) | **토스트: 되돌리기 토스트는 보존, 최대 셋, 포커스를 가리지 않는다.** `toastStore`가 목록(최대 3)을 들고, 새 토스트가 와도 "되돌리기" 단추가 있는 토스트(`ToastAction.kind === "undo"`)는 자기 8초를 채운다. 3개를 넘으면 가장 오래된 평범한 토스트부터 밀어낸다. 오류 토스트는 사람이 닫을 때까지 남고 둘까지(새 오류가 오면 가장 오래된 오류가 밀린다). 토스트 높이를 `--toast-clearance`로 알려 스크롤 칸이 `scroll-padding-bottom`으로 포커스를 가리지 않게 한다(WCAG 2.4.11). design.md "한 번에 하나" 문장을 바꾼다. | 단추가 사라져도 Ctrl+Z는 되지만 단추를 믿는 사람은 놓친다. ux.md의 "작업 내역 안내로 바뀌게"는 같은 효과를 내려고 토스트를 더 만들어야 해서 쌓아 두는 쪽이 단순하다. | 새 토스트가 올 때 이전 토스트를 "작업 내역" 안내로 바꾸기, 한 번에 하나 유지 |
| C5 (D-176) | **기다림 표시 규칙(D-167 구체화).** 300ms 미만은 아무것도 보이지 않고 이전 내용을 그대로 둔다(`placeholderData: keepPreviousData`, 머리와 본문이 같은 요청을 가리키게 마지막으로 받은 `request`를 함께 든다). 300ms가 넘으면: 이전 내용이 있으면 그대로 두고 머리 오른쪽에 `--text-muted` "불러오는 중…", 처음 로드(이전 내용 없음)면 `Skeleton` 빈 줄 3~6줄. 한 번 보이면 0.5초 유지(`useDelayedFlag(pending, { delay: 300, hold: 500 })`). 작업 버튼은 버튼 안 진행 표시(`Button busy`: 글자는 그대로, 오른쪽 작은 회전 표시, 모션 줄이기면 정지한 점 셋). 10초 넘는 진행 막대·1분 넘는 OS 알림은 하지 않는다(범위 "뺀다"). | 사용자 결정 D-167을 그대로 구현한다. 이전 내용 유지는 파일을 ↑↓로 넘길 때 번쩍임을 없앤다. | 1초 지연(Nielsen, D-159가 300ms로 정함), 이전 내용 없이 빈 줄만 |
| C6 (D-177) | **오류 안내 틀.** ① 문구: 한 문장 "무슨 일", 둘째 문장 "무엇을 하면 되는지"(해요체, 이름과 숫자만, 행동 이름은 실제 버튼 이름). ② 해결 버튼은 코드당 하나 이하: `src/features/errors/fixes.ts`의 `ERROR_FIX: Record<ErrorCode, FixKind>`(`none`·`reconnect`·`setIdentity`·`sync`·`openActivity`·`openHistory`·`clearLock`·`retry`)가 모든 코드를 덮어야 컴파일된다. `retry`는 호출한 쪽이 다시 할 일을 줄 때만 나온다. ③ 원문은 "자세히 보기"에 접고 안에 글자 버튼 "복사". ④ 새 실패 둘(`FileTooLarge`, `FileInUse`)을 `failure.rs`에 더한다(D-021: 문구마다 실제 git 재현 시험). 가장 먼저 다루는 넷: 로그인 만료(`githubExpired`·`httpAuthFailed` → 다시 연결), 100MB 넘는 파일 거절(`fileTooLarge` → 버전 기록), 다른 프로그램이 쓰는 중(`fileInUse` → 다시 해 보기), 올리기 거절(`pushRejected`·`remoteRefused` → 동기화). ⑤ 확인 창 문구는 `m.confirm.<이름> = { title, danger, close }` 묶음으로 모아 시험한다: 제목은 물음("?"·"하시겠습니까")이 아닌 결과 문장, 위험 버튼은 결과 이름("고친 것 버리기"·"덮어쓰며 올리기"), 물러나는 쪽은 "닫기"나 "다음에", 위험 버튼은 Enter 기본값이 아니다. | ux.md 5장·6장 3. 해결 버튼을 표 한 곳에 모으면 새 오류 코드가 생길 때 결정을 빠뜨릴 수 없다. | 화면마다 붙이기(지금 방식, 어긋남), 모든 코드에 버튼(없는 게 맞는 코드가 많음) |
| C7 (D-178) | **브랜치 바꾸기 구현(D-165).** `SwitchInput`에 `dirty: Option<DirtyChoice>`(`park`·`carry`, `skip_serializing_if`로 옛 journal 서명이 그대로 맞는다)를 더한다. 없고 고친 파일이 있으면 지금처럼 `park`. **가져가기(`carry`)**: ① 고친 파일을 `SnapshotFiles`로 stash 모양 커밋으로 만들고 ② `merge_check(base = 지금 버전, ours = 도착할 브랜치 끝, theirs = 스냅샷)`가 `Clean`일 때만 ③ 맡기고(`GitOp::Park`) ④ 갈아타고 ⑤ 같은 stash를 적용한다. 겹치면 갈아타지 않고 `CarryBlocked`(겹친 파일 5개까지). 적용한 stash는 `switch-carry-<브랜치>` 백업 ref로 옮겨 되돌리기에 쓴다. 되돌리기(`undo_switch`)는 입력의 `dirty`를 따라 같은 방식으로 돌아온다. 조회 `query_switch_plan(branch, remote)`가 `{ dirtyFiles, canCarry, blocked }`를 주고(객체만 쓰고 ref·index·작업 폴더는 그대로, `merge_check`와 같은 수준) core는 실행 때 다시 계산한다. 고친 파일이 없으면 묻지 않는다. 새 브랜치 만들기는 지금처럼 파일이 따라간다. **용어**: `park`의 쉬운 이름을 "맡겨 두기"에서 "여기 두기"로 바꾸고("맡겨 둔 파일 N개"→"두고 간 파일 N개") "가져가기"는 용어가 아니라 이 대화상자의 선택 이름이다. | D-165는 두 선택과 기본값만 정했다. 가져가기를 `git switch`의 "충돌 없으면 따라옴"만으로 하면 고친 파일이 목적 브랜치와 다른 파일이면 거절되어 기본값이 자주 실패한다. stash+겹침 검사는 이미 있는 D-054 방식이라 새 git 기능이 없다. D-024(한 낱말은 한 뜻) 때문에 "맡겨"와 "여기 두기"를 섞지 않는다. | 가져가기=`git switch` 그대로(자주 거절됨), 겹쳐도 충돌 표시를 남기고 가져가기(D-028 위반), 기억하는 설정 "항상 이렇게"(열린 질문) |
| C8 (D-179) | **고친 것 버리기는 되돌릴 수 있는 일이다(D-169).** `discard`를 `Caution`/`Inverse`(백업 ref에서 되살리기)로 바꾸고 확인 창과 확인 토큰을 없앤다. 토스트는 "버렸어요 · 되돌리기"(Ctrl+Z). 되돌리기는 그 파일들이 아직 버린 모습 그대로일 때만 하고(`status`에 그 경로가 없음) 아니면 `UndoStale`과 작업 내역 단추. 새 파일(추적 안 함)도 되살린다. 백업 ref를 못 쓰면 지금처럼 내용은 임시 보관 목록에 남고 `BackupFailed`로 알린다(그 경우도 잃는 내용은 없으므로 확인 창이 필요한 경우가 없다). `Irreversible` 집합에서 `discard`가 빠진다(D-136의 시험 갱신). 다시 하기는 없다(`stash`와 같음). | D-169("남기지 못하는 경우만 확인 창")를 `discard.rs`가 이미 stash로 먼저 남기는 구조에 맞게 구현하면, 남기지 못하는 경우는 버리기 자체가 실패하는 경우뿐이라 확인 창이 남을 자리가 없다. | 백업을 먼저 확인하는 질의+확인 창(쓸 일 없는 코드), 별도 `discard-anyway` 액션(같은 이유) |
| C9 (D-180) | **동기화 줄 정보.** 원격 확인 실패는 6px 점 대신 알림 줄("원격 확인 못 함 · 3분 전" `m.fetchFailed`)+`ERROR_FIX`의 해결 버튼 하나(인증 실패면 다시 연결, 그 밖은 동기화)로 보이고 닫을 수 있다. 동기화 버튼 툴팁은 할 일("2개 올리기 · 1개 가져오기", 낱말은 용어표의 `push`·`pull`)을 말하고 버튼 옆에 `--text-muted` 13px로 마지막 확인 시각(`<Time>`)을 둔다. 시각은 메모리(`syncStore.lastCheckedAt`: 동기화·자동 원격 확인 성공 때)만 쓴다. (선택, U29) 홈 노선도에 받을 버전 최대 3개를 흐린 속 빈 역으로 이어 붙인다(`HomeMapView.incoming`). | ux.md 1장 중간·6장 9. 색점은 키보드에 닿지 않고 색으로만 알린다. 시각을 파일에 저장하면 앱 데이터에 쓸 일이 늘고 다시 켜면 낡는다. | 파일(`FETCH_HEAD`) 시각 읽기(OS 파일 시각 읽기를 core에 더함), 점을 키우기 |
| C10 (D-181) | **단축키 보이기.** 상단 바·동기화·브랜치 단추의 툴팁(`TermTooltip`)에 `Kbd`로 단축키를 붙인다(`shortcut` 속성, 표에서 읽음). 새 단축키 `Ctrl+/`(`shortcutHelp`)가 단축키 표 대화상자를 열고 설정 "일반"의 "단축키 보기"와 팔레트에서도 연다. 표는 `SHORTCUTS`에서 만들며 `m.shortcutName`이 `Record<ShortcutId, string>`이라 단축키를 더하면 이름을 빠뜨릴 수 없다. | ux.md 1장 중간. 표가 이미 한 곳에 있다(D-056). | 단축키 글자를 버튼에 항상 표시(상단 바 컨트롤 5개 제한과 소란) |
| C11 (D-182) | **명령 팔레트(D-170) 세부.** `ActionMeta`에 `needs_input`을 더해(`ActionInput`의 단위 변형 여부를 `serde_json::from_value(json!({ "id": id }))`로 시험해 어긋남을 막는다, D-020 영향) `action-meta.json`·`ActionMetaView.needsInput`으로 내린다. 목록 소스: 액션 메타, 브랜치 바꾸기(이름을 치면 이어서 `switch` 흐름), 최근 프로젝트, 화면 이동(버전 기록·작업 내역·검토 요청·설정·단축키 표). 항목 종류 셋: **바로 하기**(입력 없음·`Irreversible` 아님·지금 쓸 수 있음 → `executeAction` 뒤 `notifyDone`), **화면 열기**(입력이 필요한 것은 그 기능의 화면·대화상자로: 버전 저장 → 이름 칸 포커스, 태그·새 브랜치 → 버전 기록), **제외**(`Irreversible`·역 해시가 필요한 것은 까닭과 함께 `EXCLUDED`에). 모든 액션 id가 셋 중 하나에 있어야 통과하는 시험이 새 액션에 결정을 강제한다. 초성 검색 `src/lib/hangul.ts`(순수: 완성 글자 → 초성, 호환 자모 정규화, 질의가 모두 초성이면 쉬운 이름·원래 이름의 초성열 부분 일치, 섞여 있으면 글자 단위 부분 일치), 한글 조합 중 Enter는 무시. 이름 옆에 원래 이름(`--text-muted`)과 단축키(`Kbd`). 새 의존성 없음(`cmdk` 안 씀: D-004). | D-170이 범위만 정했다. 입력이 필요한 액션을 팔레트에서 어떻게 할지는 D-020이 "그때 메타에 다시 정한다"고 남겼다. 원래 이름 열은 버튼 안 용어 툴팁을 마우스 없이 볼 길도 된다. | 모든 액션에 입력 대화상자를 새로 만들기(범위 폭증), 최근 사용 순서 기억(설정이 늘어남) |
| C12 (D-183) | **파일 목록·`<Term>` 포커스.** 파일 목록은 `role="grid"`로 Tab 한 번에 들어가고 줄(`role="row"`) 하나가 포커스 단위(roving tabindex)다. ↑↓·Home·End가 줄을 옮기며 비교할 파일을 고르고(지금 `fileList` 단축키), Space가 넣기·빼기, 메뉴 키·Shift+F10이 줄 메뉴(011-P17b)를 연다. 체크 상자는 마우스용으로 `tabIndex=-1`이되 줄 접근성 이름에 "넣음/뺌"이 있다. 머리의 체크 상자는 Tab 정지로 남는다(전체 넣기). `<Term>`은 버튼·링크·메뉴 항목·`summary`·`label` 안에 있으면 자동으로 포커스를 받지 않는다(`closest` 판정, 호출부의 `focusable={false}`는 남겨도 된다). 문장·제목 속 `<Term>`만 Tab으로 닿는다. | ux.md 1장 중간("파일 100개면 Tab 200번")·낮음. 호출부마다 `focusable={false}`를 기억하게 하는 것보다 부품이 판정하는 편이 빠뜨릴 수 없다. | 줄을 `listbox`로(체크 상자가 option 안 상호작용 요소가 됨), `focusable` 기본 거짓으로 뒤집기(문장 속 용어가 키보드로 안 읽힘) |
| C13 (D-184) | **브랜치 목록 구조.** 찾기 칸은 `role="combobox"`(`aria-expanded`·`aria-controls`·`aria-activedescendant`·`aria-autocomplete="list"`), 목록은 `role="listbox"`, 줄은 `role="option"`(`aria-selected`=고른 줄, 줄마다 `id`)이다. 줄 오른쪽 `…` 단추는 마우스용이라 `aria-hidden`+`tabIndex=-1`로 두고, 키보드는 고른 줄에서 ContextMenu 키·Shift+F10이 같은 메뉴를 연다(새 단축키를 표에 더하지 않는다). | ux.md 1장 중간. 줄 안의 상호작용 단추 때문에 `option`이 깨끗하지 않으므로 키보드 대체를 함께 둔다. | 줄을 `button` 그대로 두고 `aria-current`만(지금, 화면 읽기 프로그램이 위치를 모름) |
| C14 (D-185) | **노선도 역 고르기와 메뉴 분리.** 처음 누르면 고르기만 하고, 이미 고른 역을 다시 누르거나 고른 줄 끝의 `⋯` 단추(24px, `aria-hidden`+`tabIndex=-1`)를 누르면 메뉴가 열린다. Enter·Space·ContextMenu·Shift+F10·오른쪽 클릭은 지금처럼 연다. Shift+클릭은 구간 넓히기 그대로. D-092·D-107의 "눌러서 연다"를 이 범위에서 바꾼다. | ux.md 1장 낮음. 고르는 동작(구간 선택, 읽기)과 실행 메뉴가 한 번에 오면 메뉴가 방해한다. `⋯`가 있으면 비개발자도 메뉴를 찾는다(r-log.png의 첫 인상은 유지). | 지금 방식 유지, 메뉴를 오른쪽 클릭에만 |
| C15 (D-186) | **고정.** `recent.json` 항목에 `pinned: bool`(기본 거짓)과 `pinned_branches: Vec<String>`(기본 빈 목록, `#[serde(default)]`라 형식 버전 1 그대로)을 더한다. 프로젝트는 최대 10개이고 최근 20개 한도 밖에서도 남는다(고정 10 + 고정 아닌 20). 브랜치는 프로젝트당 10개·이름 200바이트이고 쓸 때 실제 브랜치 목록에 있는 것만 보인다(저장소가 이름을 바꿔 쓰는 글자는 표시에만 쓰이고 git 명령으로 가지 않는다). 새 명령 `recent_pin(path, pinned)`·`recent_pin_branch(repo, branch, pinned)`(열린 저장소·최근 목록 경로만 받는다). UI: 시작 화면 "고정" 묶음과 줄의 고정 단추, 프로젝트 메뉴의 최근 프로젝트, 브랜치 목록 맨 위 "고정" 묶음과 줄 메뉴의 "고정"/"고정 풀기". | ux.md 6장 14(GitHub Desktop 요청 61·30). 앱 데이터의 기존 파일을 재사용하면 읽기·쓰기·크기 검사를 새로 만들지 않는다. | `settings.json`에 이름 목록(고정 칸 규칙 "경로·글자 없음"과 어긋남), 브랜치 고정을 저장소 `.git`에(받은 저장소가 바꿀 수 있음) |
| C16 (D-187) | **작업 내역 표시.** `RecoveryItem`에 `undo: UndoHint`(`next`=Ctrl+Z가 되돌릴 줄, `earlier`=더 새 작업을 먼저, `done`=되돌림, `never`=되돌릴 수 없음(경계 앞·올린 것·`Irreversible`), `none`=앱 작업이 아님)와 `beforeOid`(journal 줄의 `before.head`)를 더한다. 계산은 `undo/run.rs`의 `stacks`와 같은 함수에서 한다. 앱 작업 줄의 단추는 "이 작업 전으로 되돌리기"(`beforeOid`로 `restore`, core가 reflog·백업에 있는지 다시 확인)이고 reflog·백업 줄은 지금 문구("이 시점으로 되돌리기")를 유지한다. 줄 아래 `--text-muted` 13px로 "Ctrl+Z로 되돌려요"/"위 작업을 먼저 되돌려야 해요"/"Ctrl+Z로 되돌릴 수 없어요"를 쓴다. 홈에서 작업 내역으로 가는 길: 상단 바 "작업 내역"(있음)에 더해 Ctrl+Z가 되돌릴 게 없거나 낡았을 때의 오류 토스트가 "작업 내역" 해결 버튼을 가진다(`ERROR_FIX`). | ux.md 6장 4(GitButler). Ctrl+Z가 될지 미리 알게 한다. | 모든 줄 단추를 "이 시점으로"로 유지(Ctrl+Z 여부를 모름) |
| C17 (D-188) | **처음 하는 일 안내 저장·모양(D-168).** `settings.json`에 `tips_on: bool`(기본 켬)과 `tips_seen: u8` 비트(첫 브랜치·첫 충돌·첫 동기화·첫 되돌리기)를 더한다(`Copy` 유지, `#[serde(default)]`, 패치는 `tips_on`·`tip_seen: TipId`·`reset_tips`). 안내는 `ui/TipLine`(알림 줄 모양, `role="note"`, 글자 `--text-muted`, 오른쪽 글자 버튼 "알겠어요")이고 행동을 막지 않는다. 닫으면 그 안내가 영영 안 뜨고, 설정 "일반"의 "처음 하는 일 안내" 켬·끔과 글자 버튼 "다시 보기"(`reset_tips`)로 되돌린다. 문장 속 용어는 `t(id)`(D-029). | D-168은 범위만 정했다. 비트 마스크는 `Copy`를 지키고 경로·글자 없는 고정 칸 규칙(D-148)에 맞는다. | 화면 `localStorage`(core가 모르고 설정 화면과 따로 놈) |
| C18 (D-189) | **이미지 비교.** 형식은 png·jpg·gif·webp·svg이고 확장자가 아니라 내용 머리로 판정한다(`core::image::sniff`, svg는 `<svg`를 가진 텍스트). 한쪽 3MB 이하일 때만, 읽기 전용 조회 `query_image_pair(repo, request)`가 이전·지금 두 쪽을 `{ mime, data(base64), bytes }`로 돌려준다(이전=HEAD나 고른 버전의 첫 부모 블롭, 지금=작업 폴더 일반 파일·링크는 읽지 않음·또는 고른 버전의 블롭). svg는 `<img>`로만 그린다(스크립트·바깥 불러오기가 실행되지 않고 내용을 DOM에 넣지 않는다). CSP는 이미 `img-src data:`를 허용하므로 바꾸지 않는다. 보기 셋(나란히·겹쳐 보기(투명도 슬라이더)·밀어 보기(가림 위치 슬라이더))은 `Choice`+`input[type=range]`로 키보드 ←→가 된다. 크기("128×64 → 130×64")는 `naturalWidth`로 읽는다. 목록 썸네일은 하지 않는다. base64는 `base64` 크레이트(이미 트리에 있음)를 core에 직접 더한다(`cargo deny` 확인). | ux.md 6장 8. 이미지를 줄이는 코드가 없어 썸네일은 원본 전체를 보내야 한다. JSON 배열로 바이트를 보내는 것보다 base64가 훨씬 작다. | 이미지 디코드·줄이기 크레이트 추가, svg를 인라인으로(스크립트 위험) |
| C19 (D-190) | **태그 역.** 세로 노선도(`layout/route.ts`)에서 태그가 붙은 버전의 역은 반지름을 키우고(테두리 `--text`) 이름 옆 글자는 그대로 둔다. "이름 붙인 버전만 보기"는 이미 있는 확대 단계 "태그만"(`ZOOMS`)으로 갈음하고 이름은 용어표 그대로 "태그"를 쓴다(D-024). | ux.md 6장 6. "태그만" 단계가 이미 있다. 새 말(이름 붙인 버전)을 만들면 "태그"와 두 낱말이 된다. | 확대 단계 이름을 "이름 붙인 버전"으로(용어 두 개) |
| C20 (D-191) | **코드 글꼴에 D2Coding을 JetBrains Mono 뒤에 둔다.** `--font-mono: "JetBrains Mono", "D2Coding", "Pretendard", ui-monospace, monospace`. npm 패키지(OFL, 예 `@kfonts/d2coding`)에서 한글 범위(`unicode-range: U+AC00-D7A3, U+1100-11FF, U+3130-318F`)·400 하나의 woff2만 import한다. 번들이 2MB를 넘거나 woff2가 없으면 하지 않고 사람에게 묻는다. `THIRD_PARTY_NOTICES.md`에 라이선스 원문을 더한다. D-014를 보완한다. | 한글이 섞인 줄이 칸에 맞는다(ux.md 5장). 지금은 한글만 Pretendard(비례폭)라 정렬이 어긋난다. | 하지 않기(정렬 어긋남), 시스템에 D2Coding이 있을 때만(OS마다 다름) |
| C21 (D-192) | **접근성 점검 방식과 이번에 하지 않는 것.** 자동 검사 도구(axe 등)는 넣지 않는다(jsdom은 대비·레이아웃을 못 재고 규칙 소란이 크며 devDependency가 느는데 효과가 작다). 대신 역할·이름·`aria-*`를 확인하는 화면별 구조 시험, 대비 시험(`contrast.test.ts`), 포커스·눌림 크기 CSS 시험을 쓰고, 화면 읽기 프로그램(NVDA·내레이터·Orca)·배율 확인은 "사람이 할 일"로 둔다. 진행 막대·OS 알림·이미지 썸네일·창 기억·폴더 위치 열기는 하지 않는다(범위 "뺀다"). | 새 의존성 없이 규칙이 코드로 남는다. | `jest-axe`/`axe-core` 추가 |

## 용어 변경
- **`park`**: 쉬운 이름 "맡겨 두기" → "여기 두기"(en "Leave here"). 설명은 그대로. 화면 문구 "맡겨 둔 파일 N개"→"두고 간 파일 N개", "맡겨 둔 파일 N개 · 겹침"→"두고 간 파일 N개 · 겹침", "맡겨 둔 파일만 남음"→"두고 간 파일만 남음", 오류 `parkBlocked`·`unparkConflict`와 작업 내역의 백업 이름(`backupLabel`)의 "맡긴"을 같은 말로. `docs/spec.md` 용어표와 화면 구성, `docs/design.md`, `src/glossary/ko.ts`·`en.ts`, `src/messages`를 **U12에서 한 커밋**으로 고친다(spec-sync 시험). `rg "맡겨|맡긴|맡기"`로 빠짐을 확인한다.
- **`discard`**: 호버 설명에 "되돌리기로 되살릴 수 있어요." 한 문장을 더하고 spec "위험도" 문단의 확인 목록에서 고친 것 버리기를 뺀다(U13).
- 새 용어는 없다. "가져가기"·명령 팔레트·고정·안내·이미지 비교는 Git 용어가 아니라 용어표에 넣지 않는다. 새 문구는 `src/messages` ko·en 둘 다 같은 커밋에: `switchDialog`(제목·`carry`·`leave`·설명·`carryBlocked(files)`), `switchedCarried(n, branch)`·`switchedLeft(n, branch)`, `disabledReasons.*`, `confirm.*`, `fix.*`(해결 버튼 이름)·`copyDetail`, `errors.fileTooLarge`·`errors.fileInUse`, `secretRule.hugeFile`, `shortcutName.*`·`shortcutsTitle`, `palette.*`, `pin`·`unpin`·`pinned`·`recentProjects`, `activityUndo.*`(`next`·`earlier`·`never`)·`restoreBefore`, `tip.*`(네 안내와 "알겠어요"·"처음 하는 일 안내"·"다시 보기"), `imageCompare.*`(`sideBySide`·`overlay`·`swipe`·`oldSide`·`newSide`), `textMenu.*`(잘라내기·복사·붙여넣기·모두 선택·`pasteWithKeys`), `incoming(n)`, `lastChecked`. 모두 `copy-rules.test.ts`·`terms-in-messages.test.ts`·`parity.test.ts`(ko·en 키 같음)를 통과해야 한다.

## 단계
각 단계 = 커밋 하나. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`(`src/lib/bindings.ts`·`action-meta.json`). 새 Tauri 명령은 `src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣는다(`tests/security_config.rs`가 확인). 화면은 git도 `invoke`도 직접 부르지 않고 `src/lib/api.ts`·`queries.ts`로만 core를 부른다(D-010). 색·간격·크기는 `tokens.css` 변수만(길이는 rem, 린트가 막음). 새 화면 문구는 ko·en 같은 커밋(이름과 숫자만, D-049; 오류·안내·확인은 짧은 문장). 용어는 `<Term>`, 문장 속 용어는 `t(id)`(D-029). OS별 코드는 `crates/platform`에만(이 계획의 앱·화면 코드에는 OS 분기가 없고, `#[cfg(windows)]`/`#[cfg(unix)]`는 시험에만 쓴다). 테스트는 사용자 설정·실제 키체인·실제 네트워크·전역 git 설정을 건드리지 않는다.

### A. 바탕: 웹 느낌·첫 화면·버튼·토스트·기다림

U1. 결정·문서 (`docs: add ux polish decisions and plan 012`)
- 목표: 이후 단계가 쓰는 결정을 먼저 확정하고 문서를 새 값에 맞춘다.
- 바꿀 파일: `docs/decision.md`(C1~C21 → D-172~D-192 "(자동 결정)", 상태 확정. 갱신 줄: D-043 "고친 것 버리기는 D-179로 확인 대상에서 빠짐", D-047 "고친 파일을 묻는 방식은 D-165·D-178", D-074 "D-179", D-092·D-107 "눌러서 연다는 D-185", D-136 "`Irreversible` 집합에서 `discard` 제외(D-179)", D-014 "코드 글꼴 보완 D-191", D-056 "`Ctrl+K`·`Ctrl+/` 표로(D-181·D-182)"), `docs/design.md`(버튼: 꺼진 이유·`busy`·24px, 토스트: 겹침, "불러오는 표시" 규칙표, 알림 줄: 원격 확인 실패, 브랜치 목록: 구조·고정, 파일 줄: Tab 한 번, 역 메뉴: 고르기/메뉴 분리와 `⋯`, 확인 대화상자 문구 규칙, 새 부품 절: 브랜치 바꾸기 대화상자·단축키 표·명령 팔레트·이미지 비교·안내 줄·앱 메뉴, "글꼴"의 코드 글꼴, 새 절 "데스크톱 느낌"), `docs/spec.md`(화면 구성 행: 작업 공간·작업 내역·설정, "진입점"에 Ctrl+K·Ctrl+/, "위험도" 문단의 확인 목록, "오류 메시지 원칙"에 해결 버튼 하나·원문 복사. 용어표는 건드리지 않는다: U12·U13이 한다), `docs/architecture.md`(명령 목록에 5개, "되돌리기"의 `discard` 설명, 프론트 절 "웹 느낌 끄기(`lib/webFeel.ts`)·`app_ready`"), `docs/roadmap.md`("UX 다듬기" 절 신설, 반복 5로 넘긴 것), `docs/progress.md`(U1~U31 체크리스트).
- 내용: 결정 문장은 위 표를 그대로 옮긴다. CLAUDE.md는 고치지 않는다.
- 테스트: `pnpm test`(`spec-sync`·`copy-rules`), `pnpm lint`.
- 완료 기준: 위 문서가 새 값으로 일치하고 테스트 통과. Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

U2. 웹 느낌 CSS와 한국어 글자 배치 (`styles: desktop-app feel and korean text layout`)
- 목표: 버튼 커서·글자 끌림·튕김·이미지 끌기를 끄고 한글 줄바꿈·숫자 폭을 바로잡는다(C1 ①②③⑥⑦).
- 바꿀 파일: `src/styles/base.css`(`html, body { overscroll-behavior: none }`, `:where(button, [role="button"], [role="option"], [role="tab"], [role="menuitem"], [role="radio"], [role="row"], summary, label, kbd) { user-select: none; cursor: default }`, `:where([data-selectable], pre, input, textarea) { user-select: text }`, `img, svg, a { -webkit-user-drag: none }`, `body { word-break: keep-all; overflow-wrap: break-word; font-variant-numeric: tabular-nums }`, `code, pre, samp, [data-selectable] { word-break: normal; overflow-wrap: anywhere }`), 22개 CSS 모듈에서 `cursor: pointer` 28곳 삭제(`linkStyle.module.css`만 남김), 선택되게 둘 곳에 `data-selectable`(`DiffView` 코드 칸, `ErrorNotice` 원문, 설정의 Git 위치·연결 코드, `FilesDialog`·`AmendDialog`·`TagDialog` 머리의 버전 번호, 복제 결과 경로), `.stylelintrc.json`(`declaration-property-value-disallowed-list`에 `cursor: /pointer/`(override로 `linkStyle.module.css` 제외)와 1px 미만 `border`·`outline` 폭 `/(^|\s)0?\.\d+px/` 추가), `scripts/stylelint-rules.test.mjs`, 신규 `src/styles/webfeel.test.ts`.
- 내용: 입력칸·글상자·`select`의 커서와 `col-resize`·`grab`·`progress`는 그대로 둔다. 줄 높이는 1.5를 유지한다(확인만).
- 테스트: stylelint 규칙 시험(`cursor: pointer` 오류·`linkStyle` 통과·`border: 0.5px solid` 오류), `webfeel.test.ts`(base.css에 위 규칙, `rg "cursor:\s*pointer" src --glob "*.css"` 결과가 `linkStyle.module.css`뿐, `data-selectable`이 있어야 하는 컴포넌트 목록을 렌더해 확인), 기존 시험. 눈으로: 개발 앱에서 버튼 위 화살표 커서, 목록을 끌어도 글자가 선택되지 않고 비교 칸은 선택되며 "버전 저장"처럼 낱말이 중간에서 끊기지 않는다.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: `overscroll-behavior`·`-webkit-user-drag`·`font-variant-numeric`은 WebView2·WebKitGTK 4.1(Ubuntu 22.04)이 모두 지원한다. 125·150% 배율에서 글자 선명도는 사람이 확인("사람이 할 일"). 보안: 해당 없음.

U3. 앱 메뉴와 브라우저 단축키 (`features: app context menu and blocked browser keys`)
- 목표: 마우스 오른쪽 클릭과 브라우저 단축키가 웹페이지처럼 동작하지 않게 한다(C1 ④⑤).
- 바꿀 파일: 신규 `src/lib/webFeel.ts`(`installWebFeel()`: `contextmenu` 캡처 리스너가 이미 `defaultPrevented`인 곳은 건드리지 않고, 입력칸이면 `textMenuStore.open`, 선택한 글자가 있고 `data-selectable` 안이면 "복사"만, 그 밖은 `preventDefault`. 입력칸에서 ContextMenu 키·Shift+F10은 요소 중앙에 연다. `keydown` 캡처에서 `RESERVED` 표의 키를 `import.meta.env.PROD`일 때만 `preventDefault`(개발 빌드는 F12·Ctrl+Shift+I/J·F5·Ctrl+R 허용). `dragover`·`drop`을 창에서 막는다), 신규 `src/ui/TextMenu.tsx`·`TextMenu.module.css`·`textMenuStore.ts`(Radix `dropdown-menu`를 보이지 않는 기준 위치에 연다. 항목: 잘라내기·복사·붙여넣기·모두 선택, 단축키 `Kbd`. 잘라내기·복사는 `document.execCommand`, 붙여넣기는 `navigator.clipboard.readText()` 뒤 입력칸에 끼워 넣고 `input` 이벤트를 보낸다. 읽기가 거절되면 항목을 `aria-disabled`로 두고 까닭 "Ctrl+V를 눌러 주세요"), `src/lib/shortcuts.ts`(`RESERVED`에 `F3`·`Ctrl+Shift+R` 추가, 시험이 표와 막는 목록이 같음을 확인), `src/main.tsx`(`installWebFeel()`), `src/messages`(`textMenu.*`), 시험.
- 내용: 한글 조합 중에는 메뉴를 열지 않는다. `RouteList`와 011-P17b의 파일 줄 메뉴처럼 직접 `contextmenu`를 처리하는 곳(`defaultPrevented`로 알아본다)은 그대로 동작해야 한다(기존 시험).
- 테스트: `webFeel.test.tsx`(버튼 위 `contextmenu`는 `defaultPrevented`, 입력칸 위는 메뉴가 열림, 이미 막은 곳은 그대로, 선택 글자 위는 "복사"만, 릴리스 모드에서 `F5`·`Ctrl+R`·`Alt+←`가 막히고 개발 모드에서 `F12`는 안 막힘, 표 하나), `TextMenu.test.tsx`(항목 동작은 `execCommand`·클립보드 모의, 읽기 거절 때 꺼진 항목, ↑↓·Esc, 닫히면 포커스가 입력칸으로 돌아감), 기존 `RouteList`·`Rewrite` 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 릴리스 빌드(`pnpm tauri build --no-bundle`)에서 오른쪽 클릭·F5·Ctrl+R·Ctrl+P·Ctrl+F·F12·Alt+←.
- Windows/Ubuntu: WebView2는 일부 브라우저 키(인쇄·찾기 등)를 JS보다 먼저 가져갈 수 있다 → 새는 키는 progress "사람이 할 일"에 적고 반복 5의 platform 함수 후보로 둔다(C1, 사람에게 묻는다). WebKitGTK는 JS 막기로 충분한지 확인. 붙여넣기 읽기 권한이 웹뷰마다 다르다. 보안: 클립보드는 사용자 동작 안에서만 읽고 값을 저장·기록하지 않는다. 메뉴 글자는 상수.

U4. 첫 화면 흰 번쩍임 (`app: show the window after the first paint`)
- 목표: 어두운 화면 모드로 켜도 흰 화면이 보이지 않는다(C2).
- 바꿀 파일: `src-tauri/tauri.conf.json`(창 `"visible": false`, 스키마가 지원하면 `backgroundColor`), `src-tauri/src/commands.rs`(`app_ready(window: WebviewWindow)`: 숨겨져 있으면 `show()`+`set_focus()`, 이미 보이면 아무것도 안 함), `src-tauri/src/bindings.rs`, `src-tauri/build.rs`(`COMMANDS`), `src-tauri/capabilities/default.json`(`allow-app-ready`), `src-tauri/src/lib.rs`(`setup`에서 3초 뒤에도 안 보이면 보이는 스레드 하나), `src/lib/api.ts`(`appReady`), `src/App.tsx` 또는 `main.tsx`(첫 렌더 커밋 뒤 `useEffect`로 한 번, 실패는 무시), 생성물, `src-tauri/tests/security_config.rs`(명령 네 곳과 창 `visible: false`·안전 타이머 상수), `src/App.test.tsx`.
- 내용: `rAF`를 쓰지 않는다. 011의 `preloadDisplay`(설정·OS 힌트를 먼저 읽고 그림)가 끝난 뒤 첫 커밋에서 부른다. 화면 오류 경계가 뜨는 경우에도 부른다.
- 테스트: `security_config`, `App.test`(마운트 뒤 `appReady` 한 번, 실패해도 렌더), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 다크 화면 모드·OS 다크에서 앱을 켜 흰 화면이 안 보이고, JS를 일부러 깨뜨리면 3초 뒤 창이 뜬다.
- Windows/Ubuntu: 숨긴 창에서도 두 웹뷰가 JS를 실행하는지 손 확인(안 되면 안전 타이머가 보인다). dev 스크린샷 스크립트는 창이 뜬 뒤에 찾는다. 보안: 인자 없는 명령, 상태를 바꾸지 않는다.

U5. 꺼진 버튼 이유·24px·포커스 점검 (`ui: disabled reasons, 24px targets and focus audit`)
- 목표: 꺼진 버튼이 읽히고 까닭을 말하며, 누를 곳이 24px 이상이고, `outline: none`이 포커스를 지우지 않는다(C3, 011-P19 ① 보완).
- 바꿀 파일: `src/ui/Button.tsx`·`Button.module.css`(`disabledReason`: `aria-disabled`+`Tooltip`+`aria-describedby`, 클릭 무시, `opacity: 0.5` 삭제, 꺼짐 색 `--text-muted`/`--surface-sunken`/`--border-strong`), `src/styles/tokens.css`(`--target-min: 1.5rem`), `src/ui/Checkbox.module.css`(`position: relative`+`::after`로 눌림 영역 24px, 보이는 상자는 16px), 쓰는 곳: `features/commit/NameBox.tsx`(고친 파일 없음), `features/stash/CompareFooter.tsx`(임시 보관: 고친 파일 없음. 꺼내기의 `title`을 `disabledReason`으로), `screens/Rewrite/Rewrite.tsx`(막힌 까닭이 있으면), `screens/Start/Start.tsx`(Git 준비 전 복제·폴더 열기), `src/messages`(`disabledReasons.*`), 신규 `src/styles/focus.test.ts`·`target.test.ts`, `src/styles/contrast.test.ts`(꺼진 버튼 색 조합 추가).
- 내용: 진행 중 꺼짐(`pending`)은 `disabled` 그대로 둔다. 눌림 크기 시험은 `Button`·`Checkbox`·`Choice`·`BranchRowMenu`·`Splitter` CSS가 `--target-min` 이상인지 읽는다(끄는 선 `Splitter`는 잡는 폭 대신 키보드 ←→가 있음을 시험이 확인). `focus.test.ts`는 모든 CSS 모듈의 `outline: none`(12곳)마다 같은 요소의 `:focus-visible`에 대신하는 표시(`outline`·`box-shadow`·`background`)가 있거나 허용 목록(`Toast` viewport, `Dialogs` content처럼 코드가 포커스를 주는 컨테이너)에 이유와 함께 있는지 확인한다. 끌기로 하는 일의 클릭·키보드 대안(WCAG 2.5.7)은 카드 지도의 "위로·아래로" 단추와 Alt+↑↓(`cardKeys.test`·`Rewrite.test`), 칸 나눔선의 ←→(`Splitter.test`)가 이미 시험으로 확인하므로 새 시험은 없고 U31의 점검 줄에 "끌기 동작에는 키보드·클릭 대안"을 더한다.
- 테스트: `Button.test`(`disabledReason`이 `aria-disabled`·툴팁·클릭 무시·키보드 포커스), 쓰는 곳 시험(까닭 문구), `target.test`·`focus.test`·`contrast.test`, `pnpm lint`.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 포커스 링 모양은 전역 규칙이라 같다. 보안: 해당 없음.

U6. 토스트 겹침과 포커스 가림 (`ui: keep undo toasts, stack up to three and clear focus`)
- 목표: 새 토스트가 "되돌리기"를 지우지 않고, 토스트가 포커스를 가리지 않는다(C4).
- 바꿀 파일: `src/ui/toastStore.ts`(`toasts: ToastMessage[]`, `show`/`showError`/`clear(id)`, `ToastAction.kind?: "undo"`), `src/ui/Toast.tsx`·`Toast.module.css`(Provider 안 Root 여럿, 세로 쌓기, 오류 둘까지, 높이를 재 `--toast-clearance`를 `<html>`에 설정), `src/styles/base.css`(`* { scroll-padding-bottom: var(--toast-clearance, 0) }`), `src/features/undo/useUndo.ts`(되돌리기 토스트에 `kind: "undo"`), 이 스토어를 읽는 시험(`toast`→`toasts` 호출 `rg "toastStore"`로 모두), `docs/design.md`는 U1.
- 내용: 3개를 넘으면 가장 오래된 평범한 토스트부터 밀어낸다. 같은 문구를 다시 띄우면 새 토스트다(`id`). 마우스를 올리면 그 토스트만 멈춘다(Radix).
- 테스트: `toastStore.test`(되돌리기 토스트 뒤에 평범한 토스트가 와도 둘 다 있음, 넷째가 오면 가장 오래된 평범한 것이 빠짐, 오류는 닫을 때까지, 오류 셋째는 가장 오래된 오류를 밀어냄, 8초 뒤 되돌리기 토스트가 사라짐(가짜 타이머)), `Toast.test`(`role="alert"`(오류)·`status`, 단추 동작), CSS 문자열 시험(`scroll-padding-bottom`), 기존 토스트 시험(여러 파일) 갱신.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 버전 저장 뒤 곧바로 "복사했어요"를 띄워도 "되돌리기"가 남고, 아래쪽 목록의 포커스 줄이 토스트에 가려지지 않는다.
- Windows/Ubuntu: 차이 없음(CSS). 보안: 해당 없음.

U7. 기다림 표시 (`ui: delayed loading with hold, previous content and skeleton`)
- 목표: 짧은 기다림에는 아무것도 보이지 않고 이전 내용이 남으며, 길면 빈 줄 모양과 버튼 안 표시가 나온다(C5, 011-P19 ② 확장).
- 바꿀 파일: `src/lib/useDelayed.ts`(011-P19의 `useDelayedFlag(pending, ms)`를 `{ delay: 300, hold: 500 }` 객체 인자로, 한 번 켜진 뒤 최소 500ms 유지), 신규 `src/ui/Skeleton.tsx`·`Skeleton.module.css`(`role="status"` 안에 `m.loading` 시각 숨김 글자, `--surface-sunken` 막대 3~6줄, 모션 줄이기면 정지), `src/ui/Button.tsx`(`busy`: `aria-busy`, 글자 그대로, 오른쪽 작은 회전 표시·모션 줄이기면 점 셋), `src/lib/queries.ts`(`useDiff`·`useStatus`·`useLogPages` 첫 쪽·`useRecovery`·`useRewritePlan`·`usePrDetail`에 `placeholderData: keepPreviousData`), `src/features/diff/DiffView.tsx`(마지막으로 받은 `{ request, data }`를 들고 머리와 본문을 그 요청으로 그림, 300ms 넘으면 머리 오른쪽 "불러오는 중…", 처음 로드는 `Skeleton`), `src/screens/Workspace/Workspace.tsx`(`<DiffView key={selected.path}>`의 `key` 제거, 파일이 바뀌면 펼침·줄 바꿈 임시 상태만 effect로 초기화), `History.tsx`·`Activity.tsx`·`Rewrite.tsx`·`PullRequests.tsx`(처음 로드 `Skeleton`), `busy`를 쓰는 작업 버튼(`NameBox` 버전 저장, `CloneDialog` 복제, `CreatePrDialog` 만들기, `GitHubSection` 연결, `ConnectPanel`), `src/messages`, 시험.
- 내용: 동기화 버튼의 "동기화 중 · N초"는 그대로(이미 버튼 안 표시). 10초 넘는 진행 막대·OS 알림은 하지 않는다.
- 테스트: `useDelayed.test`(299ms 거짓·301ms 참, 그 안에 끝나면 안 켜짐, 켜진 뒤 즉시 끝나도 500ms까지 유지), `DiffView.test`(파일을 바꾸면 이전 파일의 머리와 줄이 그대로이고 응답이 오면 함께 바뀜, 300ms 뒤 "불러오는 중…", 처음 로드는 `Skeleton`), `Skeleton.test`, `Button` `busy`(`aria-busy`·폭 변화 없음), 각 화면 시험의 "불러오는 중" 기대 갱신(즉시 끝나는 질의는 없음).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 파일을 ↑↓로 빠르게 넘겨도 비교 칸이 번쩍이지 않는다.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

### B. 오류와 큰 파일

U8. git+core: 새 실패 둘 (`git+core: classify oversized files and files in use`)
- 목표: 100MB 넘는 파일 거절과 파일을 다른 프로그램이 쓰는 중임을 가려 쉬운 문구로 낸다(C6 ④).
- 바꿀 파일: `crates/git/src/failure.rs`(`FileTooLarge`: "GH001: Large files detected"·"exceeds GitHub's file size limit"를 `PushRejected` 앞에, `FileInUse`: "unable to unlink"·"Unlink of file"·"Device or resource busy"·"being used by another process"를 SSH·HTTP 인증 뒤에), `crates/core/src/error.rs`·`view/mod.rs`(`ErrorCode::FileTooLarge`·`FileInUse`와 `From`), `src/messages/ko.ts`·`en.ts`(`errors.fileTooLarge`: "GitHub는 100MB 넘는 파일을 받지 않아요. 그 파일을 뺀 뒤 다시 동기화해 주세요.", `errors.fileInUse`: "다른 프로그램이 이 파일을 쓰고 있어요. 그 프로그램을 닫고 다시 해 주세요."), 생성물, `docs/architecture.md`(실패 분류 목록은 U1).
- 내용: 분류는 `LC_ALL=C` 영어 문구뿐이고 새 문구가 없으면 `gitFailed`로 떨어진다(D-021). 원문은 "자세히 보기"로만 나간다.
- 테스트(실제 git, 임시 저장소): `FileTooLarge`는 로컬 bare 원격의 `pre-receive` hook이 GitHub 문구를 stderr로 내며 거절하는 흉내와 표본 시험, `FileInUse`는 Unix에서 쓰기 금지 폴더의 `checkout`이 내는 "unable to unlink old" 재현(`#[cfg(unix)]`, 루트가 아닐 때), Windows에서 `OpenOptionsExt::share_mode(0)`로 연 파일 위 `git checkout` 재현(`#[cfg(windows)]`), 기존 표본이 그대로 분류됨, `git-min`(2.34.1), ko·en 문구 완전성 시험, 바인딩 최신.
- 완료 기준: `cargo test --workspace`(두 OS)·`cargo clippy --workspace --all-targets -- -D warnings`, 바인딩 최신.
- Windows/Ubuntu: 파일 사용 중은 Windows에서 흔하고 Ubuntu에서는 권한 문구로 재현한다. 보안: 분류만 하고 새 원문 경로를 화면에 내지 않는다.

U9. 오류 안내 틀과 확인 창 문구 (`features: error guide with one fix button and raw copy`)
- 목표: 모든 오류 코드가 해결 버튼 하나(또는 없음)와 같은 틀의 문구를 갖고, 원문을 복사할 수 있다(C6 ①②③⑤).
- 바꿀 파일: 신규 `src/features/errors/fixes.ts`(`FixKind`, `ERROR_FIX: Record<ErrorCode, FixKind>`, 처음 넷: `githubExpired`·`httpAuthFailed`→`reconnect`, `fileTooLarge`→`openHistory`, `fileInUse`→`retry`, `pushRejected`·`notFastForward`·`remoteRefused`→`sync`, `identityMissing`→`setIdentity`, `nothingToUndo`·`undoStale`·`undoPushed`→`openActivity`, `indexLocked`→`clearLock`, 나머지 `none`), 신규 `src/features/errors/useErrorFix.ts`(코드→`{ label, run }`: `reconnect`는 설정 GitHub 연결 열기, `sync`는 `useSync.run`, `openActivity`·`openHistory`는 `setView`, `setIdentity`는 설정 일반, `retry`는 호출자가 준 클로저만), `src/ui/ErrorNotice.tsx`(`children`이 없으면 `useErrorFix`가 단추를 만들고, "자세히 보기" 안 원문 아래 글자 버튼 "복사"(`copyText`, 성공 `copied`·실패 `copyFailed`, 원문은 `data-selectable`)), `src/ui/Toast.tsx`(오류 토스트가 같은 단추), `src/ui/ConfirmDialog.tsx`(`copy: { title, danger, close }`), 확인 창 호출부(`StuckNotice`·`ClearLockButton`와 010의 덮어쓰며 올리기·원격 브랜치 확인)가 `m.confirm.<이름>`을 쓰게, `src/messages`(`fix.*`, `copyDetail`, `confirm.*`), 신규 `src/messages/error-guide.test.ts`.
- 내용: 해결 버튼의 이름은 `m.fix.<kind>`이고 같은 이름의 실제 버튼이 있는 화면(동기화·설정 연결·버전 기록·작업 내역)에만 쓴다. 문구 틀 시험은 앞선 처리 #4의 "문구 속 행동 이름이 실제 버튼에 있다" 시험을 확장한다. 호출부가 이미 단추를 넘기면(`children`/`action`) 그것이 이긴다.
- 테스트: `error-guide.test`(모든 `errors[code]`는 1~2문장, `ERROR_FIX` 키 = `ErrorCode` 전체, `fix` 이름이 문구 속 행동 이름과 같음), `fixes` 시나리오(`githubExpired` → "다시 연결"이 설정 연결을 엶, `fileTooLarge` → "버전 기록", `pushRejected` → "동기화"가 `executeAction(sync)`를 부름, `nothingToUndo` → "작업 내역"), `ErrorNotice`(복사 단추·실패 문구), 확인 창 문구 시험(`m.confirm`의 제목은 "?"나 "하시겠습니까"로 끝나지 않음, 위험 버튼은 결과 이름, `close ∈ {"닫기","다음에"}`, 포커스가 위험 버튼이 아님), 기존 `ErrorNotice` 호출부 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 연결이 끊긴 상태로 동기화 → 오류 토스트에 "다시 연결", 원문 복사.
- Windows/Ubuntu: 복사는 `copyText`(옛 방법 폴백 있음). 보안: 복사하는 원문은 이미 `redact`를 거친 `detail`이다.

U10. 큰 파일 미리 알림 (`core: warn about files over 50MB and 100MB`)
- 목표: 버전 저장 전에 큰 파일을 알리고 100MB부터 GitHub가 거절한다고 미리 말한다(ux.md 6장 10, 011-P18의 알림 틀 위).
- 바꿀 파일: `crates/core/src/secrets.rs`(`HUGE_BYTES = 100 * 1024 * 1024`), `crates/core/src/query.rs`(`secret_check`: 크기가 `HUGE_BYTES` 이상이면 `SecretRule::HugeFile`, 아니면 `LARGE_BYTES` 이상이면 `LargeFile`, 순서는 비밀 이름 → 큰 파일 → 압축·실행 → 내용), `crates/core/src/view/secret.rs`(`HugeFile`), 생성물, `src/messages`(`secretRule.largeFile`: "50MB 넘는 파일", `secretRule.hugeFile`: "100MB 넘는 파일 · GitHub가 올리기를 거절해요", 제목은 011-P18의 `heavyTitle`을 "큰 파일·압축·실행 파일 N개"로 넓혀 두 규칙을 센다, 알림 줄 아래 한 줄 "큰 파일은 Git LFS로 두면 올릴 수 있어요"(링크 없음, LFS 관리 화면은 범위 밖)), `src/features/secrets/SecretNotice.tsx`, 시험.
- 내용: 막지 않는다("그래도 저장", 같은 입력이면 저장). 결과에는 경로와 규칙 이름만(D-077). 단추는 지금과 같다(빼기·제외할 파일에 넣기·그래도 저장).
- 테스트: `secret_check` 시험(임시 저장소에 `File::set_len`으로 49MB·50MB·99MB·100MiB 파일, 두 OS, 디스크는 거의 안 씀), 새 파일에만(추적 중인 큰 파일은 알리지 않는지 011-P18 R3과 맞춤), 압축 파일이 큰 경우 큰 파일 규칙이 우선, `SecretNotice.test`(제목·두 문구·LFS 한 줄), `copy-rules`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 큰 더미 파일을 넣고 저장 → 알림 줄.
- Windows/Ubuntu: 크기는 파일 메타데이터라 같다. 보안: 읽는 양은 기존 상한, 링크는 읽지 않는다.

### C. 되돌릴 수 있는 일

U11. core: 브랜치 바꾸기 — 여기 두기·가져가기 (`core: switch with leave-here or carry choice`)
- 목표: 고친 파일이 있을 때 두 선택으로 브랜치를 바꾸고 둘 다 Ctrl+Z로 되돌린다(D-165, C7).
- 바꿀 파일: `crates/core/src/view/input.rs`(`SwitchInput.dirty: Option<DirtyChoice>`, `#[serde(default, skip_serializing_if = "Option::is_none")]`, `journal_copy`와 시험), `crates/core/src/actions/switch.rs`(`switch_with_parking`이 선택을 받아 `carry` 흐름 추가: 스냅샷 → 겹침 검사 → 맡기기 → 갈아타기 → 적용, 겹치면 갈아타기 전에 `CoreError::CarryBlocked`, 적용한 stash를 `switch-carry-<브랜치>` 백업으로, `SwitchReport`에 `files`·`carried`), `crates/core/src/undo/run.rs`(`undo_switch`·`redo_switch`가 입력의 `dirty`를 따름), `crates/core/src/query.rs`·`view/`(신규 `SwitchPlanView { dirty_files, can_carry, blocked: Vec<String> }`, `Queries::switch_plan`), `crates/core/src/error.rs`·`ErrorCode::CarryBlocked`+문구, `src-tauri`(`query_switch_plan` 네 곳), `src/lib/api.ts`·`queries.ts`(`useSwitchPlan`), 생성물, `docs/architecture.md`.
- 내용: 고친 파일이 없으면 지금과 같다. `dirty`가 없고 고친 파일이 있으면 `park`(옛 호출·옛 journal 호환). `carry`는 core가 실행 때 다시 계산하고 화면의 `can_carry`를 믿지 않는다. 새 파일(추적 안 함)도 `Park`(`stash -u`)가 포함한다. 첫 저장 전(버전 없음)·특정 버전을 보는 중은 지금처럼 맡기지 않는다.
- 테스트(실제 git, 임시 저장소): `park`/`carry` × 고친 파일 있음/없음, 겹치는 파일 → `CarryBlocked`이고 갈아타지 않음·작업 폴더 그대로, 겹치지 않는 고침(같은 파일을 다른 줄), 새 파일, 삭제한 파일, 도착한 브랜치에 맡긴 파일이 있을 때(`unparked`), 원격 브랜치로 바꾸기(`SwitchTrack`), Ctrl+Z가 둘 다 원래대로(고친 파일 내용·브랜치), 다시 하기, 옛 journal 줄 서명이 그대로 맞음(`LINE_1B` 방식), `query_switch_plan`이 ref·index·작업 폴더를 바꾸지 않음, `git-min`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `tests/security_config.rs`(새 명령 네 곳), 바인딩 최신.
- Windows/Ubuntu: 줄바꿈(CRLF)과 경로 구분은 git이 처리. 보안: 브랜치 이름은 기존 `validate_ref_like`·`check_branch_name` 뒤에 `Oid`/배열 인자로만, 스냅샷은 ref가 아니라 객체(백업 ref는 `undo::backup`만 쓴다).

U12. screens: 브랜치 바꾸기 대화상자와 "여기 두기" 용어 (`screens: branch switch dialog and leave-here wording`)
- 목표: 브랜치를 고르면 고친 파일이 있을 때 두 선택을 묻고, 한 일을 토스트로 말한다(D-165).
- 바꿀 파일: 신규 `src/features/branches/SwitchDialog.tsx`(+css·test; `DialogFrame` 폭 narrow, 제목 "고친 파일 N개가 있어요", 주 버튼 "가져가기"(처음 포커스·Enter)와 둘째 "여기 두기"(`<Term park>`)와 글자 "닫기", 각 선택 아래 `--text-muted` 한 줄 설명, `can_carry`가 거짓이면 "가져가기"를 `disabledReason`("겹치는 파일이 있어요 · a.txt 외 2개")으로, 계획 조회 중에는 둘 다 `busy`), `src/features/branches/BranchList.tsx`(`run`이 `useStatus`의 고친 파일 수가 0보다 크면 대화상자를 열고 선택을 `executeAction({ id: "switch", input: { …, dirty } })`로 보냄. 새 브랜치 만들기는 묻지 않음), `src/lib/queries.ts`, `src/messages`(`switchDialog.*`, `switchedCarried(n, branch)`: "고친 파일 3개를 '디자인-수정'으로 가져왔어요", `switchedLeft(n, branch)`: "고친 파일 3개를 'main'에 두고 왔어요" + 도착한 브랜치에서 `unparked`면 "두고 간 파일 N개를 꺼냈어요" 덧붙임), 용어: `docs/spec.md` 용어표 `park` 행과 화면 구성 문구, `docs/design.md`, `src/glossary/ko.ts`·`en.ts`, `src/messages`의 "맡겨 둔 파일…" 계열(위 "용어 변경"), 시험.
- 내용: 토스트에는 "되돌리기"(Ctrl+Z). 선택을 기억하는 설정은 만들지 않는다(열린 질문). 팔레트·ProjectMenu 등에서 `switch`를 부르는 곳이 있으면 같은 대화상자를 거친다(`rg 'id: "switch"'`).
- 테스트: `SwitchDialog.test`(포커스가 "가져가기", Enter가 가져가기, "여기 두기"·"닫기", 가져갈 수 없으면 까닭 툴팁과 `aria-disabled`, 고친 파일 0개면 대화상자 없이 바로 바뀜, 한글 조합 중 Enter 무시), `BranchList.test`(기존 `switch` 호출 기대 갱신), 토스트 문구(두 가지·`unparked`), `spec-sync`·`terms-in-messages`·`copy-rules`·`parity`, 용어 이름 바뀜 확인(`rg "맡겨|맡긴"`).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 고친 파일이 있는 채로 브랜치를 바꿔 두 선택과 토스트, Ctrl+Z, 겹치는 경우의 까닭.
- Windows/Ubuntu: 대화상자 포커스(`onOpenAutoFocus`)를 두 웹뷰에서 한 번 더 본다. 보안: 화면은 선택 값만 보내고 core가 다시 검사한다.

U13. 버린 파일 되돌리기 (`features: undoable discard with backup`)
- 목표: 고친 것 버리기가 확인 창 없이 되고 Ctrl+Z로 파일이 돌아온다(D-169, C8).
- 바꿀 파일: `crates/core/src/actions/discard.rs`(`UndoKind::Inverse`, 확인 토큰 없음), `crates/core/src/action.rs`(`Outcome::Discarded`의 `journal_mode()`를 `Always`로: 버려도 HEAD·브랜치·index가 같을 수 있어 기본 `Journal`은 줄을 남기지 않는다. `ignore`와 같은 방식), `crates/core/src/undo/run.rs`(`undo_discard`: 입력의 경로가 지금 `status`에 없을 때만, 입력 경로와 같은 경로만 든 가장 최근 `discard-<브랜치>` 백업 stash를 `parked::refuse_unsafe` 검사 뒤 `GitOp::Unpark`로 되살림, 아니면 `UndoStale`), `crates/core/src/undo/backup.rs`(이름으로 가장 최근 백업 찾기), `crates/core/src/view/mod.rs`(`UndoKindView`는 그대로), `crates/core/tests/`(`Irreversible` 집합 시험에서 `discard` 제외, `registry-glossary`), `src/features/stash/CompareFooter.tsx`(`ConfirmDialog`·미리보기 삭제, 토스트는 `onDone(m.discarded)`의 되돌리기), 011-P17b의 파일 줄 메뉴 "이 파일 고친 것 버리기"(같은 호출), `src/messages`(`discardTitle`·`discardConfirm` 삭제), `docs/spec.md`(`discard` 설명 한 문장, "위험도" 문단의 확인 목록), 시험.
- 내용: 백업 ref를 못 쓰면 지금처럼 stash가 목록에 남고 `BackupFailed`(그 경우 Ctrl+Z 줄은 없음). 다시 하기는 없다. 되돌리기는 같은 경로의 새 파일이 이미 있으면 막는다.
- 테스트(실제 git): 수정·삭제·새 파일(추적 안 함)·이름 바꿈을 버리고 Ctrl+Z로 내용까지 복원, 그사이 같은 파일을 다시 고쳤으면 `UndoStale`, 백업 실패 주입에서 `BackupFailed`와 stash 보존, 파일 둘 중 하나만 버린 뒤 Ctrl+Z가 그 하나만, `CompareFooter.test`(확인 창 없음·토스트 "되돌리기"), 파일 줄 메뉴 시험, `action-meta.json` 최신, `git-min`.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`, 바인딩 최신. 눈으로: 파일을 버리고 Ctrl+Z.
- Windows/Ubuntu: 줄바꿈·파일 모드 복원이 같은지 확인. 보안: 되살릴 때 stash 안 경로가 모두 상대이고 `.git`이 아님을 확인(기존 `refuse_unsafe`), 해시는 백업 ref 이름으로 core가 찾는다(화면이 안 줌).

U14. 작업 내역: Ctrl+Z로 되돌릴 수 있는지 (`features: show which activity rows undo can reach`)
- 목표: 작업 내역의 줄마다 Ctrl+Z가 닿는지 보이고 "이 작업 전으로 되돌리기"가 있다(C16).
- 바꿀 파일: `crates/core/src/undo/run.rs`(`stacks`와 같은 계산의 `pub(crate) fn undo_hints(lines) -> HashMap<String, UndoHint>`), `crates/core/src/recovery.rs`(`RecoveryItem.undo`·`before_oid`, 경계 앞 줄·`Irreversible`·올린 줄은 `Never`), `crates/core/src/view/recovery.rs`, 생성물, `src/screens/Activity/Activity.tsx`(줄 아래 힌트 글자, 앱 작업 줄의 단추를 `restoreBefore`로, `restoreInput`이 `beforeOid`를 씀), `src/features/activity/model.ts`, `src/messages`(`activityUndo.*`, `restoreBefore`), 시험.
- 내용: `restore` 액션은 그대로이고 화면이 `beforeOid`를 보낸다(core가 reflog·백업에 있는지 다시 확인, 없으면 단추를 숨김). reflog·백업 줄은 문구를 바꾸지 않는다.
- 테스트: `undo_hints` 표(여러 줄·경계 줄·되돌림·다시 하기), `recovery` 시험(`next`는 하나뿐), `Activity.test`(힌트 문구 셋, 단추 이름), 기존 작업 내역 시험, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 저장·브랜치 바꾸기를 한 뒤 작업 내역에서 위 줄만 "Ctrl+Z로 되돌려요".
- Windows/Ubuntu: 차이 없음. 보안: `beforeOid`는 core가 목록 안에 있는지 다시 본다(D-085).

### D. 정보와 길 찾기

U15. 동기화 줄 (`screens: fetch failure line, last check time and sync tooltip`)
- 목표: 원격 확인 실패를 알림 줄로, 동기화 할 일을 툴팁으로, 마지막 확인 시각을 옆에 보인다(C9).
- 바꿀 파일: `src/features/sync/syncStore.ts`(`fetchFailedCode`, `lastCheckedAt`), `src/features/sync/useAutoFetch.ts`·`useSync.ts`(성공 때 시각, 실패 때 코드), 신규 `src/features/sync/FetchNotice.tsx`(알림 줄 모양, `role="status"`, "원격 확인 못 함 · 3분 전"+`ERROR_FIX` 단추 하나+글자 버튼 "닫기"), `src/screens/Workspace/Workspace.tsx`(`SyncNotice` 옆), `src/features/sync/SyncButton.tsx`(6px 점 삭제, `Tooltip` "2개 올리기 · 1개 가져오기", 옆 `--text-muted` 13px `<Time>`), `SyncButton.module.css`, `src/messages`(`syncPlan(ahead, behind)`, `lastChecked`), 시험.
- 내용: 시각은 메모리만(다시 켜면 비어 있음). 합치는 중·원격 없음이면 보이지 않는다. 800px에서는 시각 글자가 상단 바 줄 바꿈 규칙을 따른다(011-P7).
- 테스트: `FetchNotice.test`(실패 시 줄·해결 버튼·닫기, 성공하면 사라짐), `SyncButton.test`(툴팁 문구, 시각 표시, 점이 없음), `useAutoFetch.test`(인증 실패 코드 저장).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

U16. 단축키 보이기와 단축키 표 (`ui: shortcut hints and shortcut table`)
- 목표: 단축키가 툴팁에 보이고 Ctrl+/가 표를 연다(C10).
- 바꿀 파일: `src/lib/shortcuts.ts`(`shortcutHelp: Ctrl+/` scope global), `src/ui/Term.tsx`(`TermTooltip`에 `shortcut?: ShortcutId`, 툴팁 맨 아래 `Kbd`), `src/screens/Workspace/TopBar.tsx`·`SyncButton.tsx`·`BranchList.tsx`(단추 툴팁에 단축키), 신규 `src/features/shortcuts/ShortcutsDialog.tsx`(+css·test; `SHORTCUTS`를 scope별로 묶어 "어디서나·목록에서·충돌 해결·버전 기록", 줄은 이름+`Kbd`), `src/App.tsx`(`useShortcut("shortcutHelp")`), `src/features/settings/GeneralSection.tsx`("단축키 보기" 글자 버튼), `src/messages`(`shortcutName: Record<ShortcutId, string>`, `shortcutsTitle`), `src/lib/shortcuts.test.ts`.
- 내용: `Ctrl+/`는 입력칸 안에서도 동작(`inTextField: "app"`). 대화상자가 열린 동안 다른 전역 단축키는 쉰다(`modalOpen`).
- 테스트: 모든 `ShortcutId`에 이름이 있음(타입+시험), 표 겹침 시험이 `shortcutHelp`를 포함, 툴팁에 단축키(가짜 타이머), 대화상자에 표의 모든 키, Esc로 닫힘, 한글 자판에서 `Slash` 코드.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: `Ctrl+/`가 두 웹뷰에서 앱에 오는지 손 확인(한국어 자판 포함). 보안: 해당 없음.

U17. 명령 팔레트 모델 (`core: action input flag and command palette model`)
- 목표: 팔레트가 쓸 목록·초성 검색·액션 분류를 만든다(C11). 화면은 U18.
- 바꿀 파일: `crates/core/src/action.rs`·`registry.rs`·`view/mod.rs`(`ActionMetaView.needs_input`, 레지스트리가 `ActionInput` 단위 변형 여부로 채움), `crates/core/tests/`(모든 액션의 `needs_input`이 `serde_json::from_value(json!({"id": id}))` 성공 여부와 같음), 생성물, 신규 `src/lib/hangul.ts`(`choseong`, `normalizeJamo`, `matchQuery`)+`hangul.test.ts`, 신규 `src/features/palette/commands.ts`(`buildCommands({ actions, branches, recent, status, hasRemote, … })`, 분류표 `RUN_DIRECT`·`OPEN_UI`·`EXCLUDED`(이유 문자열), 정렬: 쉬운 이름 포함 > 초성 > 원래 이름·설명), `commands.test.ts`(모든 `action-meta.json` id가 셋 중 하나에 있어야 통과), `src/lib/shortcuts.ts`(`palette: Ctrl+K`를 `PLANNED`에서 표로, `inTextField: "app"`), `shortcuts.test.ts`.
- 내용: `Irreversible`과 역 해시가 필요한 액션은 `EXCLUDED`("확인 창이 있는 화면에서 해요"). 화면 이동·최근 프로젝트·브랜치 바꾸기는 액션이 아닌 항목. 초성 검색은 "ㅂㄹㅊ"→"브랜치", "ㅂㅈㅈㅈ"→"버전 저장", 섞인 "버ㅈ"→"버전 저장"을 맞춘다.
- 테스트: `hangul.test`(초성 표, 호환 자모/조합 자모 정규화, 빈 질의, 영문 섞임), `commands.test`(분류 완전성, 입력 필요 액션은 `OPEN_UI` 또는 제외, `Irreversible`은 직접 실행 목록에 없음, 정렬), core `needs_input` 시험, 바인딩·`action-meta.json` 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(순수 함수). 보안: 화면은 액션 id와 입력을 보낼 뿐 core가 모두 다시 검사한다.

U18. 명령 팔레트 화면 (`screens: command palette`)
- 목표: Ctrl+K로 열어 찾고 실행한다(D-170, C11).
- 바꿀 파일: 신규 `src/features/palette/Palette.tsx`·`Palette.module.css`·`paletteStore.ts`·`Palette.test.tsx`(`DialogFrame` 위쪽 정렬, 찾기 칸 `role="combobox"`+`listbox`+`aria-activedescendant`, 줄: 쉬운 이름 · 원래 이름(`--text-muted`) · 오른쪽 `Kbd`, 흐린 항목은 `disabledReason`, 결과 없음 "찾는 명령이 없어요", ↑↓·Enter·Esc는 `lib/listKeys.ts`, 한글 조합 중 Enter 무시·입력은 조합 중에도 필터), `src/App.tsx`(`useShortcut("palette")`, 팔레트가 열린 채 Ctrl+K는 닫기), 열기 항목의 연결(`useRepoStore.setView`, 이름 칸 포커스, `useOpenProject`, `switch` 흐름 = U12 대화상자), `src/messages`(`palette.*`), 시험.
- 내용: 시작 화면(저장소 없음)에서는 폴더 열기·복제·최근 프로젝트·설정만. 바로 하기는 `executeAction` 뒤 `notifyDone`(되돌리기 토스트)와 오류는 `showError`. 모달이라 전역 단축키는 쉰다(`modalOpen`).
- 테스트: 열기/닫기(Ctrl+K), "ㅂㄹㅊ" 입력 → "브랜치" 항목, Enter가 `executeAction` 호출(바로 하기) 또는 화면 열기, `Irreversible`이 목록에 없음, 흐린 항목의 까닭 툴팁, IME 조합 중 Enter 무시, 포커스 되돌림, 저장소 없을 때 목록.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 한글 입력 상태에서 초성 검색, 800px.
- Windows/Ubuntu: `Ctrl+K`가 WebView2·WebKitGTK에서 앱에 오는지, 한글 IME 조합(WebKitGTK의 `keyCode` 229)을 손 확인. 보안: 해당 없음.

### E. 키보드로 목록 쓰기

U19. 파일 목록 한 번 Tab과 `<Term>` 포커스 (`screens: one tab stop for the file list and terms inside controls`)
- 목표: 파일 목록을 Tab 한 번으로 쓰고 버튼 안 용어가 Tab 정지가 아니다(C12). 011-P17b 뒤에 한다.
- 바꿀 파일: `src/screens/Workspace/ChangedFiles.tsx`·`ChangedFiles.module.css`(`role="grid"`·`role="row"`·`gridcell`, roving tabindex, 줄 접근성 이름에 상태·넣음/뺌, 체크 상자 `tabIndex=-1`, ↑↓·Home·End, Space 넣기·빼기, 메뉴 키·Shift+F10이 011-P17b 메뉴를 열기, 머리 체크 상자만 별도 Tab 정지, 머리 글자 `tabIndex=0` 삭제), `src/lib/shortcuts.ts`(`toggleFile: Space`, scope `fileList`), `src/ui/Term.tsx`(`closest`로 버튼·링크·메뉴 항목·`summary`·`label` 안이면 `tabIndex` 없음), `src/features/diff/DiffView.tsx`(머리 `h2 tabIndex=0` 정리: 포커스 받을 이유가 없으면 삭제), 011-P19 ④의 `Workspace.test.tsx` Tab 순서 시험 갱신, 시험.
- 내용: 선택(비교할 파일)은 포커스를 따라간다(지금과 같음). 이름 바꿈 파일의 `unstage`는 원래 경로도 함께(기존 `pathsOf`). 파일이 하나도 없으면 목록이 없다.
- 테스트: Tab 한 번에 목록 진입·다음 Tab이 비교 칸(파일 100개 렌더에서 Tab 횟수가 3 이하), ↑↓가 비교 파일을 바꿈, Space가 `stage`/`unstage`를 한 번 부름, 메뉴 키가 줄 메뉴를 엶, 줄 이름 문구, 머리 체크, `Term` 시험(버튼 안은 `tabIndex` 없음·문장 속은 있음·`focusable` 명시가 이김), 기존 `ChangedFiles`·`Term` 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: Tab으로 한 바퀴 돌아 파일 목록에서 멈추는 횟수가 1.
- Windows/Ubuntu: Orca·NVDA에서 `grid`의 행 읽기를 손으로 확인("사람이 할 일"). 보안: 해당 없음.

U20. 브랜치 목록을 combobox·listbox로 (`features: branch list as combobox and listbox`)
- 목표: ↑↓로 고른 줄을 화면 읽기 프로그램이 알고 줄 메뉴를 키보드로 연다(C13).
- 바꿀 파일: `src/features/branches/BranchList.tsx`(찾기 칸 `role="combobox"`·`aria-expanded`·`aria-controls`·`aria-activedescendant`·`aria-autocomplete="list"`, `ul` → `role="listbox"`, 줄 → `role="option"`+`id`+`aria-selected`, `aria-current` 삭제, 집합 위치 `aria-posinset`·`aria-setsize`), `BranchRowMenu.tsx`(`…` 단추 `aria-hidden`+`tabIndex=-1`, 열림 상태를 밖에서 제어), 찾기 칸의 ContextMenu 키·Shift+F10이 고른 로컬 줄 메뉴를 열기, `BranchList.module.css`, 시험.
- 내용: 이름 바꾸기 입력(`RenameField`)이 열린 동안 키는 입력칸이 받는다(기존). 줄 안 `…`는 마우스·터치용이다.
- 테스트: 역할·이름·`aria-activedescendant`가 ↑↓를 따라감, Enter로 고름(조합 중 무시), 메뉴 키로 줄 메뉴 열고 "이름 바꾸기"까지 키보드만으로, 기존 `BranchList.test` 통과(`aria-current` 기대 갱신).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: Orca와 NVDA에서 "N개 중 M번째"를 읽는지 손 확인. 보안: 해당 없음.

U21. 노선도 역 고르기와 메뉴 분리 (`features: select a station without opening its menu`)
- 목표: 역을 누르면 고르기만 하고 메뉴는 두 번째 누름·`⋯`·키로 연다(C14).
- 바꿀 파일: `src/features/history/RouteList.tsx`(`onPointerDown`이 `onSelect`만, 이미 고른 역을 다시 누르면 `onActivate`, 고른 줄 끝 `⋯`(24px, `aria-hidden`+`tabIndex=-1`, 누르면 `onActivate`), Shift+클릭·키보드 동작은 그대로), `RouteList.module.css`, `src/screens/History/History.tsx`(`onActivate` 토글 유지), `docs/design.md`는 U1, 시험.
- 내용: 접힌 구간 줄은 지금처럼 한 번 누르면 펼친다. 오른쪽 클릭은 고르고 메뉴.
- 테스트: `RouteList.test`·`History.test`(첫 누름은 메뉴 없음, 같은 역 두 번째 누름·`⋯`·Enter·오른쪽 클릭은 메뉴, Shift+클릭 구간 선택, 접힌 구간 한 번에 펼침), `StationMenu` 시험은 그대로.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 역을 눌러 고르고 `⋯`로 메뉴를 열기.
- Windows/Ubuntu: 포인터 이벤트 동일. 보안: 해당 없음.

### F. 고정

U22. core: 프로젝트·브랜치 고정 저장 (`core: pin projects and branches`)
- 목표: 최근 목록 파일에 고정을 저장한다(C15).
- 바꿀 파일: `crates/core/src/recent.rs`(`RecentEntry.pinned`·`pinned_branches`, `touch`의 20개 한도가 고정을 밀어내지 않음, `pin(path, bool)`·`pin_branch(path, name, bool)`, 고정 10·브랜치 10·이름 200바이트), `crates/core/src/bootstrap.rs`(`Core::pin_project`·`pin_branch`: 최근 목록 경로·열린 저장소만, 브랜치는 실제 목록에 있어야 하고 `validate_ref_like`), `crates/core/src/view/`(`RecentProject.pinned`·`pinned_branches`), `src-tauri`(`recent_pin`·`recent_pin_branch` 네 곳), `src/lib/api.ts`, 생성물, `crates/core/tests/`, `docs/architecture.md`.
- 내용: 형식 버전은 1 그대로(옛 앱이 새 파일을 읽으면 고정만 무시). 고정이 가득 차면 `PinLimit`+문구. 저장소가 사라진 고정 프로젝트도 목록에 남고 "목록에서 빼기"가 고정을 푼다.
- 테스트: 옛 `recent.json`(칸 없음) 읽기, 고정 유지·한도, 21번째 최근이 와도 고정 10개 남음, 브랜치 고정이 없는 브랜치는 거절, 깨진 파일은 `.bad`로(기존), `tests/security_config.rs`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 경로 비교는 기존 `platform::same_path`. 보안: 브랜치 이름은 표시에만 쓰이고 git 명령으로 가기 전에 검사를 거친다(D-035).

U23. 프로젝트 메뉴와 고정 화면 (`screens: project menu and pinned projects and branches`)
- 목표: 프로젝트 메뉴가 쓸모 있어지고 고정한 프로젝트·브랜치가 맨 위에 남는다(C15, ux.md 1장 중간).
- 바꿀 파일: `src/screens/Workspace/ProjectMenu.tsx`(`<h1>` 안에 `Menu.Trigger`로 순서를 바꿔 단추 안 `h1`을 없앰, 항목: 최근 프로젝트(고정 먼저, 5개), 구분선, "폴더 열기 Ctrl+O", "복제 Ctrl+Shift+O", "다른 프로젝트 열기"), `ProjectMenu.module.css`, `src/screens/Start/Start.tsx`·`ProjectRow.tsx`·`Start.module.css`("고정" 묶음, 줄의 고정 단추 `aria-pressed`, 24px), `src/features/branches/BranchList.tsx`(맨 위 "고정" 묶음, `BranchRowMenu`에 "고정"/"고정 풀기"), `src/lib/queries.ts`, `src/messages`(`pin`·`unpin`·`pinned`·`recentProjects`), 시험.
- 내용: 상단 바 오른쪽 컨트롤 수는 늘지 않는다. "폴더 위치 열기"는 반복 5.
- 테스트: 프로젝트 메뉴 항목·`h1` 구조(트리거 안에 제목 없음)·최근 프로젝트 열기, 고정/풀기가 `recentPin` 호출, 고정이 맨 위, 브랜치 고정 묶음과 줄 메뉴, 키보드로 고정(Enter), 기존 `Start`·`ProjectMenu`·`BranchList` 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 프로젝트와 브랜치를 고정하고 껐다 켜도 맨 위.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

### G. 처음 하는 일 안내

U24. 안내 설정과 안내 줄 (`features: first-time tips setting and tip line`)
- 목표: 안내를 저장하고 보여 줄 부품과 설정을 만든다(D-168, C17).
- 바꿀 파일: `crates/core/src/settings.rs`(`tips_on`(기본 참, `yes()`)·`tips_seen: u8`, `Copy` 유지)·`view/settings.rs`(`TipId`, 보기 `tips_on`·`tips_seen`, 패치 `tips_on`·`tip_seen`·`reset_tips`), 생성물, 신규 `src/ui/TipLine.tsx`·`TipLine.module.css`, 신규 `src/features/tips/useTip.ts`(`useTip(id) → { visible, dismiss }`), `src/features/settings/GeneralSection.tsx`(행 "처음 하는 일 안내" `Choice` 켬·끔+글자 버튼 "다시 보기"), `src/messages`, `crates/core/tests/settings.rs`, 시험.
- 내용: 옛 `settings.json`(칸 없음)은 `tips_on` 참·`tips_seen` 0으로 읽힌다. 알 수 없는 비트는 무시한다. 안내를 닫는 동작은 즉시 `settings_set`(디바운스 없음, 드문 동작).
- 테스트: 기본값·옛 파일·비트 유지·`reset_tips`·한 칸 패치가 다른 칸을 안 바꿈·쓰기 실패 시 메모리, `TipLine`(`role="note"`, "알겠어요"가 `dismiss`), `useTip`(꺼짐/봄/안 봄), 설정 행 시험, `parity`·`copy-rules`.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`, 바인딩 최신.
- Windows/Ubuntu: 앱 데이터 경로는 기존. 보안: 패치 칸은 열거형·bool뿐.

U25. 안내 네 곳 (`features: first-time tips for branch, conflict, sync and undo`)
- 목표: 첫 브랜치·첫 충돌·첫 동기화·첫 되돌리기에 한 번만 안내가 뜬다(D-168).
- 바꿀 파일: `src/screens/Workspace/Workspace.tsx`(홈 알림 줄 자리에 첫 브랜치·첫 동기화·첫 되돌리기), `src/screens/Conflict/Conflict.tsx`(첫 충돌), `src/features/branches/BranchList.tsx`·`src/features/sync/useSync.ts`·`src/features/undo/useUndo.ts`(안내가 뜰 조건: 브랜치를 처음 만들거나 바꾼 직후, 동기화 첫 성공 직후, Ctrl+Z 첫 성공 직후), `src/messages`(`tip.firstBranch`: "브랜치는 폴더가 아니라 기록이 갈라진 선이에요. 파일은 그대로 이 폴더에 있어요.", `tip.firstConflict`: "같은 곳을 서로 다르게 고친 거예요. 한 곳씩 남길 쪽을 고르면 돼요. 언제든 그만둘 수 있어요.", `tip.firstSync`: "동기화는 새 버전을 가져온 다음 내 버전을 올려요.", `tip.firstUndo`: "방금 한 일은 Ctrl+Z로 되돌려요. 지난 작업은 작업 내역에서 되살려요."(글자 버튼 "작업 내역")), 시험.
- 내용: 문장 속 용어는 `t(id)`. 안내는 홈의 다른 알림 줄 아래에 쌓고 행동을 막지 않는다. 충돌 화면이 열려 있으면 홈 안내는 뜨지 않는다.
- 테스트: 각 조건에서 한 번 뜨고 닫으면 `tip_seen`이 저장되며 다시 안 뜸, `tips_on` 거짓이면 안 뜸, "다시 보기"로 다시 뜸, 문구 규칙 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

### H. 이미지 비교

U26. core: 이미지 비교 질의 (`core: image pair query`)
- 목표: 이전·지금 두 이미지를 안전하게 읽어 화면에 준다(C18).
- 바꿀 파일: `crates/git/src/backend.rs`·`cli/`(`blob_at(repo, rev, path) -> Result<Option<Oid>>`: `ls-tree -z`로 정확 일치, 가짜 구현 둘 갱신), 신규 `crates/core/src/image.rs`(`sniff(&[u8]) -> Option<ImageMime>`: png·jpeg·gif·webp 시그니처, svg는 BOM·공백·`<?xml`·주석을 건너뛰고 `<svg`), `crates/core/src/query.rs`(`image_pair`: 작업 폴더 비교는 이전=HEAD 블롭(이름 바꿈이면 원래 경로)·지금=작업 폴더 일반 파일(`symlink_metadata`로 링크 거절, 저장소 밖 거부), 버전 안 비교(`commit`)는 그 버전과 첫 부모의 블롭, 한쪽 3MB 초과면 `too_large`), `crates/core/src/view/image.rs`(`ImageRequest`·`ImagePairView`·`ImageBlob { mime, data, bytes }`), `crates/core/Cargo.toml`(`base64`, `cargo deny check`), `src-tauri`(`query_image_pair` 네 곳), `src/lib/api.ts`·`queries.ts`(`useImagePair`), 생성물.
- 내용: 형식이 아니거나 3MB를 넘으면 두 쪽 모두 `None`/`too_large`로 돌려 화면이 지금의 "이진 파일이에요"로 떨어진다. 읽는 곳은 object database의 블롭과 작업 폴더 일반 파일뿐이다.
- 테스트: `sniff` 표(진짜 png·jpg·gif·webp·svg, `.png` 이름의 html·스크립트 든 svg는 svg로 보되 `<img>`로만 쓰임을 문서화, 빈 파일·잘린 헤더), 임시 저장소 시험(추가·삭제·고침·이름 바꿈·버전 안 비교), 링크는 읽지 않음(Unix), 경로 `../`·절대 경로 거부, 3MB 경계, `tests/security_config.rs`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `cargo deny check`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 경로 구분·CRLF 영향 없음(바이너리). 보안: 읽기 전용, 크기 상한, 링크 안 따라감, base64 외에 해석하지 않음(디코드는 웹뷰가 `<img>`로).

U27. 이미지 비교 화면 (`features: image compare side by side, overlay and swipe`)
- 목표: 이진 이미지 파일을 나란히·겹쳐·밀어 본다(C18).
- 바꿀 파일: 신규 `src/features/diff/ImageCompare.tsx`·`ImageCompare.module.css`·`ImageCompare.test.tsx`(보기 셋 `Choice`, 겹쳐 보기 투명도 슬라이더, 밀어 보기 가림 위치 슬라이더(`input[type=range]` ←→), 체커보드 바탕은 `--surface`·`--surface-sunken`의 `conic-gradient`, 크기 "128×64 → 130×64"는 `onLoad`의 `naturalWidth`, 한쪽만 있으면 한 장과 "새 파일"/"삭제" 글자, 이미지 `alt`는 파일 이름+이전/지금), `src/features/diff/DiffView.tsx`(`diff.binary`면 `useImagePair`를 켜서 이미지면 `ImageCompare`, 아니면 지금 문구), `src/features/history/menu/dialogs/FilesDialog.tsx`(같은 비교 칸이라 자동), `src/messages`(`imageCompare.*`), 시험.
- 내용: 목록 썸네일은 하지 않는다. 큰 이미지는 비교 칸 안에서 `max-width: 100%`로 줄여 보인다(원본 크기 보기 글자 버튼). svg는 `<img src="data:image/svg+xml;base64,…">`로만 그린다.
- 테스트: 보기 전환·슬라이더 키보드(←→가 값을 바꿈)·`aria-valuenow`, 한쪽만 있을 때, `too_large` 때 지금 문구, svg가 `img` 요소로만(DOM에 `svg` 요소 없음), `FilesDialog` 안에서도 동작, CSP 문자열 시험(`security_config`가 `img-src 'self' data:`를 확인).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: png·jpg·svg를 고친 저장소에서 세 보기.
- Windows/Ubuntu: `data:` 이미지와 `conic-gradient` 두 웹뷰 지원. 고DPI에서 흐림은 사람이 확인. 보안: 서버·저장소 글자를 DOM에 넣지 않고 `<img>`만 쓴다(D-037).

### I. 노선도와 글꼴

U28. 태그 역 (`history: larger stations for tagged versions`)
- 목표: 태그가 붙은 버전의 역이 큼직하게 보인다(C19).
- 바꿀 파일: `src/features/history/layout/route.ts`(태그 있는 커밋 역 반지름 +2·테두리 `--text`)·`layout/scene.ts`·`render/SvgRenderer.tsx`·`palette.ts`(고대비 `CanvasText`), 시험(`route.test`, `SvgRenderer.test`).
- 내용: 확대 단계 "태그만"은 그대로. 역 크기는 `scale`(D-153)을 따른다.
- 테스트: 태그 있는 역의 반지름이 큼, 없는 역·합치기 역·지금 위치 표시와 겹치지 않음, scale 1.5, 다크/고대비 색 키.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. Windows/Ubuntu: SVG 동일. 보안: 해당 없음.

U29. 받을 버전 흐린 역 (선택) (`history: faded stations for incoming versions`)
- 목표: 홈 노선도에 가져올 버전을 흐린 속 빈 역으로 미리 보인다(C9). 시간이 모자라면 이 단계를 반복 5로 넘긴다.
- 바꿀 파일: `crates/git`(`log_graph`로 `upstream`의 `HEAD`에 없는 버전 3개, 필요하면 `incoming` 조회), `crates/core/src/query.rs`·`view/map.rs`(`HomeMapView.incoming: Vec<{ short_oid, summary }>`), 생성물, `src/features/history/layout/horizontal.ts`(`homeToRouteNodes`가 `incoming` 노드를 지금 줄 오른쪽에 점선으로 이음, 이름표는 `--text-muted`), `render/SvgRenderer.tsx`, `src/screens/Workspace/HomeMap.tsx`(역 툴팁 = 요약), `src/messages`(`incoming(n)`), 시험.
- 내용: 합치기 제안 블록과 겹치지 않게 최대 3개. 받을 버전이 없으면 아무것도 그리지 않는다.
- 테스트(실제 git, 로컬 bare 원격): 앞선 원격 → `incoming` 3개와 `count_between`과 일치, 없으면 비어 있음, 배치 시험(이름표 겹침 없음), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. Windows/Ubuntu: 차이 없음. 보안: 요약 글자는 React 글자로만.

U30. 코드 글꼴 D2Coding (`styles: d2coding for korean in code`)
- 목표: 한글이 섞인 코드 줄의 칸이 맞는다(C20).
- 바꿀 파일: `package.json`·`pnpm-lock.yaml`(글꼴 npm 패키지 1개), `src/styles/fonts.css` 또는 `main.tsx`의 글꼴 import(한글 범위 woff2 400 하나), `src/styles/tokens.css`(`--font-mono`에 `"D2Coding"` 추가), `THIRD_PARTY_NOTICES.md`, `docs/design.md`는 U1, 신규 `src/styles/fonts.test.ts`.
- 내용: 설치 뒤 번들 크기(`dist` 증가분)를 재어 2MB를 넘거나 woff2가 없으면 하지 않고 progress "사람이 할 일"에 적는다. 라틴은 JetBrains Mono 그대로.
- 테스트: `fonts.test`(`--font-mono`의 글꼴 순서, `unicode-range`가 한글 범위, 라이선스 원문이 `THIRD_PARTY_NOTICES.md`에 있음), `pnpm lint`, 눈으로: 비교 칸의 한글 섞인 줄 정렬, `pnpm tauri build --no-bundle` 크기.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`·`pnpm build`.
- Windows/Ubuntu: 글꼴은 번들이라 같다. 보안: 라이선스(OFL) 표시, `pnpm` 설치 스크립트 제한(`onlyBuiltDependencies`) 유지.

### J. 마무리

U31. 문서·점검·사람이 할 일 (`docs: wrap up ux polish iteration`)
- 목표: 문서를 구현에 맞추고 규칙 점검을 돌린 뒤 반복 5로 간다.
- 바꿀 파일: `docs/design.md`·`docs/spec.md`·`docs/architecture.md`(구현과 다른 값 정리, 명령 목록·OS별 처리·웹 느낌 절), `docs/roadmap.md`("UX 다듬기 완료", 반복 5에 넘긴 것: 창 크기·위치·폴더에서 보기·폴더 위치 열기·OS 알림·진행 막대 여부·Windows 브라우저 키 platform 함수), `docs/progress.md`(체크 U1~U31, "사람이 할 일" 새 항목), `.claude/agents/code-analyst.md`(점검 줄: `cursor: pointer`는 `linkStyle`만, `outline: none`은 대신하는 표시와 함께, 새 `ErrorCode`는 `ERROR_FIX`·ko·en, 새 `ShortcutId`는 `shortcutName`, 새 액션은 팔레트 분류표, `ConfirmDialog` 문구는 `m.confirm`, `Irreversible` 집합 변경은 D-179 이후 값, 이미지는 `<img>`로만, 화면의 `window`/`navigator.clipboard` 직접 사용은 `lib`에서만), `docs/research/img/ux/`(데스크톱 느낌·브랜치 바꾸기 대화상자·오류 토스트·명령 팔레트·단축키 표·이미지 비교·고정 그림, 개인 정보 없는 데모 저장소).
- 내용: code-analyst에게 "반복 끝 점검"(계층 규칙·용어 규칙·보안 점검표)을 맡기고 나온 위반을 고친 뒤 커밋한다. CI 두 OS·`git-min`·`cargo deny check`를 `gh run list`로 확인한다.
- 테스트: `pnpm check:full`, CI.
- 완료 기준: "완료 기준 (반복 전체)"의 항목이 모두 자동 시험 또는 "사람이 할 일"에 있다.
- Windows/Ubuntu: Windows 확인은 사람이 할 일. 보안: 그림에 개인 정보가 없다.

## 위험
- **규모**: 31단계라 길다. 그룹(A~J)은 서로 독립이고 한 단계씩 끝내도 앱이 깨지지 않는다. 시간이 모자라면 뒤에서부터(U29 선택 → U30 → U28 → H 이미지)를 반복 5로 넘기고 U31에 적는다.
- **011·010이 아직 안 끝난 가정**: U5·U7·U9·U10·U19는 011-P17b·P18·P19와 앞선 처리(오류 토스트·`ConfirmDialog`)가 끝난 코드를 가정한다. 구현 첫날 `rg`로 현재 상태를 확인하고 어긋나면 progress에 적고 단계 순서를 맞춘다.
- **"가져가기"와 "가져오기"는 한 글자 차이**다. 사용자 결정(D-165)이라 그대로 쓰되 대화상자 설명에 동작을 풀어 쓰고 `pull`의 쉬운 이름은 바꾸지 않는다. 혼동되면 사용자에게 물을 질문으로 올렸다.
- **가져가기 계산 비용과 정확성**: 스냅샷+`merge_check`는 큰 저장소에서 느릴 수 있다(대화상자가 조회 중 `busy`, core는 실행 때 다시 계산). 겹침 검사는 줄 단위 3-way라 git이 실제로 적용할 때와 어긋날 수 있다 → 적용이 충돌하면 갈아타기를 되돌리고(기존 실패 정리) `CarryBlocked`로 알린다. 시험으로 고정한다.
- **버리기 되돌리기의 stale**: 사용자가 버린 파일을 곧바로 다시 만들거나 고쳤으면 되돌리지 않는다(`UndoStale`). 조용히 덮어쓰지 않는다.
- **브라우저 키 막기의 한계**: WebView2는 일부 키를 JS보다 먼저 가져갈 수 있다. 새는 키 목록을 손 확인으로 찾고 반복 5의 platform 함수(`webview2-com`, unsafe 이유 주석)로 옮길지 사람에게 묻는다.
- **앱 메뉴의 붙여넣기**: 웹뷰가 `navigator.clipboard.readText()`를 막을 수 있다. 막히면 항목이 꺼지고 Ctrl+V 안내가 나온다(기능 손실 작음).
- **첫 화면 숨김**: 첫 렌더가 늦거나 JS가 실패하면 창이 3초 뒤에야 보인다(타이머). 숨긴 창에서 WebView2·WebKitGTK의 타이머가 느려지는지 손 확인.
- **초성 검색과 IME**: 한글 조합 중(`isComposing`, WebKitGTK `keyCode 229`)에는 Enter를 무시하되 입력은 필터한다. 호환 자모(ㅂ)와 조합 자모(ᄇ)를 같게 본다(시험).
- **이미지**: 3MB 안에서도 디코드가 메모리를 쓴다. svg는 `<img>`로만 그려 스크립트가 돌지 않는다. 포맷 판정은 내용 머리라 이름만 바꾼 파일이 그림으로 취급되지 않는다.
- **`settings.json`·`recent.json` 호환**: 칸을 더하되 형식 버전은 1이라 새 파일을 옛 앱이 읽으면 `deny_unknown_fields`(settings)로 기본값으로 돌아간다(개발 중에만 해당, 011-C3과 같은 감수).
- **코드 글꼴 크기**: D2Coding 한글 범위가 커서 번들이 늘 수 있다(2MB 상한, 넘으면 하지 않음).
- **접근성은 자동으로 다 못 잰다**: 대비는 시험이 있지만 화면 읽기 프로그램 동작·배율 선명도·키보드 흐름은 사람이 확인한다("사람이 할 일").
- **Term 툴팁의 키보드 접근**: 버튼 안 `<Term>`의 원래 이름·명령은 마우스 호버로만 뜬다(포커스 정지를 없앤 대가). 키보드 사용자는 단축키 표·명령 팔레트의 원래 이름 열에서 본다.
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md`에 "(자동 결정)"으로 올린 뒤 이어 간다. 사용자 결정 D-165~D-171과 어긋나는 구현이 필요해 보이면 멈추고 progress "사람이 할 일"에 적는다.
- code-analyst 검토 때 먼저 볼 곳: U3(`RESERVED` 표 하나로 막는지, 개발 빌드 예외), U4(`app_ready`가 상태를 바꾸지 않는지, 네 곳), U6(토스트 상한과 오류 보존), U8(실제 git 재현 시험이 두 OS에 있는지), U9(`ERROR_FIX` 완전성, 문구 속 행동 이름), U11(옛 journal 서명 호환, `carry`의 core 재계산, 실패 정리), U13(`Irreversible` 집합 시험 값, 되돌리기 stale), U22(고정 브랜치 이름이 git 명령으로 가기 전 검사), U26(읽기 전용·링크·상한), U27(svg가 `<img>`로만).

## 사람이 할 일
`docs/progress.md`의 "사람이 할 일"에 U31에서 옮긴다. **사람만 줄 수 있는 것**은 아래 "사람에게 묻는 것"뿐이고, 나머지는 실제 컴퓨터에서 확인할 것이다.
- **화면 읽기 프로그램**: Windows는 NVDA와 내레이터, Ubuntu는 Orca로 폴더 열기 → 버전 저장 → 동기화를 한 번씩 해 본다. 브랜치 목록에서 ↑↓가 "N개 중 M번째"로 읽히는지, 파일 목록이 Tab 한 번에 들어가 줄 이름(상태·넣음/뺌)이 읽히는지, 역 메뉴·토스트(오류는 즉시 읽힘)·대화상자 제목, 명령 팔레트. 긴 목록(1만 버전)에서 Orca가 느리지 않은지(가상화 유지).
- **배율**: Ubuntu 100·125·150·200%에서 글자 선명도(WebKitGTK 흐림 버그)와 1px 선, Windows 100·125·150·200%에서도. 800px 창에서 상단 바·토스트·대화상자가 겹치지 않는지.
- **웹 느낌**(릴리스 빌드): 오른쪽 클릭(버튼·목록·입력칸·선택한 글자), F5·Ctrl+R·Ctrl+P·Ctrl+F·F12·Alt+←·Ctrl+Shift+I 중 새는 키, 붙여넣기 메뉴, 버튼·목록 위 커서, 글자 끌림(목록·비교 칸·오류 원문 선택), 이미지·링크 끌기, 화면 끝 튕김. 다크 모드(OS 다크와 설정 다크)로 켤 때 흰 화면. 두 웹뷰의 숨긴 창에서 시작이 느려지지 않는지.
- **기다림·토스트**: 큰 저장소에서 파일을 빠르게 넘겨도 비교 칸이 번쩍이지 않는지, 300ms 뒤 "불러오는 중…"·빈 줄 모양, 버전 저장 직후 다른 토스트가 와도 "되돌리기"가 남는지, 오류 토스트가 닫을 때까지 남는지, 토스트가 포커스 줄을 가리지 않는지.
- **오류 안내**: 로그인 만료(키체인 토큰을 지워 보기), 100MB 넘는 파일을 GitHub에 올려 보기(실제 계정), Windows에서 다른 프로그램이 연 파일 위로 브랜치 바꾸기, 올리기 거절(뒤처진 브랜치). 문구와 해결 버튼, 원문 복사.
- **브랜치 바꾸기**: 고친 파일(수정·삭제·새 파일 섞음)이 있는 채로 두 선택, 겹치는 경우의 까닭, 토스트와 Ctrl+Z, 도착한 브랜치에 두고 간 파일이 있는 경우, 원격 브랜치로 바꾸기, Windows 줄바꿈 파일.
- **버리기·작업 내역**: 파일을 버리고 Ctrl+Z, 그사이 같은 파일을 고쳤을 때, 새 파일 버리기, 작업 내역의 "Ctrl+Z로 되돌려요" 표시와 "이 작업 전으로".
- **팔레트·단축키**: Ctrl+K·Ctrl+/가 WebView2·WebKitGTK에서 앱에 오는지(한국어 자판), 한글 IME로 초성 검색, 단축키 표의 키와 실제 동작.
- **고정·안내·이미지·노선도·글꼴**: 고정이 껐다 켜도 남는지, 안내 네 가지가 한 번만 뜨고 설정에서 다시 켜지는지, png·jpg·gif·webp·svg와 투명 png·큰 이미지(3MB 근처)의 세 보기, 태그 역 크기, 받을 버전 흐린 역(만들었다면), 비교 칸의 한글 줄 정렬.
- **사람에게 묻는 것**: ① 10초 넘는 작업의 진행 막대·1분 넘는 작업의 OS 알림이 필요한지(D-080·새 의존성), ② "가져가기"가 "가져오기"와 헷갈리지 않는지·선택을 기억하는 설정이 필요한지, ③ Windows에서 브라우저 단축키가 새면 platform 함수(`webview2-com`)를 허용할지, ④ D2Coding 번들 크기 허용치, ⑤ 최소 창 크기(011의 질문 그대로).

## 단계 순서 요약
| 단계 | 내용 | 크기 | 크레이트·화면 | 커밋 메시지 제안 |
| --- | --- | --- | --- | --- |
| U1 | 결정 D-172~D-192·문서 | 소 | docs | `docs: add ux polish decisions and plan 012` |
| U2 | 웹 느낌 CSS·한국어 글자 배치·린트 | 중 | styles | `styles: desktop-app feel and korean text layout` |
| U3 | 앱 메뉴·브라우저 단축키 막기 | 중 | 화면(lib, ui) | `features: app context menu and blocked browser keys` |
| U4 | 첫 화면 흰 번쩍임 | 소 | app, 화면 | `app: show the window after the first paint` |
| U5 | 꺼진 버튼 이유·24px·포커스 점검 | 중 | 화면(ui, styles) | `ui: disabled reasons, 24px targets and focus audit` |
| U6 | 토스트 겹침·포커스 가림 | 중 | 화면(ui) | `ui: keep undo toasts, stack up to three and clear focus` |
| U7 | 기다림 표시(0.5초 유지·이전 내용·빈 줄) | 중 | 화면 | `ui: delayed loading with hold, previous content and skeleton` |
| U8 | 새 실패 둘(100MB 거절·파일 사용 중) | 중 | git, core | `git+core: classify oversized files and files in use` |
| U9 | 오류 안내 틀·해결 버튼 표·확인 창 문구 | 대 | 화면 | `features: error guide with one fix button and raw copy` |
| U10 | 큰 파일 50MB·100MB 미리 알림 | 소 | core, 화면 | `core: warn about files over 50MB and 100MB` |
| U11 | 브랜치 바꾸기 core(여기 두기·가져가기) | 대 | core, app | `core: switch with leave-here or carry choice` |
| U12 | 브랜치 바꾸기 대화상자·"여기 두기" 용어 | 중 | 화면, 문서(용어) | `screens: branch switch dialog and leave-here wording` |
| U13 | 버린 파일 되돌리기 | 중 | core, 화면 | `features: undoable discard with backup` |
| U14 | 작업 내역 Ctrl+Z 표시·"이 작업 전으로" | 중 | core, 화면 | `features: show which activity rows undo can reach` |
| U15 | 동기화 줄(원격 확인 실패·시각·툴팁) | 소 | 화면 | `screens: fetch failure line, last check time and sync tooltip` |
| U16 | 단축키 보이기·단축키 표 Ctrl+/ | 중 | 화면 | `ui: shortcut hints and shortcut table` |
| U17 | 팔레트 모델(`needsInput`·초성 검색·분류표) | 중 | core, 화면(lib) | `core: action input flag and command palette model` |
| U18 | 명령 팔레트 화면 Ctrl+K | 중 | 화면 | `screens: command palette` |
| U19 | 파일 목록 Tab 한 번·`<Term>` 포커스 | 중 | 화면 | `screens: one tab stop for the file list and terms inside controls` |
| U20 | 브랜치 목록 combobox·listbox | 중 | 화면 | `features: branch list as combobox and listbox` |
| U21 | 역 고르기·메뉴 분리와 `⋯` | 소 | 화면(history) | `features: select a station without opening its menu` |
| U22 | 프로젝트·브랜치 고정 저장 | 중 | core, app | `core: pin projects and branches` |
| U23 | 프로젝트 메뉴 확장·고정 화면 | 중 | 화면 | `screens: project menu and pinned projects and branches` |
| U24 | 처음 하는 일 안내 저장·부품·설정 | 중 | core, 화면 | `features: first-time tips setting and tip line` |
| U25 | 안내 네 곳 | 소 | 화면 | `features: first-time tips for branch, conflict, sync and undo` |
| U26 | 이미지 비교 질의 | 대 | git, core, app | `core: image pair query` |
| U27 | 이미지 비교 화면 | 대 | 화면 | `features: image compare side by side, overlay and swipe` |
| U28 | 태그 역 크게 | 소 | 화면(history) | `history: larger stations for tagged versions` |
| U29 | 받을 버전 흐린 역(선택) | 중 | git, core, 화면 | `history: faded stations for incoming versions` |
| U30 | 코드 글꼴 D2Coding | 소 | styles | `styles: d2coding for korean in code` |
| U31 | 문서·점검·사람이 할 일 | 소 | docs | `docs: wrap up ux polish iteration` |

순서의 까닭: U1이 모든 단계 앞(결정·문서). 효과가 크고 작은 일부터(A: 웹 느낌·첫 화면·버튼·토스트·기다림은 모든 화면에 보이고 서로 독립), 그다음 오류(B: Rust 분류 U8 → 화면 틀 U9 → 큰 파일 U10; 해결 버튼 표 `ERROR_FIX`는 이후 단계 U13·U14·U15가 쓴다), 되돌릴 수 있는 일(C: 브랜치 바꾸기 core → 화면, 그다음 버리기, 작업 내역; U13이 U6의 되돌리기 토스트를 쓴다), 길 찾기(D: U16 단축키 표가 U18 팔레트의 `Kbd` 열을 쓴다), 키보드 목록(E: U19는 011-P17b 뒤, U21은 U5의 `⋯` 24px 규칙을 쓴다), 고정(F: 저장 → 화면, 화면은 U20 브랜치 목록 구조 위), 안내(G: 부품·설정 → 네 곳, U25의 첫 되돌리기·첫 브랜치는 U12·U13 뒤), 이미지(H), 노선도·글꼴(I: 독립이라 맨 뒤), 마무리(J). 각 단계는 CI가 초록이어야 다음으로 간다.

## 검토 반영 (code-analyst, 구현 전)
구현은 아래를 위 단계 설명보다 우선한다. U1이 결정을 올릴 때도 이 절로 고친 값을 쓴다. 결정 번호는 D-172부터로 고정하지 않고 **U1 때 decision.md의 마지막 번호 + 1부터** 쓴다(그 사이 011·010·4c가 결정을 더할 수 있다).

- **R1 (U6, C4) 쌓인 되돌리기 토스트.** 모든 "되돌리기"는 가장 최근 줄(`undo_last`)을 되돌리므로(`useUndo.ts`), 새 되돌리기 토스트가 오면 앞 토스트의 단추는 "작업 내역" 글자 버튼으로 바뀐다(ux.md 원안). 시험: 저장 토스트 뒤 브랜치 바꾸기 토스트가 오면 앞 토스트에 "되돌리기"가 없다.
- **R2 (U13, C8) 버린 파일 되돌리기.** 지금 `unpark`는 작업 폴더에 고친 파일이 하나라도 있으면 `WouldOverwrite`로 거절한다(`park.rs`). 그래서 stash가 가진 경로만 되살리는 새 `GitOp`(경로 한정 적용, 새 파일인 세 번째 부모 포함, `refuse_unsafe` 검사)를 U13 앞부분에 둔다. 백업은 "경로가 같은 가장 최근 `discard-<브랜치>` 백업"으로 찾지 않고 `Outcome::Discarded`에 백업 oid를 담아 서명된 journal에 남긴다(D-063, D-085). `UndoState::is_valid`가 그 ref를 받도록 넓히는지 확인한다. `journal_mode`의 `Always`는 `count > 0`일 때만. `Irreversible` 집합 시험(D-136)이 그때까지 없으면 U13이 만든다(`discard`를 빼고 `clear-lock`·`force-push`·원격 브랜치 지우기·이름 바꾸기).
- **R3 (U8, U9) 새 실패 둘.** `fileTooLarge`는 큰 파일이 이미 로컬 버전에 들어 있어 작업 폴더에서 빼도 풀리지 않는다. 문구는 "그 파일이 든 버전에서 빼야 해요"이고 해결 버튼은 "버전 기록"이다. `failure.rs`의 `RULES`에서 `RemoteUnreachable`·`HttpAuthFailed` 앞에 둔다. `fileInUse`는 `Unlink of file`, `Device or resource busy`, `being used by another process`로만 판정하고 "Permission denied"는 권한 문제라 넣지 않는다(Ubuntu 시험도 이 문구로).
- **R4 (U11·U12, C7) 가져가기의 빈틈.** (a) 도착 브랜치에 맡긴 파일이 있으면 "가져가기"를 끄고 까닭 툴팁을 보이며 "여기 두기"만 둔다(두 번 꺼낼 수 없다). (b) 가져가기를 되돌릴 때 겹쳐 막히면 `UndoStale`과 "작업 내역" 안내. (c) 원격 브랜치로 바꿀 때 `merge_check`의 도착 이름은 `<원격>/<브랜치>`. (d) `query_switch_plan`의 스냅샷은 파일 500개·합계 50MB를 넘으면 계산하지 않고 "가져가기"를 까닭과 함께 끈다(조회마다 해시하지 않게). (e) 이미 있는 새 파일과 겹치는 경우를 시험에 넣는다. (f) 화면은 상태를 읽는 중이면 계획 조회를 기다리고, 낡은 `dirty`로 core가 조용히 맡기지 않게 core도 실행 때 다시 본다.
- **R5 (U7) 이전 내용 유지.** `placeholderData`는 함수로 두어 같은 repo의 이전 결과일 때만 쓴다(다른 프로젝트 목록이 남지 않게). 파일이 바뀔 때의 임시 상태는 effect로 지우지 않고 안쪽 컴포넌트에 `key`를 건다(react-hooks `set-state-in-effect`).
- **R6 (U22, C15) 고정.** `recent.rs`의 `touch`는 항목을 새로 만들므로 `pinned`·`pinned_branches`를 물려받게 한다. 고정 프로젝트는 20개 한도 밖에서도 남고, 사라진 브랜치 이름은 읽을 때 고정 목록에서 뺀다.
- **R7 (U14, C16) 작업 내역 표시.** `Never`에 "올린 버전 저장"을 넣지 않는다(되돌리는 새 버전을 만들어 되돌린다, D-073). 힌트는 `undo()`와 같은 규칙 함수에서 만들고 "되돌리는 새 버전을 만들어요" 변형을 둔다. "이 작업 전으로 되돌리기"는 HEAD를 옮기는 줄 중 `before.branch == after.branch == 지금 브랜치`일 때만 보이고 core가 입력 때 같은 검사를 한다. `restore`에 올림 검사(`guard::refuse_published`)가 없으면 이 단계에서 더한다.
- **R8 (U30, C20) 코드 글꼴.** JetBrains Mono 라틴(0.6em)과 D2Coding 한글(1em)을 섞으면 칸이 맞지 않는다. 비교 칸은 라틴도 D2Coding으로 하거나 한글 `@font-face`에 `size-adjust`를 준다(측정 시험을 둔다, 문자열 시험만으로 끝내지 않음). 글꼴은 공식 배포 woff2를 `src/assets`에 두고 라이선스를 적는다(npm 패키지 이름은 확인된 것만).
- **R9 (U3, C1) 앱 메뉴와 브라우저 키.** 클립보드 접근은 `src/lib/clipboard.ts` 한 곳(`ui`에서 `navigator.clipboard` 직접 사용 금지). 붙여넣기는 `execCommand("insertText")`로 React 입력칸에 넣는다. `contextmenu`는 `window` bubble 단계에서 처리한다(capture에서는 React의 `defaultPrevented`를 볼 수 없다). 입력칸 메뉴의 "모두 선택"은 결정문에 적는다(D-171 보충). 전역 `contextmenu` 막기는 `import.meta.env.PROD`일 때만. WebView2의 `readText` 권한 창과 Radix 포커스 가두기 겹침은 손 확인.
- **R10 (U16·U18, C10·C11) 단축키 표와 팔레트.** `useShortcut`은 모달이 열려 있으면 쉰다(`modalOpen`). 그래서 팔레트 안의 Ctrl+K(닫기)와 표 안의 Ctrl+/는 그 컴포넌트가 직접 키를 처리한다(시험). 확인 창이 있는 액션은 목록에서 빼지 않고 흐리게 보이며 까닭 툴팁을 준다(D-170 "모든 액션"). 위험도 `caution` 이상이면서 안내 흐름이 있는 액션은 바로 실행하지 않고 그 화면을 연다(분류 시험).
- **R11 (U26, C18) 이미지 읽기.** 크기는 읽기 전에 `cat-file --batch-check`(또는 작업 폴더 `symlink_metadata` 길이)로 확인한다. `path`는 `status`나 `ls-tree`에 있는 경로만 받고 부모 구성요소도 링크가 아닌지 본다(D-037). 쿼리 `gcTime`은 짧게. `base64`는 이미 트리에 있는 버전에 맞춘다(cargo deny 중복 경고).
- **R12 (U9, C6) 오류 안내 틀의 계층.** `ui/ErrorNotice`는 `features`를 부르지 않는다. 해결 버튼은 prop이나 context로 받고 `ERROR_FIX` 표만 `features/errors`에 둔다. "이름과 이메일 설정"은 설정 대화상자를 "일반" 구역으로 여는 인자를 `settingsStore`에 더한다.
- **R13 (U10) 큰 파일 알림.** "Git LFS로 두면 올릴 수 있어요" 줄은 앱에 LFS 설정이 없고 용어표 밖 낱말이라 넣지 않는다.
- **R14 (계획 안 어긋남).** 새 `ErrorCode`는 4개(U8 둘, U11 `CarryBlocked`, U22 `PinLimit`)이고 `ERROR_FIX`가 모든 코드를 요구하므로 U11·U22 커밋이 항목을 함께 넣는다. 줄 번호는 구현 첫날 `rg`로 다시 맞춘다(예: `Rewrite.tsx`의 `m.loading`, `queries.ts`의 `useDiff`).
- **R15 (작은 것).** U2: 비접두사 `user-select`가 WebKitGTK에서 되는지 기존 코드(`RouteList.module.css`, `base.css`의 `:root[data-resizing]`)로 먼저 확인한다. U5: `Splitter`의 24px 면제 까닭은 "끌기 전용, 키보드 ←→와 더블클릭 대체". U19: Home/End/Space/메뉴 키 판정은 `src/lib`에(D-056), `<Term>`의 `closest` 목록에 `[role="option"]`·`[role="row"]`·`[role="tab"]`을 더한다. U11: `SwitchInput.dirty`만 `skip_serializing_if`로 두어 옛 journal 줄 서명이 맞게 한다.

이미 011에서 처리된 것(이 계획 작성 뒤): ux.md 높음 4건(공용 `ConfirmDialog`와 모달 중 단축키 쉼 67273ba, 오류 토스트 4a0a247, 문구 속 버튼 이름 32c50f0, 이름·이메일 152d586·9b245af·4727783), 칸 나눔선(D-164, ad775dc), 홈 노선도 점선·5역(011-P12, c4d31cf).
