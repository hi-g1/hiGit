# 014 OS 통합과 배포 (배포 전 보안 묶음·실행 인자·단일 인스턴스·우클릭 메뉴·파일 열기·설치 파일)
- 관련 결정: D-002, D-003, D-009, D-010, D-014, D-021, D-032, D-034, D-035, D-037, D-038, D-039, D-040, D-041, D-051, D-063, D-067, D-072, D-078, D-086, D-102, D-108, D-110, D-111, D-118, D-127, D-137, D-138, D-146, D-147, D-148, D-156, D-160, D-161, D-162, D-163, D-164. 012의 결정 후보 C1(웹 느낌)·C2(첫 화면)·C6(오류 안내 틀)은 012-U1이 올린 번호로 읽는다. 새로 정할 것은 아래 "결정 후보" C1~C16이고, **구현 첫 단계(R1)가 `docs/decision.md` 맨 아래 다음 빈 번호부터 "(자동 결정)"으로 올린다.** 번호는 이 계획에 적지 않는다(그 사이 010·013·012가 결정을 더한다). 사람에게 묻지 않고 추천안으로 진행하되, C16(D-051 미룸)만 "상태: 제안"으로 올려 사람 답을 기다린다(구현은 C16과 무관하다).
- 관련 명세: spec.md "진입점"(폴더 우클릭 "Git으로 관리", 명령줄 `higit <폴더경로>`, 단일 인스턴스), "화면 구성"의 작업 공간(고친 파일 줄 오른쪽 클릭)·설정 행·충돌 해결 화면("고른 것은 이 기기에 남겨 두었다가"). architecture.md "OS별 처리"(우클릭·실행 인자·단일 인스턴스 행과 "설치 프로그램에서는 기본 등록" 문장), "보안"(위협 표, 열린 저장소만 받기, 명령 네 곳), "프론트와 백엔드 계약"(명령·이벤트 목록), "새 액션 추가 절차"(이번에 새 액션은 없다). design.md "브랜드"·"부품 규칙"(파일 줄 메뉴, 알림 줄, 설정 대화상자).
- 근거 자료(결정 아님): `docs/research/security.md`("믿을 수 없는 저장소 설정", "Tauri와 웹뷰"의 단일 실행, "공급망과 배포"의 코드 서명·체크섬, "언제 넣나"의 첫 배포 전 항목). 조사 내용을 근거로 한 구현은 모두 아래 결정 후보를 거쳐 decision.md에 오른다.
- 앞선 계획: 010(반복 4b), 013(반복 4c, 이 계획을 쓴 커밋 f11b0b0에는 아직 없음), 011(다듬기), 012(UX 다듬기). **이 계획은 010 나머지·013·012가 끝난 코드 위에서 시작한다.** 아래 "현재 상태"는 f11b0b0 기준이고, 012가 바꿀 곳(웹뷰 메뉴·단축키 끄기, 흰 첫 화면, 파일 목록 키보드, 프로젝트 메뉴, 오류 안내 틀)은 "012-Un이 한다"로만 적었다. 구현 첫날 `rg`로 "현재 상태"를 다시 확인하고 어긋나면 `docs/progress.md`에 적은 뒤 단계를 맞춘다.
- 단계 이름은 R1~R20이고 `docs/progress.md`와 커밋 본문에는 "014-R3"처럼 계획 번호를 붙여 적는다(005·010의 H, 011의 P, 012의 U와 겹치지 않게). 진행: 사람 확인 없이 Claude가 계획 → code-analyst 검토 → 아래 순서대로 구현, 한 단계 = 한 커밋(메시지 `<영역>: <요약>`).

## 목표
받은 저장소가 되돌리기 기록·충돌 선택·올리는 곳·작업 폴더를 몰래 바꾸지 못하게 막은 뒤, 사람들이 실제로 깔아 쓰게 한다. Windows는 .msi, Ubuntu는 .deb와 AppImage로 설치하고, 탐색기·파일 앱에서 폴더를 오른쪽 클릭하거나 `higit <폴더>`로 열면 이미 켜진 창 하나에서 그 폴더가 열린다. 고친 파일은 "폴더에서 보기"·더블클릭으로 기본 앱에서 열리고(실행 파일은 열지 않는다), 창 크기와 위치가 남는다.

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04, Windows 10·11)에서 손으로, 가능한 것은 자동 시험으로 확인한다. 손 확인은 `docs/progress.md` "사람이 할 일"에 모은다(R20).
- 5-0 보안: 다른 저장소나 같은 저장소의 다른 자리에서 옮겨 붙인 journal 줄, 순서를 바꾸거나 끼워 넣은 줄은 Ctrl+Z에 쓰이지 않는다(시험). 앱을 새 버전으로 바꿔도 이전 Ctrl+Z 기록이 이어진다(한 번 옮기기, 시험). 앱 데이터에 키를 저장하지 못하면 시작 화면과 홈에 한 줄 알림이 뜬다. 키 파일을 읽지 못해도 덮어쓰지 않는다(시험).
- 충돌 "직접 고치기" 글이 웹뷰 `localStorage`에 남지 않는다(ESLint가 `localStorage`·`sessionStorage`를 막고, 옛 키는 시작 때 지운다). 고른 것은 저장소 git 폴더 안 서명한 파일에 남아 껐다 켜도 이어지고, 받은 zip에 심은 파일은 쓰이지 않는다(시험).
- 받은 저장소의 `remote.*.pushurl`·`url.*.pushInsteadOf`·`url.*.insteadOf`·`core.worktree`·`http.*`·`remote.*.proxy`가 신뢰 확인에 나오고, "끄고 열기"를 고르면 그 저장소에서 원격 작업이 모두 막히며 `core.worktree`는 무시된다(실제 git 시험). 화면의 Escape·Enter 판정은 `src/lib` 한 곳뿐이다(린트).
- 실행 인자·단일 인스턴스: `higit <폴더>`(상대 경로 포함)로 켜면 시작 화면을 건너뛰고 그 폴더가 열린다. 앱이 켜져 있으면 새 창이 생기지 않고 기존 창이 앞으로 나와 그 폴더를 연다. `--git-credential`로 다시 실행된 helper는 기존 창으로 넘어가지 않는다(시험).
- 우클릭 메뉴: Windows 탐색기에서 폴더와 폴더 빈 곳을 오른쪽 클릭하면 "Git으로 관리"가 있고(Windows 11은 "더 많은 옵션 표시" 안), Ubuntu 파일 앱은 오른쪽 클릭 "스크립트"에 같은 항목이 있다. 설정 "일반"에서 끄고 켠다. 설치한 빌드는 처음 켤 때 저절로 켜지고, MSI를 지우면 메뉴도 사라진다.
- 파일 열기: 고친 파일 줄 메뉴의 "폴더에서 보기"·"기본 앱으로 열기"와 더블클릭, 프로젝트 메뉴 "폴더 위치 열기"가 된다. 실행 파일·스크립트·바로 가기·링크·저장소 밖 경로는 "기본 앱으로 열기"가 꺼지고 core도 거절한다(시험).
- 창: 크기·위치·최대화가 다시 켜도 남고, 모니터를 뺐거나 다른 배율이면 화면 안으로 돌아온다. 최소 크기는 800×560 그대로다.
- Ubuntu는 시스템 글꼴 크기(`font-name`의 pt)를 따라 본문 크기가 정해진다(기본 11pt → 15px 그대로). 비교 칸에서 마지막 묶음 뒤 생략된 줄도 "변경 없는 N줄 · 펼치기"로 보인다.
- 설치 파일: CI가 .msi·.deb·AppImage를 만들고 .deb는 깨끗한 Ubuntu 22.04 컨테이너에 설치되며(`git` 의존 포함), .msi는 Windows 러너에서 조용히 설치·제거된다. 16·24px 아이콘은 단순한 아이콘이다(시험). 태그 `v*`를 올리면 release 워크플로가 서명 없는 시험판 초안 릴리스와 `SHA256SUMS`를 만들고, 서명 비밀을 넣으면 같은 워크플로가 서명한다(비밀 없을 때 건너뜀을 시험).
- `SECURITY.md`가 있고 `-z` 파서들이 무작위 바이트에 패닉하지 않는다(시험). `pnpm check:full`, 두 OS CI(`git-min` 포함)·`cargo deny check` 통과. code-analyst 점검 위반 0건.

## 범위
- **이번에 한다**: roadmap "반복 5" 절 전부. 5-0 보안 묶음(D-118·D-137 그대로: journal 묶기와 키 알림, 충돌 글 `localStorage`, 받은 저장소의 보내는 곳·작업 폴더 설정, Escape·Enter 판정 모으기), 폴더 우클릭 메뉴, 실행 인자와 단일 인스턴스, 설치 파일, D-161이 넘긴 넷(폴더에서 보기·기본 앱으로 열기, 창 크기·위치, 시스템 글꼴 pt, 비교 칸 끝 접힌 줄). 012가 반복 5로 넘긴 프로젝트 메뉴 "폴더 위치 열기"도 같은 길이라 함께 한다. D-038의 "첫 배포 전" 항목 중 체크섬·`SECURITY.md`·파서 무작위 시험과 서명 자리.
- **뺀다**(까닭과 갈 곳):
  - 코드 서명 인증서로 실제 서명: 사람이 인증서를 아직 갖고 있지 않다. 서명 없이 만들고 서명을 끼울 자리만 둔다(C14).
  - 자동 업데이트(Tauri updater): roadmap에 없고 updater 서명 키 관리가 따로 필요하다. 첫 정식 배포 전 계획으로 넘긴다(C15).
  - `cargo-fuzz` 퍼징: nightly 도구와 별도 CI가 든다. 이번에는 의존성 없는 무작위 바이트 시험으로 하고 cargo-fuzz는 서명과 함께 첫 정식 배포 전으로(C15).
  - 명령 인자를 저장소 ID로 바꾸기(D-051 "반복 5 전"): 모든 명령 시그니처가 바뀌는 큰 일이고 `OpenedRepos`가 이미 열린 저장소만 받는다. 미루는 결정을 사람에게 묻는다(C16).
  - Ubuntu 파일 앱 맨 위 항목(nautilus-python 확장): `python3-nautilus` 의존과 Nautilus 42·43 이후 API 차이가 있다. 이번에는 "스크립트" 하위 메뉴(C6). 맨 위가 필요하면 사람에게 묻는다.
  - Windows 11 새 오른쪽 클릭 메뉴(첫 화면 메뉴): 묶음 앱 등록과 COM 구현이 필요하다. 이번에는 "더 많은 옵션 표시" 안(C6).
  - 012가 사람에게 물은 둘은 사용자가 정했다(2026-10-16): 1분 넘는 작업의 OS 알림은 넣지 않고, Windows WebView2 브라우저 키 막기는 012에서 한다. 이 계획에서 따로 하지 않는다.
  - .rpm·NSIS·macOS: D-002 밖이거나 사용자가 고르지 않았다.
- **012가 한다**(중복 단계를 만들지 않는다):

| 항목 | 012 단계 | 이 계획이 더하는 것 |
| --- | --- | --- |
| 웹뷰 기본 오른쪽 클릭 메뉴·브라우저 단축키 끄기 | 012-U3 | 없음. 새 파일 메뉴 항목은 012-U3의 메뉴 틀 위에 얹는다 |
| 흰 첫 화면, 창 `visible: false`와 `app_ready` | 012-U4 | 단일 인스턴스가 창을 앞으로 가져올 때·창 위치를 되살릴 때 숨긴 창과 맞춘다(R7·R12) |
| 파일 목록 Tab 한 번·줄 메뉴 키 | 012-U19 | 새 항목 둘과 더블클릭만 더한다(R11) |
| 프로젝트 메뉴 확장 | 012-U23 | "폴더 위치 열기" 한 줄(R11) |
| 오류 안내 틀 `ERROR_FIX` | 012-U9 | 새 오류 코드 셋의 해결 버튼(R5·R10) |
| 커서·글자 선택·끌기 | 012-U2 | 없음 |

- 새 Tauri 명령 7개(`app_take_launch_folder`, `app_shell_menu_status`, `app_shell_menu_sync`, `app_shell_menu_set`, `file_reveal`, `file_open`, `project_reveal`)와 `conflict_draft_save` 1개, 새 이벤트 1개(`open-folder`). 새 액션 없음. 바뀌는 `core::view`: `AppStatus.undoKey`, `ConflictStateView.draft`(신규 `ConflictDraftView`), `ProjectOpenResult::NeedsTrust.redirects`, `ShellMenuView`(신규), `AppSettingsView/Patch.shellMenu`, `FileDiffView.newLineCount`, `ErrorCode` 3개(`RemoteSettingsOff`, `OpenRefused`, `OpenFailed`), 상수 `OPEN_BLOCKED_EXTENSIONS`. 새 의존성 후보 3개(C7): `tauri-plugin-single-instance`, `tauri-plugin-window-state`, `winreg`(Windows만, 이미 트리에 있음).

## 현재 상태 (코드로 확인한 사실, 커밋 f11b0b0)
- 창과 묶음: `src-tauri/tauri.conf.json`의 창은 `main` 1200×780, `minWidth 800`·`minHeight 560`, `zoomHotkeysEnabled: false`이고 `visible`·`backgroundColor`는 없다(012-U4가 더한다). `bundle`은 `"active": true`, `"targets": "all"`, 아이콘 5개(`32x32.png`·`128x128.png`·`128x128@2x.png`·`icon.icns`·`icon.ico`)뿐이고 `windows`·`linux` 설정·`publisher`가 없다. `version` 0.1.0, `identifier` `dev.higit.desktop`, `mainBinaryName` `higit`.
- 아이콘: `assets/brand/`에 `app-icon.svg`·`app-icon-1024.png`·`logo-light.svg`·`logo-dark.svg`만 있고 16·24px용 단순한 아이콘은 없다. `src-tauri/icons/`는 `pnpm tauri icon assets/brand/app-icon-1024.png`로 만든 것이라(D-163, progress "브랜드 아이콘") `icon.ico`의 16·24px도 큰 그림을 줄인 것이다.
- 앱 진입: `src-tauri/src/main.rs`가 첫 인자 `--git-credential`이면 `app_lib::run()` 전에 helper로 갈라진다(D-111). `src-tauri/src/lib.rs`는 플러그인 `dialog`·`opener`만 등록하고 `setup`에서 진행 알림 싱크만 건다. 실행 인자를 읽는 곳이 없다. 주석에 "app은 `higit-core`만 부른다(git, platform을 직접 쓰지 않는다)".
- 열기: `crates/core/src/bootstrap.rs:517` `open_project(path, how)`가 유일한 입구이고 `OpenHow`는 `Picked`·`Recent` 둘이다. 523줄에서 신뢰 확인보다 먼저 `git.discover`(`crates/git/src/cli/mod.rs:251`, `rev-parse --show-toplevel`)를 부른다. 이 값은 `core.worktree`를 따르므로 받은 저장소가 `core.worktree=$HOME`이면 열린 저장소 경로가 홈 폴더가 된다. 화면은 `src/state/useStartup.ts`가 앱을 켤 때 최근 목록 첫 항목을 `"recent"`로 연다.
- 신뢰 확인: `crates/core/src/trust.rs`는 "실행되는 설정"(`safe_value`: fsmonitor·hooksPath·sshCommand·filter·diff·credential 등)과 hooks 파일만 보고 지문 해시로 `trust.json`에 결정을 남긴다. "끄고 열기"는 `set_repo_overrides`로 `-c` 덮어쓰기. `pushurl`·`insteadOf`·`pushInsteadOf`·`core.worktree`·`http.*`는 보지 않는다. `local_config`는 local·worktree 범위를 `--includes`로 읽는다(`cli/mod.rs:767`). `http.*`·`remote.*.proxy`는 helper를 붙이지 않는 데만 쓰인다(`core/src/github/mod.rs:512` `redirects_transport`). `remote_urls`는 insteadOf·pushInsteadOf·pushurl을 적용한 실제 주소를 준다(`git/src/types.rs:169`).
- journal: `crates/core/src/undo/journal.rs`는 `VERSION = 1`, 줄 = `{v, ...본문, sig}`이고 서명 글자는 `{v, 본문}`뿐이라 저장소·순서에 묶이지 않는다(같은 키로 서명한 줄을 다른 저장소 journal에 옮겨도 맞는다). 뒤쪽 4MB만 읽는다. 옛 줄 호환 시험 `LINE_1B`(309줄)이 있다(D-072). `crates/core/src/marker.rs`는 이미 git 폴더 실제 경로를 서명에 묶는다(D-102, `signed_text`).
- 키: `crates/core/src/undo/key.rs`의 `load_or_create`는 파일을 읽지 못하거나 길이가 틀리면 새 키를 만들어 **덮어쓴다**(일시적인 읽기 실패에도 모든 journal 서명이 무효가 된다). 쓰기에 실패하면 `persistent: false`이고 이 사실은 `pick.rs:192`·`rewrite.rs:267`의 멈출 수 있는 작업만 본다. `AppStatus`(`view/mod.rs:77`)에 키 상태 칸이 없고 화면 알림도 없다.
- 충돌 선택: `src/features/conflict/conflictStore.ts`가 `localStorage`의 `higit.conflict.<저장소 경로>|<합칠 버전 짧은 해시>`에 고른 것 전부를 JSON으로 둔다(`custom` 선택의 직접 고친 글 포함, `model.ts:4`). 키는 `screens/Conflict/Conflict.tsx:95`가 만든다. 그만두기·마치기를 앱 밖에서 하면 지워지지 않는다.
- 키 판정: `e.key === "Escape"`·`"Enter"`가 `src/lib` 밖 일곱 파일(`Rewrite.tsx`, `Activity.tsx`, `PullRequests.tsx`, `IdentityForm.tsx`, `rewrite/Card.tsx`, `CloneDialog.tsx`, `BranchList.tsx`)에 있다(progress 203·233, D-137이 5-0으로 넘김). `src/lib`에는 `listKeys.ts`·`choiceKeys.ts`·`cardKeys.ts`가 있다.
- 파일 줄: `src/screens/Workspace/FileRowMenu.tsx`가 "경로 복사"·"이 파일 고친 것 버리기" 두 항목(011 R13). 더블클릭 처리 없음. opener 플러그인(2.7.2, `reveal_item_in_dir` 있음)은 app의 URL 열기와 `app_open_log_folder`(`commands.rs:166`, `open_path`)에서만 쓰고, 웹뷰 권한에는 `opener:`가 없다(`tests/security_config.rs:54`가 막음).
- platform: 모듈 `app_dir`·`display`·`exe`·`file`·`git_exe`·`link`·`path`·`process`·`secret`. 우클릭 메뉴 코드가 없다(Cargo 설명에만 "우클릭 메뉴"). `exe::current_exe_path`는 AppImage일 때 마운트된 임시 경로를 준다(`APPIMAGE` 환경 변수를 모른다). `display.rs`의 `BODY_PX`는 상수(Linux 15, 그 밖 14)이고 Linux는 `gsettings`로 `color-scheme`만 읽는다(D-162).
- 설정: `crates/core/src/settings.rs`의 `AppSettings`(`Copy`, `allow_published_rewrite`·`language`·`font_size`·`theme`·`term_help`·`diff_wrap`·칸 폭 셋). 우클릭 메뉴 칸 없음. 설정 화면은 `features/settings/GeneralSection.tsx`·`GitHubSection`·`AdvancedSection`·`InfoSection`.
- 비교 칸: `view/diff.rs`의 `FileDiffView { binary, truncated, hunks, link }`, `DiffRequest.fullContext`. 접힌 줄은 `src/features/diff/model.ts`가 묶음 사이와 첫 묶음 앞만 센다(D-156). 비교 상한은 `git/src/cli/diff.rs`의 `MAX_LINES 5000`·`MAX_BYTES 1MB`.
- CI: `.github/workflows/ci.yml`의 작업 셋(`deny`, `git-min`, `build-and-test` 두 OS). 빌드는 `pnpm tauri build --no-bundle`이고 main push·수동 실행에서 Windows 포터블 exe와 `THIRD_PARTY_NOTICES.md`·`packaging/windows/README.txt`를 14일 올린다. 설치 파일·릴리스 워크플로·`SECURITY.md`가 없다. `src-tauri/deny.toml`은 두 OS 대상, 라이선스 허용 목록, `multiple-versions = "warn"`.
- 의존성 트리: `Cargo.lock`에 `winreg` 0.56(embed-resource의 빌드 의존)과 `zbus` 5.19가 이미 있다. `tauri-plugin-single-instance`·`tauri-plugin-window-state`는 없다.
- 문서: architecture.md "OS별 처리" 표(438~451줄)는 우클릭을 "Nautilus 스크립트 또는 nautilus-python"으로, "설치 프로그램에서는 기본 등록"으로 적었다. D-051은 "명령 인자를 저장소 ID로 바꾸는 일은 반복 5(첫 배포) 전"이라 적었다.

## 결정 후보 (R1이 decision.md 맨 아래 다음 빈 번호부터 "(자동 결정)"으로 올린다)
추천안을 적었다. C16만 "상태: 제안"으로 올린다.

| 번호 | 결정(추천안) | 까닭 | 대안 |
| --- | --- | --- | --- |
| C1 | **journal 줄을 저장소·순번·앞 줄에 묶는다.** 형식 `v: 2`. 서명 글자 = `"higit-journal\n2\n<git 폴더 실제 경로>\n<seq>\n<앞 줄 sig 또는 빈 글자>\n<본문 정규 JSON>"`. 줄에 `seq`(1부터 하나씩)를 적고 앞 줄 sig는 적지 않는다(읽을 때 앞 줄에서 얻음). 읽기는 v2 줄을 차례로 보며 `seq`가 앞 줄 + 1이고 사슬이 맞는 동안만 "믿는 줄"이다. 사슬이 끊기면 그 뒤 줄은 작업 내역에 보이기만 한다. 뒤쪽 4MB만 읽어 앞이 잘렸을 때만 첫 줄의 `seq`가 1이 아니어도 된다. 덧붙일 때 마지막 줄의 `seq`·sig를 뒤쪽 64KB에서 읽는다(저장소 쓰기 잠금 안). **옛 v1 줄은 더 믿지 않되**, 저장소를 열 때(신뢰 확인 뒤, `open_project`의 백업 정리 자리) 파일에 v2 줄이 하나도 없으면 지금 키로 서명이 맞는 v1 줄을 v2 사슬로 한 번 다시 써 옮긴다(임시 파일 + rename, `check_place` 그대로, 서명이 안 맞는 v1 줄은 그 자리에 그대로 둠). 옮기기에 실패하면 아무것도 바꾸지 않는다. | D-063의 서명은 줄 하나가 진짜인지만 본다. 같은 키로 서명한 다른 저장소의 줄, 지운 줄, 순서를 바꾼 `undo`·`redo` 줄을 막으려면 저장소와 순서에 묶어야 한다(D-108·D-118이 넘김). 한 번 옮기기는 D-072의 "앱을 업데이트하면 Ctrl+Z 기록이 사라지면 안 된다"를 지킨다. 저장소 경로 묶기는 D-102 표식과 같은 방식이다. | v1 줄을 계속 믿기(옮겨 붙이기 공격이 남음), v1 줄 버리기(D-072 위반), 줄마다 저장소 id를 새로 만들어 `.git`에 두기(받은 zip이 같이 가져옴) |
| C2 | **키를 못 쓸 때 알리고 덮어쓰지 않는다.** `load_or_create`는 파일이 없을 때만 새로 만든다. 파일이 있는데 읽지 못하거나 길이가 틀리면 덮어쓰지 않고 이번 실행만 임시 키를 쓴다. 상태는 `UndoKeyState`(`saved`·`temporary`)로 `AppStatus.undoKey`에 싣는다. `temporary`면 시작 화면 맨 위 알림 줄과 홈 알림 줄(이번 실행에 한 번, 닫기 있음)에 "되돌리기 기록을 이 컴퓨터에 저장하지 못해요. 앱을 다시 켜면 이전 작업은 Ctrl+Z로 되돌릴 수 없어요."를 보이고 "자세히 보기"에 앱 데이터 폴더 경로(`redact` 거침)를 접는다. 해결 버튼은 "로그 폴더 열기"가 아니라 없음. | 지금은 조용히 덮어써 모든 저장소의 Ctrl+Z가 사라질 수 있고, 저장하지 못해도 사람이 모른다(D-108의 "키를 못 쓸 때 알리기"). | 키를 키체인에 두기(잠금 해제 창이 앱 시작마다 뜰 수 있음), 알림 없이 로그만 |
| C3 | **충돌 선택은 git 폴더 안 서명한 파일에 둔다.** `<git 폴더>/higit/conflict-draft.json`, `marker.rs`와 같은 방식(도메인 `higit-conflict-draft`, git 폴더 실제 경로, 합칠 쪽 전체 해시를 서명에 묶음). 내용은 파일별 선택(`hunks`·`ours`·`theirs`, `custom`의 글 포함), 전체 8MB·글 하나 1MB(`ConflictFinishInput` 상한과 같음), 경로는 지금 충돌 경로에 있는 것만. 읽기는 `query_conflict_state`가 `draft`로 주고(서명·대상·경로·곳 수가 맞을 때만), 쓰기는 새 명령 `conflict_draft_save(folder, draft)`(화면은 500ms 디바운스). 마치기·그만두기 액션과 합칠 대상이 바뀐 것을 본 조회가 파일을 지운다. 화면의 `localStorage`·`sessionStorage`는 ESLint로 막고, 앱 시작 때 옛 `higit.conflict.*` 키를 한 번 지운다(`src/lib/legacyStorage.ts`만 예외). | spec이 약속한 "껐다 켜도 이어진다"를 지키면서 코드 글을 웹뷰 저장소(저장소 밖, 앱 밖에서 끝낸 합치기에도 남음)에 두지 않는다. git 폴더는 이미 그 코드를 가진 곳이다. 서명은 받은 zip에 심은 선택이 미리 채워져 "마치기" 한 번에 쓰이는 것을 막는다. | 직접 고친 글은 메모리에만(껐다 켜면 사라짐, spec 위반), 앱 데이터 폴더(저장소 밖에 코드 사본이 남음) |
| C4 | **보내는 곳·작업 폴더를 바꾸는 설정도 신뢰 확인에 넣는다.** 대상(local·worktree 범위): `remote.*.pushurl`, `url.*.insteadof`, `url.*.pushinsteadof`, `remote.*.proxy`, `remote.*.receivepack`, `remote.*.uploadpack`, `http.*` 전부, `core.worktree`. 지문 해시에 함께 넣는다(설정이 바뀌면 다시 묻는다). 대화상자는 지금 목록 아래에 "올리는 곳이나 작업 폴더를 바꾸는 설정" 목록을 따로 보이고 주소 값은 호스트만, 경로는 `redact`로 보인다. **"끄고 열기"**: 실행되는 설정은 지금처럼 덮어쓰고, 보내는 곳 설정이 있으면 그 저장소의 원격 작업(fetch·pull·push·동기화·저절로 하는 원격 확인·태그 올리기·원격 브랜치 지우기 등 네트워크 `GitOp` 전부)을 git 크레이트 한 곳(`CliBackend`의 저장소 정책 `RepoPolicy { overrides, network_off }`)에서 `RemoteSettingsOff`로 거절한다. 저절로 하는 원격 확인은 조용히 건너뛴다. 오류 문구 "이 프로젝트는 올리는 곳을 바꾸는 설정을 끄고 열었어요. 믿고 다시 열면 동기화할 수 있어요.", 해결 버튼 "믿고 다시 열기"(`ERROR_FIX`에 새 종류 `retrust`: 신뢰 확인을 다시 띄움). **`core.worktree`**는 "믿고 열기"가 없다. `discover`가 `rev-parse --absolute-git-dir --show-toplevel`로 작업 폴더가 git 폴더의 부모가 아님을 알아내면(연결된 worktree는 제외) 그 부모 폴더를 저장소로 보고 늘 `-c core.worktree=<그 폴더>`를 붙인다. git 폴더 이름이 `.git`이 아니면 열지 않는다(`NotARepo`). | D-118이 남긴 "받은 저장소가 자격 증명 없이 코드를 다른 곳으로 올리게 하는 위험"을 막는다. `pushurl`·`insteadOf`는 여러 값 설정이라 `-c`로 지울 수 없어서 덮어쓰기 대신 원격 작업을 막는다. `core.worktree`는 지금 신뢰 확인보다 먼저 저장소 경로를 바꿔 버리므로 열기 단계에서 무력화해야 한다. 한 곳에서 막아야 013이 더한 원격 쓰기까지 빠짐이 없다. | 매번 실제 주소를 보여 주고 올릴 때마다 묻기(D-043과 어긋남), `-c`로 덮어쓰기(여러 값이라 안 됨), `http.*` 키마다 안전한 값 표(값마다 의미가 달라 틀리기 쉬움) |
| C5 | **실행 인자와 단일 인스턴스.** `tauri-plugin-single-instance`를 첫 플러그인으로 등록한다(`main.rs`의 `--git-credential` 분기 뒤라 helper는 영향 없음). 인자 해석은 core 순수 함수 `launch::parse_args(args, cwd) -> Option<PathBuf>`: 첫 인자는 실행 파일, `--`가 나오면 그 뒤 첫 인자가 경로, 그 전에는 `-`로 시작하는 인자를 건너뛰고 첫 위치 인자를 경로로 본다. `--git-credential`이 있으면 `None`. 상대 경로는 그 실행의 현재 폴더(두 번째 실행은 플러그인이 준 `cwd`) 기준. 끝에 붙은 `"`(탐색기가 `"C:\"`를 넘길 때 생김)는 떼어 낸다. 4096바이트·NUL 거절. 첫 실행의 경로는 `AppState`가 들고 `app_take_launch_folder`(한 번만 줌)로 화면이 가져가 마지막 프로젝트 대신 `project_open(path, "picked")`으로 연다. 두 번째 실행은 Rust 콜백이 창을 보이고(`show`·`unminimize`·`set_focus`) 이벤트 `open-folder { path }`를 보낸다. 화면은 도는 작업·열린 확인 창이 끝난 뒤 연다. 경로가 없으면 시작 화면 알림 "열려던 폴더를 찾지 못했어요." 저장소가 아니면 지금처럼 "버전 관리 시작" 안내. | spec "진입점"의 우클릭·명령줄과 "이미 실행 중이면 그 창에서". 앱 두 개가 같은 최근 목록·journal·설정을 쓰는 일도 없어진다(D-032가 남긴 위험). 열기는 늘 `open_project` 한 곳(D-032)이라 신뢰 확인을 건너뛸 길이 없다. | 새 `OpenHow::Launch`(Picked와 같은 동작이라 필요 없음), 경로를 이벤트로만(첫 실행 때 화면이 아직 듣지 않음) |
| C6 | **폴더 우클릭 메뉴.** Windows: `HKCU\Software\Classes\Directory\shell\higit`와 `Directory\Background\shell\higit`에 `MUIVerb`(항목 이름), `Icon`(`"<exe>",0`), `command` 기본값 `"<exe>" -- "%V"`. 관리자 권한이 없다. Windows 11은 "더 많은 옵션 표시" 안에 보인다. Ubuntu: `~/.local/share/nautilus/scripts/<항목 이름>` 실행 스크립트(0755, 첫 줄 아래 `# higit-shell-menu v1` 표식, 실행 파일 경로는 작은따옴표로 감쌈). 스크립트는 실행 파일이 없으면 자기를 지우고 끝나고, 고른 항목이 있으면 `"$1"`, 없으면 지금 폴더로 `exec "<exe>" -- …`. 실행 파일 경로는 `platform::launcher_path()`: AppImage면 `APPIMAGE`(절대 경로·있는 파일·줄바꿈 없음일 때만), 아니면 `current_exe_path`. 항목 이름은 화면 언어의 `shellMenuLabel`("Git으로 관리" / "Manage with hiGit")을 화면이 넘기고 core가 64바이트·제어 문자 없음을 검사한다. 상태는 설정 파일이 아니라 실제 등록(표식·명령이 이 실행 파일인지)을 읽는다. 설정 `shell_menu: Option<bool>`은 "사람이 정했는지"만 남긴다. 화면이 시작 때 `app_shell_menu_sync(label)`을 부르면 core가: 값이 없고 **릴리스 빌드**면 등록하고 `true`로, `true`인데 실행 파일·이름이 바뀌었으면 다시 등록, `false`면 아무것도 안 함. 설정 "일반"의 "폴더 오른쪽 클릭 메뉴" 켬·끔이 `app_shell_menu_set`. MSI는 지울 때 위 HKCU 키를 지운다(WiX 조각). .deb는 사용자 폴더를 못 지우므로 스크립트가 스스로 지운다. | spec "진입점". HKCU와 사용자 스크립트는 관리자 권한·설치 파일 없이 포터블·AppImage에서도 같은 켬·끔으로 된다. 개발 빌드가 저절로 등록하면 개발 PC 메뉴가 `target/debug`를 가리키므로 릴리스만. "설치 프로그램에서는 기본 등록"(architecture)은 "설치한 앱을 처음 켤 때 등록"으로 바꾼다. | MSI가 등록(HKLM은 관리자 필요·설정 끄기와 어긋남), nautilus-python 확장(맨 위 항목, `python3-nautilus` 의존·API 두 판), `.desktop` `MimeType=inode/directory`(폴더 기본 앱이 바뀔 위험), 처음에 묻기 |
| C7 | **새 의존성 셋.** `tauri-plugin-single-instance` 2(app, Linux는 이미 있는 `zbus` 사용), `tauri-plugin-window-state` 2(app, C9), `winreg` 0.56(platform, `cfg(windows)`만, 트리에 이미 있는 버전). 셋 다 웹뷰 권한(capability)을 열지 않고 Rust에서만 쓴다. 정확한 버전·기능은 두 OS CI와 `cargo deny check`로 정하고 라이선스는 허용 목록 안이어야 한다(아니면 그 단계를 멈추고 progress에 적는다). `THIRD_PARTY_NOTICES.md`는 바이너리에 들어가는 것만 갱신. | 단일 실행은 security.md가 이 플러그인을 적었다. 레지스트리 쓰기를 `reg.exe`로 하면 `"`·`%V`가 든 값의 인자 따옴표 처리가 틀리기 쉽다. D-147이 `winreg`를 고르지 않은 까닭은 "새 의존성"이었는데 이미 트리에 있다. | `reg.exe`로 쓰기(따옴표), `windows-registry` 크레이트(새 트리), 직접 만든 소켓 단일 실행 |
| C8 | **파일 열기 규칙.** core `open_target(folder, path, purpose)`가 열린 저장소(`OpenedRepos`)의 상대 경로만 받아 정규화하고, 저장소 밖·`.git` 안·경로 구성요소 중 링크·없는 파일을 거절한다(D-037). `reveal`(폴더에서 보기)은 있는 파일·폴더면 된다. `open`(기본 앱으로 열기)은 일반 파일이고 다음이 아닐 때만: 확장자 차단표 `OPEN_BLOCKED_EXTENSIONS`(대소문자 무시, 마지막 확장자, 예 `exe com bat cmd ps1 psm1 vbs vbe js jse wsf wsh hta msi msp msix appx lnk url scr pif cpl reg inf jar py pyw rb pl sh bash zsh run bin appimage desktop deb rpm dll sys ocx application gadget settingcontent-ms library-ms` — 구현 단계가 Windows "위험한 첨부 파일 형식" 목록과 맞춰 고정), 앞 바이트 `\x7fELF`·`MZ`·`#!`, 실행 권한 비트(`platform::is_executable`, Windows는 늘 거짓), Windows 장치 이름·대체 데이터 스트림(`:`)이 든 경로(`platform` 경로 검사). 차단표는 바인딩 상수로 화면에 내려 메뉴 항목을 미리 끄고(까닭 "실행할 수 있는 파일은 열지 않아요") core가 실행 때 다시 본다. OS가 여는 일은 app이 opener 플러그인(`reveal_item_in_dir`, `open_path`)으로 하고 웹뷰에는 opener 권한을 주지 않는다(D-037 그대로). 폴더 위치 열기는 저장소 루트를 `open_path`. 실패하면 `OpenFailed`("이 파일을 열 앱을 찾지 못했어요."). 로그에는 결과 코드만. | D-161이 요구한 "실행 파일 거절·경로 검증". 기본 앱 열기는 Windows에서 `.exe`·`.js`·`.py`를 그대로 실행할 수 있고 Linux에서 `.desktop`·실행 비트 스크립트를 실행할 수 있다. 판정을 core 한 곳에 두고 화면은 미리 보기만 한다. OS 차이는 opener 플러그인과 platform 함수 둘에만 있다. | 확인 창을 띄우고 열기(초보자는 확인을 누름), 허용 목록(텍스트·그림만, 너무 좁음), platform이 `xdg-open`·`explorer.exe`를 직접 부르기(이미 있는 opener와 중복) |
| C9 | **창 크기·위치 기억은 `tauri-plugin-window-state`.** 상태 깃발은 `SIZE`·`POSITION`·`MAXIMIZED`만(`VISIBLE`·`DECORATIONS`·`FULLSCREEN` 제외, 012-U4의 숨긴 창을 먼저 보이지 않게). 파일은 플러그인 기본 위치(앱 설정 폴더 `.window-state.json`). 되살린 뒤 app이 한 번 고친다: 창 사각형이 어느 모니터 작업 영역과도 절반 넘게 겹치지 않으면 주 모니터 가운데로, 크기는 모니터보다 크면 줄이고 800×560보다 작으면 늘린다(Tauri 공통 API, OS 분기 없음). 웹뷰 권한(`window-state:*`)은 열지 않는다. 최소 크기 800×560은 그대로(사용자 결정). | D-161이 이 플러그인 검토를 적었다. 최대화 직전 크기·배율 바뀜 같은 경우를 플러그인이 이미 다룬다. 모니터 확인은 받은 값을 믿지 않는 안전장치다. | `settings.json`에 직접 저장(새 의존성 없음, 최대화·배율 처리 직접), 기억하지 않기 |
| C10 | **Ubuntu만 시스템 글꼴 pt를 읽는다.** `display_hints()`가 Linux에서 `/usr/bin/gsettings get org.gnome.desktop.interface font-name`(예 `'Ubuntu 11'`, 소수 허용)을 다크 읽기와 함께 읽어 `body_px = round(pt × 4/3)`을 13~20으로 자른다. 읽지 못하면 15. Windows는 14 그대로다(Windows 10 1703부터 시스템 글꼴 크기 설정이 없고 "텍스트 크기"는 이미 `TextScaleFactor`로 곱한다). `BODY_PX` 상수는 기본값으로 남는다. 절대 경로·1.5초·256바이트 규칙은 D-147 그대로. | D-138 "시작 크기는 OS 기본을 따른다"를 Ubuntu 글꼴 설정까지 넓힌다. 기본 11pt면 지금과 같은 15px라 아무것도 바뀌지 않는다. 글꼴 pt는 `text-scaling-factor`(웹뷰가 이미 반영, D-162)와 따로라 두 번 커지지 않는다. | Windows도 `NONCLIENTMETRICS` 읽기(unsafe·의미 없음), 둘 다 상수 유지 |
| C11 | **비교 칸 마지막 묶음 뒤 접힌 줄.** `FileDiffView.newLineCount: Option<u32>`(새 쪽 전체 줄 수). 이진·링크·`truncated`·새 쪽 1MB 초과·지운 파일이면 `None`. 작업 폴더는 `platform::open_regular_no_follow`로 1MB까지 읽어 줄을 세고, 버전 안 비교는 그 버전 블롭을 같은 상한으로 센다(012-U26의 `blob_at`이 있으면 쓰고 없으면 이 단계가 `ls-tree -z` 정확 일치로 더한다). 화면 `model.ts`는 `newLineCount > 마지막 묶음 끝 줄`이면 끝에 "변경 없는 N줄 · 펼치기"를 더하고 펼치기는 기존 `fullContext`(D-156)로 한다. | D-156이 "파일 길이를 몰라" 남긴 것. 읽는 양은 비교 상한과 같다. | 펼친 뒤에만 보이기(지금), `git diff -U` 큰 수로 늘 읽기(비교마다 두 배) |
| C12 | **16·24px은 단순한 아이콘.** `assets/brand/app-icon-small.svg`(먹색 둥근 사각, 굵은 본선과 갈래 하나, 역·색 번짐 없음, 1px 정렬)와 그 렌더 `app-icon-16.png`·`app-icon-24.png`를 둔다(사용자가 파일을 주면 그것, 아니면 R15가 만들어 사람 확인). `icon.ico`는 `pnpm tauri icon assets/brand/app-icon-1024.png`로 만든 뒤 `scripts/brand-icons.mjs`(의존성 없는 Node)가 16·24 항목을 작은 PNG로 바꿔 다시 쓴다. Linux는 `src-tauri/icons/16x16.png`·`24x24.png`(작은 그림)를 묶음 아이콘 목록에 더해 hicolor 16·24에 깔린다. 32px 이상은 지금 그림. D-163 영향 줄에 "16·24px은 작은 아이콘" 한 줄을 더한다. | 사용자 결정. 큰 그림을 16px로 줄이면 역 세 개와 번짐이 뭉개진다(progress "브랜드 아이콘"). | 모든 크기를 큰 그림에서(지금), 래스터 도구를 devDependency로(렌더는 한 번이면 됨) |
| C13 | **설치 파일.** `bundle.targets = ["msi", "deb", "appimage"]`(rpm·nsis 없음). `publisher`·`shortDescription`·`category: "DeveloperTool"`. **MSI**: WiX 언어 `ko-KR` 하나, `upgradeCode`를 한 번 만든 값으로 고정, WebView2는 기본 `downloadBootstrapper`(조용히), 제거 때 우클릭 HKCU 키를 지우는 WiX 조각(`packaging/windows/shell-menu-cleanup.wxs`, `fragmentPaths`·`componentRefs`), `THIRD_PARTY_NOTICES.md`를 실행 파일 옆에(`bundle.resources`). Program Files 설치라 관리자 확인이 뜬다. PATH에는 넣지 않는다(명령줄은 전체 경로). **deb**: `depends: ["git"]`, `section: "vcs"`, 자체 `.desktop` 틀(`packaging/linux/higit.desktop`: `Name=hiGit`, `Comment`·`Comment[ko]`, `Categories=Development;RevisionControl;`, `MimeType` 없음), `THIRD_PARTY_NOTICES.md`를 `/usr/share/doc/higit/`에. `/usr/bin/higit`이라 `higit <폴더>`가 된다. **AppImage**: `bundleMediaFramework: false`. Ubuntu 22.04는 `libfuse2`가 없으면 안 뜨므로 안내 문서에 적는다. 버전은 `package.json`·`tauri.conf.json`·Cargo 작업 공간 셋이 같아야 한다(시험). | 사용자가 고른 세 형식. 버전 맞춤은 설치 파일 업그레이드의 바탕이다. `git` 의존은 앱이 git CLI 없이는 켜지지 않아서(D-003). | NSIS 사용자 단위 설치(관리자 확인 없음, 사용자가 .msi를 고름), 두 언어 MSI(파일 둘) |
| C14 | **서명 없이 만들고 서명 자리를 둔다.** 릴리스는 CI에서만(D-038): 새 `.github/workflows/release.yml`이 태그 `v*`(와 수동 실행)에 두 OS 묶음을 만들고 `SHA256SUMS`를 붙여 **초안** GitHub 릴리스를 만든다(`gh` 명령, 새 액션 없음, 이 작업만 `contents: write`, GitHub Environment `release`). 서명 자리: Windows는 환경 비밀 `WINDOWS_SIGN_COMMAND`가 있으면 `src-tauri/tauri.sign.json`(`bundle.windows.signCommand`)을 만들어 `pnpm tauri build --config`로 넘긴다(exe와 msi 둘 다 서명됨). Linux는 `LINUX_GPG_KEY`가 있으면 `SHA256SUMS.asc`를 만든다. 비밀이 없으면 두 단계는 건너뛰고 릴리스 이름에 "서명 없는 시험판"을 붙인다. client id는 환경 비밀 `HIGIT_GITHUB_CLIENT_ID`(D-109). 방법은 `packaging/README.md`에 적는다. | 인증서를 사면 워크플로를 고치지 않고 비밀만 넣으면 된다. 초안이라 사람이 보고 게시한다. | 지금 자체 서명 인증서(SmartScreen 경고는 그대로), 로컬 빌드 배포(D-038 위반) |
| C15 | **D-038의 "첫 배포 전" 항목을 둘로 나눈다.** 이번 설치 파일은 "서명 없는 시험판"이고 이번에 하는 것: `SECURITY.md`(GitHub 비공개 취약점 신고), `SHA256SUMS`, 파서 무작위 시험(`-z` 파서마다 고정 씨앗 무작위 바이트·잘린 입력 2만 번, 패닉 없음, 의존성 없는 시험 도우미). 첫 정식 배포(서명한 설치 파일) 전으로 넘기는 것: 코드 서명, `cargo-fuzz`, 자동 업데이트와 updater 서명, Dependabot 지연 설정. D-038 영향 줄에 이 구분을 적는다. | 인증서가 없어 정식 배포의 조건을 다 채울 수 없다. 시험판은 체크섬과 신고 창구만으로도 위험이 작고, 파서 견고성은 지금도 값싸게 확인할 수 있다. | 서명 전에는 설치 파일을 만들지 않기(사용자 요청과 어긋남) |
| C16 (제안) | **명령 인자를 저장소 ID로 바꾸는 일(D-051)은 첫 정식 배포 전으로 미룬다.** 지금은 모든 저장소 명령이 `OpenedRepos`(열기·신뢰 확인을 거친 경로만)로 막혀 있고, 화면이 넘긴 경로는 열린 목록과 같을 때만 쓰인다. | 모든 명령 시그니처·바인딩·화면 호출이 바뀌는 큰 일이고 이번 반복의 목표(설치·OS 통합)와 겹치지 않는다. 막는 효과는 `OpenedRepos`가 이미 낸다. | 이번에 하기(단계 하나 이상 추가, R5 뒤) — **사람에게 묻는다** |

## 용어 변경
없음. 우클릭 메뉴·폴더에서 보기·기본 앱으로 열기·설치·창 기억은 Git 용어가 아니라 `docs/spec.md` 용어표와 glossary에 넣지 않는다. 신뢰 확인 대화상자의 설정 키는 지금처럼 원래 키 글자로 보인다. 새 화면 문구는 `src/messages` ko·en 같은 커밋에(괄호 없이, 해요체, 짧게):
- `undoKeyTemporary`: "되돌리기 기록을 이 컴퓨터에 저장하지 못해요. 앱을 다시 켜면 이전 작업은 Ctrl+Z로 되돌릴 수 없어요." / "Can't save undo history on this computer. After a restart, Ctrl+Z can't undo earlier work."
- `trustRedirects`: "올리는 곳이나 작업 폴더를 바꾸는 설정" / "Settings that change where code goes", `errors.remoteSettingsOff`: "이 프로젝트는 올리는 곳을 바꾸는 설정을 끄고 열었어요. 믿고 다시 열면 동기화할 수 있어요." / "This project was opened with its push settings turned off. Reopen it as trusted to sync.", `fix.retrust`: "믿고 다시 열기" / "Reopen as trusted"
- `launchMissing`: "열려던 폴더를 찾지 못했어요." / "Couldn't find the folder to open."
- `shellMenuLabel`: "Git으로 관리" / "Manage with hiGit", `settings.shellMenu`: "폴더 오른쪽 클릭 메뉴" / "Folder right-click menu", `shellMenuHint.explorer`: "탐색기에서 폴더를 오른쪽 클릭해 바로 열어요." / "Open a folder from its right-click menu in File Explorer.", `shellMenuHint.nautilusScripts`: "파일 앱 오른쪽 클릭 메뉴의 스크립트에 넣어요." / "Adds it under Scripts in the Files right-click menu.", `shellMenuHint.unsupported`: "이 컴퓨터에서는 쓸 수 없어요." / "Not available on this computer."
- `revealInFolder`: "폴더에서 보기" / "Show in folder", `openDefault`: "기본 앱으로 열기" / "Open with default app", `openBlocked`: "실행할 수 있는 파일은 열지 않아요" / "Executable files aren't opened", `openMissing`: "지운 파일이에요" / "This file was deleted", `projectReveal`: "폴더 위치 열기" / "Open folder location", `errors.openRefused`: "실행할 수 있는 파일이라 열지 않았어요. 폴더에서 보기로 확인해 주세요." / "This file can run, so it wasn't opened. Check it with Show in folder.", `errors.openFailed`: "이 파일을 열 앱을 찾지 못했어요." / "No app was found to open this file."
- 모두 `copy-rules.test.ts`·`terms-in-messages.test.ts`·ko·en 키 같음 시험을 통과해야 한다.

## 단계
각 단계 = 커밋 하나. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령·이벤트가 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`(`src/lib/bindings.ts`·`action-meta.json`). 새 명령은 `src-tauri/src/commands.rs`·`src/bindings.rs`·`build.rs`의 `COMMANDS`·`capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에(`tests/security_config.rs`가 확인). 화면은 git도 `invoke`도 직접 부르지 않는다(D-010). **OS별 코드는 `crates/platform`에만** 둔다: app·core·화면에 `#[cfg(windows)]`·`#[cfg(unix)]`·`navigator.platform` 분기가 없고(시험 전용 `cfg`는 예외), 설치 설정(`tauri.conf.json`의 `windows`·`linux` 절, `packaging/`)과 CI는 코드가 아니라 묶음 설정이다. 시험은 사용자 레지스트리·사용자 Nautilus 폴더·실제 키체인·전역 git 설정을 건드리지 않는다(시험용 루트를 인자로 받는 함수로).

### A. 결정과 문서

R1. 결정·문서 (`docs: add os integration and release decisions and plan 014`)
- 목표: 이후 단계가 쓰는 결정을 확정하고 문서를 새 방식에 맞춘다.
- 바꿀 파일: `docs/decision.md`(C1~C15를 다음 빈 번호부터 "(자동 결정)", 상태 확정. C16은 "상태: 제안". 갱신 줄: D-038 영향 "첫 배포 전 항목의 구분은 C15", D-063 "저장소·순번 묶기는 C1", D-072 "v1 줄은 한 번 옮긴 뒤 믿지 않음(C1)", D-108·D-118·D-137 "5-0 이행", D-161 "이행", D-163 "16·24px은 작은 아이콘(C12)", D-147 "Ubuntu 글꼴 pt는 C10"), `docs/architecture.md`("OS별 처리" 표: 우클릭 행을 C6으로, 실행 인자 행에 `--`, 새 행 "파일 열기·폴더에서 보기"(opener와 `platform::is_executable`)·"창 크기·위치"·"설치 파일", "설치 프로그램에서는 기본 등록" 문장을 "설치한 앱을 처음 켤 때 등록, MSI는 지울 때 지움"으로. "보안" 위협 표에 "받은 저장소의 보내는 곳·작업 폴더 설정"과 "열린 파일이 실행됨" 두 줄, journal·충돌 선택 저장 방식 한 줄씩. "프론트와 백엔드 계약"의 명령·이벤트 목록은 각 단계에서), `docs/spec.md`("진입점"의 우클릭·명령줄 문장을 Windows 11 위치·Ubuntu 스크립트 메뉴와 맞춤, 작업 공간 행의 파일 줄 오른쪽 클릭·더블클릭, 설정 행 "일반"에 "폴더 오른쪽 클릭 메뉴", 충돌 화면 "이 기기에 남겨 두었다가"를 "저장소 안에 남겨 두었다가"로), `docs/design.md`(파일 줄 메뉴 항목, 신뢰 확인 대화상자의 두 목록, 키 알림 줄, "브랜드"의 작은 아이콘), `docs/roadmap.md`("반복 5" 절에 계획 014와 단계 요약), `docs/progress.md`("반복 5 단계" 체크리스트 R1~R20).
- 내용: 결정 문장은 위 표를 그대로 옮긴다. 용어표·glossary는 건드리지 않는다. CLAUDE.md는 고치지 않는다.
- 테스트: `pnpm test`(`spec-sync`·`copy-rules`), `pnpm lint`.
- 완료 기준: 문서가 서로 맞고 시험 통과. Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

### B. 5-0 배포 전 보안 묶음

R2. journal 줄 묶기와 한 번 옮기기 (`core: bind journal lines to repo and sequence`)
- 목표: 서명한 journal 줄을 다른 저장소로 옮기거나 순서를 바꿔도 Ctrl+Z에 쓰이지 않게 하고, 옛 기록은 한 번 옮겨 이어 쓴다(C1).
- 바꿀 파일: `crates/core/src/undo/journal.rs`(`VERSION = 2`, `seq`, 서명 글자에 도메인·실제 git 폴더·`seq`·앞 줄 sig, `read`/`read_all`의 사슬 검사, `append`가 뒤쪽 64KB에서 마지막 `seq`·sig를 읽음, 신규 `migrate_v1(&self) -> Result<usize>`), `crates/core/src/marker.rs`(실제 경로 함수 `real_dir`를 `pub(crate)`로 함께 씀), `crates/core/src/bootstrap.rs`(`open_project`의 백업 정리 옆에서 `migrate_v1`, 실패는 무시), `crates/core/src/recovery.rs`(작업 내역의 "믿는 줄" 판정이 새 규칙을 따름), 시험.
- 내용: 덧붙이기는 지금처럼 저장소 쓰기 잠금(실행기) 안에서만 일어난다는 것을 확인하고 주석으로 적는다. 옮기기는 `higit/journal.jsonl.tmp`에 쓰고 `sync_data` 뒤 rename. 저장소 폴더를 옮기면 경로가 바뀌어 이전 줄은 보이기만 한다(표식과 같음, 위험 절).
- 테스트: 같은 키로 서명한 줄을 다른 임시 저장소 journal에 옮기면 `read`가 버림, 줄 하나 지우기·두 줄 바꾸기·같은 줄 두 번이면 그 뒤가 "믿지 않는 줄", 4MB 넘는 파일의 뒤쪽 읽기에서 첫 줄 `seq`가 1이 아니어도 됨, v1 줄만 있는 파일을 옮기면 Ctrl+Z가 그대로 되고 서명이 안 맞는 v1 줄은 그대로 남음, v2 줄이 있으면 옮기지 않음, 옮기는 중 링크로 바뀐 `higit` 폴더 거절, `LINE_1B` 시험은 "옮긴 뒤 같은 동작"으로 바꿈, 기존 undo 시험 전부, `git-min`.
- 완료 기준: `cargo test --workspace`(두 OS), clippy.
- Windows/Ubuntu: 실제 경로는 `std::fs::canonicalize`(Windows `\\?\` 접두사까지 같은 함수로 만들고 비교하므로 그대로). rename은 Windows에서 대상이 열려 있으면 실패할 수 있다 → 실패하면 옮기지 않음. 보안: 키·서명은 로그에 없다.

R3. 키를 못 쓸 때 알리기 (`core+screens: keep the undo key and warn when it is temporary`)
- 목표: 키 파일을 덮어쓰지 않고, 저장하지 못하면 사람에게 알린다(C2).
- 바꿀 파일: `crates/core/src/undo/key.rs`(`load_or_create`: 없을 때만 만듦, 읽기 실패·길이 틀림은 임시 키, `state() -> UndoKeyState`), `crates/core/src/view/mod.rs`(`UndoKeyState`, `AppStatus.undo_key`), `crates/core/src/bootstrap.rs`(`app_status`), 생성물, `src/screens/Start/`의 알림 줄 자리, `src/screens/Workspace/Workspace.tsx`(홈 알림 줄, 이번 실행에 한 번), `src/messages`(`undoKeyTemporary`), 시험.
- 내용: 알림 줄은 012의 알림 줄 틀을 쓰고 해결 버튼이 없다(`ERROR_FIX`가 아니라 상태라서). "자세히 보기"는 앱 데이터 폴더 경로를 `redact` 뒤에.
- 테스트: 없는 폴더에서 만듦·다시 읽기 같음(기존), 길이 틀린 파일은 그대로 남고 임시 키, 읽기 권한 없는 파일(Unix `#[cfg(unix)]`, 루트가 아닐 때)은 덮어쓰지 않음, `AppStatus` 직렬화, 화면 시험(임시면 두 곳에 알림, 닫으면 이번 실행 동안 안 뜸), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 권한 시험은 Unix만, Windows는 읽기 전용 폴더로 쓰기 실패만 시험. 보안: 키 값·경로 원문은 화면에 내지 않는다.

R4. 충돌 선택을 저장소 안으로 (`core+features: store conflict picks in the git folder`)
- 목표: 직접 고친 글이 웹뷰 저장소에 남지 않고, 고른 것은 서명한 파일로 이어진다(C3).
- 바꿀 파일: `crates/core/src/marker.rs`(이름 `ConflictDraft`, 상한을 이름별로: 표식 256KB, 초안 8MB), 신규 `crates/core/src/conflict_draft.rs`(쓰기·읽기·지우기, 대상 해시·경로·곳 수 검사), `crates/core/src/view/conflict.rs`(`ConflictDraftView`, `ConflictStateView.draft`), `crates/core/src/query.rs`(`conflict_state`가 초안을 붙이고 대상이 바뀌었으면 지움), `crates/core/src/actions/conflict.rs`(마치기·그만두기가 초안을 지움), `crates/core/src/bootstrap.rs`(`Core::save_conflict_draft`), `src-tauri`(`conflict_draft_save` 네 곳), `src/lib/api.ts`, 생성물, `src/features/conflict/conflictStore.ts`(`localStorage` 대신 `api.saveConflictDraft` 디바운스, 열 때 `state.draft`로 채움), `src/screens/Conflict/Conflict.tsx`(키 만들기 삭제), 신규 `src/lib/legacyStorage.ts`(시작 때 `higit.conflict.*` 지우기)·`src/main.tsx`, `eslint.config.*`(`localStorage`·`sessionStorage` 금지, `legacyStorage.ts`만 예외), `scripts/eslint-rules.test.mjs`, 시험.
- 내용: 저장 실패는 화면을 막지 않는다(지금처럼). 다른 도구가 시작한 합치기는 초안을 쓰지 않는다(D-101 표식이 없는 경우 지금 화면이 안 열리므로 영향 없음 확인).
- 테스트(실제 git): 겹치는 합치기에서 초안 저장 → 다시 조회하면 같은 선택, 받은 zip처럼 다른 키로 서명한 초안은 무시, 합칠 대상이 바뀌면 무시하고 지움, 충돌 경로에 없는 경로·곳 번호 넘침 거절, 8MB 넘김 거절, `higit` 폴더가 링크면 쓰지 않음, 마치기·그만두기 뒤 파일 없음, `Conflict.test`(껐다 켠 흉내: 스토어를 비우고 조회로 다시 채움), ESLint 규칙 시험, `security_config`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 파일은 git 폴더 안이라 같다. 보안: 초안 글은 로그·오류에 넣지 않는다. 화면이 보낸 경로는 core가 지금 충돌 경로와 다시 맞춘다.

R5. 보내는 곳·작업 폴더 설정 신뢰 확인 (`git+core: trust check for push targets, url rewrites, http and worktree settings`)
- 목표: 받은 저장소가 코드를 엉뚱한 곳으로 올리거나 다른 폴더를 작업 폴더로 쓰게 하지 못한다(C4).
- 바꿀 파일: `crates/git/src/cli/mod.rs`(`discover`가 git 폴더·작업 폴더를 함께 읽어 `core.worktree`로 옮겨진 경우를 알림, 저장소 정책 `RepoPolicy { overrides, network_off }`와 네트워크 `GitOp` 거절), `crates/git/src/backend.rs`(`set_repo_overrides` → `set_repo_policy`, `Discovered`), 가짜 구현 둘(`core/src/executor.rs` 시험용·`core/tests/common`), `crates/git/src/failure.rs`·`error.rs`(`RemoteSettingsOff`), `crates/core/src/trust.rs`(`redirect_key`, 지문, `Executing.redirects`, `network_off`), `crates/core/src/bootstrap.rs`(`open_project`가 옮겨진 작업 폴더를 부모 폴더로 열고 덮어쓰기, `NeedsTrust.redirects`), `crates/core/src/github/mod.rs`(`redirects_transport`를 `trust::redirect_key`로 합침), `crates/core/src/error.rs`·`view/`(`ErrorCode::RemoteSettingsOff`), 생성물, `src/features/open-project/TrustDialog.tsx`(두 번째 목록), `src/features/errors/fixes.ts`(`retrust`), `src/messages`, 시험.
- 내용: 저절로 하는 원격 확인은 `RemoteSettingsOff`를 조용히 건너뛴다(012-U15의 실패 줄에 안 뜸). "믿고 열기"는 지금처럼 helper만 붙이지 않는다(D-111). 이미 `trust.json`에 결정이 있는 저장소도 보내는 곳 설정이 있으면 지문이 바뀌어 한 번 다시 묻는다.
- 테스트(실제 git, 임시 저장소, 로컬 bare 원격): `pushurl`·`pushInsteadOf`·`insteadOf`·`http.sslVerify`·`http.<url>.extraHeader`·`remote.origin.proxy`가 각각 신뢰 질문을 만들고, "끄고 열기" 뒤 fetch·push·동기화·태그 올리기가 `RemoteSettingsOff`이며 원격이 바뀌지 않음, "믿고 열기" 뒤에는 지금처럼 동작, 전역 설정의 같은 키는 묻지 않음(시험용 `HOME`), `core.worktree=<다른 임시 폴더>`인 저장소를 열면 `.git`의 부모가 저장소이고 status가 그 폴더를 보며 다른 폴더는 그대로, `extensions.worktreeConfig`의 `config.worktree`도 같음, `git worktree add`로 만든 연결된 worktree는 영향 없음, `.git`이 아닌 이름의 git 폴더는 열지 않음, 지문 변화로 다시 묻기, 대화상자 시험(두 목록, 호스트만 보임), `git-min`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`(두 OS)·clippy, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: `core.worktree` 경로 비교는 `platform::same_path`(Windows 대소문자 무시). Windows `rev-parse` 출력의 `/` 구분자를 `normalize_repo_path`로 맞춘다. 보안: 화면에 보내는 값은 키와 호스트·`redact`한 경로뿐, 주소 속 사용자 정보는 내지 않는다(D-040).

R6. Escape·Enter 판정 모으기 (`ui: move escape and enter checks into src/lib`)
- 목표: 키 판정을 `src/lib` 한 곳에 모아 한글 조합 처리를 빠뜨리지 않게 한다(D-137의 5-0 항목).
- 바꿀 파일: 신규 `src/lib/keys.ts`(`isEscape(e)`, `isSubmitEnter(e)`: `isComposing`·`keyCode 229`면 거짓), 위 일곱 파일과 012가 더한 대화상자(`rg 'e\.key === "(Escape|Enter)"'`), ESLint `no-restricted-syntax`로 `src/lib` 밖의 `e.key === "Escape"|"Enter"` 비교 금지, `scripts/eslint-rules.test.mjs`, 신규 `src/lib/keys.test.ts`.
- 테스트: 조합 중 Enter 무시, Shift+Enter는 제출 아님(글상자), 기존 화면 시험 그대로, 린트 규칙 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`, `rg` 결과가 `src/lib`뿐.
- Windows/Ubuntu: WebKitGTK의 조합 `keyCode 229`를 같은 함수가 본다. 보안: 해당 없음.

### C. OS 통합

R7. 실행 인자와 단일 인스턴스 (`app: launch folder argument and single instance`)
- 목표: `higit <폴더>`로 켜면 그 폴더가 열리고, 이미 켜져 있으면 기존 창에서 연다(C5).
- 바꿀 파일: `src-tauri/Cargo.toml`(작업 공간·app에 `tauri-plugin-single-instance`), `Cargo.lock`, 신규 `crates/core/src/launch.rs`(`parse_args`), `crates/core/src/lib.rs`, `src-tauri/src/lib.rs`(첫 플러그인으로 등록, 콜백이 `parse_args(argv, cwd)` → 창 보이기·포커스 → `open-folder` 이벤트, 첫 실행 인자를 `AppState.launch: Mutex<Option<PathBuf>>`), `src-tauri/src/commands.rs`·`bindings.rs`(`app_take_launch_folder`, `OPEN_FOLDER_EVENT` 상수)·`build.rs`·`capabilities/default.json`, `src/lib/api.ts`·`events.ts`, `src/state/useStartup.ts`(실행 인자가 있으면 마지막 프로젝트 대신), 신규 `src/features/open-project/useLaunchRequests.ts`(이벤트를 받아 도는 작업·열린 대화상자 뒤에 `open(path, "picked")`), `src/state/repoStore.ts`(`launchMissing` 알림), `src/messages`, 생성물, `src-tauri/tests/security_config.rs`, `THIRD_PARTY_NOTICES.md`(필요하면), `deny.toml`(필요하면).
- 내용: 콜백 안에서 core를 부르지 않는다(열기는 화면이 `project_open`으로, 신뢰 확인을 거치게). 창을 보일 때 012-U4의 `app_ready` 전이면 안전 타이머와 같은 함수를 쓴다. 개발 빌드와 릴리스 빌드는 같은 identifier라 서로 넘겨질 수 있다(위험 절).
- 테스트: `parse_args` 표(경로 없음, 상대·절대, `--` 뒤 `-`로 시작하는 폴더 이름, 옵션 건너뛰기, `--git-credential`이면 `None`, 끝 `"` 떼기 `C:"` → `C:`, 한글·공백 경로, 4096바이트 넘김·NUL 거절), `security_config`(새 명령 네 곳, `main.rs`에서 `--git-credential` 분기가 `run()`보다 앞, `lib.rs`의 첫 `.plugin(`이 single-instance), `useStartup.test`(실행 인자가 있으면 최근 프로젝트를 열지 않음, 없는 폴더면 시작 화면 알림), `useLaunchRequests.test`(이벤트가 오면 열기, 작업 중이면 끝난 뒤), `cargo deny check`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`, CI `cargo-deny`.
- Windows/Ubuntu: 플러그인이 Windows는 창 메시지, Linux는 D-Bus 세션 버스를 쓴다. D-Bus가 없는 세션(일부 원격 접속)에서는 단일 인스턴스가 안 될 수 있다(손 확인). 상대 경로의 기준 폴더는 두 OS 모두 플러그인이 준 `cwd`. 보안: 두 번째 실행의 인자는 같은 사용자 프로세스에서 오지만 사람이 고른 폴더와 똑같이 `open_project`·신뢰 확인을 거친다. 이벤트에는 경로만.

R8. platform: 우클릭 메뉴 등록·해제 (`platform: folder context menu for explorer and nautilus`)
- 목표: 두 OS에서 폴더 우클릭 항목을 만들고 지우고 상태를 읽는 함수를 만든다(C6, C7).
- 바꿀 파일: `crates/platform/Cargo.toml`(`cfg(windows)`에 `winreg`), 신규 `crates/platform/src/shell_menu.rs`(`ShellMenuKind { Explorer, NautilusScripts, Unsupported }`, `status(launcher, label)`, `install(launcher, label)`, `uninstall()`, 시험용 `*_at(root)`), `crates/platform/src/exe.rs`(`launcher_path()`: `APPIMAGE` 검사), `crates/platform/src/lib.rs`, `Cargo.lock`, `deny.toml`(필요하면), `THIRD_PARTY_NOTICES.md`(필요하면).
- 내용: Windows 값: `MUIVerb`=이름, `Icon`=`"<exe>",0`, `command`=`"<exe>" -- "%V"`. 실행 파일 경로에 `"`·줄바꿈이 있으면 등록하지 않는다. 지우기는 hiGit 키(`higit`)만 지운다. Ubuntu 스크립트: 파일이 링크면 손대지 않음, 표식 없는 같은 이름 파일은 덮어쓰지 않고 `Conflict` 상태, 임시 파일 + rename, 권한 0755, 작은따옴표 안 `'`는 `'\''`로. 상태는 표식·실행 파일 경로·이름이 모두 맞을 때 `installed`.
- 테스트: Windows `#[cfg(windows)]`는 `HKCU\Software\hiGit-test-<무작위>` 아래를 루트로 설치·상태·다시 설치(실행 파일 바뀜)·지우기·남의 키는 그대로, 시험 끝에 루트 삭제. Linux `#[cfg(unix)]`는 임시 폴더를 루트로 스크립트 내용(표식, 따옴표 처리: `'`·공백·한글이 든 경로), 권한, 링크 거절, 표식 없는 파일 보존, 스크립트를 `sh`로 실제 실행해 가짜 실행 파일이 받은 인자가 `--`와 경로인지(선택 있음·없음), 가짜 실행 파일을 지우면 스크립트가 스스로 지워짐. `launcher_path`는 `APPIMAGE`가 상대 경로·없는 파일이면 무시.
- 완료 기준: `cargo test --workspace`(두 OS)·clippy·`cargo deny check`.
- Windows/Ubuntu: 이 단계 전체가 OS별이라 platform 안에만 있다. Windows 11 새 메뉴에는 안 나온다(C6). 보안: 레지스트리·스크립트에 들어가는 글자는 실행 파일 경로와 검사한 이름뿐이고 명령은 `--` 뒤에 경로를 따옴표로 넘긴다(폴더 이름이 옵션으로 읽히지 않게).

R9. 우클릭 메뉴 설정과 처음 등록 (`core+screens: shell menu setting and first-run registration`)
- 목표: 설정에서 우클릭 메뉴를 켜고 끄며, 설치한 앱은 처음 켤 때 저절로 켠다(C6).
- 바꿀 파일: `crates/core/src/settings.rs`·`view/settings.rs`(`shell_menu: Option<bool>`, `#[serde(default)]`, `Copy` 유지), 신규 `crates/core/src/shell_menu.rs`(`status`, `sync(label, auto)`, `set(on, label)`: 이름 검사, platform 호출, 설정 기록), `crates/core/src/view/`(`ShellMenuView { kind, installed }`), `src-tauri/src/commands.rs`(`app_shell_menu_status`, `app_shell_menu_sync(label)`: `auto = !cfg!(debug_assertions)`을 app이 넘김, `app_shell_menu_set(on, label)`) 외 세 곳, `src/lib/api.ts`·`queries.ts`, `src/features/settings/GeneralSection.tsx`(행 "폴더 오른쪽 클릭 메뉴" `Choice` 켬·끔과 종류별 한 줄 설명, `unsupported`면 행을 숨김), 시작 때 한 번 `sync`를 부르는 곳(`src/state/useStartup.ts` 또는 `App.tsx`), `src/messages`, 생성물, `crates/core/tests/settings.rs`, 시험.
- 내용: `sync`는 Git이 없어도 돈다(Core 없이 쓰는 `app_display_hints`처럼 무상태). 실패는 조용히 로그 코드만 남긴다. 언어를 바꾸면 `sync`를 다시 불러 이름을 바꾼다.
- 테스트: 옛 `settings.json`(칸 없음) 읽기, `sync`의 세 경우(값 없음+자동, 값 없음+개발 빌드는 아무것도 안 함, `true`인데 실행 파일 바뀜 → 다시 등록, `false`는 그대로), 이름 검사(65바이트·제어 문자 거절), `set(false)` 뒤 `sync`가 다시 켜지 않음. platform은 시험용 루트를 쓰도록 core가 `ShellMenuPort` 트레이트 뒤로 받는다(가짜 구현). 화면 시험(행·설명·켬끔 호출, 언어 바꿈에 `sync`), `security_config`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 설명 문구는 `kind`로 고르므로 화면에 OS 분기가 없다. 보안: 화면이 넘기는 것은 항목 이름 하나이고 core가 검사한다.

R10. 파일 열기 검사와 명령 (`core+app: validated reveal and open-with-default-app`)
- 목표: 열린 저장소 안의 안전한 파일만 OS에 넘긴다(C8).
- 바꿀 파일: 신규 `crates/core/src/open_target.rs`(`Purpose::{Reveal, Open}`, 경로 검사, 차단표, 앞 바이트, 실행 비트), `crates/core/src/view/`(`OPEN_BLOCKED_EXTENSIONS` 상수 내보내기), `crates/core/src/error.rs`·`view/mod.rs`(`OpenRefused`·`OpenFailed`), `crates/platform/src/file.rs`(`is_executable(&Metadata)`), `crates/platform/src/path.rs`(Windows 장치 이름·`:` 구성요소 판정 `is_plain_component`), `crates/core/src/bootstrap.rs`(`Core::open_target`·`project_root`), `src-tauri/src/commands.rs`(`file_reveal(folder, path)`→`reveal_item_in_dir`, `file_open(folder, path)`→`open_path`, `project_reveal(folder)`→루트 `open_path`) 외 세 곳, `src/lib/api.ts`, 생성물, `src/features/errors/fixes.ts`(`openRefused` → "폴더에서 보기"를 부르는 새 종류 `reveal`, `openFailed` → 없음), 시험.
- 내용: 해석은 core, OS에 넘기는 일은 app의 opener 호출 한 줄이다. 결과 경로는 화면에 돌려주지 않는다. 링크 확인은 저장소 루트부터 구성요소마다 `symlink_metadata`.
- 테스트: `../`·절대 경로·Windows 드라이브 문자·`.git/config`·저장소 밖을 가리키는 링크·중간 폴더가 링크·없는 파일 거절, 차단표(대소문자, `a.pdf.exe`, 확장자 없는 ELF·`#!` 스크립트, Unix 실행 비트 텍스트 파일), 열 수 있는 것(`.txt`·`.png`·`.md`), 폴더는 `Reveal`만, Windows `CON`·`a.txt:stream` 거절(`#[cfg(windows)]`), 열리지 않은 저장소는 `NotOpened`, 차단표 상수가 바인딩에 있음, `security_config`(네 곳, 웹뷰에 `opener:` 권한 없음은 기존).
- 완료 기준: `cargo test --workspace`(두 OS)·clippy, 바인딩 최신.
- Windows/Ubuntu: Windows는 `ShellExecute`, Ubuntu는 `xdg-open`·`gio`를 opener가 쓴다. Ubuntu `reveal_item_in_dir`는 파일 관리자 D-Bus가 없으면 폴더만 연다(손 확인). 보안: 실행될 수 있는 파일을 OS 기본 앱에 넘기지 않는 것이 이 단계의 목적이다. 차단표에 없는 새 실행 형식은 남는 위험(위험 절).

R11. 파일 줄·프로젝트 메뉴 화면 (`screens: show in folder, open with default app and open folder location`)
- 목표: 고친 파일을 폴더에서 보고 기본 앱으로 열며, 프로젝트 폴더를 연다(D-161).
- 바꿀 파일: `src/screens/Workspace/FileRowMenu.tsx`(항목 "폴더에서 보기"·"기본 앱으로 열기", 지운 파일·차단표 확장자면 `aria-disabled`와 까닭 툴팁), `src/screens/Workspace/ChangedFiles.tsx`(줄 더블클릭 → 기본 앱으로 열기, 막힌 파일은 오류 토스트와 "폴더에서 보기" 해결 버튼), 012-U23의 `src/screens/Workspace/ProjectMenu.tsx`("폴더 위치 열기"), 신규 `src/features/files/useOpenFile.ts`(차단표 판정 `src/lib/openable.ts`와 api 호출), `src/messages`, 시험.
- 내용: 키보드는 012-U19의 줄 메뉴 키로 같은 항목에 닿는다(새 단축키 없음). 더블클릭의 첫 클릭은 지금처럼 파일 고르기다.
- 테스트: 메뉴 항목·꺼진 까닭(지운 파일, `.exe`, `.sh`), 더블클릭이 `fileOpen`을 한 번 부름, 거절 오류에 "폴더에서 보기"가 `fileReveal`을 부름, 프로젝트 메뉴 항목, `openable` 표가 core 상수와 같음(바인딩에서 읽음), 기존 `ChangedFiles`·`ProjectMenu` 시험.
- 완료 기준: `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: 차이 없음(OS 일은 R10). 보안: 화면은 상대 경로만 보낸다.

R12. 창 크기·위치 기억 (`app: remember window size and position`)
- 목표: 다시 켜도 창이 같은 크기·자리에 뜨고 화면 밖으로 나가지 않는다(C9).
- 바꿀 파일: `src-tauri/Cargo.toml`(`tauri-plugin-window-state`), `Cargo.lock`, `src-tauri/src/lib.rs`(플러그인 등록과 깃발, `setup`에서 되살린 뒤 화면 안 맞추기), 신규 `src-tauri/src/window_fit.rs`(순수 함수 `fit(rect, monitors, min) -> rect`와 Tauri 창에 적용하는 얇은 함수), `src-tauri/tests/security_config.rs`(웹뷰 권한에 `window-state:`가 없음, 최소 크기 800×560 그대로), `deny.toml`·`THIRD_PARTY_NOTICES.md`(필요하면).
- 내용: 012-U4의 숨긴 창이 보이기 전에 되살린다. 저장은 플러그인 기본(닫을 때). 최소 크기는 바꾸지 않는다.
- 테스트: `fit` 표(모니터 밖·절반만 겹침·두 모니터 사이·모니터보다 큼·최소보다 작음·배율 다른 모니터), `security_config`, `cargo deny check`.
- 완료 기준: `cargo test --workspace`, clippy, CI `cargo-deny`.
- Windows/Ubuntu: Ubuntu 22.04 Wayland 세션에서는 창 위치를 앱이 읽고 정할 수 없어 크기·최대화만 남는다(손 확인, X11은 위치도). Windows는 배율이 다른 두 모니터에서 손 확인. 보안: 상태 파일은 숫자뿐이고 받은 값은 `fit`이 다시 맞춘다.

R13. Ubuntu 시스템 글꼴 pt (`platform: read the gnome interface font size`)
- 목표: Ubuntu의 글꼴 크기 설정을 본문 크기에 반영한다(C10).
- 바꿀 파일: `crates/platform/src/display.rs`(Linux에서 `font-name`을 `color-scheme`과 함께 읽음, `parse_font_pt`, `body_px_from_pt`), `docs/architecture.md`의 OS별 처리 행은 R1에서.
- 테스트: `parse_font_pt` 표(`'Ubuntu 11'`·`'Cantarell 11.5'`·`'Noto Sans CJK KR Bold 12'`·숫자 없음·`0`·`100`), `body_px_from_pt`(11 → 15, 9 → 13 하한, 16 → 20 상한), 실패 기본 15, 기존 display 시험. 화면은 `body_px`를 이미 쓰므로 바뀌는 곳이 없음을 `fontScale.test`로 확인.
- 완료 기준: `cargo test --workspace`(두 OS).
- Windows/Ubuntu: Windows는 그대로 14. 보안: 출력은 숫자 파서로만, 절대 경로·시간 제한 그대로.

R14. 비교 칸 끝 접힌 줄 (`core+features: fold unchanged lines after the last hunk`)
- 목표: 마지막 묶음 뒤 생략된 줄도 "변경 없는 N줄 · 펼치기"로 보인다(C11).
- 바꿀 파일: `crates/core/src/view/diff.rs`(`new_line_count`), `crates/core/src/query.rs`(작업 폴더·버전 블롭 줄 세기, 상한 1MB), 필요하면 `crates/git/src/backend.rs`·`cli/`(`blob_at`, 012-U26에 없을 때만, 가짜 구현 갱신), 생성물, `src/features/diff/model.ts`(끝 접힌 줄), `DiffView.tsx`, 시험.
- 테스트: 줄 세기(끝 줄바꿈 있음·없음, CRLF, 빈 파일, 1MB 넘김 → `None`, 링크는 읽지 않음), 버전 안 비교, 지운 파일 `None`, `model.test`(끝 접힌 줄 수, 마지막 묶음이 끝까지면 없음, 펼치기 뒤 사라짐), `DiffView.test`, 바인딩 최신, `git-min`.
- 완료 기준: `cargo test --workspace`, `pnpm test`·`pnpm lint`·`pnpm typecheck`.
- Windows/Ubuntu: CRLF 파일도 `\n` 수로 센다(git의 줄 번호와 같음). 보안: 읽기 전용, 상한 안, 링크 안 따라감.

### D. 설치 파일과 배포

R15. 작은 아이콘과 아이콘 묶음 (`brand: simple 16 and 24 px icons`)
- 목표: 작업 표시줄·앱 목록의 16·24px이 단순한 아이콘이다(C12).
- 바꿀 파일: `assets/brand/app-icon-small.svg`·`app-icon-16.png`·`app-icon-24.png`(사용자 파일이 있으면 그것), 신규 `scripts/brand-icons.mjs`(ICO 읽기·쓰기, 16·24 항목 바꾸기, `icons/16x16.png`·`24x24.png` 복사), 신규 `scripts/brand-icons.test.mjs`(검사 스크립트 등록 방식은 기존 `scripts/*.test.mjs`와 같게), `src-tauri/icons/*`(다시 만든 것), `src-tauri/tauri.conf.json`(`bundle.icon`에 `icons/16x16.png`·`icons/24x24.png`), `package.json`(`"brand:icons"` 스크립트), `docs/design.md` "브랜드"는 R1, `docs/research/img/brand/icon-sizes.png` 다시 만듦.
- 내용: PNG 렌더는 한 번만(`rsvg-convert -w 16`·`-w 24`, 방법을 스크립트 머리 주석에). 저장소에 래스터 도구 의존을 더하지 않는다.
- 테스트: `icon.ico`에 16·24·32·48·64·256 항목이 있고 16·24 바이트가 작은 PNG와 같음, `tauri.conf.json` 아이콘 목록의 파일이 모두 있고 크기가 이름과 같음(PNG 머리 읽기), `pnpm test`.
- 완료 기준: `pnpm test`·`pnpm lint`, `pnpm tauri build --no-bundle`.
- Windows/Ubuntu: Windows 작업 표시줄 100·150%는 24·32px 항목, Ubuntu hicolor 16·24는 패널·창 목록이 쓴다(손 확인). 보안: 해당 없음.

R16. 설치 파일 설정 (`bundle: msi, deb and appimage settings`)
- 목표: 세 설치 파일이 맞는 이름·의존·메뉴·라이선스로 만들어진다(C13).
- 바꿀 파일: `src-tauri/tauri.conf.json`(`bundle.targets`, `publisher`·`shortDescription`·`category`, `resources`, `windows.wix`(`language`, `upgradeCode`, `fragmentPaths`, `componentRefs`), `linux.deb`(`depends`, `section`, `desktopTemplate`, `files`), `linux.appimage`), 신규 `packaging/windows/shell-menu-cleanup.wxs`, 신규 `packaging/linux/higit.desktop`, `packaging/windows/README.txt`(설치 파일 기준으로 다시 씀: SmartScreen "추가 정보 → 실행", 관리자 확인, 우클릭 위치), 신규 `packaging/linux/README.txt`(`sudo apt install ./higit_*.deb`, AppImage와 `libfuse2`), 신규 `scripts/version-sync.test.mjs`(세 버전 같음).
- 내용: 서명 설정은 넣지 않는다(R19). `upgradeCode`는 한 번 만든 GUID를 그대로 둔다.
- 테스트: `version-sync`, `tests/security_config.rs`에 "`bundle.targets`가 셋, `.desktop`에 `MimeType` 없음, WiX 조각이 HKCU `Directory\shell\higit`·`Directory\Background\shell\higit`만 지움" 시험 추가, 로컬 `pnpm tauri build --bundles deb,appimage`(Ubuntu)로 파일이 생김.
- 완료 기준: `cargo test --workspace`, `pnpm test`, Ubuntu에서 deb·AppImage 생성(.msi는 R17의 CI가 확인).
- Windows/Ubuntu: 이 단계는 묶음 설정이라 코드 분기가 없다. WiX는 Windows에서만, deb·AppImage는 Linux에서만 만들어진다. 보안: 묶음에 비밀이 없다(client id는 빌드 환경 변수).

R17. CI 설치 파일 확인 (`ci: build and smoke-test installers`)
- 목표: main push와 수동 실행마다 설치 파일이 만들어지고 깔리는지 CI가 본다.
- 바꿀 파일: `.github/workflows/ci.yml`(새 작업 `bundle`: `build-and-test` 뒤, `pull_request`에서는 돌지 않음, 두 OS. Ubuntu: `pnpm tauri build --bundles deb,appimage`, `dpkg-deb -I`(Depends에 `git`)·`dpkg-deb -c`(`usr/bin/higit`, `.desktop`, hicolor 16·24·32·128·256, 라이선스 파일), `desktop-file-validate`, `ubuntu:22.04` 컨테이너에서 `apt-get install -y ./*.deb`. Windows: `pnpm tauri build --bundles msi`, `msiexec /i … /qn` → `C:\Program Files\hiGit\higit.exe`와 `THIRD_PARTY_NOTICES.md` 있음 → 시험용으로 HKCU 우클릭 키를 만든 뒤 `msiexec /x … /qn` → 실행 파일과 키가 없어짐. 두 OS 결과물을 14일 올림. 액션은 이미 쓰는 SHA 고정 액션만. 기존 Windows 포터블 결과물 작업은 그대로).
- 테스트: CI 자체. 실패 메시지가 무엇을 못 찾았는지 말한다.
- 완료 기준: `gh run list`로 main의 `bundle` 작업 초록.
- Windows/Ubuntu: Windows 러너는 관리자라 MSI 설치가 된다. AppImage는 러너에서 실행하지 않고 만들어지는지만 본다(FUSE). 보안: 작업 권한 `contents: read` 그대로, 비밀을 쓰지 않는다.

R18. 파서 무작위 시험 (`git: random input tests for machine-readable parsers`)
- 목표: `-z`·porcelain 파서가 이상한 바이트에 패닉하지 않는다(C15, D-038).
- 바꿀 파일: 신규 `crates/git/src/cli/random_tests.rs`(고정 씨앗 xorshift, 파서마다 무작위 바이트·실제 출력 표본을 자르고 뒤섞은 입력 2만 번), 대상은 status·log·diff·branch·reflog·stash·config(`local_config`)·`ls-tree`·rebase 할 일 목록 파서(`rg "fn parse_" crates/git/src`로 목록 확정), `crates/core`의 journal 줄 읽기와 `remote_urls` 결과 해석도 같은 방식으로 한 번.
- 테스트: 위 시험이 패닉 없이 `Ok`나 `Err`로 끝나고 2초 안. CI에서 두 OS.
- 완료 기준: `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음. 보안: 이 단계가 D-038 "파서 퍼징"의 시험판 몫이다.

R19. 릴리스 워크플로·서명 자리·SECURITY.md (`ci: draft release with checksums and a signing slot`)
- 목표: 태그를 올리면 서명 없는 시험판 초안 릴리스가 체크섬과 함께 생기고, 서명은 비밀만 넣으면 켜진다(C14, C15).
- 바꿀 파일: 신규 `.github/workflows/release.yml`(트리거 `push: tags: ['v*']`와 `workflow_dispatch`, 기본 권한 `contents: read`. 작업: `version`(태그 = 세 버전), `build`(두 OS, `HIGIT_GITHUB_CLIENT_ID` 환경 비밀, Windows 서명 단계는 `env.WINDOWS_SIGN_COMMAND != ''`일 때 `src-tauri/tauri.sign.json`을 만들어 `--config`로), `checksums`(`sha256sum` → `SHA256SUMS`, `LINUX_GPG_KEY`가 있으면 `SHA256SUMS.asc`), `publish`(environment `release`, `contents: write`, `gh release create --draft`, 비밀이 없으면 이름에 "서명 없는 시험판")), 신규 `SECURITY.md`(지원 버전, GitHub 비공개 취약점 신고, 응답 기간, 범위), 신규 `packaging/README.md`(만드는 법, 체크섬 확인 `sha256sum -c`·`Get-FileHash`, 서명 비밀 이름과 넣는 법, signtool·Azure Trusted Signing 예시 명령 형태), `.gitignore`(`src-tauri/tauri.sign.json`), `scripts/version-sync.test.mjs`(태그 비교 함수).
- 내용: 액션은 이미 SHA로 고정한 것만 쓰고 릴리스 만들기는 러너의 `gh`로 한다(D-038). `pull_request_target` 없음. 서명 명령은 저장소에 쓰지 않고 비밀에서만 온다.
- 테스트: `version-sync`(태그 `v0.1.0` 맞음·`v0.1.1` 틀림), 워크플로를 `workflow_dispatch`로 한 번 돌려 비밀 없이 서명 단계가 건너뛰어지고 초안이 생기는지(사람이 지움), `actionlint`가 있으면 로컬 검사.
- 완료 기준: 수동 실행 한 번 초록, 초안 릴리스에 .msi·.deb·AppImage·`SHA256SUMS`.
- Windows/Ubuntu: 서명 자리가 OS마다 다르다(Authenticode·GPG). 보안: 쓰기 권한은 `publish` 작업 하나, 비밀은 environment `release`에서만 읽힌다(security.md "비밀"). 서명 없는 Windows 설치 파일은 SmartScreen 경고가 뜬다(사람이 할 일·위험).

### E. 마무리

R20. 문서·점검·사람이 할 일 (`docs: wrap up iteration 5`)
- 목표: 문서를 구현에 맞추고 규칙 점검을 돌린다.
- 바꿀 파일: `docs/architecture.md`(명령·이벤트 목록, OS별 처리, 보안 표), `docs/spec.md`, `docs/design.md`, `docs/roadmap.md`("반복 5 완료", 넘긴 것: 코드 서명·cargo-fuzz·updater·D-051(C16 답에 따라)·nautilus-python·Windows 11 새 메뉴), `docs/progress.md`(체크 R1~R20, "사람이 할 일" 새 항목), `.claude/agents/code-analyst.md`(점검 줄: 새 설정 키 중 보내는 곳·작업 폴더를 바꾸는 것은 `trust::redirect_key`, 네트워크 `GitOp`는 저장소 정책을 거침, 화면의 `localStorage` 금지, 파일을 OS에 넘기는 것은 `open_target`을 거침, 우클릭·단일 인스턴스 OS 코드는 platform에만, 새 플러그인은 웹뷰 권한 없이), `docs/research/img/release/`(설치 화면·우클릭 메뉴·신뢰 확인 두 목록 그림, 개인 정보 없음).
- 내용: code-analyst "반복 끝 점검"(계층·용어·보안 점검표) 뒤 위반을 고치고 커밋. CI 두 OS·`git-min`·`cargo-deny`·`bundle`을 `gh run list`로 확인.
- 테스트: `pnpm check:full`, CI.
- 완료 기준: "완료 기준 (반복 전체)"이 모두 자동 시험이나 "사람이 할 일"에 있다. Windows/Ubuntu: Windows 확인은 사람이 할 일. 보안: 그림에 개인 정보 없음.

## 위험
- **journal 묶기와 저장소 옮기기**: 저장소 폴더를 옮기거나 이름을 바꾸면 실제 경로가 바뀌어 이전 Ctrl+Z 기록이 "보이기만" 된다(D-102 표식과 같은 감수). 같은 PC에서 폴더를 옮기는 사람이 많으면 다음 반복에서 "옮긴 저장소 다시 묶기"를 정한다. v1 → v2 옮기기는 한 번뿐이라 그때 실패하면 옛 기록은 되돌리기에 못 쓴다.
- **보내는 곳 설정의 오탐**: 회사 프록시(`http.proxy`)나 미러(`insteadOf`)를 저장소 로컬에 둔 사람은 처음 한 번 질문을 본다. "믿고 열기"면 지금과 같다. 전역 설정은 묻지 않는다.
- **`core.worktree` 무력화**: 일부 고급 사용자가 일부러 쓰는 설정이다. 이번에는 늘 무시하고 열린 폴더를 작업 폴더로 쓴다. 불만이 있으면 사람에게 묻는다.
- **단일 인스턴스와 개발 빌드**: 개발 빌드와 설치한 앱이 같은 identifier라 한쪽이 켜져 있으면 다른 쪽 실행이 넘겨진다. 개발 때는 설치한 앱을 끄고 쓴다(progress에 적음). Linux D-Bus 세션이 없으면 플러그인이 동작하지 않을 수 있다.
- **helper와 설치 경로**: Windows GUI 실행 파일의 표준 입출력(009 위험)과 `C:\Program Files\hiGit\` 공백 경로, AppImage의 마운트 경로에서 helper가 도는지는 설치한 앱으로 손 확인해야 한다. 안 되면 별도 console 실행 파일을 묶음에 더하는 결정을 사람에게 묻는다.
- **우클릭 메뉴 한계**: Windows 11은 "더 많은 옵션 표시" 안에만 나오고, Ubuntu는 "스크립트" 하위 메뉴다. 처음 켤 때 저절로 등록하는 것은 사람에게 확인한다. 포터블 exe를 지우면 Windows 메뉴 항목이 남아 누르면 Windows 오류 창이 뜬다(MSI 제거는 지움, 앱 설정에서 끌 수 있음). MSI 제거는 지우는 사용자의 HKCU만 지운다.
- **파일 열기 차단표**: 확장자·앞 바이트·실행 비트로 막아도 새 실행 형식이나 연결 프로그램이 스크립트를 돌리는 경우가 남는다. 표는 core 한 곳이라 고치기 쉽다. 차단 대상은 "폴더에서 보기"로는 열 수 있다(파일 관리자에서 사람이 직접 실행하는 것은 막지 않음).
- **창 위치**: Wayland에서는 위치가 남지 않는다. 플러그인 파일은 Windows에서 로밍 앱 설정 폴더에 생겨 다른 PC로 따라갈 수 있다(`fit`이 화면 안으로 맞춤).
- **설치 파일**: 서명이 없어 Windows SmartScreen 경고가 뜨고 일부 백신이 막을 수 있다. MSI는 관리자 확인이 뜬다. AppImage는 Ubuntu 22.04에 `libfuse2`가 없으면 안 뜬다. .deb의 WebKitGTK 의존은 Tauri가 붙이고 `git`은 이 계획이 더한다. 설치 파일 크기(AppImage는 WebKitGTK를 묶어 큼)는 R17 결과로 progress에 적는다.
- **새 의존성 셋**: `cargo deny`의 라이선스·중복 경고가 늘 수 있다(중복은 경고). 라이선스가 허용 목록 밖이면 그 단계를 멈추고 대안(C7·C9의 대안)을 사람에게 묻는다.
- **012·013 가정**: R3·R5·R10·R11은 012의 알림 줄 틀·`ERROR_FIX`·파일 줄 키보드·프로젝트 메뉴, R7·R12는 012-U4의 숨긴 창, R5는 013이 더한 원격 쓰기를 가정한다. 구현 첫날 확인하고 다르면 progress에 적는다.
- **규모**: 20단계. B(5-0)는 배포 전에 반드시, C는 서로 독립, D는 R15 → R16 → R17 → R19 순서. 시간이 모자라면 R13·R14·R12 순으로 다음 반복에 넘기고 R20에 적는다.
- 구현 중 알게 된 것은 `docs/progress.md`에 적고, 결정이 필요하면 decision.md에 "(자동 결정)"으로 올린 뒤 이어 간다.
- code-analyst 검토 때 먼저 볼 곳: R2(사슬 검사와 옮기기의 원자성, 잠금 안 덧붙이기), R4(초안 서명 범위·경로 재검사), R5(네트워크 `GitOp` 목록이 빠짐없는지, `discover`의 worktree 판정, 연결된 worktree 예외), R7(helper 분기 순서, 콜백에서 core를 부르지 않음), R8(레지스트리·스크립트 값의 따옴표, 남의 파일 보존), R10(링크·구성요소 검사, 차단표), R12(웹뷰 권한 없음), R19(권한과 비밀 범위).

## 사람이 할 일
`docs/progress.md`의 "사람이 할 일"에 R20에서 옮긴다. **사람만 줄 수 있는 것**은 아래 "사람에게 묻는 것"과 인증서·비밀이고, 나머지는 실제 컴퓨터에서 확인할 것이다.
- **Windows 설치(10과 11)**: 초안 릴리스의 .msi를 받아 `Get-FileHash`로 `SHA256SUMS`와 맞는지 보고 설치한다. SmartScreen "추가 정보 → 실행"과 관리자 확인, 시작 메뉴 바로 가기와 아이콘(16·24·32·48px, 100·150% 배율), 처음 켤 때 우클릭 메뉴가 저절로 생기는지. 탐색기에서 폴더·폴더 빈 곳·드라이브 루트(`C:\`)·한글과 공백이 든 폴더를 오른쪽 클릭 "Git으로 관리"(Windows 11은 "더 많은 옵션 표시"). 앱이 켜진 채로 다른 폴더에서 다시 눌러 기존 창이 앞으로 나오고 그 폴더가 열리는지. 설정에서 끄고 켜기. 제거하면 메뉴·파일이 사라지는지. 설치 경로(`C:\Program Files\hiGit\`)에서 GitHub 비공개 저장소 https 동기화가 GCM 창 없이 되는지(helper).
- **Ubuntu 설치(22.04, 가능하면 24.04도)**: `sudo apt install ./higit_*.deb`로 `git`까지 깔리는지, 앱 목록 아이콘, 처음 켤 때 파일 앱의 오른쪽 클릭 "스크립트 > Git으로 관리"가 생기는지(폴더 위·빈 곳), 한글·공백 폴더, 앱이 켜진 채로 다시 눌러 기존 창에서 열리는지, `higit .`·`higit ~/프로젝트`. AppImage는 `libfuse2` 없이·있이 실행, 우클릭 메뉴가 AppImage 경로를 가리키는지, AppImage를 옮긴 뒤 다시 켜면 메뉴가 고쳐지는지. deb 제거 뒤 메뉴를 누르면 스크립트가 사라지는지. X11과 Wayland 세션 둘 다.
- **5-0 보안**: 받은 zip처럼 `remote.origin.pushurl`을 다른 주소로 바꾼 저장소를 열어 질문의 두 번째 목록, "끄고 열기" 뒤 동기화가 막히고 "믿고 다시 열기"가 되는지. `core.worktree`를 다른 폴더로 둔 저장소를 열어 그 폴더가 바뀌지 않는지. 합치기 충돌에서 직접 고친 뒤 앱을 껐다 켜도 남는지, 웹 검사기(개발 빌드)의 `localStorage`가 비어 있는지. 앱 데이터 폴더를 읽기 전용으로 만들고 켜서 키 알림이 뜨는지(끝나면 되돌림).
- **파일 열기**: 고친 `.txt`·`.png`·`.md`를 더블클릭과 메뉴로 열기, `.exe`·`.bat`·`.sh`·`.desktop`·실행 비트 파일은 꺼져 있고 "폴더에서 보기"는 되는지, 파일 관리자가 그 파일을 고른 채로 열리는지(Ubuntu Nautilus, Windows 탐색기), 프로젝트 메뉴 "폴더 위치 열기".
- **창·글꼴·비교**: 창 크기·자리·최대화를 바꾸고 다시 켜기, 모니터 두 개에서 바깥 모니터에 두고 끈 뒤 모니터를 빼고 켜기, 배율이 다른 모니터. Ubuntu "설정 > 접근성"이나 `gsettings set org.gnome.desktop.interface font-name 'Ubuntu 13'` 뒤 앱 글자 크기(끝나면 되돌림). 긴 파일 중간만 고쳐 비교 칸 끝 "변경 없는 N줄 · 펼치기".
- **작은 아이콘**: R15가 만든 16·24px 아이콘이 작업 표시줄·창 목록에서 알아보기 쉬운지.
- **사람에게 묻는 것**: ① C16 — 명령 인자를 저장소 ID로 바꾸는 일(D-051)을 첫 정식 배포 전으로 미뤄도 되는지. ③ 처음 켤 때 우클릭 메뉴를 묻지 않고 켜도 되는지, Ubuntu "스크립트" 하위 메뉴로 충분한지(맨 위 항목이면 nautilus-python과 `python3-nautilus` 의존). ④ MSI가 관리자 확인이 뜨는 Program Files 설치여도 되는지(아니면 NSIS 사용자 단위 설치), MSI 언어 한국어 하나로 되는지, Windows PATH에 `higit`을 넣을지. ⑤ 코드 서명 인증서(Authenticode 또는 Azure Trusted Signing)와 Linux GPG 키를 언제 마련할지, 그때 넣을 GitHub Environment 비밀 이름(`WINDOWS_SIGN_COMMAND`, `LINUX_GPG_KEY`, `HIGIT_GITHUB_CLIENT_ID`). ⑥ 앱 자체 라이선스(설치 파일·deb `copyright`에 적을 것). ⑦ 012에서 넘어온 OS 알림·WebView2 브라우저 키 platform 함수를 이 계획 뒤에 할지.

## 단계 순서 요약
| 단계 | 내용 | 크기 | 크레이트·화면 | 커밋 메시지 제안 | CI 변경 |
| --- | --- | --- | --- | --- | --- |
| R1 | 결정 C1~C16·문서 | 소 | docs | `docs: add os integration and release decisions and plan 014` | 없음 |
| R2 | journal 줄 묶기·한 번 옮기기 | 중 | core | `core: bind journal lines to repo and sequence` | 없음 |
| R3 | 키 덮어쓰기 금지·임시 키 알림 | 소 | core, 화면 | `core+screens: keep the undo key and warn when it is temporary` | 없음 |
| R4 | 충돌 선택을 git 폴더 서명 파일로, `localStorage` 금지 | 중 | core, app, 화면 | `core+features: store conflict picks in the git folder` | 없음 |
| R5 | 보내는 곳·작업 폴더 설정 신뢰 확인, 원격 작업 막기 | 대 | git, core, 화면 | `git+core: trust check for push targets, url rewrites, http and worktree settings` | 없음 |
| R6 | Escape·Enter 판정을 `src/lib`로 | 소 | 화면 | `ui: move escape and enter checks into src/lib` | 없음 |
| R7 | 실행 인자·단일 인스턴스 | 중 | core, app, 화면 | `app: launch folder argument and single instance` | `cargo-deny`가 새 의존성 검사 |
| R8 | platform 우클릭 메뉴 등록·해제 | 중 | platform | `platform: folder context menu for explorer and nautilus` | Windows 러너가 레지스트리 시험 |
| R9 | 우클릭 메뉴 설정·처음 등록 | 중 | core, app, 화면 | `core+screens: shell menu setting and first-run registration` | 없음 |
| R10 | 파일 열기 검사와 명령 | 중 | core, platform, app | `core+app: validated reveal and open-with-default-app` | 없음 |
| R11 | 파일 줄·프로젝트 메뉴 화면 | 소 | 화면 | `screens: show in folder, open with default app and open folder location` | 없음 |
| R12 | 창 크기·위치 기억 | 소 | app | `app: remember window size and position` | `cargo-deny`가 새 의존성 검사 |
| R13 | Ubuntu 글꼴 pt | 소 | platform | `platform: read the gnome interface font size` | 없음 |
| R14 | 비교 칸 끝 접힌 줄 | 소 | core, 화면 | `core+features: fold unchanged lines after the last hunk` | 없음 |
| R15 | 16·24px 작은 아이콘 | 소 | assets, scripts | `brand: simple 16 and 24 px icons` | 없음 |
| R16 | 설치 파일 설정 | 중 | 묶음 설정, packaging | `bundle: msi, deb and appimage settings` | 없음 |
| R17 | CI 설치 파일 만들기·설치 확인 | 중 | CI | `ci: build and smoke-test installers` | 새 작업 `bundle`(main push·수동) |
| R18 | 파서 무작위 시험 | 소 | git, core | `git: random input tests for machine-readable parsers` | 없음 |
| R19 | 릴리스 워크플로·서명 자리·SECURITY.md | 중 | CI, docs | `ci: draft release with checksums and a signing slot` | 새 워크플로 `release.yml` |
| R20 | 문서·점검·사람이 할 일 | 소 | docs | `docs: wrap up iteration 5` | 없음 |

순서의 까닭: R1이 모든 단계 앞. roadmap대로 5-0 보안 묶음(R2~R6)을 맨 앞에 두어 설치 파일이 나가기 전에 끝낸다(R2 → R3은 같은 키 파일, R4는 R2의 서명 방식을 쓰고, R5는 독립). OS 통합은 실행 인자(R7)가 우클릭 메뉴(R8·R9)의 바탕이고, 파일 열기(R10 → R11)·창(R12)·글꼴(R13)·비교 칸(R14)은 서로 독립이다. 배포는 아이콘(R15)과 묶음 설정(R16)이 있어야 CI가 설치 파일을 확인하고(R17), 파서 시험(R18)은 릴리스(R19) 전 D-038 몫이다. 각 단계는 CI가 초록이어야 다음으로 간다.

## 사용자 답과 덧붙인 단계 (2026-10-16, 위 단계 설명보다 우선)
R1은 아래를 사용자 결정으로 올린다("(자동 결정)"을 붙이지 않는다). 이 계획은 code-analyst 검토 없이 구현하고, 지금 코드와 다른 곳은 그 자리에서 맞추며 커밋 본문에 한 줄 적는다.
- **C16(D-051)**: 명령 인자를 저장소 ID로 바꾸는 일은 첫 정식 배포 전으로 미룬다. 시험판은 지금 방식(사용자 결정, 상태 확정).
- **우클릭 메뉴**: 처음 켤 때 묻지 않고 등록하고 설정에서 끌 수 있게 한다. Ubuntu는 Nautilus 스크립트로 충분하다.
- **Windows 설치(R16 바뀜)**: MSI 대신 관리자 확인이 없는 사용자 단위 NSIS. 설치 언어는 한국어와 English. PATH에는 넣지 않는다. Linux는 deb·AppImage 그대로.
- **서명**: 코드 서명 인증서와 GPG 키는 정식 배포 전에 사용자가 마련한다. 지금은 계획대로 서명 자리만 둔다(R19).
- **앱 라이선스**: MIT(저작권자 "hiGit contributors", 2026). 저장소 루트 `LICENSE`와 `package.json`·모든 크레이트 `Cargo.toml`의 `license = "MIT"`는 4b 도중(2026-10-16)에 먼저 넣었다. 글꼴 OFL 원문은 `assets/fonts/licenses/`에 두고 설치 파일에 함께 넣는다(D2Coding은 012-U30이 글꼴과 함께 넣는다).
- **git은 설치 파일에 넣지 않는다**(넣게 되면 GPL 고지가 따로 필요하다, decision에 적음).
- **R15 작은 아이콘**: 사용자가 그린 그림으로 이미 했다(3d67c55, D-163 덧붙임). 이 단계는 건너뛰고 progress에 적는다.
- **앱 이름**: hiGit 그대로. Git 상표 정책 확인은 사람이 할 일(progress "사람만 줄 수 있는 것"). GitHub client id가 없으면 GitHub 연결은 "준비 안 됨"인 채로 배포한다.
- 012에서 넘어온 OS 알림은 하지 않는다(D-172). WebView2 브라우저 키는 012-U3b.

덧붙인 단계(번호는 끼워 넣는 자리):
- **R16b. 서드파티 라이선스 고지 만들기** (`bundle: generate third-party notices from cargo and pnpm`): `cargo metadata`(또는 `cargo about`)와 `pnpm licenses list --json`에서 실제로 앱에 들어가는 의존성의 이름·버전·라이선스·원문을 모아 `THIRD_PARTY_NOTICES`(설치 파일용 txt/html)를 만드는 스크립트(`scripts/notices.mjs`), 글꼴 OFL 원문(`assets/fonts/licenses/`)과 앱 `LICENSE`를 함께 묶음 자원(`bundle.resources`)으로 넣는다. CI가 만들어진 고지가 의존성 목록과 맞는지 확인한다(새 의존성이 고지에 빠지면 실패). 시험: 스크립트가 알려진 의존성 몇 개와 라이선스를 담는지.
- **R16c. 설정 "정보"의 "오픈 소스 라이선스"와 "사용법" 줄** (`screens: open source licenses and user guide in settings`): 정보 구역에 글자 버튼 "오픈 소스 라이선스"(묶음 자원의 고지를 OS 기본 앱이나 앱 안 창으로 연다, 경로는 core·platform이 정하고 화면은 인자를 넘기지 않음)와 "사용법"(묶음에 넣은 `docs/guide/index.html`, 앱 언어가 English면 `en.html`)을 둔다.
- **R20 바뀜(반복 끝 점검)**: code-analyst 규칙·보안 점검은 이번 한 번만(릴리스 직전), 보안 항목 위주로 짧게. 이어서 릴리스 직전 스크린샷(홈·버전 기록·설정, Ubuntu)과 사용법 문서.
- **R20b. 사용법 문서** (`docs: user guide as one html file`): `docs/guide/index.html` 한 파일(CSS·그림을 모두 안에, 인터넷 없이 열림). 읽는 사람은 Git을 모르는 사람이고 화면 낱말은 앱과 같게, Git 원래 이름은 처음 나올 때 한 번만 옆에. 차례: 설치하기와 Windows 경고 창 넘기기, 처음 켜기와 이름·이메일, 폴더 열기와 복제, 버전 저장, 비교 보기, 버전 기록 노선도, 되돌리기와 작업 내역, 브랜치 만들기와 바꾸기, 합치기와 충돌 해결, 임시 보관, GitHub 연결과 동기화, 검토 요청, 설정, 단축키 표, 자주 묻는 것. 장마다 짧은 설명 한두 문장, 따라 하는 순서, 그림 한 장(Ubuntu 실제 화면을 줄여 넣음). 괄호 보충 없이 해요체. design.md 색·글꼴, 라이트·다크, 왼쪽 차례(좁으면 위로). 시간이 되면 `docs/guide/en.html`.
- **R21. v0.1.0 시험판** (`release: v0.1.0 pre-release`): 버전을 0.1.0으로 맞추고(`package.json`·`Cargo.toml`·`tauri.conf.json`), R19 워크플로로 설치 파일·체크섬과 `hiGit-guide.html`(사용법 문서)을 GitHub 릴리스에 "pre-release"로 올린다. 서명은 자리만. 릴리스 본문은 짧게(설치 방법, Windows 경고 창, 알려진 한계, 사람이 확인할 것).
