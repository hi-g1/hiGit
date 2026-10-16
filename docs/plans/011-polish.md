# 011 다듬기 (글자 크기·언어·밑줄·노선도·파일 줄)
- 관련 결정: D-006, D-010, D-015, D-024, D-026, D-029, D-031, D-043, D-044, D-046, D-049, D-056, D-057, D-060, D-062, D-090, D-093, D-095, D-116, D-127, **사용자 결정 D-138~D-145**(뒤집지 않는다). 새로 정할 것은 아래 "결정 후보" C1~C16이고, 구현 첫 단계(P1)가 `docs/decision.md` 맨 아래에 D-146부터 "(자동 결정)"으로 올린다. 사용자 결정 D-138~D-145에는 "(자동 결정)"을 붙이지 않는다. 사람에게 묻지 않고 추천안으로 진행한다.
- 관련 명세: spec.md "화면 구성"의 시작 화면·작업 공간·버전 기록·설정 행, "진입점"(단축키), 용어표(바뀌는 것 없음). design.md "글꼴"·"간격과 모양"·"부품 규칙"(Term 툴팁, 목록 행, 홈 노선도)·"설정 대화상자"·"다크 모드". architecture.md "Query"·"프론트와 백엔드 계약"·"버전 기록 그리기 계층"·"OS별 처리".
- 근거 자료(결정 아님): `docs/research/polish.md`(사용자 의견), 그림 `docs/research/img/flow4/r-home.png`(홈: 테두리 없는 상단 바 단추, 모두 켠 체크, 파일 이름 위·폴더 아래, "변경 없는 12줄 · 펼치기"), `r-log.png`(버전 기록), `img/start-normal.png`·`start-normal-dark.png`(시작 화면).
- 앞선 계획: 010(반복 4b, 010-H2까지 구현). **이 반복은 010의 H3 앞에 들어가고, 010의 나머지(010-H3부터)는 다듬기가 끝난 뒤 이어서 한다.** 단계 이름은 P1~P20이고 `docs/progress.md`와 커밋 본문에는 "011-P3"처럼 계획 번호를 붙여 적는다(005의 H·010의 H와 겹치지 않게). 진행: 사람 확인 없이 Claude가 계획 → code-analyst 검토 → 아래 순서대로 구현, 한 단계 = 한 커밋.

## 목표
글자가 두 OS에서 읽기 편하게 커지고(OS 기본 크기와 OS 글자 배율을 따르며, 설정과 Ctrl+=·Ctrl+−·Ctrl+0으로 바꾼다), 언어·화면 모드·용어 설명 보기를 설정에서 고르고, 밑줄·"…"·잘린 경로·모두 빈 체크 같은 어색한 곳이 사라진다. 고친 파일은 이름 위·폴더 아래로 보이고, 압축·실행 파일을 넣으려 하면 한 번 알려 준다.

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04, Windows 11)에서 손으로, 가능한 것은 자동 테스트로 확인한다. 손 확인은 `docs/progress.md` "사람이 할 일"에 모은다.
- 글자: `src/styles/tokens.css`의 크기 토큰이 D-138 값(보조 13, 본문·코드 14, 화면 제목 17, 버전 이름 칸 22, 시작 화면 이름 28)이고 모두 rem이다. 12px보다 작은 글자가 없다(가장 작은 "작게"에서도 보조 글자 ≥ 12px, 시험). CSS 모듈에 길이·크기의 `px`가 남지 않는다(1~3px 선·테두리·`@media` 제외, 린트가 막는다). 루트 글자 크기 하나를 바꾸면 행 높이·간격·아이콘·노선도 역과 글자가 함께 커진다.
- 시작 크기: Windows는 본문 14px, Ubuntu는 15px. OS 글자 배율(Windows 접근성 "텍스트 크기", Ubuntu `text-scaling-factor`)을 곱한다. 설정 "글자 크기"(작게·보통·크게·아주 크게)와 Ctrl+=·Ctrl+−·Ctrl+0이 같은 값을 바꾸고 다시 켜도 유지된다. 웹뷰 자체 확대는 꺼져 있고 버전 기록의 `+`·`-`·Ctrl+휠(D-095)은 그대로 노선도만 확대한다.
- 800px 너비와 "아주 크게"에서 상단 바가 넘치지 않는다(줄 바꿈). Ubuntu에서 홈·버전 기록·브랜치 목록·설정 스크린샷이 `docs/research/img/polish/`에 있다(P9).
- 언어: 설정 "언어"(시스템 따라가기·한국어·English)를 고르면 다시 시작하지 않고 바로 바뀌고 저장된다. 시스템 따라가기는 OS 언어가 한국어면 한국어, 그 밖은 English. 날짜·"3분 전"·버전 이름 제안도 따른다. English에서 이름이 같은 용어 툴팁은 이름 줄이 빠진다. ko·en 문구 키가 한쪽에만 있으면 시험이 실패한다.
- 밑줄: `<Term>`은 밑줄·점선 없이 툴팁만(지연 600ms), 설정 "용어 설명 보기"를 끄면 툴팁도 없다. 버튼·상단 바 글자에 밑줄이 없고 글자 버튼은 호버하면 `--surface-hover` 바탕이다. 밑줄은 링크 모양 파일 하나에서만, 호버할 때만 쓰고 린트가 그 밖의 `text-decoration: underline`·`border-bottom: dotted`를 막는다.
- 화면 모드: 설정 "화면 모드"(시스템 따라가기·라이트·다크)가 바로 적용되고 저장된다.
- 노선도: 홈 노선도가 "…" 글자 없이 왼쪽 끝 24px 점선과 그 아래 "이전 N개"(누르면 버전 기록)를 보이고, 역은 5개까지, 이름표 폭에 맞춰 간격이 넓고, "지금 여기"는 `--primary` 바탕 이름표로 합치기 제안 블록 앞까지 오른쪽으로 가서 폭을 다 쓴다. 역에 호버·키보드 포커스를 하면 전체 이름 툴팁. 비교 칸의 접힌 줄은 "변경 없는 N줄 · 펼치기"이고 누르면 펼친다.
- 고친 파일: 넣은 파일이 0개면 모든 체크가 켜진 모양이고 하나를 끄면 나머지가 들어간다. 줄은 위 파일 이름·아래 폴더(`--text-muted`), 상태 글자는 "고침". 브랜치 단추와 프로젝트 이름은 둘 다 테두리 없이 호버 때 바탕이 생긴다. 비교 칸 머리에 "줄 바꿈"(기본 켬)이 있다.
- 압축(.zip 등)·실행 파일(.exe, ELF 등)을 넣고 저장하면 비밀 파일 알림 줄과 같은 꼴로 한 번 알리고 막지 않는다.
- 불러오는 표시는 300ms가 넘을 때만 보인다. 긴 이름은 끝을(파일 경로는 앞을) 말줄임하고 호버하면 전체가 보인다. 포커스 테두리는 모든 컨트롤이 같은 `--accent` 2px이고 Tab 순서는 상단 바·노선도·이름 칸·파일 목록·비교 칸이다. 설정 대화상자에 구역 "일반"·"GitHub"·"고급"(Git 위치·로그 폴더 열기)이 있다.
- `pnpm check:full`, 두 OS CI(`git-min` 포함)·`cargo deny check` 통과. code-analyst 점검 위반 0건.

## 범위
- **이번에 한다**: polish.md 1~5장 전부와 D-138~D-145, 그리고 6장 중 화면 모드·설정 구역(D-145)·시각 표시 규칙·말줄임·300ms 로딩·포커스 테두리와 Tab 순서·스크롤 막대 폭.
- **반복 5로 미룬다**(C16): 파일 줄 마우스 오른쪽 메뉴·더블클릭(폴더에서 보기·경로 복사·기본 앱으로 열기: 저장소 안 경로 검증과 실행 파일 거절이 필요한 보안 설계), 창 크기·위치 기억과 최소 960x600(D-138의 800px 시험과 부딪히고 모니터·배율 손 확인이 필요), 시스템 글꼴 pt 읽기.
- 새 Tauri 명령 1개(`app_display_hints`). 새 액션·새 git 쓰기 없음. 새 용어 없음(글자 크기·언어 등은 Git 용어가 아니라 용어표에 넣지 않는다). 바뀌는 `core::view`: `AppSettingsView/Patch`, `DisplayHintsView`(신규), `AppStatus.gitPath`, `DiffRequest.fullContext`, `SecretRule` 변형 2개.
- 새 의존성 없음(Windows `reg.exe`·Ubuntu `gsettings`를 `Command`로 부르고, 바이너리 경로는 절대 경로).

## 현재 상태 (코드로 확인한 사실)
- `src/styles/tokens.css`: 크기 토큰이 px(`--font-size-sm 12`, `-md 13`, `-title 15`, `-home 20`(쓰는 곳 없음), `-name 20`, `-display 26`), 간격 `--space-1~8` 4~32px, `--radius-*`, `--row-height 32px`, `--scrollbar-size 8px`. 화면 CSS는 글자 크기를 모두 이 토큰으로 쓰고(`font-size: <값>` 직접 쓴 곳 없음), 길이 px 직접 값이 약 120곳(35개 파일: 대화상자 폭 `min(440px, …)`, 높이 `min-height: 56px`, 점 `8px`, 체크 상자 `16px` 등). 다크 모드는 이미 `@media (prefers-color-scheme: dark)`와 `:root[data-theme="dark"|"light"]`가 있다.
- 노선도: 배치(`layout/horizontal.ts`, `layout/route.ts`)가 px 좌표 장면(`Scene`)을 만들고 `SvgRenderer`가 `width={scene.width}`·`viewBox` 같은 값으로 그린다. 글자 폭은 `geometry.ts`의 `textWidth` 추정(한글 1em, 그 밖 0.55em)이고 `horizontal.ts`의 `FONT = 12`가 `--font-size-sm`과 따로 적혀 있다. 세로 목록은 `ROW_H = 48` 고정 가상 스크롤(`RouteList.tsx`가 `ROW_H`로 스크롤·`rowAt`·`top` 계산). 홈 노선도 높이 `MAP_HEIGHT = 104`를 `HomeMap.tsx`가 style로, 시작 화면 작은 노선도는 `RoutePreview.tsx`의 `MINI_WIDTH = 240`을 쓴다.
- `horizontal.ts`: 한 줄 간격 `STEP = 120`(최소 84)이라 노선도가 왼쪽에 몰리고 "지금 여기"는 `lastX + STEP`에 놓인다(폭을 안 씀). 접힌 앞쪽 역은 198줄이 선 위에 "…" 글자로 그린다. 역 이름은 `fitText(summary, step - 8, 12)`로 자른다. "지금 여기"는 글자(`labels.here`)이고 `SceneTag`(`--primary` 바탕 `rect` 64x18 고정)는 upstream 이름표("GitHub")만 쓴다. 역 호버는 SVG `<title>`뿐이라 키보드 포커스에는 안 뜬다.
- `src/lib/shortcuts.ts`의 `RESERVED`에 `ctrl("Equal")`·`ctrl("Minus")`·`ctrl("Digit0")`이 있고, 버전 기록 확대(`zoomIn`/`zoomOut`)는 Ctrl 없는 `Equal`·`NumpadAdd`·`Minus`·`NumpadSubtract`(scope `history`)다. `matches`는 `ctrlKey`를 정확히 비교한다. `tauri.conf.json`에 `zoomHotkeysEnabled`가 없고 `capabilities/default.json`에는 웹뷰 확대 권한(`core:webview:allow-set-webview-zoom`)이 없다. 창은 `minWidth 800`·`minHeight 560`.
- 설정: `crates/core/src/settings.rs`(`SettingsStore`, 파일 `settings.json` = `{"v":1,"allow_published_rewrite":bool}`, `deny_unknown_fields`, 4KB, 링크면 읽지 않음, 틀린 모양은 기본값, 쓰기 실패는 메모리), `AppSettings`(`Copy`, `ExecCtx.settings`), `view/settings.rs`(`AppSettingsView/Patch`), 명령 `settings_get/set`, `api.settings()/setSettings()`, `useAppSettings()`(`staleTime: Infinity`). `SettingsDialog`는 "GitHub" 구역 하나(`GitHubSection`), `DialogFrame size="wide"`.
- 언어: `useUiStore`에 `locale: "ko"|"en"`(초기 "ko", 설정 없음)과 `termDisplay`. `useMessages()`·`Term`·열다섯 화면이 `locale`을 읽는다. `src/lib/time.ts`는 `formatRelative`(1분 미만 "지금", 시간·일·주·달·해 단위)와 `formatDate`(올해 "10월 7일", 아니면 연도 포함). 호출부 열한 곳(`ProjectRow`, `LastProject`, `SyncButton`, `CompareFooter`, `RouteList`, `BranchList`, `PrRow`, `Card`, `FilesDialog`, `HunkPanel`, `Activity`) — `Activity`는 `formatDate`를 날짜 묶음 키로, `Intl.DateTimeFormat`을 시각 글자로 직접 쓴다. 버전 이름 제안(`features/commit/suggest.ts`)은 `m.suggest`를 써서 이미 언어를 따른다. `src/messages/en.ts`는 `Messages` 형이라 키가 어긋나면 `tsc`가 막지만 런타임 시험은 없다.
- 밑줄: `ui/Term.module.css`의 `.term`이 `border-bottom: 1px dotted var(--text-muted)`와 `cursor: help`, `Term`은 `tabIndex=0`, `ui/Tooltip.tsx`는 지연 400ms(`TOOLTIP_DELAY_MS`) 하나를 모두가 쓴다. `text-decoration: underline`은 `screens/History/History.module.css`(`.scopeButton` 상시)와 `ui/Toast.module.css`(`.action:hover`) 두 곳. 링크 부품은 없고 `ErrorNotice`의 `<summary>`("자세히 보기")가 문장 속 접기 표시다. `.stylelintrc.json`은 색 규칙만 있다.
- 홈 위쪽: `TopBar.module.css`의 `.bar`는 한 줄 flex(줄 바꿈 없음, `min-height: 44px`), 프로젝트 메뉴(`ProjectMenu.module.css .trigger`)는 테두리 없음·호버 바탕, 브랜치 단추(`BranchList.module.css .trigger`)는 `1px solid var(--border)` 테두리 상자. 오른쪽 글자 버튼 `.textButton`은 호버 바탕.
- 파일 줄: `screens/Workspace/ChangedFiles.tsx`가 체크 `checked={file.staged}`, 줄은 `[상태 64px][경로 한 줄 말줄임][이름 바꿈 전 경로]`(32px), 상태 글자는 `ko.ts`의 `changeKind.modified: "수정"`. 머리 체크는 `included === files.length`일 때 켜짐, 일부면 가운데 줄. `useStageMutation`이 `stage`/`unstage` 액션을 부른다. 넣은 파일이 0개면 `suggestName`·`secret_check`·버전 저장이 이미 "모두 넣고 저장"(D-062)으로 동작한다(`features/commit/suggest.ts filesToSave`, `query.rs secret_check`의 `stage_all`).
- 비교 칸: `features/diff/DiffView.tsx` 43줄이 `row.section || "…"`로 묶음 머리 줄을 그린다(앞 묶음과 사이에 몇 줄이 생략됐는지는 안 보임). 표 글자는 `--font-size-sm`(12px), 줄은 `white-space: pre`·가로 스크롤. `git/cli/diff.rs`는 `-U3` 고정이고 `DiffRequest { path, origPath, untracked, commit }`에 문맥 줄 수 입력이 없다. `Hunk { old_start, old_len, new_start, new_len, section, lines }`, `DiffLineView`에 `old_line`·`new_line`이 있어 묶음 사이 생략 줄 수는 화면에서 셀 수 있다.
- 비밀 파일 알림: `core/src/secrets.rs`(`name_rule`·`content_rule`·`LARGE_BYTES`), `query.rs secret_check`(넣은 파일 또는 `stage_all`, 이름 규칙 → 큰 파일 → 내용 규칙 순, 파일 200개·앞 256KB), `view/secret.rs`(`SecretRule`, `SecretFinding { path, rule, line }`), 화면 `features/secrets/SecretNotice.tsx`(제목 `m.secretTitle(n)`, 줄마다 `m.secretRule[rule]`, 단추 "빼기"·"제외할 파일에 넣기"·"그래도 저장"), `NameBox`가 한 번 알리고 같은 입력이면 저장.
- `docs/progress.md`: 010-H2 끝, 다음은 이 계획. `scratchpad`에 `env.sh`·`shot.sh`(DISPLAY :1, `xprop`로 "hiGit" 창을 찾아 캡처)·`xkey.py`(XTest 키 입력)·`xclick.py`·`stop-dev.sh`가 있다.

## 결정 후보 (P1이 D-146부터 "(자동 결정)"으로 올린다)
추천안을 적었다. 기존 사용자 결정은 뒤집지 않는다(D-095의 "Ctrl+=·Ctrl+−를 쓰지 않는다"는 D-138이 이미 바꿨고 P1이 D-095 상태 줄에 그 사실을 적는다).

| 번호 | 결정(추천안) | 까닭 | 대안 |
| --- | --- | --- | --- |
| C1 (D-146) | **글자 크기 단위와 값.** 모든 길이 토큰은 rem이고 `1rem = 16px × 화면 배율`이다(디자인 문서의 px 숫자는 배율 1일 때 값). JS가 `document.documentElement.style.fontSize = 16 × 배율 px`만 바꾼다(CSS 변수·미디어 쿼리 없음). 배율 = `OS 기본 본문 px / 14 × OS 글자 배율 × 단계 값`을 0.9~2.0으로 자른다. 단계: 작게 13/14, 보통 1, 크게 16/14, 아주 크게 18/14. 토큰: `--font-size-sm 0.8125rem`(13), `-md`·`-code 0.875rem`(14, `-code`는 새 토큰: 비교 칸·파일 경로 코드), `-title 1.0625rem`(17), `-name 1.375rem`(22), `-display 1.75rem`(28), `-home` 삭제. 간격 `--space-1~8`은 `0.25/0.5/0.75/1/1.5/2rem`, `--radius-control 0.375rem`·`-panel 0.5rem`, `--row-height 2rem`(두 줄 행용 `--row-height-two 3.25rem` 신설), `--scrollbar-size 0.5rem`. `--border-width 1px`·선택 막대 2px·포커스 2px는 px 그대로(선은 글자와 함께 커지지 않는다). 보통 + Windows 기본이면 배율이 1이라 본문 14px, Ubuntu는 15/14라 15px. 가장 작은 보조 글자는 작게+Windows에서 13 × 13/14 = 12.07px로 12px 아래로 가지 않는다. | D-138의 "한 곳만 바꾸면 전체가 커진다"를 루트 글자 크기 하나로 이룬다. 16px 기준이면 디자인 숫자를 16으로 나눠 옮기기 쉽고, JS가 켜지기 전(첫 화면)에도 기본 16px = 본문 14px로 Windows 기본과 같다. 배율 값은 토큰·`fontScale.ts` 상수 두 곳뿐이라 사용자가 스크린샷(P9)을 보고 조정하기 쉽다. | CSS `zoom`(WebKitGTK·글자 선명도 차이), `em` 연쇄(중첩 오류), 단계별 토큰 묶음(토큰 수 곱절) |
| C2 (D-147) | **OS 글자 설정은 `platform::display_hints()` 한 곳이 읽는다.** `DisplayHints { body_px, text_scale_percent, dark }`. `body_px`는 Windows 14, Ubuntu 15, 그 밖 14(상수, `cfg`). `text_scale_percent`: Ubuntu는 `/usr/bin/gsettings get org.gnome.desktop.interface text-scaling-factor`(`LC_ALL=C`, `1.25` → 125), Windows는 `%SystemRoot%\System32\reg.exe query "HKCU\Software\Microsoft\Accessibility" /v TextScaleFactor`(REG_DWORD 100~225, 없으면 100). `dark: Option<bool>`: Ubuntu `gsettings get org.gnome.desktop.interface color-scheme`(`'prefer-dark'`면 참, `'default'`·`'prefer-light'`면 거짓), Windows `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize" /v AppsUseLightTheme`(0이면 다크), 읽지 못하면 `None`. 모든 호출은 절대 경로·출력 256바이트까지·1.5초 제한이고 실패·범위 밖(50~300 밖)은 기본(100, `None`). core `Core::display_hints()`가 그대로 전달하고 새 명령 `app_display_hints`(async + blocking, 네 곳)로 화면이 읽는다. 화면은 앱 시작 때 한 번, 창 포커스를 얻을 때(5초에 한 번까지) 다시 읽는다. | OS 판단은 `crates/platform`에만(D-002). 웹뷰는 OS 글자 배율을 반영하지 않고 WebKitGTK 22.04는 GNOME 다크 설정을 못 볼 수 있어 앱이 읽어 준다. 절대 경로는 PATH 바꿔치기를 막는다(`terminate_tree`의 `taskkill`보다 엄격하게). 새 의존성(`winreg`, `dconf` crate) 없이 이미 쓰는 `Command` 방식. | `winreg`·`gio` crate(의존성·deny 검토), 웹뷰 `window.devicePixelRatio`만 쓰기(텍스트 배율을 못 읽음), 시작 때 한 번만 읽기(배율을 바꿔도 반영 안 됨) |
| C3 (D-148) | **설정 저장은 `settings.json`의 고정 칸을 넓힌다. 파일 버전 `v`는 1 그대로.** 새 칸: `language`(`system`·`ko`·`en`), `font_size`(`small`·`normal`·`large`·`xlarge`), `theme`(`system`·`light`·`dark`), `term_help`(참), `diff_wrap`(참). 새 칸은 `#[serde(default)]`라 옛 `{"v":1,"allow_published_rewrite":…}`가 그대로 읽힌다. 칸에 없는 값·알 수 없는 칸·4KB 초과는 기존 규칙대로 파일 전체 기본값(안전한 쪽: 올린 버전 바꾸기 꺼짐). `AppSettings`는 계속 `Copy`(열거형·bool만). 새 열거형은 `core::view`에 두고(`LanguageSetting`·`FontSizeSetting`·`ThemeSetting`, specta, camelCase 값) 패치는 `Option` 칸이다. 옛 앱이 새 파일을 읽으면(`deny_unknown_fields`) 기본값으로 돌아가는 것은 감수한다. 화면 쪽 `localStorage`에는 두지 않는다. | D-139가 "앱 데이터에 저장"을 정했고 010-H2가 만든 저장소를 그대로 쓰면 읽기·쓰기·메모리 대체·4KB 검사를 다시 만들지 않는다. `v` 유지는 호환이 쉽다(개발 중 파일뿐). | `v: 2` 이전(옛 파일 처리 코드가 더 듦), 화면 `localStorage`(core가 모르고 첫 화면 전에 읽는 길이 따로 필요함, 반복 5-0의 `localStorage` 평문 이월과 같은 길) |
| C4 (D-149) | **시스템 언어는 웹뷰의 `navigator.languages`로 판단한다.** 첫 항목이 `ko`로 시작하면 한국어, 그 밖은 English(`src/lib/locale.ts`의 `resolveLocale(setting, languages)` 순수 함수 + `systemLanguages()` 한 곳, OS 분기 없음). 앱 시작 때 첫 화면을 그리기 전에 적용한다: `main.tsx`가 `settings_get`·`app_display_hints`를 먼저 읽어(0.8초 제한, 실패하면 기본값) 값을 쿼리 캐시와 루트 요소(`lang`, `font-size`, `data-theme`)·`useUiStore`에 넣은 뒤 화면을 그린다. 바뀔 때는 `useDisplaySync`가 같은 적용 함수를 부른다. | WebView2는 Windows 표시 언어, WebKitGTK는 로케일 환경을 `navigator.languages`로 준다(둘 다 OS 언어). 판단 코드가 하나이고 `platform`에 새 읽기를 만들지 않는다. 먼저 읽고 그리면 영어 사용자가 한국어를 한 번 보는 깜빡임이 없다. | `platform`에서 로케일 읽기(OS 코드 늘림), 설정을 기다리지 않고 기본 한국어로 먼저 그림(깜빡임) |
| C5 (D-150) | **시각 표시 규칙(polish 6장)을 `src/lib/time.ts` 한 곳으로.** `formatTime(unix, now, locale)`: 1분 안·미래(시계 오차) "지금", 1시간 안 "N분 전", 같은 날 "오후 3:12"(en "3:12 PM"), 같은 해 "10월 7일", 그 전 "2025년 10월 7일". 호버에는 정확한 시각(`formatExact`, `dateStyle: long`·`timeStyle: medium`). 새 부품 `ui/Time.tsx`(`<time dateTime title>`)가 둘을 묶는다. `formatRelative`는 `formatTime`으로 대체하고 `formatDate`(날짜만)는 작업 내역의 날짜 묶음 키로 남긴다. `Intl.DateTimeFormat`·`Intl.RelativeTimeFormat`은 `src/lib/time.ts` 밖에서 쓰지 못하게 ESLint `no-restricted-properties`로 막는다(`Activity`의 직접 호출을 `formatClock`으로). | 호출부 열한 곳이 제각각이라(어떤 곳은 날짜만, 어떤 곳은 상대) 규칙을 한 함수에 모은다. 언어를 따르는지도 한 곳에서 시험한다. | 호출부마다 유지(어긋남) |
| C6 (D-151) | **화면 모드 적용.** `theme`가 `light`·`dark`면 `document.documentElement.dataset.theme`에 넣고(tokens.css가 이미 처리), `system`이면 속성을 지운다. `system`일 때는 `display_hints().dark`가 알려진 값이면 그 값으로 `data-theme`을 정하고(WebKitGTK가 OS 다크를 못 볼 때를 위해) `None`이면 속성을 지워 CSS `prefers-color-scheme`에 맡긴다. 고르는 즉시 바뀐다. | tokens.css가 `data-theme`를 이미 지원하고 부품 코드는 같다(design.md "다크 모드"). | CSS 미디어 쿼리만(WebKitGTK 22.04에서 시스템 따라가기가 틀릴 수 있음) |
| C7 (D-152) | **글자 크기 단축키.** 표(`shortcuts.ts`)에 `fontLarger`(Ctrl+`Equal`, Ctrl+Shift+`Equal`(=Ctrl++), Ctrl+`NumpadAdd`, 화면 글자 "Ctrl+="), `fontSmaller`(Ctrl+`Minus`, Ctrl+`NumpadSubtract`, "Ctrl+-"), `fontReset`(Ctrl+`Digit0`, Ctrl+`Numpad0`, "Ctrl+0"), 모두 `scope: "global"`·`inTextField: "app"`. `RESERVED`에서 `ctrl("Equal")`·`ctrl("Minus")`·`ctrl("Digit0")`을 뺀다. 단계는 네 칸 사이를 한 칸씩 움직이고 끝에서는 아무 일 없다. 0은 "보통"으로 돌아가는 것이다. 웹뷰 자체 확대는 `tauri.conf.json`에 `"zoomHotkeysEnabled": false`를 명시해 끄고(WebView2의 `IsZoomControlEnabled`), 웹뷰 확대 권한(`core:webview:allow-set-webview-zoom`)은 capabilities에 열지 않으며(Linux·macOS의 Tauri 폴리필은 그 권한이 있어야 돈다), 앱 핸들러가 `preventDefault`한다. 시험이 이 둘을 확인한다. 버전 기록 확대(Ctrl 없는 `+`·`-`·Ctrl+휠)와는 `ctrl` 값이 달라 겹치지 않는다. 설정은 즉시 `uiStore`에 반영하고 저장은 `settings_set`으로. | D-138의 단축키를 표 한 곳에 넣는다(D-056). `code`로 판정해 한글 입력·다른 자판에서도 같다. | Ctrl 없는 키(입력칸과 겹침), 웹뷰 확대와 앱 확대 둘 다 허용(두 번 커짐) |
| C8 (D-153) | **SVG 노선도는 "디자인 단위로 배치하고 그릴 때 배율을 곱한다."** 배치(`layout/*`)는 지금의 px 수치를 그대로 단위로 쓰고(시험 그대로) `Renderer`가 `scale` 인자(루트 글자 크기/16, 기본 1)를 받아 `<svg width={scene.width × scale} height={scene.height × scale} viewBox=장면 그대로>`로 그린다(선 굵기·역 반지름·글자가 함께 커진다). SVG 글자 크기는 CSS 토큰이 아니라 장면 단위 상수 `SCENE_FONT = 13`을 `fontSize` 속성으로 준다(배율이 한 번만 곱해지게; `horizontal.ts`의 `FONT`도 이 상수). 배치에 주는 너비는 `px / scale`이고, 가상 스크롤 목록은 행 높이 `ROW_H × scale`(px)과 `scale`로 나눈 장면 좌표를 쓴다. 이름표(`SceneTag`)는 `width`를 글자 폭에서 계산한다. 배율은 `useUiStore.uiScale`(루트 글자 크기를 정할 때 같이 넣음, 기본 1)이다. | 배치·판정·시험을 건드리지 않고 한 경계에서만 곱한다(D-017: 판정은 장면 좌표). `getComputedStyle`을 매번 읽지 않아 jsdom 시험도 된다. | 배치 함수 전체에 `scale` 인자(시험·상수 전부 수정), CSS `transform: scale`(레이아웃 크기가 안 바뀜) |
| C9 (D-154) | **밑줄·툴팁 세부.** `Term`은 `border`·`cursor: help` 없이 글자만 두고 툴팁만 뜬다(포커스 테두리는 유지). `Tooltip`이 `delay` 인자를 받고(기본 400ms 그대로, 아이콘 단추 툴팁 등) `Term`만 600ms(`TERM_TOOLTIP_DELAY_MS`). 설정 `term_help`가 꺼지면 `TermTooltip`이 툴팁을 만들지 않고 `Term`의 `tabIndex`도 없앤다(`useUiStore.termHelp`). 링크 모양은 `src/ui/linkStyle.module.css` 한 파일(호버 때만 `text-decoration: underline`, 평소엔 `--text`·밑줄 없음, 포커스 `--accent` 테두리)이고 지금은 `ErrorNotice`의 "자세히 보기" `<summary>`가 쓴다(문장 속 접기). 린트는 stylelint `declaration-property-value-disallowed-list`로 `text-decoration`·`text-decoration-line`의 `underline`과 `border-*`의 `dotted`를 막고 `linkStyle.module.css`만 예외로 한다. `History`의 `.scopeButton` 상시 밑줄은 없애고(고른 것은 600·`--text`, 호버는 `--surface-hover`), `Toast`의 `.action:hover`는 `--primary-hover` 바탕으로 바꾼다. | D-140을 그대로. 링크 부품이 지금 쓸 곳이 하나라 `ErrorNotice` 접기에 붙여 쓸모 없는 부품을 만들지 않는다. 토스트는 `--primary` 바탕이라 `--surface-hover`가 맞지 않아 `--primary-hover`(이미 토큰)를 쓴다. | 모든 툴팁 지연 600ms(아이콘 단추가 느려짐), 링크 부품 없이 린트만(허용 목록이 없음) |
| C10 (D-155) | **홈 노선도 배치 세부(장면 단위).** 홈(`density: "home"`)은 지금 줄의 역을 5개까지만 그린다(나머지는 `folded`에 더한다). 시작 화면 큰 노선도는 지금처럼 너비에 맞는 만큼. 접힌 앞쪽(`folded > 0`)은 "…" 글자 대신, 지금 줄의 왼쪽 끝(한 줄이면 `LEFT`, 두 줄이면 갈라진 곡선이 끝난 곳)에서 24단위를 `--border-strong` 점선으로 긋고(팔레트 키 `~fold-line`), 그 아래 `SCENE_FONT` `--text-muted`로 "이전 N개"(N = 접힌 수)를 둔다. 홈에서는 이 글자를 SVG가 아니라 장면의 `fold { x, y, count }` 좌표 위에 겹친 진짜 `<button>`(글자 버튼 모양, 누르면 버전 기록)으로 그리고, 시작 화면은 SVG 글자만(누르는 곳이 아님). 역 사이 간격 = `min(남은 폭/(n−1), max(96, 가장 긴 이름표 폭(최대 200단위) + 16))`. 이름표는 `fitText(summary, 간격 − 8)`. 인접한 이름표가 겹치지 않게 시험한다. "지금 여기"는 `SceneTag`(`--primary` 바탕, 폭 = 글자 폭 + 16, 높이 18)이고 "지금 여기"의 x는 합치기 제안 블록 앞(장면 너비 − 오른쪽 여유 72)이다. 그 사이는 점선(기존 `dashed`)이 이어진다. upstream 이름표와 같은 역에 겹치면 upstream 이름표를 한 칸 위로 올린다. 호버·키보드 포커스 툴팁은 각 역 위에 겹친 포커스 가능한 `<span tabIndex=0 role="img" aria-label=전체 이름>`을 디자인 시스템 `Tooltip`으로 감싼다(단추로 만들지 않는다: 누르는 동작이 없음). | D-141을 구체화. "이전 N개"가 누를 수 있어야 하고(키보드) 포커스 순서에 들어가야 해서 SVG 글자 대신 HTML을 겹친다. 역 툴팁은 `<title>`이 키보드에 안 뜨는 점을 푼다. | SVG 안 클릭 영역(키보드 포커스 불가), 모든 역을 단추로(눌러도 아무 일 없는 단추) |
| C11 (D-156) | **비교 칸 접힌 줄·펼치기·줄 바꿈.** 묶음(hunk) 사이와 맨 앞의 생략 줄 수는 화면이 줄 번호로 센다(앞 묶음 마지막 `newLine`+1 ~ 다음 묶음 첫 `newLine`−1, 첫 묶음 앞은 `첫 newLine − 1`). 줄은 "변경 없는 N줄 · 펼치기"(`--surface-sunken`, 글자 `--text-muted`)이고 묶음 머리의 함수 이름(`section`)은 오른쪽에 `--text-muted`로 덧붙인다. 마지막 묶음 뒤는 파일 길이를 몰라 펼친 뒤에만 보인다(r-home.png의 끝 줄은 이번에 못 맞춘다). 누르면 core `DiffRequest.fullContext: bool`(기본 거짓, `#[serde(default)]`)로 다시 읽어(`git diff -U<큰 수>`, 기존 비교 상한 안에서 `truncated`로 알림) 그 구간의 문맥 줄만 끼워 넣는다. 화면에 한 번 읽은 전체 문맥을 둔다(파일을 바꾸면 버림). 상한에 걸리면 "펼치기" 대신 `m.diffTruncated`. "줄 바꿈"은 비교 칸 머리 오른쪽 글자 버튼 켬·끔(`aria-pressed`), 기본 켬, 설정 `diff_wrap`에 저장: 켜면 `white-space: pre-wrap; overflow-wrap: anywhere`, 줄 번호·기호는 윗줄에 붙는다. 끄면 지금처럼 가로 스크롤. | 짧게 읽는 화면(D-049)에서 "…"만 보이는 곳을 그림대로. 펼치기는 읽기 전용이라 새 액션이 아니고 `git diff` 인자 하나(`-U`)뿐이다(D-003). 파일 길이를 core가 세려면 블롭을 읽어야 해 이번에는 뺀다. | 맨 끝 줄을 위해 `FileDiffView.totalLines`(블롭 전체 읽기), 줄 단위 펼치기(복잡) |
| C12 (D-157) | **넣은 파일 0개일 때 체크 모양 세부.** `virtualAll = files.length > 0 && included === 0`. `virtualAll`이면 모든 줄 체크와 머리 체크가 켜진 모양(`checked`), 행 체크를 누르면 눌린 파일을 뺀 나머지를 `stage`한다. 나머지가 없으면(파일 하나뿐) 그 파일을 `stage`해 확정한다(모양은 같다). `virtualAll`에서 머리 체크를 누르면 모두 `stage`(확정, 모양 같음). index는 저절로 바꾸지 않는다(D-062). 알려진 한계: 넣은 파일을 하나씩 끄다 마지막 하나를 끄면 0개가 되어 다시 모두 켠 모양으로 돌아온다(버전 저장이 모두 넣고 저장하므로 모양이 맞다). 접근성 이름은 `<Term stage>`·`<Term unstage>`(지금처럼). | D-142 구현. "하나를 끄면 나머지를 넣는다"가 파일 하나일 때 비는 것을 메운다. | 머리 체크를 막기(툴팁으로만 알림은 어색함) |
| C13 (D-158) | **압축·실행 파일 알림.** `SecretRule`에 `Archive`·`Executable`을 더한다(비밀이 아니어도 같은 알림 줄 틀). 판정은 이름(확장자, 소문자)과 앞 4바이트(매직)로: 압축 `.zip .7z .rar .tar .gz .tgz .bz2 .xz .zst`/`PK\x03\x04`·`\x1f\x8b`·`7z\xBC\xAF`·`Rar!`, 실행 `.exe .msi .dll .dmg .deb .rpm .appimage`/`\x7fELF`·`MZ`. 쉘·배치 스크립트와 실행 비트(100755)는 알리지 않는다(코드이므로). 한 파일에 하나만: 비밀 이름 규칙 → 큰 파일(50MB) → 압축·실행 → 내용 규칙(압축·실행 파일 머리는 비밀 내용 검사에서 뺀다). 알림 제목은 걸린 것에 비밀 규칙이 하나라도 있으면 지금 `secretTitle`, 모두 압축·실행이면 `heavyTitle(n)` "압축·실행 파일 N개". 줄마다 `m.secretRule[rule]`("압축 파일"·"실행 파일"). 막지 않는다("그래도 저장", 같은 입력이면 저장). 단추는 지금과 같다(빼기·제외할 파일에 넣기·그래도 저장). | D-040·D-077의 알림 틀을 그대로 쓰되 사용자가 말한 `home.zip`·실행 파일을 같은 길에 넣는다. 매직 4바이트는 이미 읽는 앞 256KB 안이라 비용이 없다. | 이름만(확장자 없는 ELF를 놓침), 알림 줄을 새로 만들기 |
| C14 (D-159) | **작은 디테일 규칙.** ① 포커스 테두리: 토큰 `--focus-width 2px`·`--focus-offset 0.0625rem`·`--focus-offset-inset -2px`와 `base.css`의 전역 `:focus-visible { outline: var(--focus-width) solid var(--accent); outline-offset: var(--focus-offset) }` 하나로 통일하고 개별 CSS의 같은 선언은 지운다(예외: 토스트 단추는 `--primary-text`, 목록 줄·목록 자체는 안쪽 `--focus-offset-inset`). ② 로딩: 훅 `src/lib/useDelayed.ts`(`useDelayedFlag(pending, 300)`)로 `m.loading` 표시 다섯 곳(`Workspace`·`DiffView`·`History`·`Activity`·`Rewrite`)이 300ms 넘을 때만 보인다. ③ 말줄임: 부품 `ui/Truncate.tsx`(끝 말줄임, `from="start"`면 앞 말줄임을 `direction: rtl` + `<bdi>`로, 잘렸을 때만 `Tooltip`으로 전체; 마우스 진입·포커스에서 `scrollWidth > clientWidth` 측정)를 프로젝트 이름·브랜치 이름·파일 이름과 폴더·버전 이름(목록)·검토 요청 제목 다섯 곳에 쓴다. ④ Tab 순서: 작업 공간 DOM 순서를 상단 바 → 노선도(역·이전 N개) → 이름 칸 → 파일 목록 → 비교 칸으로 시험으로 고정(양수 `tabIndex` 없음). ⑤ 스크롤 막대: 지금 전역 얇은 막대(`--scrollbar-size`)를 비교 칸·목록 모두 같은 폭으로 쓰는 것을 확인하고 `scrollbar-gutter`는 쓰지 않는다(OS 모양을 따르는 부분은 이미 `scrollbar-width: thin`). | 6장 항목을 규칙 한 곳으로. | 항목별로 따로(어긋남) |
| C15 (D-160) | **설정 대화상자 구역.** 탭이 아니라 세로로 쌓은 구역 제목 셋: "일반"(`GeneralSection`: 글자 크기·언어·화면 모드·용어 설명 보기. 반복 4b의 010-H5가 "이름과 이메일"을 이 구역 맨 위에 더한다), "GitHub"(기존 `GitHubSection`), "고급"(`AdvancedSection`: Git 위치(코드 글꼴, 읽기 전용, `AppStatus.gitPath`)·글자 버튼 "로그 폴더 열기"(기존 `app_open_log_folder`). 반복 4b의 010-H10이 "올린 버전도 바꾸기"를 이 구역에 더한다). 구역 사이는 `--border` 선, 구역 제목은 15px(`--font-size-title`) 600. 고르는 부품은 `ui/Choice.tsx`(`role="radiogroup"` + 글자 버튼 `role="radio" aria-checked`, 방향키 이동, 고른 것은 600·`--text`). 010의 "이름과 이메일 구역"·"고급 구역" 설명이 이 틀과 어긋나는 곳은 010 "검토 반영"이 아니라 이 계획 끝 "010에 미치는 영향"으로 고친다. | D-145의 구역 이름을 지키고 010이 이미 정한 항목을 어디에 넣을지 정한다. | 탭(400px 높이에서 800px 폭 시험이 더 어려움) |
| C16 (D-161) | **이번에 하지 않는 것.** 파일 줄 마우스 오른쪽 메뉴(폴더에서 보기·경로 복사·이 파일 고친 것 버리기)와 더블클릭 기본 앱 열기, 창 크기·위치 기억과 최소 960x600, 시스템 글꼴 크기(pt) 읽기는 반복 5에서 한다(roadmap "반복 5"에 적는다). 이유: 기본 앱으로 파일을 여는 일은 실행 파일·스크립트를 돌릴 수 있어 저장소 안 경로 검증·확장자 거절·확인 설계가 필요하고, 창 최소 960은 D-138의 "800px 너비 시험"과 부딪히며 창 위치는 모니터·배율 손 확인이 많다. 창 최소 크기는 800x560 그대로 둔다. | 범위를 줄여 1~5장과 결정 D-138~D-145를 완전히 한다. | 이번에 모두(보안 설계가 새 결정 묶음이 됨) |

## 용어 변경
없음. 글자 크기·언어·화면 모드·용어 설명 보기·줄 바꿈은 Git 용어가 아니라 `docs/spec.md` 용어표와 glossary에 넣지 않는다. 화면 낱말만 `src/messages` ko·en에 더한다: `settings.general`·`settings.advanced`·`settings.fontSize`(작게·보통·크게·아주 크게)·`settings.language`(시스템 따라가기·한국어·English·언어 이름은 자기 말로)·`settings.theme`(시스템 따라가기·라이트·다크)·`settings.termHelp`("용어 설명 보기")·`settings.gitPath`·`settings.openLogFolder`·`earlierVersions(n)`("이전 N개")·`diffFold(n)`("변경 없는 N줄 · 펼치기")·`wrapLines`("줄 바꿈")·`heavyTitle(n)`("압축·실행 파일 N개")·`secretRule.archive`("압축 파일")·`secretRule.executable`("실행 파일"). en은 같은 키에 영어("N earlier", "N unchanged lines · Expand", "Line wrap", "N archive or executable files"). `ko.ts`의 `changeKind.modified`는 "수정" → "고침"(en은 그대로 "Modified"), design.md 색 표의 "수정된 파일" → "고친 파일"(D-143). 새 문구는 `copy-rules.test.ts`·`terms-in-messages.test.ts`(괄호·옛 용어·합쇼체 없음)를 통과해야 한다.

## 단계
각 단계 = 커밋 하나, 메시지는 `<영역>: <요약>`(아래 제안). Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`(`src/lib/bindings.ts`·`action-meta.json`). 새 Tauri 명령(`app_display_hints`)은 `src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣는다(`tests/security_config.rs`가 확인). 화면은 git도 `invoke`도 직접 부르지 않고 `src/lib/api.ts`·`queries.ts`로만 core를 부른다(D-010, ESLint가 막음). 색·간격·글꼴·크기는 `tokens.css` 변수만. 새 화면 문구는 `src/messages` ko·en 둘 다 같은 커밋에(이름과 숫자만, D-049). 용어는 `<Term>`, 문장 속 용어는 `t(id)`(D-029). 테스트는 사용자 설정·실제 키체인·실제 네트워크를 건드리지 않는다. `docs/progress.md`는 단계마다 "011-P3 끝"처럼 갱신한다.

### A. 결정과 단위 바탕

P1. 결정·문서 (`docs: add polish decisions and plan 011`)
- 목표: 이후 단계가 쓰는 결정을 먼저 확정하고 문서를 새 값에 맞춘다.
- 바꿀 파일: `docs/decision.md`(C1~C16을 D-146~D-161 "(자동 결정)", 상태 확정. D-095 상태 줄에 "(Ctrl+=·Ctrl+−는 D-138·D-152가 글자 크기에 씀, 버전 기록 `+`·`-`는 그대로)", D-116 상태 줄에 "구역은 D-145·D-160", D-006 영향에 "Term 표시 방법은 D-140"), `docs/design.md`("글꼴": 크기 목록을 D-138 값과 rem·"문서의 px는 배율 1일 때 디자인 값" 한 줄, "간격과 모양": rem·`--row-height-two`, "부품 규칙": Term 툴팁 지연 600ms·밑줄 없음·링크 규칙·글자 버튼 호버, 목록 행 두 줄·체크 모양(D-142), 홈 노선도 "이전 N개"·"지금 여기" 이름표·5역, 비교 칸 접힌 줄·줄 바꿈 새 줄, "설정 대화상자" 구역 셋·고르는 부품, "다크 모드" 화면 모드 설정, 색 표 `--modified` "고친 파일", "시작 화면 색"의 "…" 문장을 점선·"이전 N개"로), `docs/spec.md`(설정 행: 구역 "일반"·"GitHub"·"고급"과 010이 더할 것, 작업 공간·버전 기록 행의 파일 줄·접힌 구간 문구 한 줄씩, 단축키가 나오는 "진입점"에 Ctrl+=·−·0), `docs/architecture.md`(OS별 처리 표에 "글자 배율·다크 읽기" 행, 프론트와 백엔드 계약의 명령 목록에 `app_display_hints`는 P5에서), `docs/roadmap.md`("다듬기" 절을 단계 요약과 "반복 5로 넘김"(C16)으로, "반복 5"에 파일 줄 메뉴·창 크기·위치 추가), `docs/progress.md`("다듬기 단계" 체크리스트 P1~P20).
- 내용: 결정 문장은 이 표를 그대로 옮긴다. 용어표·glossary는 건드리지 않는다(용어 변경 없음). CLAUDE.md는 고치지 않는다(규칙은 린트로 막는다).
- 테스트: `pnpm test`(`spec-sync`·`copy-rules`), `pnpm lint`.
- 완료 기준: 위 문서가 새 값으로 일치하고 테스트 통과.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

P2. rem 토큰과 CSS 단위 (`styles: switch size tokens to rem and raise font sizes`)
- 목표: 모든 길이·크기를 rem으로 바꾸고 D-138 값으로 키워 루트 글자 크기 하나로 화면 전체가 같이 커지게 한다. 아직 JS 배율은 없어 기본 루트 16px = 본문 14px.
- 바꿀 파일: `src/styles/tokens.css`(C1 표 그대로; `--font-size-home` 삭제; 새 `--font-size-code`·`--row-height-two`), `src/styles/base.css`(`--motion-shift: 0px`은 `0rem`로, 코드 글꼴 요소 `code,kbd,pre,samp`에 `font-size: var(--font-size-code)`), 화면 CSS 모듈 35개(아래 규칙), `.stylelintrc.json`(규칙), 신규 `src/styles/tokens.test.ts`, 신규 `scripts/stylelint-rules.test.mjs`(규칙 시험, 기존 `scripts/eslint-rules.test.mjs`·`check.mjs`가 시험을 도는 방식에 맞춰 등록), `src/features/diff/DiffView.module.css`(표 글자 `--font-size-sm` → `--font-size-code`), `src/ui/ListRow.module.css`(`.mono` → `--font-size-code`).
- 내용: ① 바꿈 규칙: `rg "[0-9.]+px" src --glob "*.css"`로 찾아 길이 값 `px ÷ 16`을 rem으로(대화상자 폭 `min(440px,…)` → `min(27.5rem,…)`, 560→35rem, 480→30rem, 520→32.5rem, 960→60rem, 640→40rem, 360→22.5rem, 320→20rem, 220→13.75rem, 56→3.5rem, 44→2.75rem, 36→2.25rem, 28→1.75rem, 16→1rem, 8→0.5rem, 6→0.375rem, `Workspace`의 `minmax(280px, 340px)` → `minmax(17.5rem, 21.25rem)`, `Activity`·`FilesDialog`·`Conflict` 그리드 열 폭 등, `NameBox`의 `calc(… + 4px)` → `0.25rem`, `Toast`의 `translateY(4px)` → `var(--motion-shift)`, `Checkbox`의 체크 표시 `8px/4px`·`translateY(-1px)` → rem, `CardMap`의 `letter-spacing: -2px` → `-0.125rem`). ② px로 남기는 것(선): `border`·`outline`·`box-shadow`·`border-*-width`(1~3px 선과 띠, 선택 막대 `inset 2px`), `@media (width <= 900px)`의 조건. ③ 보이지 않는 요소의 `1px × 1px`(`History`·`StationMenu`·`CardMap`)은 `/* stylelint-disable-next-line declaration-property-unit-disallowed-list -- 화면 밖 접근성 요소 */`로 둔다. ④ `.stylelintrc.json`에 `declaration-property-unit-disallowed-list`: `"/^(font-size|line-height|width|height|min-width|min-height|max-width|max-height|padding.*|margin.*|gap|row-gap|column-gap|top|right|bottom|left|inset.*|grid-template-.*|grid-auto-.*|flex-basis|border-radius|letter-spacing|transform)$/": ["px"]`(`tokens.css`는 `--` 변수라 걸리지 않는다. 함수 안(`min()`·`calc()`)의 px도 걸린다). ⑤ `tokens.test.ts`: `tokens.css`를 읽어 `--font-size-*`가 모두 rem이고 가장 작은 것이 0.8125rem이며(`-home` 없음), `--space-*`·`--radius-*`·`--row-height*`·`--scrollbar-size`가 rem이고, 토큰 밖에서 쓸 px는 `--border-width`·`--focus-*`뿐인지 확인. `scripts/stylelint-rules.test.mjs`: stylelint API로 `width: 10px`·`min(10px, 1rem)`은 오류, `border: 1px solid`·`outline-offset: 2px`·`@media (width <= 900px)`는 통과.
- 테스트: `pnpm test`·`pnpm lint`·`pnpm typecheck`(CSS라 jsdom은 변화 없음), 위 두 시험. 눈으로: 개발 앱에서 홈·버전 기록·설정이 깨지지 않는지(스크린샷은 P9).
- 완료 기준: `rg "[0-9.]+px" src --glob "*.css"` 결과가 선·미디어 쿼리·`tokens.css`의 선 토큰·허용한 `stylelint-disable` 줄뿐이다.
- Windows/Ubuntu: 두 웹뷰 모두 rem·`min()`을 지원한다. 기본 루트 16px라 지금은 두 OS가 같은 크기(본문 14px). 보안: 해당 없음.

P3. 노선도 비례 (`history: scale the route map with the root font size`)
- 목표: SVG 노선도·가상 스크롤 목록이 `scale` 하나로 같이 커지게 한다(C8). 배치 시험은 배율 1에서 그대로 통과한다.
- 바꿀 파일: `src/state/uiStore.ts`(`uiScale: number` 기본 1, `setUiScale`), 신규 `src/lib/useUiScale.ts`(스토어에서 읽기), `src/features/history/layout/scene.ts`(`SCENE_FONT = 13` 상수, `SceneTag.width`), `layout/horizontal.ts`(`FONT` → `SCENE_FONT`, 태그 폭을 글자 폭으로), `render/Renderer.ts`·`render/SvgRenderer.tsx`(`scale?: number` 기본 1, 글자는 `fontSize={SCENE_FONT}` 속성, 태그 `rect` 폭 `t.width`)·`SvgRenderer.module.css`(글자 `font-size` 두 줄·태그 글자 줄 삭제), `screens/Workspace/HomeMap.tsx`(`MAP_HEIGHT × scale`, 배치 너비 `width / scale`), `features/history/RoutePreview.tsx`·`RoutePreview.module.css`(`.mini` 폭 `15rem`와 장면 폭 `MINI_WIDTH` 단위 일치, 큰 노선도 너비 `/ scale`), `features/history/RouteList.tsx`(행 `rowPx = ROW_H × scale`로 스크롤·`rowAt`·`top`·`height`·`visibleRows`·`scrollToRow`, 그래프 칸 `paddingLeft: scene.width × scale`, `viewport`는 장면 단위 `first × ROW_H`), 관련 시험 파일.
- 내용: `scale`은 모든 곳에서 `useUiScale()` 하나(루트 글자 크기를 정하는 P6이 값을 넣기 전까지 1). `layoutHorizontal`·`verticalScene`은 단위 계산 그대로. `SvgRenderer`는 `width={scene.width × scale}`·`height={scene.height × scale}`·`viewBox` 그대로. 판정(`rowAt`)은 `clientY`를 `scale`로 나눠 단위로 바꿔 부르거나 `rowPx`로 부른다(한 가지로 정해 `RouteList` 안에 한 번만).
- 테스트: `SvgRenderer.test`(scale 1.5 → `width`·`height`가 1.5배, `viewBox`는 그대로, 글자 `fontSize` 13), `RouteList.test`(scale 2에서 행 top이 `i×96`, 클릭 위치 → 같은 행 선택, 방향키 스크롤), `horizontal.test`(scale과 무관, 기존 통과), 태그 폭이 글자 폭에 비례, `RoutePreview`의 작은 노선도가 scale 1.25에서 `width = 240 × 1.25`.
- 완료 기준: `pnpm test`·`pnpm typecheck`·`pnpm lint`. 눈으로는 P6 뒤.
- Windows/Ubuntu: SVG `fontSize` 속성은 두 웹뷰 동일. 고DPI Windows에서 소수 배율 선명도는 손 확인. 보안: 역 이름은 지금처럼 React 글자로만(D-037).

### B. 글자 크기(OS 읽기·설정·단축키)

P4. platform: OS 글자 배율·다크 읽기 (`platform: read os text scale and color scheme`)
- 목표: OS 기본 본문 크기·글자 배율·다크 여부를 `crates/platform` 한 곳이 읽는다(C2).
- 바꿀 파일: 신규 `crates/platform/src/display.rs`(+시험), `crates/platform/src/lib.rs`(`pub use display::{DisplayHints, display_hints}`), `docs/architecture.md`(OS별 처리 표 행).
- 내용: `DisplayHints { body_px: u8, text_scale_percent: u16, dark: Option<bool> }`, `display_hints() -> DisplayHints`(실패하지 않고 기본값). `body_px`는 `cfg!(windows)` 14, `cfg!(target_os = "linux")` 15, 그 밖 14. 읽기: 자식 프로세스를 `configure_child`(Windows `CREATE_NO_WINDOW`)로 띄우고 stdout을 256바이트까지만, 1.5초 안에 안 끝나면 `kill`하고 값 없음(작은 비공개 도우미 `run_limited(cmd) -> Option<String>`; 자식은 `Stdio::null()` stdin). Linux: `/usr/bin/gsettings`(없으면 건너뜀)에 `get org.gnome.desktop.interface text-scaling-factor`·`color-scheme`(`LC_ALL=C`), Windows: `<SystemRoot>\System32\reg.exe`(`SystemRoot` 환경 변수, 없으면 `C:\Windows`)에 `query "HKCU\Software\Microsoft\Accessibility" /v TextScaleFactor`·`query "HKCU\…\Themes\Personalize" /v AppsUseLightTheme`. 파서는 순수 함수로 분리: `parse_scaling_factor("1.25\n") → Some(125)`(소수 둘째 자리까지, 0.5~3.0 밖이면 `None`), `parse_color_scheme("'prefer-dark'\n") → Some(true)`, `parse_reg_dword("    TextScaleFactor    REG_DWORD    0x96\r\n") → Some(150)`(10진 환산, 100~225 밖이면 `None`), `AppsUseLightTheme 0x0 → dark Some(true)`. 읽지 못한 칸은 `text_scale_percent = 100`, `dark = None`.
- 테스트: 파서 표(정상·소수점·쉼표 로케일 `1,25`는 `None`·범위 밖·빈 문자열·이상한 글자·256바이트 넘는 줄), `display_hints()`가 어떤 환경에서도 패닉 없이 범위 안 값을 돌려줌(gsettings가 없는 CI도 통과), 상수 `body_px`가 OS별 값(`cfg` 시험). 실제 OS 값은 사람이 확인(`cargo test -p platform -- --ignored os_display_hints_print`는 값을 출력만 하는 무시 시험).
- 완료 기준: `cargo test -p platform`(두 OS), `cargo clippy --workspace --all-targets -- -D warnings`.
- Windows/Ubuntu: Windows `reg.exe` 출력의 값 이름·형식은 언어와 무관하다(머리글만 현지화). Ubuntu에서 GNOME이 아닌 데스크톱(KDE 등)은 `gsettings`가 없거나 키가 없어 기본값(100, `None`). 보안: 절대 경로만 실행(PATH 사용 안 함), 인자는 상수, 출력은 숫자 파서로만 쓰고 로그·화면에 그대로 내보내지 않는다. 새 의존성 없음.

P5. core: 설정 칸 넓히기와 `app_display_hints` (`core: extend app settings and add display hints command`)
- 목표: 글자 크기·언어·화면 모드·용어 설명 보기·줄 바꿈을 저장하고 OS 읽기 값을 화면에 준다(C3, C2).
- 바꿀 파일: `crates/core/src/settings.rs`(`AppSettings`에 칸 다섯, 수동 `Default`(`term_help`·`diff_wrap` 참), `SettingsFile`에 `#[serde(default)]` 칸, 쓰기·읽기·`set`의 패치 적용), `crates/core/src/view/settings.rs`(`LanguageSetting`·`FontSizeSetting`·`ThemeSetting`, `AppSettingsView`·`AppSettingsPatch` 칸, 신규 `DisplayHintsView { body_px, text_scale_percent, dark }`), `view/mod.rs`(내보내기), `crates/core/src/bootstrap.rs`(`Core::display_hints()` = `platform::display_hints()` 변환), `src-tauri/src/commands.rs`·`bindings.rs`·`build.rs`(`COMMANDS`)·`capabilities/default.json`(`app_display_hints`), `src/lib/api.ts`(`displayHints`), `src/lib/queries.ts`(`useDisplayHints`: `refetchOnWindowFocus: true`, `staleTime: 5000`), 생성물(`bindings.ts`), `docs/architecture.md`(명령 목록에 `app_display_hints`, 설정 칸 설명).
- 내용: `settings.json` 예 `{"v":1,"allow_published_rewrite":false,"language":"system","font_size":"normal","theme":"system","term_help":true,"diff_wrap":true}`. 옛 파일 `{"v":1,"allow_published_rewrite":true}`는 새 칸이 기본값으로 읽힌다. 틀린 값·알 수 없는 칸·4KB 초과·링크는 지금처럼 파일 전체 기본값. 패치는 `Option` 칸이고 값이 같으면 쓰지 않는다. `ExecCtx.settings`의 `AppSettings`는 `Copy`로 유지(실행기는 새 칸을 읽지 않는다). 화면 문구·크기 계산은 core가 하지 않는다(값만 준다).
- 테스트: 기본값 표(다섯 칸), 새 칸을 쓰고 다시 열어 유지, 옛 v1 파일 호환, 틀린 열거값·알 수 없는 칸은 기본값, 파일 쓰기 실패 시 메모리 값 유지, 한 칸만 패치해도 다른 칸이 그대로, 쓴 파일 문자열 모양(고정), `tests/settings.rs`·`tests/security_config.rs`(새 명령 네 곳), `registry-glossary`·바인딩 최신(`gen-bindings` 뒤 diff 없음).
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신, `pnpm typecheck`.
- Windows/Ubuntu: 앱 데이터 경로는 기존 `platform::app_data_dir`. 보안: 패치 칸은 고정 열거형·bool뿐(경로·주소·글자 없음), 받은 저장소가 바꿀 수 없다. `app_display_hints`는 저장소를 받지 않는 읽기 전용이고 인자가 없다.

P6. 글자 크기: 적용·설정·단축키 (`features: font size setting, os scale and shortcuts`)
- 목표: 시작할 때 OS 기본과 배율로 루트 글자 크기를 정하고, 설정과 Ctrl+=·−·0으로 바꾼다(C1, C7, C15의 구역 틀).
- 바꿀 파일: 신규 `src/lib/fontScale.ts`(`FONT_STEPS = ["small","normal","large","xlarge"]`, `STEP_FACTOR`(13/14, 1, 16/14, 18/14), `computeUiScale(hints, step) = clamp(hints.bodyPx/14 × hints.textScalePercent/100 × factor, 0.9, 2)`, `stepUp/stepDown`), 신규 `src/lib/display.ts`(`applyDisplay(settings, hints)`: 루트 `style.fontSize = 16×scale px`, `useUiStore.setUiScale`, `lang`·`data-theme`는 P10·P11에서 이어 넣을 자리, 순수 입력→DOM 한 함수), 신규 `src/lib/preload.ts`(`preloadDisplay(client)`: `api.settings()`·`api.displayHints()`를 0.8초 제한으로 읽어 쿼리 캐시에 넣고 `applyDisplay`, 실패하면 기본값), 신규 `src/lib/useDisplaySync.ts`(설정·힌트 쿼리가 바뀌면 `applyDisplay`), `src/main.tsx`(`void start()`: 쿼리 클라이언트 만들고 `preloadDisplay` 뒤 `createRoot().render`), `src/App.tsx`(`App({ client })`, `useDisplaySync()`, 글자 크기 단축키 세 개 `useShortcut`), `src/lib/shortcuts.ts`(C7: 세 항목 추가, `RESERVED`에서 `Equal`·`Minus`·`Digit0` Ctrl 조합 제거)·`shortcuts.test.ts`(겹침 시험이 새 키를 포함), `src/state/uiStore.ts`(`fontStep`), 신규 `src/ui/Choice.tsx`·`Choice.module.css`(+시험), `src/features/settings/SettingsDialog.tsx`(구역 틀: `<section>`에 제목 "일반"·"GitHub", 구역 사이 `--border` 선), 신규 `src/features/settings/GeneralSection.tsx`(글자 크기 행: `Choice` 네 칸, 고르면 `api.setSettings({ fontSize })` 뒤 캐시 갱신), `SettingsDialog.module.css`, `src/messages/ko.ts`·`en.ts`, `src-tauri/tauri.conf.json`(`"zoomHotkeysEnabled": false`), `src-tauri/tests/security_config.rs`(그 값과 capabilities에 웹뷰 확대 권한이 없음을 확인), 시험 파일들(각 화면 시험이 `vi.mock`한 `api`에는 `settings`·`displayHints`가 없어 쿼리가 실패하는데, `useDisplaySync`와 `useAppSettings`를 쓰는 곳은 실패·미응답을 기본값으로 처리해야 하므로 기존 시험을 고치지 않고 통과해야 한다. 그렇지 않은 곳만 시험의 `api` 모의에 두 함수를 더한다).
- 내용: 앱 시작 순서: `main.tsx` → `preloadDisplay`(설정·힌트 읽기, `applyDisplay`로 루트 글자 크기와 `uiScale`을 첫 화면 전에 정함) → 그리기. 이후 `useDisplaySync`가 설정·힌트(창 포커스 때 새로 읽음)를 따라 `applyDisplay`. 단축키: `fontLarger/fontSmaller/fontReset` 핸들러는 `useUiStore.getState().fontStep`에서 다음 단계를 계산해 먼저 스토어에 넣고(빨리 눌러도 따라감) `api.setSettings({ fontSize })`를 보낸다(끝 단계에서는 호출하지 않음). 입력칸 안에서도 동작(`inTextField: "app"`). `Ctrl+=`가 버전 기록 화면의 `+`(확대)와 겹치지 않음은 `ctrl` 값이 다르기 때문이고 시험으로 고정한다. 설정의 글자 크기 행은 단계 이름("작게·보통·크게·아주 크게")만 보이고 설명 문장은 없다. 설정이 저장되지 않아도(쓰기 실패) 이번 실행 동안은 바뀐 값이 유지된다(core가 메모리로 내려간다).
- 테스트: `fontScale`(Windows 힌트 {14,100} 보통 → 1, Ubuntu {15,100} 보통 → 15/14, 배율 125% 곱, 0.9~2.0 자르기, 모든 단계×힌트 {14,100}에서 `16 × scale × 0.8125 ≥ 12`), `stepUp/stepDown` 끝, `applyDisplay`가 루트 `style.fontSize`를 `16×scale px`로 쓰고 `uiScale`을 넣음, 단축키(Ctrl+`Equal`·Ctrl+Shift+`Equal`·Ctrl+`NumpadAdd`가 큰 쪽, Ctrl+`Minus`·`NumpadSubtract`가 작은 쪽, Ctrl+`Digit0`이 보통, 입력칸 안에서도 동작, `xlarge`에서 더 키워도 `setSettings` 호출 없음, 조합 중이면 무시), `Choice`(radiogroup 이름·`aria-checked`·방향키), `GeneralSection`(고르면 `setSettings` 입력), `main` 시작 순서(`preloadDisplay`가 실패해도 렌더), `security_config`(`zoomHotkeysEnabled` false, 확대 권한 없음), `SettingsDialog` 기존 시험, `shortcuts.test`(겹침·`RESERVED`에서 뺀 키가 표에 있음).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`, `cargo test --workspace`(security_config). 눈으로: 개발 앱에서 설정 → 글자 크기를 바꾸면 홈·노선도·설정이 바로 같이 커지고 다시 켜도 유지, Ctrl+=·−·0이 앱에 온다.
- Windows/Ubuntu: Ctrl+=가 WebView2·WebKitGTK에서 앱에 오는지·웹뷰 확대가 같이 일어나지 않는지는 손 확인("사람이 할 일"). 한국어 자판에서도 `code`가 같다. 고DPI Windows에서 소수 루트 크기. 보안: 설정 패치는 고정 열거형.

P7. 상단 바: 테두리 맞춤과 넘침 (`screens: top bar buttons without borders and wrap at large sizes`)
- 목표: 브랜치 단추와 프로젝트 이름을 같은 모양(테두리 없음, 호버 때 바탕)으로 맞추고 800px 너비·"아주 크게"에서 상단 바가 넘치지 않게 한다(D-138, polish 5장).
- 바꿀 파일: `src/features/branches/BranchList.module.css`(`.trigger`: 테두리·`--surface` 바탕 삭제, `border: none; background: none;`, 호버·열림(`[data-state="open"]`)·포커스를 `ProjectMenu`와 같게), `src/screens/Workspace/ProjectMenu.module.css`(필요하면 같은 값으로 정리), `src/screens/Workspace/TopBar.module.css`(`.bar { flex-wrap: wrap; row-gap: var(--space-1); }`, 이름·브랜치 이름 `min-width: 0` + `max-width: 20rem` 말줄임, `.spacer`는 줄이 바뀌어도 오른쪽 묶음이 오른쪽에 붙게), `TopBar.tsx`(오른쪽 컨트롤 묶음을 `<div className={styles.right}>`로 감싸 한 덩어리로 줄 바꿈), 시험.
- 내용: 동기화 단추(`--border-strong` 테두리 버튼)는 그대로(D-045). 폭이 모자라면 오른쪽 덩어리가 통째로 아래 줄로 내려오고 그래도 모자라면 그 덩어리 안에서 줄이 바뀐다. 가로 스크롤이 생기지 않는다.
- 테스트: `TopBar.module.css`·`BranchList.module.css`를 읽어 `.trigger`에 `border: none`·`.bar`에 `flex-wrap: wrap`이 있음을 확인하는 CSS 문자열 시험(jsdom은 레이아웃을 못 재므로), `TopBar.test`(기존 + 오른쪽 덩어리가 하나의 요소 안), 접근성(이름·`aria-pressed` 그대로). 800px·"아주 크게"는 P9 스크린샷과 사람 손 확인.
- 완료 기준: `pnpm test`·`pnpm lint`. 눈으로: 개발 앱 창을 800px로 줄이고 "아주 크게"에서 상단 바가 가로로 넘치지 않는다.
- Windows/Ubuntu: 웹뷰 `flex-wrap` 동일. 보안: 해당 없음.

### C. 밑줄

P8. 밑줄 줄이기와 용어 설명 보기 (`ui: term tooltip without underline and a switch to turn it off`)
- 목표: `<Term>`은 표시 없이 툴팁만(600ms), 버튼·상단 바 글자에 표시가 없고, 밑줄은 링크 모양 한 파일에서만 호버 때(린트가 보장), 설정 "용어 설명 보기"로 툴팁을 끈다(D-140, C9).
- 바꿀 파일: `src/ui/Term.module.css`(`.term`에서 `border-bottom`·`cursor: help` 삭제, 포커스 테두리는 유지), `src/ui/Term.tsx`(`TermTooltip`이 `useUiStore.termHelp`가 거짓이면 `children`만 돌려줌, `Term`은 그때 `tabIndex` 없음, 지연 `TERM_TOOLTIP_DELAY_MS = 600`을 `Tooltip`에 넘김), `src/ui/Tooltip.tsx`(`delay?: number` 인자, 기본 `TOOLTIP_DELAY_MS`), `src/state/uiStore.ts`(`termHelp: boolean` 기본 참, `setTermHelp`), `src/lib/display.ts`(`applyDisplay`가 `termHelp`도 넣음), `src/features/settings/GeneralSection.tsx`(행 "용어 설명 보기": `Choice` 켬·끔 또는 체크 상자, 값은 `settings_set`), `src/screens/History/History.module.css`(`.scopeButton` 밑줄 삭제, 고른 것 600·`--text`, 호버 `--surface-hover` 바탕·`--radius-control`), `src/ui/Toast.module.css`(`.action:hover` → `background: var(--primary-hover)`), 신규 `src/ui/linkStyle.module.css`(`.link` 평소 밑줄 없음, 호버 `text-decoration: underline`, 포커스는 전역 규칙), `src/ui/ErrorNotice.tsx`·`ErrorNotice.module.css`(`<summary>`에 `.link`), `.stylelintrc.json`(`declaration-property-value-disallowed-list`: `text-decoration`·`text-decoration-line` 값 `/underline/`, `border`·`border-bottom`·`border-top`·`border-left`·`border-right` 값 `/dotted/`; `overrides`로 `src/ui/linkStyle.module.css`만 해제), `scripts/stylelint-rules.test.mjs`(규칙 시험 추가), `src/messages`(`settings.termHelp`), 시험 파일들.
- 내용: 글자 버튼(상단 바·히스토리 범위·토스트 단추)의 호버 알림은 바탕 변화로만. `Term`을 쓰는 곳이 많아도 바꿀 호출부는 없다(부품 안에서 처리). `Tooltip`을 직접 쓰는 아이콘 단추 등은 400ms 그대로.
- 테스트: `Term.test`(밑줄·점선 CSS 없음 — 모듈에 `border`·`dotted` 없음, 호버 600ms 뒤에 툴팁이 뜨고 400ms엔 안 뜸(가짜 타이머), `termHelp` 거짓이면 툴팁·`tabIndex` 없음, git 이름으로 보기 설정과 함께 동작), `GeneralSection`(켬·끔이 `setSettings`), `History` 시험(범위 단추 `aria-pressed`는 그대로), stylelint 규칙 시험(`text-decoration: underline`·`border-bottom: 1px dotted var(--x)`는 오류, `linkStyle.module.css`는 통과), `Toast`의 호버 스타일은 CSS 문자열 시험. `rg "underline|dotted" src --glob "*.css"` 결과가 `linkStyle.module.css`와 `tokens.css` 주석뿐.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 홈·버전 기록·설정 어디에도 점선·상시 밑줄이 없고 "용어 설명 보기"를 끄면 툴팁이 뜨지 않는다.
- Windows/Ubuntu: 차이 없음(CSS). 보안: 해당 없음.

### D. 스크린샷

P9. Ubuntu 스크린샷 (`docs: add ubuntu screenshots after font size and underline changes`)
- 목표: 사용자가 글자 크기와 밑줄을 눈으로 확인할 수 있게 개발 앱을 띄워 홈·버전 기록·브랜치 목록·설정 그림을 `docs/research/img/polish/`에 넣는다. 이 단계 앞의 P2~P8만 반영된 모습이다(노선도 "…"·파일 줄 등은 아직 그대로).
- 바꿀 파일: 신규 `docs/research/img/polish/p9-home.png`, `p9-home-xlarge.png`, `p9-home-800-xlarge.png`(창을 800px로 줄일 수 있을 때만), `p9-log.png`(역 하나를 골라 역 메뉴가 열린 모습, r-log.png와 같은 장면), `p9-branches.png`(브랜치 목록 팝오버), `p9-settings.png`(설정, 일반·GitHub 구역과 글자 크기 줄), `docs/progress.md`(사람이 볼 그림 목록과 "크기·밑줄이 마음에 들지 않으면 `tokens.css`·`fontScale.ts`의 값만 고친다" 안내).
- 내용: 코드는 고치지 않는다. 준비는 scratchpad에서만: `. <scratchpad>/env.sh` 뒤 `HIGIT_DATA_DIR=<scratchpad>/polish-data pnpm tauri dev`(앱 데이터가 scratchpad라 사용자 설정·최근 목록에 닿지 않음), 데모 저장소는 scratchpad의 것(없으면 scratchpad 스크립트로 만든다: 이름 있는 `main` + 기능 브랜치가 두 칸 앞서고 고친 파일 3개 — 그중 긴 경로 `docs/plans/010-settings-github-more.md`와 `home.zip`(작은 더미 압축), 긴 줄을 바꾼 `.md`, 접힌 앞쪽이 생길 만큼 30개 이상의 버전, `.env` 없음). 그림은 scratchpad의 `shot.sh`를 원본 해상도(축소 없이)로 저장하도록 복사해 쓴다(`DISPLAY=:1`). 키는 `xkey.py`(Ctrl+= 두 번으로 "아주 크게", `ctrl+h` 버전 기록, `ctrl+b` 브랜치 목록). 끝나면 `stop-dev.sh`로 개발 서버를 끄고 scratchpad 임시 데이터는 둔다(저장소에는 넣지 않음). 다크·언어는 아직 설정이 없으므로 찍지 않는다(P20이 다시 찍는다).
- 테스트: 그림 파일이 열리고 비어 있지 않다(눈으로). 새 파일이 `docs/research/img/polish/` 밖에 없다. `home.zip`(저장소 루트 사용자 파일)은 커밋하지 않는다.
- 완료 기준: 그림 6장 안팎이 저장소에 있고 `docs/progress.md`에 파일 이름과 무엇을 보면 되는지 한 줄씩 있다.
- Windows/Ubuntu: Ubuntu(WebKitGTK) 한 환경의 그림이다. Windows는 사람이 할 일. 보안: 데모 저장소에 비밀·개인 정보가 없다.

### E. 언어·화면 모드

P10. 언어 설정과 시각 표시 (`features: language setting and time display rules`)
- 목표: 설정 "언어"로 바로 바꾸고, 날짜·상대 시각이 앱 언어와 한 규칙을 따르며, English 용어 툴팁이 같은 이름 줄을 뺀다(D-139, C4, C5).
- 바꿀 파일: 신규 `src/lib/locale.ts`(`resolveLocale(setting, languages)`, `systemLanguages()` = `navigator.languages`), `src/lib/display.ts`(`applyDisplay`가 `useUiStore.setLocale`·`document.documentElement.lang`을 넣음), `src/features/settings/GeneralSection.tsx`(행 "언어": 시스템 따라가기·한국어·English, 언어 이름은 자기 말로 고정 글자), `src/lib/time.ts`(`formatTime`·`formatExact`·`formatClock`, `formatRelative`를 `formatTime`으로 대체, `formatDate`는 날짜 묶음 키로 유지), 신규 `src/ui/Time.tsx`(`<time dateTime title>`)·시험, 호출부 열한 곳(`screens/Start/ProjectRow.tsx`·`LastProject.tsx`, `features/sync/SyncButton.tsx`, `features/stash/CompareFooter.tsx`, `features/history/RouteList.tsx`, `features/branches/BranchList.tsx`, `features/pr/PrRow.tsx`, `features/rewrite/Card.tsx`, `features/history/menu/dialogs/FilesDialog.tsx`, `features/conflict/HunkPanel.tsx`, `screens/Activity/Activity.tsx`)을 `<Time>`·`formatTime`으로, `src/ui/Term.tsx`(English에서 `friendly`와 `gitName`이 대소문자 무시로 같으면 툴팁의 이름 줄 생략), `eslint.config.js`(`no-restricted-properties`: `Intl.DateTimeFormat`·`Intl.RelativeTimeFormat`을 `src/lib/time.ts` 밖에서 금지)·`scripts/eslint-rules.test.mjs`(규칙 시험), 신규 `src/messages/parity.test.ts`, 시험 파일들.
- 내용: `resolveLocale("system", ["ko-KR","en-US"]) = "ko"`, `("system", ["en-US","ko"]) = "en"`(첫 항목만 본다), 빈 목록·알 수 없는 언어는 English(D-139 "그 밖은 English"). 앱 시작 때는 P6의 `preloadDisplay`가 이미 `applyDisplay`를 부르므로 깜빡임 없이 바른 언어로 그린다. 버튼·화면 문구는 `useMessages()`가 이미 `locale`을 따르므로 호출부 변경이 없다. `formatTime`(C5): 1분 안·미래 "지금", 1시간 안 "N분 전"(`Intl.RelativeTimeFormat`), 같은 날 "오후 3:12"(`Intl.DateTimeFormat` 시·분·12시간), 같은 해 "10월 7일", 그 전 "2025년 10월 7일". `Time`은 정확한 시각을 `title`로(툴팁 부품이 아닌 네이티브 title은 시각이 짧은 글자라 허용: 이 부품 하나에만). 버전 이름 제안은 `m.suggest`가 이미 언어를 따르므로 English 문구가 영어 문법인지만 확인(`suggest.test`에 en 표).
- 테스트: `locale.test`(표), `time.test`(고정 시각 `new Date(y, m, d, h, mi)`로 시간대와 무관하게: 30초·5분·같은 날 오후 3시 12분·같은 해·작년·미래 시계 오차, ko·en 둘 다, 날짜가 바뀌는 자정 경계), `Time`(title이 정확한 시각), `parity.test`(`ko`와 `en`의 키 집합·값 종류(문자열·함수)·중첩 구조가 같고 빈 문자열이 없다, 함수는 샘플 인자로 호출해 문자열), `Term.test`(en: "Push"와 "push"는 이름 줄 없음, "Commit"과 "commit"도 없음, 다르면 있음, ko는 항상 있음), `GeneralSection`(고르면 `setSettings({ language })`, 즉시 `useMessages`가 영어로, `document.documentElement.lang`), 호출부 시험이 새 글자(예 `오후 3:12`)로 통과, ESLint 규칙 시험, `terms-in-messages`·`copy-rules`.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: English로 바꾸면 홈·버전 기록·설정·브랜치 목록·충돌 화면에 한국어가 남지 않는다.
- Windows/Ubuntu: `navigator.languages`가 WebView2는 Windows 표시 언어 목록, WebKitGTK는 `LANG`·`LANGUAGE`를 따르는지 손 확인(Ubuntu를 영어 로케일로 띄워 "시스템 따라가기"가 English가 되는지). `Intl`의 12시간 표기는 두 웹뷰가 같다. 보안: 해당 없음(시각 글자는 숫자로 만든다).

P11. 화면 모드 (`features: color scheme setting`)
- 목표: 설정 "화면 모드"(시스템 따라가기·라이트·다크)가 바로 적용되고 저장된다(D-145, C6).
- 바꿀 파일: `src/lib/display.ts`(`applyDisplay`가 `data-theme`을 정함: `light`·`dark`면 그 값, `system`이면 `hints.dark`가 `true`/`false`면 그 값, `None`이면 속성을 지움), `src/features/settings/GeneralSection.tsx`(행 "화면 모드": `Choice` 세 칸), `src/messages`(이미 P6에서 더한 키 사용), 시험 파일.
- 내용: `tokens.css`는 바꾸지 않는다(이미 `:root[data-theme="light"|"dark"]`와 `prefers-color-scheme` 처리). 창 포커스를 얻을 때 `useDisplayHints`가 다시 읽으므로 OS 모드를 바꾸고 돌아오면 `system`이 따라간다.
- 테스트: `applyDisplay`(light·dark가 `data-theme`을 넣음, system + `dark: true`가 dark, system + `dark: null`이 속성 없음, system에서 light → system으로 바꾸면 속성이 사라짐), `GeneralSection`(고르면 `setSettings({ theme })`), 대비 시험(`contrast.test.ts`)은 그대로 통과.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 다크로 바꾸면 바로 어두워지고 start-normal-dark.png와 비슷하다.
- Windows/Ubuntu: Ubuntu 22.04의 WebKitGTK가 OS 다크를 못 볼 수 있어 `hints.dark`가 대신한다(손 확인: GNOME 설정에서 다크로 바꾸고 앱이 "시스템 따라가기"에서 따라가는지). Windows WebView2는 둘이 같다. 고대비 모드(`forced-colors`)는 그대로. 보안: 해당 없음.

### F. 노선도

P12. 노선도 배치: 접힌 앞쪽·5역·이름표 폭·"지금 여기" (`history: dotted fold, five stations, label-width spacing and now tag`)
- 목표: "…" 글자 대신 점선과 "이전 N개", 이름표 폭에 맞춘 간격, 폭을 다 쓰는 "지금 여기" 이름표를 장면에 만든다(D-141, C10). 화면 연결은 P13.
- 바꿀 파일: `src/features/history/layout/horizontal.ts`, `layout/scene.ts`(`fold: { x, y, count } | null`, 이름표 `kind`·`width`), `render/palette.ts`(팔레트 키 `~fold-line` → `var(--border-strong)`, 고대비 `CanvasText`), `render/SvgRenderer.tsx`(점선 `strokeDasharray`는 장면 선의 `dashed`, `fold`가 있고 `interactive`가 아니면 SVG 글자로 "이전 N개"를 그림), `layout/horizontal.test.tsx`, `src/messages/ko.ts`·`en.ts`(`earlierVersions(n)`), `HorizontalLabels`에 `earlier(n)`·`interactiveFold`.
- 내용: ① `home` 밀도는 `MAX_STATIONS = 5`(앞쪽을 더 잘라 `folded`에 더함, `currentMore + 잘린 수`), `preview`·`mini`는 지금 규칙(너비에 맞는 만큼). ② `STEP`·`MIN_STEP` 고정값을 없애고 간격 = `min(남은 폭/(n−1), max(96, 가장 긴 이름표 폭(최대 200) + 16))`(C10). ③ "지금 여기": 장면 너비 − 72 위치(합치기 제안 블록 앞)에 `SceneTag` 이름표(폭 = 글자 폭 + 16, 높이 18)를 두고 마지막 역에서 거기까지 기존 `dashed` 선이 이어진다. 고친 파일 숫자 글자는 그 아래(`labels.changed`). upstream 이름표와 겹치면 upstream을 한 칸 위로. ④ 접힘: `folded > 0`이면 "…" 글자를 지우고 선의 왼쪽 끝(한 줄 `LEFT`, 두 줄 갈라진 곡선이 끝난 곳 `forkX + curve`)에서 24단위 점선(`~fold-line`, `dashed`)과 `scene.fold`를 만든다. "이전 N개" 글자는 점선 시작 x·역 이름 줄 y에 `anchor: "start"`. 첫 역의 x는 `max(기존 값, 접힘 글자 끝 + 첫 이름표 반폭 + 8)`로 밀어 이름표와 "이전 N개"가 겹치지 않게 한다. ⑤ `labels.interactiveFold`가 참이면(홈) 장면에 `fold` 좌표만 넣고 SVG 글자는 만들지 않는다. 시작 화면은 거짓(SVG 글자만).
- 테스트: 한 줄·두 줄에서 `folded`가 있을 때 "…" 글자가 장면에 없고 점선(`~fold-line`)과 `fold.count`가 있다, 홈에서 역이 6개 이상 들어와도 그려지는 역은 5개이고 `fold.count`가 늘어난다, 이름이 짧으면 간격이 `max(96, …)`·넉넉하면 이름표가 잘리지 않고 길면 `fitText`로 잘리며 인접 이름표 폭의 합이 간격 − 8 이하(겹침 없음), "지금 여기" 이름표 x가 `width − 72`이고 폭에 비례(`textWidth + 16`), 이름표 높이 18, "이전 N개" 글자와 첫 이름표가 겹치지 않음(`textWidth`로 계산), `interactiveFold`가 `fold`만 만들고 SVG 글자는 없음, `preview`는 접힘이 있으면 SVG 글자, `mini`는 그대로(`mini` 시험 변화 없음), 좁은 폭(320)에서도 음수 간격이 없다.
- 완료 기준: `pnpm test`·`pnpm typecheck`·`pnpm lint`.
- Windows/Ubuntu: 글자 폭 추정(`textWidth`)이 번들 글꼴(Pretendard) 기준이라 두 웹뷰에서 같다. 보안: 이름표 글자는 React 글자로만(D-037).

P13. 홈 노선도 화면 연결 (`screens: home route map fold button, tooltips and full width`)
- 목표: 홈 노선도가 폭을 다 쓰고, "이전 N개" 단추가 버전 기록을 열고, 역에 호버·포커스하면 전체 이름이 뜬다(D-141).
- 바꿀 파일: `src/screens/Workspace/HomeMap.tsx`·`HomeMap.module.css`, `src/features/history/StationTips.tsx`(신규: 장면의 역 좌표 위에 겹친 `<span tabIndex=0 role="img" aria-label=전체 이름>`을 `Tooltip`으로 감쌈, 좌표는 `× scale`), `src/features/history/RoutePreview.tsx`(시작 화면은 SVG 글자만, 툴팁 없음 — 줄 전체가 이미 단추), `src/messages`, 시험 파일.
- 내용: `HomeMap`이 `layoutHorizontal(..., { interactiveFold: true, earlier: m.earlierVersions })`를 부르고 `scene.fold`가 있으면 그 좌표(`× scale`)에 글자 버튼 "이전 N개"(`--text-muted` 12~13px, 호버 `--surface-hover`)를 겹친다. 누르면 `setView("history")`. 역 툴팁은 5개 역만(전체 `summary`). 노선도 폭은 `.route { flex: 1 }`가 이미 합치기 제안 블록 앞까지이므로 배치 너비를 그 상자 너비로 주는 것으로 충분하다(P12가 폭을 쓰도록 바꿈). Tab 순서는 노선도 안에서 왼쪽 → 오른쪽(이전 N개 → 역들)이고 DOM 순서가 그 순서.
- 테스트: `HomeMap.test`(접힌 버전이 있으면 "이전 N개" 단추가 있고 누르면 `view`가 `history`, 없으면 단추 없음, 역에 포커스하면 전체 이름 툴팁(가짜 타이머 600ms는 Term 용, 이 툴팁은 400ms), 긴 이름이 라벨에선 잘리고 툴팁엔 전체, scale 1.5에서 단추 위치가 1.5배), 시작 화면 시험 변화 없음(노선도 SVG 글자 "이전 N개" 존재).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 폭이 넓은 창에서 노선도가 오른쪽까지 가고 "지금 여기"가 합치기 제안 앞에 있다.
- Windows/Ubuntu: 겹친 HTML 요소의 위치는 두 웹뷰에서 같다. 고DPI·소수 배율에서 겹침 오차는 손 확인. 보안: 역 이름은 `aria-label`·툴팁에 React 글자로.

### G. 비교 칸

P14. 비교 질의: 전체 문맥 (`git+core: optional full-context diff`)
- 목표: 접힌 줄 "펼치기"가 쓸 수 있게 비교를 전체 문맥으로 다시 읽을 수 있다(C11). 화면은 P15.
- 바꿀 파일: `crates/git/src/cli/diff.rs`(`-U3` 상수를 문맥 줄 수 인자로: 기본 3, 전체는 큰 수 `FULL_CONTEXT = 1_000_000`), `crates/git/src/types.rs`·`backend.rs`(요청에 `context: u32` 또는 `full_context: bool`, 가짜 구현 둘 `NoGit`·`FakeGit` 갱신), `crates/core/src/view/diff.rs`(`DiffRequest.full_context: bool`, `#[serde(default)]`), `crates/core/src/query.rs`(변환), `src/lib/queries.ts`(`useDiff` 키에 `fullContext`)·`api.ts`, 생성물, 시험.
- 내용: `-U<큰 수>`는 기존 `truncated` 상한(`001-10b` 큰 파일 비교 상한) 안에서만 문맥을 늘린다. 경로 검사·링크·이진 처리는 지금과 같다. `commit`이 있는 비교(이때 파일 보기)도 같은 인자를 받는다.
- 테스트(실제 git, 임시 저장소): `full_context`가 참이면 묶음이 하나로 합쳐지고 모든 줄이 나오며 거짓이면 지금과 같다, 큰 파일은 `truncated`, 이진·링크·새 파일(`untracked`)은 변화 없음, `git-min`에서 `-U1000000`이 되는지, `DiffRequest` JSON에 필드가 없어도 기본 거짓(옛 요청 호환), `core` 시험(가짜 백엔드가 인자를 받음), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm typecheck`, 바인딩 최신.
- Windows/Ubuntu: `-U` 인자는 같고 CRLF 줄은 파서가 끝 `\r`을 뺀다(기존). 보안: 읽기 전용, 경로 검사 그대로, 인자는 정수 상수(사용자 글자 없음).

P15. 비교 칸: 접힌 줄·펼치기·줄 바꿈 (`diff: unchanged-lines fold, expand and line wrap toggle`)
- 목표: "…"만 보이던 접힌 줄을 "변경 없는 N줄 · 펼치기"로 쓰고 누르면 펼치며, 머리의 "줄 바꿈"(기본 켬)으로 긴 줄을 본다(D-141, D-144, C11).
- 바꿀 파일: `src/features/diff/model.ts`(`toDiffRows(diff, expanded?)`: 앞 묶음과 다음 묶음 사이·첫 묶음 앞 생략 줄 수를 줄 번호로 계산해 `fold` 행(`{ type: "fold", key, count, from, to }`)으로, 펼친 구간은 전체 문맥에서 `newLine`이 `from..to`인 문맥 줄로 대체, `section`은 fold 행 오른쪽 글자로), `src/features/diff/DiffView.tsx`(fold 행 단추 "변경 없는 N줄 · 펼치기"(`--surface-sunken` 바탕·글자 `--text-muted`·호버 `--surface-hover`, 키보드 Enter), 펼칠 때 `useDiff(repo, { ...request, fullContext: true })`를 켬, 머리 오른쪽 글자 버튼 "줄 바꿈"(`aria-pressed`)), `DiffView.module.css`(`.wrapOn .code { white-space: pre-wrap; overflow-wrap: anywhere }`, 줄 번호·기호 칸 `vertical-align: top`, 표 글자 `--font-size-code`, 접힌 줄 스타일), `src/state/uiStore.ts`(`diffWrap`)·`src/lib/display.ts`(`applyDisplay`가 넣음)·`src/features/diff`(켜고 끌 때 `setSettings({ diffWrap })`), `src/messages`(`diffFold(n)`·`wrapLines`), 시험 파일.
- 내용: `row.section || "…"`을 없앤다. 앞쪽 묶음 사이 줄 수 0이면 fold 행을 만들지 않는다(묶음이 붙은 경우). 마지막 묶음 뒤 fold는 전체 문맥을 읽은 뒤에만(C11). 전체 문맥 읽기가 `truncated`이거나 실패하면 fold 행의 "펼치기" 대신 `m.diffTruncated`·오류(`ErrorNotice`). 펼친 상태는 선택한 파일이 바뀌면 버린다(`DiffView`가 `key={path}`로 다시 만들어짐). "줄 바꿈"을 끄면 지금처럼 가로 스크롤(`overflow: auto`)이고 켜면 가로 스크롤이 생기지 않는다.
- 테스트: `model.test`(묶음 사이 N줄 계산: 첫 묶음 앞 12줄·사이 8줄·붙은 묶음은 fold 없음·`fullContext` 응답에서 구간만 펼침, 맨 끝 fold는 펼친 뒤에만), `DiffView.test`(fold 행 글자 "변경 없는 12줄 · 펼치기", 누르면 `diff` 질의가 `fullContext: true`로 한 번 더 가고 그 구간 줄이 나옴, `section`이 오른쪽에 보임, 줄 바꿈 단추가 기본 `aria-pressed=true`이고 누르면 `setSettings({ diffWrap: false })`·표에 `wrapOn` 클래스 없음, 긴 줄 시험), 기존 DiffView 시험 통과(기본 켬에 맞춰 갱신).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 홈에서 r-home.png처럼 "변경 없는 N줄 · 펼치기"가 보이고 긴 줄이 아래로 이어진다.
- Windows/Ubuntu: `overflow-wrap: anywhere`는 두 웹뷰가 같다. 한글·CRLF 줄. 보안: 줄 글자는 React 글자로만, 펼친 줄도 같은 길.

### H. 고친 파일 줄

P16. 파일 줄 두 줄과 "고침" (`screens: two-line file rows and the word edited`)
- 목표: 고친 파일 줄이 위 파일 이름·아래 폴더로 보이고 상태 낱말이 "고침"이다(D-143, polish 5장).
- 바꿀 파일: `src/screens/Workspace/ChangedFiles.tsx`(줄 구조: `[체크][이름·폴더 두 줄][상태]`), `ChangedFiles.module.css`(두 줄 행 높이 `--row-height-two`, 이름 `--font-size-md`·코드 글꼴, 폴더 `--font-size-sm`·`--text-muted`, 상태는 오른쪽 `--font-size-sm` 600), 신규 `src/lib/path.ts`(`splitPath("docs/plans/010.md") → { name: "010.md", dir: "docs/plans" }`, 최상위 파일은 dir 빈 문자열), `src/ui/ListRow.module.css`(두 줄 행), `src/messages/ko.ts`(`changeKind.modified: "고침"`), `docs/design.md` 색 표 줄은 P1에서, 시험 파일.
- 내용: 이름은 끝 말줄임, 폴더는 앞 말줄임(P19의 `Truncate`가 오기 전에는 CSS `text-overflow: ellipsis` + `title`로 두고 P19가 `Truncate`로 바꾼다). 폴더가 없으면 아래 줄은 비운다(높이는 두 줄로 고정해 행 높이가 들쭉날쭉하지 않게). 이름 바꿈은 아래 줄에 `m.renamedFrom(origPath)`를 `--text-muted`로. 상태 글자 색은 지금(`--added`·`--modified`·`--deleted`)과 같고 글자("새 파일"·"고침"·"삭제")로도 알린다(design.md "상태 표시": 색만으로 알리지 않음). 경로 구분은 git의 `/`(Windows도 같음).
- 테스트: `path.test`(최상위·깊은 경로·한글·공백·끝 `/` 없음), `ChangedFiles.test`(이름이 위·폴더가 아래, 상태 "고침", 이름 바꿈 표시, 선택·체크·키보드 ↑↓ 기존 통과), `Start`·`suggest` 시험의 "수정" 문구 갱신(`rg "수정"` 확인: `ko.ts` 한 곳).
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: r-home.png의 줄 모양.
- Windows/Ubuntu: 폴더 구분 `/`는 git이 줄 때 이미 같다. 한글 폴더. 보안: 경로는 React 글자로만.

P17. 체크 모양: 넣은 파일 0개면 모두 켠 모양 (`screens: show all files checked when none is staged`)
- 목표: 넣은 파일이 0개일 때 모든 체크가 켜진 모양이고 하나를 끄면 나머지를 넣는다(D-142, C12). index는 저절로 바뀌지 않는다(D-062).
- 바꿀 파일: `src/screens/Workspace/ChangedFiles.tsx`(`virtualAll`, 체크 `checked={file.staged || virtualAll}`, 머리 체크 `all || virtualAll`, 행 체크·머리 체크 핸들러), 필요하면 `src/features/stage/useStageMutation.ts`(변경 없음), 시험 파일.
- 내용: `virtualAll = files.length > 0 && included === 0`. 행 체크 `onChange`: `virtualAll`이면 눌린 파일을 뺀 나머지 파일의 `path`로 `stage`(나머지가 없으면 그 파일을 `stage`), 아니면 지금처럼 토글. 머리 체크: `virtualAll`이면 모든 파일 `stage`(확정), 일부만 넣었으면 지금처럼. 가운데 줄(indeterminate)은 일부만 넣은 실제 상태에서만. 접근성 이름은 `<Term stage>`·`unstage`(지금처럼), `virtualAll`일 때 이름은 "넣기 해제"(눌러서 빼는 동작에 맞춤). 버전 저장·`suggestName`·`secret_check`는 이미 0개를 "모두"로 보므로 변경 없음.
- 테스트: `ChangedFiles.test`(0개 넣음 → 모든 체크 켜짐·머리 체크 켜짐, 하나 끄면 `stage`가 나머지 경로로 한 번 호출됨(unstage 호출 없음), 파일 하나뿐이면 그 파일을 `stage`, 머리 체크가 `virtualAll`에서 모두 `stage`, 일부 넣음 → 가운데 줄 머리 체크, 모두 넣음 → 모두 켜짐·머리 누르면 모두 `unstage`, 이름 바꿈 파일은 `unstage`에서만 원래 경로도), 저장 흐름 `NameBox.test` 변화 없음.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 고친 파일이 있는 홈에서 r-home.png처럼 모두 켜진 체크.
- Windows/Ubuntu: 차이 없음. 보안: 액션은 기존 `stage`·`unstage`(위험도 safe)뿐.

### I. 알림과 디테일

P18. 압축·실행 파일 알림 (`core: warn about archive and executable files before saving`)
- 목표: `home.zip` 같은 압축 파일과 실행 파일을 넣고 저장하면 비밀 파일 알림 줄과 같은 꼴로 한 번 알린다. 막지 않는다(C13).
- 바꿀 파일: `crates/core/src/secrets.rs`(`name_rule`에 확장자 목록은 별도 `kind_rule(path, head) -> Option<SecretRule>`로, `Archive`·`Executable`), `crates/core/src/view/secret.rs`(`SecretRule::Archive`·`Executable`), `crates/core/src/query.rs`(`secret_check`의 판정 순서: 비밀 이름 → 큰 파일 → 압축·실행 → 내용 규칙, 압축·실행 파일은 내용 규칙 검사에서 뺌), 생성물(`bindings.ts`), `src/messages/ko.ts`·`en.ts`(`secretRule.archive`·`executable`, `heavyTitle(n)`), `src/features/secrets/SecretNotice.tsx`(제목 선택: 비밀 규칙이 하나라도 있으면 `secretTitle`, 모두 압축·실행이면 `heavyTitle`), 시험 파일.
- 내용: 매직은 이미 읽는 앞 256KB의 첫 바이트 몇 개. 판정 표는 C13. 쉘·배치 스크립트와 실행 비트는 알리지 않는다. 알림 단추는 지금과 같다(빼기·제외할 파일에 넣기(모두 새 파일일 때)·그래도 저장). 같은 입력(서명)이면 다음 저장은 통과(`NameBox`의 기존 흐름). 결과에는 경로와 규칙 이름만(글자 일치 내용 없음).
- 테스트: `secrets.rs` 표(`home.zip`·`a.TAR.GZ`·`setup.exe`·`lib.dll`·`app.AppImage`·확장자 없는 ELF 머리·`MZ` 머리·`PK\x03\x04` 머리, `notes.md`·`run.sh`·`build.bat`·`zipper.ts`·`exe.txt`는 아님, `.pem`은 키 파일이 우선), `secret_check` 시험(임시 저장소: 넣은 파일·`stage_all`·큰 압축은 큰 파일 규칙 우선·압축 파일 안의 비밀 문자열은 내용 검사하지 않음·링크는 읽지 않음), `SecretNotice.test`(제목 두 가지·줄 문구·단추), `NameBox.test`(압축 파일만 있을 때 한 번 알리고 "그래도 저장"이면 저장), `copy-rules`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: 저장소에 `home.zip`을 두고 버전 저장 → 알림 줄.
- Windows/Ubuntu: 확장자는 소문자 비교(Windows 대소문자 무시와 맞음). 실행 비트는 보지 않으므로 파일 시스템 차이가 없다. 보안: 읽는 양은 기존 상한(200개·앞 256KB), 링크는 따라가지 않는다.

P19. 디테일: 로딩 지연·말줄임·포커스·Tab 순서·고급 구역 (`ui: delayed loading, truncation, unified focus ring and advanced section`)
- 목표: polish 6장의 작은 규칙 다섯과 설정 "고급" 구역(D-145, C14, C15)을 넣는다.
- 바꿀 파일: ① 포커스: `src/styles/tokens.css`(`--focus-width 2px`·`--focus-offset`·`--focus-offset-inset`), `src/styles/base.css`(전역 `:focus-visible`), 화면 CSS의 개별 `outline: 2px solid var(--accent)` 블록 정리(Term·ProjectMenu·TopBar·Start·ChangedFiles·BranchList·PullRequests·SyncButton·CompareFooter·History·Checkbox·Button·StationMenu 등, 예외: `Toast`), `scripts/stylelint-rules.test.mjs`(개별 `outline`에 `--accent`를 직접 쓰는 것을 막는 규칙은 두지 않고 `rg`로 확인). ② 로딩: 신규 `src/lib/useDelayed.ts`(`useDelayedFlag(pending, ms = 300)`)·시험, `Workspace.tsx`·`DiffView.tsx`·`History.tsx`·`Activity.tsx`·`Rewrite.tsx`의 `m.loading` 표시. ③ 말줄임: 신규 `src/ui/Truncate.tsx`·`Truncate.module.css`·시험, 적용 `ProjectMenu`(프로젝트 이름)·`TopBar`/`BranchList` 단추(브랜치 이름)·`ChangedFiles`(이름 끝·폴더 앞)·`RouteList`(버전 이름)·`PrRow`(제목). ④ Tab 순서: `src/screens/Workspace/Workspace.test.tsx`(순서 시험). ⑤ 스크롤 막대: `base.css`(그대로 `--scrollbar-size`)·비교 칸·목록의 `overflow: auto`가 같은 규칙을 받는지 확인(시험은 CSS 문자열). ⑥ 고급: `crates/core/src/view/mod.rs`(`AppStatus.git_path: String`), `crates/core/src/bootstrap.rs`(`app_status()`가 찾은 git 경로를 담음), 생성물, `src/features/settings/AdvancedSection.tsx`(Git 위치 코드 글꼴 읽기 전용 + 글자 버튼 "로그 폴더 열기" → `api.openLogFolder()`), `SettingsDialog.tsx`(구역 "고급" 추가), `src/messages`, 시험 파일.
- 내용: 전역 포커스는 `outline: var(--focus-width) solid var(--accent); outline-offset: var(--focus-offset)`이고 목록·줄·칸 안쪽은 `--focus-offset-inset`. 로딩은 `pending`이 참이 된 뒤 300ms가 지나서야 `true`(그 안에 끝나면 아무것도 안 그림). `Truncate`는 `title`을 쓰지 않고 `Tooltip`(400ms)으로 전체를 보인다. 경로는 `direction: rtl` + `unicode-bidi: plaintext`와 `<bdi>`로 앞을 줄인다(한글·영문 경로가 뒤집히지 않게 시험). Tab 순서: 상단 바(프로젝트·브랜치·오른쪽 단추) → 노선도(이전 N개·역) → 이름 칸 → 파일 목록 → 비교 칸(줄 바꿈·접힌 줄). 설정 "고급"은 읽기 전용 값과 단추 하나뿐(쓰기 없음). 010-H10이 나중에 "올린 버전도 바꾸기"를 이 구역에 더한다.
- 테스트: `useDelayed.test`(가짜 타이머: 299ms는 거짓, 301ms는 참, 그 전에 끝나면 안 켜짐), 로딩 다섯 곳 시험(즉시 끝나는 질의는 "불러오는 중"이 없다), `Truncate.test`(잘리지 않으면 툴팁 없음, 잘리면 마우스 진입·포커스에서 전체 툴팁(`scrollWidth` 모킹), 경로는 앞이 줄고 끝 이름이 보임), 개별 `outline … var(--accent)` 선언이 `base.css`·예외 밖에 없다(`rg`를 시험으로: `src/**/*.css`를 읽어 확인), `Workspace` Tab 순서, `AdvancedSection`(경로 표시·단추가 `openLogFolder` 호출), `app_status` 시험(`git_path` 비어 있지 않음), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`. 눈으로: Tab으로 한 바퀴 돌 때 모든 컨트롤의 포커스 테두리가 같다.
- Windows/Ubuntu: `direction: rtl`·`bdi`는 두 웹뷰 동일(한글 경로 손 확인). Windows `git_path`는 `C:\Program Files\Git\cmd\git.exe` 같은 모양이고 화면에서 말줄임. 보안: `git_path`는 설정 화면에만 보이고 로그·오류 문구로 보내지 않는다(D-040의 개인 경로 가리기는 로그·오류에만 적용).

### J. 마무리

P20. 문서·재촬영·점검 (`docs: wrap up polish iteration`)
- 목표: 문서를 구현에 맞추고, 다듬은 모습을 다시 찍고, 규칙 점검을 돌린 뒤 010-H3으로 돌아간다.
- 바꿀 파일: `docs/design.md`(구현과 다른 값 정리), `docs/spec.md`, `docs/architecture.md`(OS별 처리 표 행, 명령 목록 `app_display_hints`, 프론트 "화면 배율·설정 시작" 절: 루트 글자 크기·`uiScale`·`preloadDisplay`·단위 노선도), `docs/roadmap.md`(다듬기 "2026-10-16 완료", 반복 4b "010-H3부터"), `docs/progress.md`(P1~P20 체크, 사람이 할 일 새 항목, 다음 할 일 010-H3), `.claude/agents/code-analyst.md`(점검 줄 추가: 길이 `px`·`font-size` 직접 값 금지, `text-decoration: underline`·`dotted`는 `linkStyle.module.css`만, `Intl.DateTimeFormat`은 `time.ts`만, `navigator.language(s)`는 `locale.ts`만, OS 분기는 `platform`만(`display.rs` 허용), 새 설정 칸은 `view/settings.rs`·`AppSettingsPatch`에 함께), `docs/research/img/polish/`에 P9와 같은 장면의 `p20-*.png`(홈 라이트·다크, 홈 "아주 크게" 800px, 버전 기록, 브랜치 목록, 설정(일반·고급), 비교 칸 접힌 줄과 줄 바꿈, 고친 파일 줄, 압축 파일 알림 줄, English 홈).
- 내용: code-analyst에게 "반복 끝 점검"을 맡기고(계층 규칙·용어 규칙·보안 점검표) 나온 위반을 고친 뒤 커밋한다. CI 두 OS·`git-min`·`cargo deny check`가 초록인지 `gh run list`로 본 뒤 010-H3 이어가기.
- 테스트: `pnpm check:full`, CI.
- 완료 기준: "완료 기준 (반복 전체)"의 항목이 모두 자동 시험 또는 "사람이 할 일"에 있다.
- Windows/Ubuntu: Windows 확인은 사람이 할 일로 남긴다. 보안: 그림에 개인 정보가 없다.

## 010에 미치는 영향
다듬기가 끝난 뒤 010-H3부터 이어 갈 때 아래를 적용한다(010 문서는 고치지 않고 구현 첫 단계가 이 절을 우선한다).
- **010-H5**: "설정 구역 틀: `IdentitySection`·`GitHubSection` 순서" 대신 이 계획의 틀 "일반·GitHub·고급"에 맞춘다: `IdentityForm`을 `GeneralSection`의 맨 위(제목 "이름과 이메일" 줄)에 넣는다. `docs/design.md`의 설정 대화상자 설명도 그 구조로.
- **010-H10**: "고급" 구역은 이미 `AdvancedSection`(P19)이 있고 "올린 버전도 바꾸기" 스위치를 그 안에 더한다(Git 위치·로그 폴더 열기 아래).
- **모든 010 화면 CSS**: 길이는 rem(린트가 막음), 글자 크기는 `--font-size-*`(`-code` 포함), 밑줄·점선 금지, 새 대화상자의 포커스는 전역 규칙, 시각은 `<Time>`, 긴 이름은 `<Truncate>`.
- **010-H6** 공용 확인 대화상자·**010-H13** 원격 줄 메뉴는 이 계획과 겹치는 파일이 없어 그대로.
- progress의 010-H3 "다음 할 일"은 다듬기가 끝나면 그대로 이어진다.

## 위험
- **웹뷰의 OS 글자 설정 반영이 겹칠 수 있다**: WebView2(Windows 텍스트 크기)나 WebKitGTK가 이미 글자 배율을 반영하면 앱이 곱해 두 번 커진다. 루트 글자 크기를 px로 직접 정하므로 보통 겹치지 않지만, 두 OS에서 접근성 배율을 바꿔 한 번만 커지는지 손으로 본다. 겹치면 `display_hints()`가 그 OS에서 `text_scale_percent`를 1로 돌려주는 한 줄로 푼다(platform 안에서만).
- **Ctrl+=가 웹뷰에 먼저 잡힐 수 있다**: `zoomHotkeysEnabled: false`와 확대 권한 없음, `preventDefault`를 쓰지만 WebView2·WebKitGTK에서 실제로 앱에 오는지(한국어 자판의 `Equal`·숫자패드 포함) 손 확인한다. 안 오면 `keydown` 캡처 단계로 옮기거나 설정만 남긴다(단축키는 선택).
- **OS 읽기 실패**: `gsettings`가 없는 데스크톱·`reg.exe` 거부·느린 dbus는 기본값(배율 100, 다크 모름)으로 내려가고 앱은 막히지 않는다(1.5초 제한, 시작 때 0.8초 안에 못 읽으면 기본값으로 그리고 나중에 `useDisplaySync`가 따라간다).
- **px → rem 대량 변경**: 약 120곳이라 눈으로 놓칠 수 있다. 린트(`declaration-property-unit-disallowed-list`)와 `rg` 점검이 완료 기준이고, P9 그림으로 사용자가 크기를 확인한다. 첫 화면을 그리기 전에 루트 글자 크기를 정하지 못한 환경(JS 실패)은 16px = 본문 14px로 Windows 기본과 같아 읽을 수는 있다.
- **상단 바 넘침은 jsdom이 못 잰다**: CSS 문자열 시험과 P9·P20 그림, 사람 손 확인으로 막는다. "아주 크게"+OS 배율 150%+800px는 두 줄 이상으로 늘어날 수 있다(가로 스크롤은 없음).
- **글자 크기 단계의 값**(13/14·1·16/14·18/14)은 사용자가 그림을 보고 바꿀 수 있다. 값은 `fontScale.ts` 한 곳, 토큰은 `tokens.css` 한 곳이고 "12px 아래 금지" 시험이 하한을 지킨다.
- **언어 판단**(`navigator.languages`)이 OS 표시 언어와 어긋날 수 있다(웹뷰 설정). 그때는 설정에서 직접 고른다. 판단 코드는 `locale.ts` 하나라 바꾸기 쉽다.
- **`settings.json` 호환**: `v: 1`을 유지하고 칸을 더하므로 새 파일을 옛 앱이 읽으면 기본값으로 돌아간다(개발 중에만 해당, "올린 버전도 바꾸기"도 꺼짐 = 안전한 쪽).
- **접힌 줄 "펼치기"는 읽기 전용**이지만 큰 파일에서 `-U` 큰 수가 느릴 수 있다. 기존 비교 상한 안에서만 읽고 `truncated`면 펼치지 않는다. 마지막 묶음 뒤 접힌 줄은 파일 길이를 몰라 펼친 뒤에만 보인다(r-home.png의 끝 줄과 다름, 알려진 차이).
- **체크 "모두 켠 모양"**: 넣은 파일을 하나씩 끄다 마지막 하나를 끄면 모두 켠 모양으로 돌아온다(D-142·D-062의 결과). 사용자가 놀랄 수 있어 손 확인에 넣는다.
- **압축·실행 알림이 너무 자주 뜰 수 있다**: 프로젝트에 `.zip` 테스트 자료나 `.dll`이 정상으로 들어 있으면 저장할 때마다 한 번 알린다(같은 입력이면 통과). 막지 않고 "제외할 파일에 넣기"가 있다. 불편하면 이후 반복에서 파일별 "다시 묻지 않기"를 더한다.
- **고DPI Windows·소수 배율**에서 SVG와 겹친 HTML 단추(이전 N개·역 툴팁 범위)의 위치가 1px쯤 어긋날 수 있다. 손 확인.
- **범위**: 20단계라 길다. P1~P9가 사용자가 말한 앞쪽(글자·밑줄·그림)이고, P10 이후는 서로 독립이라 앞에서부터 하나씩 끝내도 앱이 깨지지 않는다. 시간이 모자라면 P19(디테일)의 ③ 말줄임·⑤ 스크롤 막대는 반복 5로 넘기고 P20에 적는다.
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md`에 "(자동 결정)"으로 올린 뒤 이어 간다. 사용자 결정 D-138~D-145와 어긋나는 구현이 필요해 보이면 멈추고 progress "사람이 할 일"에 적는다.
- code-analyst 검토 때 먼저 볼 곳: P4(절대 경로·출력 상한·시간 제한·파서 범위, `platform` 밖에 OS 분기가 없는지), P5(`settings.json` 호환·4KB·알 수 없는 값 기본값, 새 명령 네 곳), P6(`RESERVED`에서 뺀 키가 표에 있는지, 웹뷰 확대 완전히 꺼짐, 시작 순서·실패 시 기본값), P2·P8(린트 규칙이 실제로 막는지, 예외 목록이 최소인지), P3(배율이 한 경계에서만 곱해지는지), P10(`Intl` 직접 호출이 `time.ts` 밖에 없는지, ko·en 키 동일), P12·P13(이름표 겹침, `<title>`→`Tooltip` 키보드), P15(`fullContext` 인자가 정수 상수뿐인지), P17(index를 저절로 바꾸지 않는지), P18(읽는 양 상한).

## 사람이 할 일
`docs/progress.md`의 "사람이 할 일"에 P20에서 옮긴다. **사람만 줄 수 있는 것**은 새로 없다. 실제 컴퓨터에서 확인할 것:
- **P9 그림 확인(가장 먼저)**: `docs/research/img/polish/p9-*.png`를 보고 글자 크기·밑줄이 마음에 드는지. 마음에 들지 않으면 알려 주면 `tokens.css`·`fontScale.ts` 값만 고친다.
- **글자 크기(두 OS)**: Ubuntu에서 설정 → 디스플레이/접근성의 "큰 글자"(`text-scaling-factor` 1.25)와 Windows 설정 → 접근성 → 텍스트 크기(150%)를 바꾸고 앱을 켜서 한 번만 커지는지(겹치지 않는지), 앱을 켠 채 OS 배율을 바꾸고 창으로 돌아오면 따라가는지. 설정 "글자 크기" 네 단계, Ctrl+=·Ctrl++·Ctrl+−·Ctrl+0(한국어 자판·숫자패드)이 WebView2·WebKitGTK에서 앱에 오고 페이지 확대가 같이 일어나지 않는지, 버전 기록의 `+`·`-`·Ctrl+휠은 노선도만 확대하는지. 800px·"아주 크게"에서 상단 바가 넘치지 않는지, 홈·버전 기록·브랜치 목록·설정·충돌 화면이 겹치거나 잘리지 않는지.
- **언어**: Ubuntu를 영어 로케일(`LANG=en_US.UTF-8`)로 띄워 "시스템 따라가기"가 English인지, Windows 표시 언어를 영어로 바꿔 같은지, 설정에서 바꾸면 다시 시작 없이 바뀌고 껐다 켜도 유지되는지. English 용어 툴팁에 이름 줄이 겹치지 않는지, 시각 표시("N분 전"·"오후 3:12"·"10월 7일"·"2025년 10월 7일")와 호버의 정확한 시각.
- **화면 모드**: 다크·라이트·시스템 따라가기, OS 모드를 바꾼 뒤 창으로 돌아오면 시스템 따라가기가 따라가는지(Ubuntu 22.04에서 특히). start-normal.png·start-normal-dark.png와 모양 비교.
- **밑줄·툴팁**: 어디에도 점선·상시 밑줄이 없는지, 용어 툴팁이 0.6초 뒤에 뜨는지, "용어 설명 보기"를 끄면 안 뜨는지, 글자 버튼 호버 바탕.
- **노선도**: 버전이 30개 넘는 저장소에서 "이전 N개"가 보이고 누르면 버전 기록이 열리는지, 역 호버·Tab 포커스 툴팁, "지금 여기" 이름표와 합치기 제안 블록의 간격, 한 줄·두 줄·저장 1~2개짜리 프로젝트, 다크, 800px. 시작 화면의 큰·작은 노선도가 안 깨졌는지.
- **비교 칸**: "변경 없는 N줄 · 펼치기"(첫 묶음 앞·묶음 사이), 펼친 뒤 줄 번호, "줄 바꿈" 켬·끔과 긴 한글·영문 줄, 큰 파일에서 느리지 않은지.
- **파일 줄**: 이름 위·폴더 아래(긴 경로·한글 폴더), 상태 "고침", 넣은 파일이 0개일 때 모두 켠 체크, 하나를 끄면 나머지만 들어가는지(`git status`로 확인), 마지막 하나를 끄면 모두 켠 모양으로 돌아오는 것이 괜찮은지.
- **압축·실행 파일 알림**: `home.zip`·`.exe`/ELF를 넣고 저장하면 한 번 알리고 "그래도 저장"으로 저장되는지, 비밀 파일이 함께 있으면 기존 제목인지.
- **디테일**: 로딩 표시가 짧은 작업에서 깜빡이지 않는지(느린 디스크·큰 저장소에서는 300ms 뒤에 보이는지), 긴 이름 말줄임과 호버 전체 이름, Tab으로 한 바퀴 돌 때 포커스 테두리와 순서(상단 바 → 노선도 → 이름 칸 → 파일 목록 → 비교 칸), 설정의 "고급"(Git 위치·로그 폴더 열기, 두 OS에서 파일 관리자가 열리는지).
- **웹뷰 공통**: 한글 IME(설정 입력은 없지만 이름 칸·글상자), Windows 고대비에서 점선·선이 `CanvasText`로 보이는지.

## 단계 순서 요약
| 단계 | 내용 | 크레이트·화면 | 커밋 메시지 제안 |
| --- | --- | --- | --- |
| P1 | 결정 D-146~D-161·문서 | docs | `docs: add polish decisions and plan 011` |
| P2 | rem 토큰·CSS 단위·린트 | styles | `styles: switch size tokens to rem and raise font sizes` |
| P3 | 노선도·목록 배율 | 화면(history) | `history: scale the route map with the root font size` |
| P4 | OS 글자 배율·다크 읽기 | platform | `platform: read os text scale and color scheme` |
| P5 | 설정 칸 넓히기·`app_display_hints` | core, app | `core: extend app settings and add display hints command` |
| P6 | 글자 크기 적용·설정·Ctrl+=·−·0 | 화면, app 설정 | `features: font size setting, os scale and shortcuts` |
| P7 | 상단 바 테두리 맞춤·넘침 | 화면 | `screens: top bar buttons without borders and wrap at large sizes` |
| P8 | 밑줄 줄이기·용어 설명 보기·린트 | 화면(ui) | `ui: term tooltip without underline and a switch to turn it off` |
| P9 | Ubuntu 스크린샷 | docs | `docs: add ubuntu screenshots after font size and underline changes` |
| P10 | 언어 설정·시각 표시 규칙 | 화면 | `features: language setting and time display rules` |
| P11 | 화면 모드 | 화면 | `features: color scheme setting` |
| P12 | 노선도 배치: 점선·5역·간격·"지금 여기" | 화면(history) | `history: dotted fold, five stations, label-width spacing and now tag` |
| P13 | 홈 노선도 연결: 이전 N개·툴팁·폭 | 화면 | `screens: home route map fold button, tooltips and full width` |
| P14 | 비교 질의 전체 문맥 | git, core | `git+core: optional full-context diff` |
| P15 | 비교 칸 접힌 줄·펼치기·줄 바꿈 | 화면(diff) | `diff: unchanged-lines fold, expand and line wrap toggle` |
| P16 | 파일 줄 두 줄·"고침" | 화면 | `screens: two-line file rows and the word edited` |
| P17 | 체크 모양(0개면 모두 켬) | 화면 | `screens: show all files checked when none is staged` |
| P18 | 압축·실행 파일 알림 | core, 화면 | `core: warn about archive and executable files before saving` |
| P19 | 로딩 지연·말줄임·포커스·Tab·고급 구역 | core, 화면 | `ui: delayed loading, truncation, unified focus ring and advanced section` |
| P20 | 문서·재촬영·점검 | docs | `docs: wrap up polish iteration` |

순서의 까닭: P1이 모든 단계 앞(결정·문서). 글자 크기가 가장 먼저 보이는 일이라 P2(단위)→P3(노선도 배율)→P4·P5(OS 읽기·설정)→P6(적용·단축키)→P7(상단 바 넘침)을 앞에 두고, 밑줄 P8 직후 P9에서 사용자가 볼 그림을 만든다. 언어·화면 모드(P10·P11)는 P5의 설정 칸과 P6의 `applyDisplay`·구역 틀 위에 얹힌다. 노선도(P12·P13)는 P3의 `scale`과 `SCENE_FONT` 위에서, 비교 칸(P14·P15)과 파일 줄(P16·P17)·알림(P18)은 서로 독립이다. P19는 앞 단계가 만든 `Truncate`를 쓸 곳이 다 생긴 뒤. 각 단계는 CI가 초록이어야 다음으로 간다(진행 방식). 반복이 끝나면 010-H3으로 돌아간다.

## 검토 반영 (code-analyst, 구현 전)
구현은 아래를 위 단계 설명보다 우선한다. P1이 결정 D-146~D-161을 올릴 때도 위 표가 아니라 이 절로 고친 값을 쓴다.

- **R1 (P6, C1) 배율 하한.** `computeUiScale`의 하한을 0.9가 아니라 `MIN_SCALE = 12 / (16 × 0.8125)`(≈0.923)으로 한다. 시험은 `bodyPx` 14·15, `textScalePercent` 50~300, 네 단계 전부의 조합에서 `16 × scale × 0.8125 ≥ 12`를 확인한다.
- **R2 (P5, C2) OS 읽기는 Core 없이.** `app_display_hints`는 `AppState`의 Core를 쓰지 않는 무상태 명령이다(`higit_core::display_hints()`가 `platform::display_hints()`를 변환). Git이 없거나 오래돼 Core가 없을 때도 "Git 받기" 화면이 OS 배율로 그려진다. 그때 `settings_get`은 실패하므로 `preloadDisplay`는 설정을 기본값으로 쓴다(그 화면에서는 설정 대화상자를 열 수 없다). 시험: `preloadDisplay`가 설정 읽기 실패(GitMissing 오류)에서도 힌트를 적용한다.
- **R3 (P18, C13) 압축·실행 판정.**
  - 새 파일에만 알린다: 인덱스 쪽은 `index == Added`, `stage_all` 쪽은 `Untracked`. 이미 추적 중인 파일을 고친 것은 알리지 않는다.
  - 문서·패키지 컨테이너 확장자(`.docx .xlsx .pptx .odt .ods .odp .jar .apk .epub .whl .nupkg .vsix .xpi`)는 PK 매직을 무시한다(알리지 않음).
  - `MZ`는 확장자가 `.exe .dll .msi .sys .scr`일 때만 매직으로 쓰거나, 확장자가 없으면 PE 머리(0x3C의 4바이트 오프셋이 가리키는 곳이 `PE\0\0`)까지 맞아야 실행 파일이다.
  - 내용 규칙 검사는 매직이 맞은 파일만 건너뛴다. 확장자만 맞은 파일(`foo.zip`이라는 이름의 글자 파일)은 지금처럼 내용을 검사하고, 비밀이 나오면 비밀 규칙이 이긴다.
  - 제목은 두 수를 따로 센다: 비밀만이면 `secretTitle(n)`, 압축·실행만이면 `heavyTitle(n)`, 둘 다면 둘을 " · "로 잇는다.
  - "링크는 읽지 않음" 시험은 작업 폴더 경로가 `symlink_metadata` 확인 뒤 여는 구조임을 알고 쓴다(틈을 메우는 것은 이번 범위가 아니다).
- **R4 (P6) 설정 대화상자 스크롤.** `DialogFrame` 본문에 `overflow-y: auto; min-height: 0`을 준다. 완료 기준에 "창 높이 560px·아주 크게에서 설정이 스크롤되고 버튼을 누를 수 있다"를 더한다(P9 그림·손 확인).
- **R5 (P7, P9, P20) 넘침 확인은 실제 웹뷰에서.** jsdom 시험은 CSS가 적혀 있는지만 본다. P7의 "800px 시험"은 P9의 손 확인이다. P9·P20은 창을 800px로 줄이고 "아주 크게"에서 찍어 가로 스크롤 막대가 없고 오른쪽 덩어리가 보이는지를 그림으로 확인하고 결과를 progress에 적는다(웹뷰 안 JS를 자동으로 돌릴 길이 없어 그림으로 본다). `Workspace.module.css`의 왼쪽 칸은 `minmax(min(17.5rem, 45vw), 21.25rem)`로 해 큰 배율에서 절반을 넘지 않게 한다(P2).
- **R6 (P5, C3) 새 bool 기본값.** `term_help`·`diff_wrap`은 `#[serde(default = "yes")]`(참을 돌려주는 함수)로 읽는다. 옛 `{"v":1,"allow_published_rewrite":true}`를 읽으면 `term_help`·`diff_wrap`이 참인지 시험한다. `AppSettingsPatch`에 `Default`를 derive하고 기존 구조체 리터럴은 `..Default::default()`로 고친다. `AppSettings`를 실행용·화면용으로 나누지는 않는다(파일 하나, `Copy` 유지).
- **R7 (P4, P6) 시작 때 읽기 속도.**
  - P4: Linux의 `gsettings` 두 번은 스레드 두 개로 함께 부른다. `run_limited`는 stdin·stderr를 `Stdio::null()`로 두고, 시간을 넘기면 `kill` 뒤 `wait`까지 한다.
  - P6: `preloadDisplay`는 설정과 힌트를 따로 기다린다(각 0.8초). 늦게 온 값은 `useDisplaySync`가 나중에 적용한다(한 번 튈 수 있음, 손 확인). 힌트를 `settings.json`에 캐시하지 않는다.
- **R8 (P6) `Choice`의 키 판정.** 방향키 판정은 `src/lib/choiceKeys.ts`(`listKeys.ts`처럼, 조합 중·수식키 무시)로 한다(D-056). Radix RadioGroup은 쓰지 않는다(새 의존성 없음).
- **R9 (P12, C10) 노선도 간격.** 앞쪽 역을 접는 최소 간격 루프는 남긴다(`MIN_STEP = 72` 단위). 간격 = `max(MIN_STEP, min(남은 폭/(n−1), max(96, 가장 긴 이름표 폭(최대 200) + 16)))`이고 `MIN_STEP`으로도 안 들어가면 앞 역을 더 접는다. 시험에 "scale 1.3, 장면 폭 560"을 넣는다. "지금 여기"를 `width − 72`에 두는 것은 `home`에만 적용하고 `preview`·`mini`는 지금 위치다(시작 화면 시험 변화 없음). 접힌 구간 마름모(`kind: "fold"` 역, `homeToRouteNodes`)는 점선으로 바꿀 때 함께 없앤다.
- **R10 (P2, P3) 배율을 곱할 곳 추가.** P3 "바꿀 파일"에 `screens/Start/LastProject.tsx`(`MAP_HEIGHT`), `screens/Start/Start.module.css`의 `.rowMap`(장면 `MINI_WIDTH`·`MINI_HEIGHT`와 짝, rem으로 맞춤), `screens/History/History.tsx`(`shown.length * ROW_H` 비교와 역 메뉴 anchor), `layout/hit.ts`의 `hitTest`를 부르는 곳(`clientX/Y`를 `scale`로 나눔)을 더한다. P2 커밋에서 `horizontal.ts`의 `FONT`를 13으로 함께 바꾼다(`--font-size-sm`이 13px이 되므로).
- **R11 (P2, P8) 토큰·린트 규칙.**
  - `tokens.test.ts`는 크기 토큰(`--font-size-*`·`--space-*`·`--radius-*`·`--row-height*`·`--scrollbar-size`·`--motion-shift`)이 rem인지만 본다. 선·그림자 토큰(`--border-width`·`--selected-bar`·`--shadow-float`·`--focus-*`)은 px로 둔다.
  - 코드 글꼴: `font-family: var(--font-mono)`를 쓰는 규칙은 `font-size: var(--font-size-code)`를 함께 쓴다(D-138 "코드 14"). 시험(`src/styles/mono-size.test.ts`)이 CSS 모듈을 읽어 확인한다. 예외가 필요하면 시험의 목록에 이유와 함께 둔다.
  - P8의 stylelint 규칙에 `/^border(-(top|right|bottom|left))?-style$/`의 `dotted`도 넣는다.
- **R12 (P10, C5) 시각 표시.**
  - 문자열이 필요한 곳(`SyncButton`의 `aria-label`·`title`, `RouteList` 행 aria 라벨, `LastProject`의 문장, `BranchList`, `HunkPanel`)은 `formatTime` 문자열을 쓰고, 화면 글자만 필요한 곳은 `<Time>`을 쓴다.
  - `RouteList.module.css`의 `.date` 폭은 "2025년 10월 7일"이 들어가게 rem으로 넓힌다.
  - `<Time>`의 정확한 시각은 네이티브 `title`로 둔다. 시각 글자는 목록 줄·단추 안에 많이 들어가 툴팁 트리거를 겹칠 수 없기 때문이다. 이 예외를 D-150 문장에 적는다.
  - ESLint `no-restricted-properties`·`no-restricted-syntax`로 `Date.prototype.toLocale*String`과 `navigator.language`·`navigator.languages`도 막는다(`time.ts`·`locale.ts`만 예외).
  - `parity.test`는 샘플 인자를 `copy-rules.test.ts`와 같은 표에서 가져온다(표를 `src/messages/samples.ts`로 옮겨 둘이 함께 쓴다).
- **R13 (C16) 미루는 것 다시 정함.**
  - 파일 줄 오른쪽 메뉴의 "경로 복사"·"이 파일 고친 것 버리기"는 새 보안 설계가 필요 없으므로(`src/lib/copy.ts`, 기존 `discard` 액션과 그 확인 흐름) 이번에 한다: **P17b**(P17 뒤) `screens: file row context menu with copy path and discard`. 메뉴 부품은 역 메뉴와 같은 것을 쓰고 키보드(메뉴 키·Shift+F10)로도 연다.
  - 반복 5로 넘기는 것: "폴더에서 보기"·더블클릭 "기본 앱으로 열기"(실행 파일 거절·경로 검증), 창 크기·위치 기억(`tauri-plugin-window-state` 새 의존성·capability·cargo-deny 검토, 여러 모니터 손 확인), 시스템 글꼴 pt 읽기(D-138이 상수로 정함).
  - 최소 창 크기 960×600은 D-138의 "800px 너비 시험"과 맞지 않으므로 800×560을 유지하고, progress "사람이 할 일"에 사용자에게 물을 것으로 올린다.
- **R14 (P14, P15, C11) 접힌 줄 계산.**
  - `HunkView`에 `oldStart`·`oldLen`·`newStart`·`newLen`을 내려 사이 줄 수를 이 값으로 센다(줄 번호가 없는 삭제만 있는 묶음도 맞게). `bindings.ts` 갱신 대상이다.
  - 전체 문맥이 비교 상한(`MAX_LINES`·`MAX_BYTES`)에 걸리면, 펼칠 구간이 받은 줄 안에 다 있으면 펼치고 아니면 그 구간만 `m.diffTruncated`로 둔다.
  - 마지막 묶음 뒤 접힌 줄(r-home.png의 끝 줄)은 반복 5 목록에 올린다.
  - `-U3`를 `COMMON_ARGS`에서 빼고 `DiffTarget`에 문맥 줄 수를 둔다(리터럴 사용처를 함께 고침). 인자는 정수 상수 두 개(3, `FULL_CONTEXT`)에서만 만든다.

참고로 반영할 것:
- **설정 저장 연타**: 글자 크기 단축키와 줄 바꿈은 스토어에 바로 넣고 `settings_set`은 300ms 디바운스로 마지막 값만 보낸다(P6).
- **시험 상태**: `src/test/setup.ts`의 `afterEach`가 `useUiStore`를 초기값으로 되돌린다(P6).
- **P9 대비**: Ctrl+=가 WebKitGTK에서 앱에 오지 않으면 설정에서 "아주 크게"를 클릭으로 고른다(`xclick.py`). 고친 파일 3개·긴 경로·`home.zip` 데모는 파일 줄을 바꾼 뒤인 P20 재촬영에서 쓴다.
- **010-H5 연결**: D-145의 "일반" 목록에는 이름과 이메일이 없지만 010이 정한 항목이라 "일반" 맨 위에 둔다(P1이 D-160에 한 줄로 적는다).
- **C6 화면 모드**: Windows는 WebView2의 `prefers-color-scheme`이 OS를 바로 따르므로 `display_hints().dark`를 Windows에서는 읽지 않고 `None`을 준다(`reg.exe` 호출 하나 줄어듦). Ubuntu만 `gsettings color-scheme`을 읽는다.
- **토스트 호버**(`--primary-hover`)가 `--primary`와 가까워 약할 수 있다: 손 확인에 넣는다.
- **P1 문서**: D-095 상태 줄과 함께 D-127 영향에 "settings.json 칸 추가(D-148)"를 적는다.
