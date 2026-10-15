# 003 시작 화면
- 2026-10-15 확정: D-031~D-033, Q1~Q15, 추가 두 가지(아래 "결정 결과").
- 관련 결정: D-031, D-032, D-033 (제안, 이 계획과 함께 추가). 기대는 확정 결정: D-002, D-003, D-004, D-006, D-009, D-010, D-019, D-024, D-025, D-026, D-027, D-029, D-030. D-017(제안)이 확정되어야 하는 부분은 따로 표시한다.
- 관련 명세: spec.md "화면 구성"의 "시작 화면" 행과 "진입점" 절(이 계획과 함께 고침, 제안 상태 표시). 용어표는 건드리지 않는다.
- 참고 자료(결정 아님): `docs/research/start-screen.md`, `docs/research/img/start-normal.png`, `start-first.png`(다크는 `-dark`).
- 앞선 계획: 001(반복 1a 구현 완료, 1b 남음), 002(용어·디자인 개편 완료).

## 목표
앱을 다시 켜면 어제 하던 프로젝트가 바로 열린다. 작업 공간의 프로젝트 메뉴에서 "다른 프로젝트 열기"를 누르면 시작 화면이 나오고, 마지막으로 하던 프로젝트가 크게 보이며 "이어서 하기" 한 번으로 돌아간다. 아래 목록에서 다른 프로젝트의 상태를 문장으로 보고 고를 수 있다.

## 현재 상태 (반복 1a 완료 시점)
이 계획이 기대는 사실만 적는다.
- `src/App.tsx`: 라우터 없이 `useRepoStore().repo`가 있으면 `Workspace`, 없으면 `Home`. 앱을 켜면 늘 `Home`.
- `src/screens/Home/Home.tsx`: 제목 "hiGit", 한 줄 소개, 주 버튼 "폴더 열기", 저장소가 아닌 폴더면 안내와 `ActionButton({ id: "init" })`. 최근 프로젝트 목록은 없다. git이 없으면 `appStatus.error`를 `ErrorNotice`로.
- `src/screens/Workspace/TopBar.tsx`: 왼쪽 프로젝트 이름(`h1`, 경로는 `title`), 브랜치 표시, 올릴·가져올 수. 오른쪽 글자 버튼 "다른 폴더 열기"(`closeRepo`). `TopBar.test.tsx`가 이 버튼을 확인한다.
- `src/state/repoStore.ts`: `repo`, `leftTab`, `selectedPath`, `openRepo`, `closeRepo`.
- `src/features/history/model.ts`: `HistoryRow`(oid, parents, isMerge …). `layout/`, `render/`은 아직 없다(D-017 제안, 반복 0~1a는 단순 목록).
- core: `Queries`는 `status`, `log`, `diff`. `Core::open_folder`가 폴더를 정규화하고 `discover`로 저장소 최상위를 찾는다. `view::BranchView { name, hasCommits, upstream, ahead, behind }`, `CommitRow`가 있다. `git::LogQuery.rev`는 시작점 하나만 받는다. 브랜치 목록(`branches`)은 1b 6단계에서 생긴다.
- platform: `find_git`, `normalize_repo_path`(dunce), `configure_child`, `symlink_file`. 의존성 `which`, `dunce`뿐.
- 앱 데이터를 파일로 남기는 곳이 아직 없다. Tauri 플러그인은 `dialog`만.
- 단축키: 버전 저장 상자의 Ctrl+Enter(`CommitBox.tsx`, `isComposing` 검사), 탭의 좌우 화살표. 명령 팔레트 Ctrl+K는 architecture.md에 계획만 있다.
- 디자인 토큰: 글자 크기 12·13·15·20px, `--graph-1`~`--graph-6`은 라이트 값만 있고 다크 덮어쓰기가 없다. `contrast.test.ts`는 글자·테두리 쌍만 확인하고 그래프 색은 확인하지 않는다.

## 반복 1b에 넣을 것과 미룰 것
| 항목 | 언제 | 왜 / 무엇이 필요한가 |
| --- | --- | --- |
| 최근 프로젝트 목록 저장(`recent.json`, platform 경로) | 1b, 2~3단계 | D-032. Git 기능이 아니라 독립적 |
| 앱을 켜면 마지막 프로젝트 바로 열기, 사라짐·저장소 아님 처리 | 1b, 3·7단계 | D-032 |
| 작업 공간 프로젝트 메뉴 "다른 프로젝트 열기" | 1b, 7단계 | 지금의 "다른 폴더 열기"를 바꿈. 1b 19단계 브랜치 드롭다운과 같은 Radix 부품 |
| 프로젝트 상태 요약 Query(`query_project_summary`) | 1b, 4~5단계 | 지금 있는 status·log만으로 만든다 |
| 시작 화면 배치: 상단 바(프로젝트, 폴더 열기 Ctrl+O), 마지막으로 하던 곳(시각, 이름, 경로, 상태 문장, 이어서 하기, 바뀐 파일 N개 보기), 다른 프로젝트 목록(이름, 상태, 마지막으로 연 때), 사라진 줄과 "목록에서 빼기", 빈 상태 | 1b, 8단계 | D-031. 노선도 자리는 비워 둔다 |
| 단축키 표와 Ctrl+O | 1b, 6단계 | D-033 |
| 글자 크기 토큰 26px | 1b, 1·8단계 | D-031 |
| 큰 노선도(브랜치 색), 작은 노선도(회색) | D-017 확정 후, 노선도 반복과 함께(나중 단계 N1~N3) | 배치 계산·Renderer가 버전 기록 화면과 같아야 하므로 노선도 반복보다 먼저 만들지 않는다 |
| 복제 글자와 Ctrl+Shift+O | 반복 4(나중 단계 N4) | 복제 액션이 반복 4. 그 전에는 숨김(Q3) |
| 상단 바 "설정" | 설정 화면이 생기는 반복 | 갈 곳이 없다 |
| 처음 실행: 이름·이메일 설정 | 1b 뒤 별도 계획(Q6) | `git config --global` 쓰기 = 새 GitOp·Action·용어 필요. 001에서 범위 밖(H-16) |
| 처음 실행: "Git을 찾았어요" 줄 | 1b, 8단계 | `app_status`로 이미 안다. 이름·이메일 줄 없이 빈 상태 위에 한 줄 |
| 처음 실행: GitHub 연결, 이메일이 GitHub와 다를 때 | 반복 4 | 로그인이 반복 4 |
| Git이 없을 때 OS별 안내(Windows 내려받기 링크, Ubuntu 설치 명령 복사, 다시 확인) | 나중(N6) | OS 판별(platform)과 외부 링크 열기 플러그인이 필요. 지금은 기존 `gitMissing` 안내 |
| 폴더를 창에 끌어다 놓기 | 나중(N5), Q7 | Windows에서 Tauri 파일 끌어놓기를 켜면 웹뷰 안 HTML5 끌어놓기가 막힌다. 노선도의 끌어놓기 합치기(spec 원칙 4)와 함께 정해야 함 |
| 저장소 아닌 폴더 안내를 작업 공간 위로 옮기기 | 나중, Q8 | 작업 공간이 저장소 없는 상태를 다뤄야 함 |
| "합치는 중" 같은 진행 중 작업 상태 | 반복 2 | 합치기가 반복 2. 1b는 충돌 파일 수만 |
| 우클릭·명령줄로 열 때 시작 화면 건너뛰기 | 반복 5 | 단일 인스턴스·실행 인자와 함께 |

## 완료 기준
- Ubuntu 22.04와 Windows 11에서 `pnpm tauri dev`로 저장소 A를 열고 앱을 닫았다가 다시 켜면 시작 화면을 거치지 않고 A의 작업 공간이 열린다.
- 작업 공간 상단 바의 프로젝트 이름을 누르면 메뉴가 열리고, "다른 프로젝트 열기"를 누르면 시작 화면이 나온다. 위쪽에 A가 "마지막으로 하던 곳 · 지금"과 함께 보이고, "이어서 하기"를 누르면 A로 돌아간다.
- A에 바뀐 파일이 3개 있으면 상태 문장이 "main 브랜치에서 파일 3개를 고치다 멈췄어요. 마지막 버전은 10월 7일에 저장한 "…"예요."처럼 보이고, "바뀐 파일 3개 보기"를 누르면 변경 사항 탭에 첫 파일 비교가 열린다.
- 저장소 B, C를 차례로 열면 시작 화면 아래 목록에 이름·짧은 상태·마지막으로 연 때가 최근 순으로 보인다. 충돌 파일이 있는 프로젝트의 상태만 빨간 글자다.
- A 폴더를 지우고 앱을 다시 켜면 시작 화면이 나오고 맨 위에 "마지막으로 연 A 폴더를 찾지 못했어요." 한 줄이 보인다. 위쪽에는 그다음 프로젝트 B가 보이고, A 줄은 흐린 글자와 "목록에서 빼기"로 남는다. A의 `.git`만 지운 경우에도 시작 화면이 나오고 A 줄에 "버전 관리를 하지 않는 폴더예요"가 보인다. 어느 경우에도 다른 프로젝트가 저절로 열리지 않는다.
- 시작 화면과 작업 공간에서 Ctrl+O로 폴더 고르기 창이 열린다. 한글 입력 상태에서도 열린다. Ctrl+Shift+O는 아무 일도 하지 않는다(반복 4 전).
- 최근 목록 파일이 Windows `%LOCALAPPDATA%\dev.higit.desktop\recent.json`, Ubuntu `~/.local/share/dev.higit.desktop/recent.json`에 생긴다. 파일을 일부러 깨뜨려도 앱이 뜨고 목록이 빈 채로 시작한다.
- `pnpm test`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm lint` 통과. `gen-bindings`를 두 번 돌려도 `git diff`가 비어 있다.
- 화면 코드에서 git·`invoke`를 직접 부르지 않는다. `rg "cfg\((windows|target_os)" src-tauri --glob '!crates/platform/**'`가 D-030 예외 말고 비어 있다.

## 단계
각 단계 = 커밋 하나. 1b의 001 단계(6, 8, 9, 10 브랜치 조회, 10b, 12, 13, 19, 20)와 순서는 자유롭지만 7단계는 001 19단계(브랜치 드롭다운)와 같은 Radix 부품을 쓰므로 그 앞이나 뒤에 붙여서 한다. 바인딩이 바뀌는 단계는 같은 커밋에서 `cargo run -p app --bin gen-bindings`로 다시 만든다. 1b 마무리(001 22단계)에 이 계획의 점검 항목을 더한다.

1. 결정 확정과 문서 반영 (`docs: confirm start screen decisions`)
   - 전제: 사람이 아래 "사람 확인 필요"에 답한다.
   - 변경 파일:
     - `docs/decision.md`: D-031~D-033 상태를 `확정`으로(보류한 것은 제안 그대로). 답에 따라 문구 수정.
     - `docs/spec.md`: "진입점" 절 머리의 "제안 상태" 문장과 "시작 화면" 행의 "제안"을 확정에 맞게 고친다. 저장소 아닌 폴더 안내 자리(Q8) 답을 적는다.
     - `docs/design.md`: 글꼴 크기 줄을 "13px(기본), 12px(보조), 15px(화면 제목, 시작 화면 상태 문장), 20px(시작 화면 처음 실행 제목), 26px(시작 화면 프로젝트 이름)"로. 부품 규칙에 "굵은 글자 버튼: 테두리 없이 600, 옆에 단축키를 `--text-muted` 12px 코드 글꼴로"와 "흐린 줄: 투명도 대신 `--text-muted`". 새 절 "시작 화면 색": 큰 노선도 선·점만 `--graph-*`, 작은 노선도 `--border-strong`·갈라진 브랜치 `--text-muted`, 상태 글자는 `--text`이고 충돌만 `--deleted`, 노선도 옆 글자는 `--text`·`--text-muted`.
     - `docs/architecture.md`: "프론트와 백엔드 계약"의 명령 목록에 `recent_list`, `project_open`(기존 `repo_open` 대체, 프로젝트 열기의 유일한 입구), `recent_remove`, `query_project_summary`. "OS별 처리" 표에 "앱 데이터 폴더" 행(Windows `%LOCALAPPDATA%\dev.higit.desktop`, Ubuntu `$XDG_DATA_HOME` 또는 `~/.local/share` 아래 `dev.higit.desktop`)과 "경로 같음 비교"(Windows 대소문자 무시) 행. 계층 그림 `screens/`의 "홈"을 "시작 화면"으로. core 설명에 "최근 프로젝트 목록". "버전 기록 그리기 계층"에 한 줄: "시작 화면의 노선도 미리보기도 같은 layout·render를 쓴다(D-031)".
     - `docs/roadmap.md`: 반복 1b에 "시작 화면: 마지막 프로젝트 바로 열기, 최근 프로젝트, 상태 문장(계획 003)". 반복 4에 "시작 화면 복제 글자와 Ctrl+Shift+O". "노선도 보기" 절에 "시작 화면 큰·작은 노선도(계획 003 N1~N3)".
   - 완료 기준: `pnpm test`(spec.md를 건드리므로 spec-sync 재확인) 통과. 용어표는 바뀌지 않는다.
   - Windows/Ubuntu: 문서라 차이 없음.
   - 위험: D-017이 계속 제안이면 노선도 부분은 이 계획에서 구현되지 않는다(의도).

2. platform: 앱 데이터 폴더와 경로 비교 (`platform: add app data dir and same-path check`)
   - 변경 파일: `src-tauri/Cargo.toml`(workspace 의존성에 `dirs`), `crates/platform/Cargo.toml`(`dirs`), `crates/platform/src/app_dir.rs`(신규), `crates/platform/src/path.rs`(`same_path`), `crates/platform/src/lib.rs`(re-export).
   - 함수:
     - `pub fn app_data_dir(app_id: &str) -> io::Result<PathBuf>`: 환경 변수 `HIGIT_DATA_DIR`이 있으면 그것(테스트·개발용), 없으면 `dirs::data_local_dir()`(Windows `%LOCALAPPDATA%`, Linux `$XDG_DATA_HOME` 또는 `~/.local/share`) 아래 `app_id`. 폴더가 없으면 만든다. 찾을 수 없으면 오류.
     - `pub fn same_path(a: &Path, b: &Path) -> bool`: Windows는 대소문자 무시(`to_lowercase` 비교, 정규화된 경로끼리라는 전제), 그 밖은 바이트 비교. `cfg(windows)` 분기는 이 함수 안에만.
   - 테스트: `HIGIT_DATA_DIR`를 임시 폴더로 주면 그 아래를 돌려주고 폴더를 만든다(환경 변수 테스트는 한 테스트 함수 안에서 차례로, 병렬 실행 간섭을 피하려고 `serial` 없이 하나로 묶는다). 환경 변수 없이 부르면 절대 경로이고 `app_id`로 끝난다. `same_path`: Windows에서 `C:\A\b`와 `c:\a\B`가 같음, Linux에서 `/a/B`와 `/a/b`는 다름(`cfg(windows)`/`cfg(not(windows))` 테스트는 platform 안이라 규칙 위반 아님).
   - 완료 기준: `cargo test -p platform` 두 OS 통과. `cargo tree -p platform -e normal`에 내부 크레이트 없음.
   - Windows/Ubuntu: Windows `data_local_dir`은 `FOLDERID_LocalAppData`(로밍 아님, D-032). Ubuntu는 `XDG_DATA_HOME`이 상대 경로이면 XDG 규칙상 무시해야 한다 → `dirs`가 처리하는지 테스트로 확인하고, 아니면 이 함수에서 절대 경로만 받는다. Tauri의 `app_local_data_dir`과 같은 위치인지 두 OS에서 손으로 한 번 비교한다(WebView2 데이터 폴더 `EBWebView`가 같은 폴더에 있어야 함).
   - 위험: `dirs` 버전이 Windows 경로 규칙을 바꾸면 기존 목록을 못 찾는다 → 버전은 잠금 파일로 고정, 바꿀 때 확인.

3. core: 최근 프로젝트 목록 (`core: add recent projects store`)
   - 변경 파일: `crates/core/Cargo.toml`(`serde_json`을 `[dependencies]`로 옮김), `crates/core/src/recent.rs`(신규), `crates/core/src/bootstrap.rs`, `crates/core/src/lib.rs`, `crates/core/src/view/mod.rs`(또는 `view/recent.rs`), `crates/core/tests/recent.rs`(신규), `src-tauri/src/lib.rs`의 테스트 모듈 또는 `src-tauri/tests/app_id.rs`(identifier 대조).
   - `recent.rs`:
     - 파일 형식 `{ "version": 1, "projects": [{ "path": "…", "name": "…", "lastOpenedAt": 1760000000 }] }`. 최근 것이 앞, 최대 20개.
     - `RecentStore::open(dir: PathBuf)`, `list() -> Vec<RecentEntry>`, `touch(repo: &Path)`(맨 앞으로 옮기고 시각 갱신, 같은 경로는 `platform::same_path`로 하나만), `remove(path)`.
     - 쓰기는 `recent.json.tmp`에 쓰고 `std::fs::rename`으로 바꿔 끼운다. 읽기 실패(형식 오류, 모르는 `version`)는 원본을 `recent.json.bad`로 옮기고 빈 목록으로 시작한다. 쓰기 실패는 폴더 열기를 실패시키지 않는다(무시하고 다음 쓰기에서 다시).
     - 저장하는 경로는 `platform::normalize_repo_path`를 거친 저장소 최상위다. 경로를 UTF-8로 그대로 옮길 수 없으면(`to_str()`가 `None`) 기록하지 않는다.
     - 동시 접근은 `Mutex`로 한 프로세스 안에서만 막는다.
   - `bootstrap.rs`:
     - `pub const APP_ID: &str = "dev.higit.desktop";`(D-009).
     - `init()`이 `platform::app_data_dir(APP_ID)`로 `RecentStore`를 만든다. 폴더를 못 만들면 목록 없이(메모리 전용) 계속한다. `with_backend(git, recent: RecentStore)`로 테스트가 임시 폴더를 넘긴다(기존 테스트 호출부 수정, 또는 `with_backend`는 그대로 두고 `with_backend_and_data_dir` 추가 — 구현자가 작은 쪽을 고른다).
     - `open_folder`에서 저장소가 확인되면 `touch`. `execute`의 결과가 `RepoCreated`면 `touch`.
     - `recent_list() -> Vec<view::RecentProject>`: 파일만 읽는다. 폴더가 있는지는 여기서 보지 않는다(네트워크 드라이브가 응답하지 않으면 목록 전체가 멈추므로, 줄마다 4단계 Query에서 확인).
     - **프로젝트 열기는 한 곳(사용자 지시, 2026-10-15)**: `Core::open_project(path, how) -> Result<view::ProjectOpenResult>` 하나만 둔다. 앱을 켤 때 마지막 프로젝트 열기, 최근 목록에서 열기, 폴더 고르기(Ctrl+O·"폴더 열기"), 나중의 프로젝트 메뉴 최근 항목, 우클릭·명령줄(반복 5)이 모두 이 함수를 거친다. 기존 `open_folder`는 이 함수로 대체하고(또는 안에서 이 함수를 부르는 얇은 별칭으로 두었다가 다음 커밋에서 지움), `open_project`라는 별도 함수는 만들지 않는다.
       - `how`: `Picked`(사용자가 방금 고른 폴더, 저장소가 아니면 버전 관리 시작 안내로) / `Recent`(목록·자동 열기, 최상위가 기록과 `same_path`여야 함).
       - 순서: 폴더 정규화(실패 → `Missing`) → `discover` → `Recent`이면 최상위가 `path`와 `same_path`가 아니면 `NotARepo`(`.git`만 지워져 바깥 저장소가 잡히는 경우를 막는다), `Picked`이면 저장소가 없을 때 `NotARepo { folder }` → **저장소 신뢰 확인 자리**(지금은 항상 통과, 보안 계획에서 이 한 곳에 넣는다는 주석과 `trust_check` 빈 함수) → `touch` → `Opened { repo }`.
       - 테스트: 세 진입점(앱 켜기·목록·폴더 고르기)에 해당하는 화면 흐름이 모두 같은 명령 `project_open`을 부르는지(프론트 테스트), core는 `how`별 결과.
     - `remove_recent(path)`.
   - view 타입(바인딩 derive는 view에만, D-019):
     ```rust
     pub struct RecentProject { pub path: String, pub name: String, #[specta(type = u32)] pub last_opened_at: i64 }
     #[serde(tag = "kind", rename_all = "camelCase")]
     pub enum ProjectOpenResult { Opened { repo: RepoInfo }, Missing, NotARepo { folder: String } }
     ```
   - 테스트(`tests/recent.rs`, 가짜 백엔드 + 임시 폴더): 열면 맨 앞에 생김, 같은 저장소를 두 번 열면 하나, 21개 열면 20개, 깨진 파일 → 빈 목록 + `.bad`, 모르는 `version` → 같음, `open_project`: 지운 폴더 → `Missing`, 저장소 아닌 폴더 → `NotARepo`, 바깥 저장소 안의 하위 폴더를 기록한 뒤 그 `.git`이 없는 경우(`Recent`) → `NotARepo`, 같은 하위 폴더를 `Picked`로 고르면 바깥 저장소가 열림(지금 동작 유지), init 성공 → 기록됨, 쓰기 실패(읽기 전용 폴더)에도 `open_folder`는 성공. identifier 대조 테스트: `tauri.conf.json`의 `identifier`가 `higit_core::APP_ID`와 같다.
   - 완료 기준: `cargo test --workspace` 두 OS 통과, clippy 통과.
   - Windows/Ubuntu: Windows의 `rename`은 대상 파일을 다른 프로그램(백신)이 열고 있으면 실패할 수 있다 → 실패는 무시하고 `.tmp`를 남긴다(다음 쓰기가 덮어씀). Windows 경로는 `C:\…` 형식(dunce)으로 저장되어 JSON에서 `\\`로 이스케이프된다. Ubuntu에서 홈이 암호화되었거나 `~/.local/share`가 없을 때 폴더를 만든다(2단계).
   - 위험: 단일 인스턴스(반복 5) 전에는 앱 두 개가 같은 파일을 쓰면 나중 쓰기가 이긴다(D-032에 적음). 목록 크기 20은 화면 높이 기준 임의 값(Q5).

4. core: 프로젝트 상태 요약 Query (`core: add project summary query`)
   - 새 액션 추가 절차 4의 "조회 기능이면 `core::query`에 메서드와 view 타입" 경로(D-010). git 크레이트는 바뀌지 않는다(기존 `status`, `log` 사용).
   - 변경 파일: `crates/core/src/query.rs`, `crates/core/src/view/mod.rs`(또는 `view/summary.rs`), `crates/core/tests/query.rs`.
   - `Queries::project_summary(&self, repo: &Path) -> Result<ProjectSummaryView>`:
     - 폴더 정규화 실패 → `FolderNotFound`. `discover`의 최상위가 `repo`와 `same_path`가 아니거나 없으면 → `NotARepo`(3단계와 같은 규칙, 바깥 저장소 정보를 보여주지 않음).
     - `status` 한 번, `log`(limit 1) 한 번. 저장이 없으면 log를 부르지 않는다.
     ```rust
     pub struct ProjectSummaryView {
         pub branch: BranchView,        // 기존 타입 재사용
         pub changed_files: u32,        // 충돌 포함 전체
         pub conflicted_files: u32,
         pub last_commit: Option<CommitRow>,
     }
     ```
   - 문장은 core가 만들지 않는다. 문장·언어는 화면(messages)이 정한다.
   - 노선도 데이터는 이 Query에 넣지 않는다(나중 단계 N1의 별도 Query).
   - 테스트: 가짜 백엔드로 바뀐 파일 수·충돌 수 세기, 저장 없는 저장소(`last_commit: None`, log 호출 없음), detached, 없는 폴더 → `FolderNotFound`, 바깥 저장소가 잡히는 하위 폴더 → `NotARepo`. 실제 git 통합 테스트 1개(파일 2개 수정 + 버전 1개).
   - 완료 기준: `cargo test --workspace` 통과.
   - Windows/Ubuntu: 큰 저장소의 `status`는 Windows에서 특히 느리다(수 초). 화면이 줄마다 따로 불러 늦게 채우므로 목록 표시는 막히지 않는다.
   - index 잠금(사용자 지시, 2026-10-15): 여러 프로젝트를 한꺼번에 읽으므로 다른 작업(사용자의 터미널 git, 다른 앱)과 `index.lock`이 부딪히면 안 된다. 모든 git 호출은 이미 `Runner`가 `GIT_OPTIONAL_LOCKS=0`을 붙이며(1a), 이 Query는 그 경로만 쓴다(새 git 호출을 따로 만들지 않는다). 테스트로 고정한다: 실제 git 저장소에 `.git/index.lock`을 만들어 다른 작업이 잡고 있는 상태를 흉내 낸 뒤 `project_summary`가 성공하고, 끝난 뒤에도 `index.lock`이 그대로이며 `.git/index`의 수정 시각이 바뀌지 않는다. `Runner`가 이 환경 변수를 붙이는지 확인하는 git 크레이트 단위 테스트도 하나 둔다(지금은 감시 되먹임 테스트로만 간접 확인).
   - 위험: 가져올 버전 수(`behind`)는 마지막 새로 고침(fetch) 기준이다. 시작 화면은 프로젝트마다 네트워크를 쓰지 않으므로 "최신 상태"가 실제와 다를 수 있다(Q9).

5. app + lib: 명령과 바인딩 (`app: expose recent projects and project summary`)
   - 변경 파일: `src-tauri/src/commands.rs`(`recent_list`, `project_open(path, how)`, `recent_remove(path)`, `query_project_summary(repo)`, 모두 `blocking`. 기존 `repo_open`은 `project_open`으로 바꾼다), `src-tauri/src/bindings.rs`(명령 목록), 생성물 `src/lib/bindings.ts`, `src/lib/api.ts`(`recentProjects()`, `openProject(path, how)`, `removeRecent(path)`, `projectSummary(repo)`와 타입 re-export. `openFolder`는 없앤다), `src/features/open-project/useOpenProject.ts`(신규: 화면에서 프로젝트를 여는 유일한 훅. 결과에 따라 `openRepo`·시작 화면 안내·버전 관리 시작 안내로 나눈다), `src/lib/queries.ts`(`queryKeys.recent`, `queryKeys.summary(repo)`, `useRecentProjects()`, `useProjectSummary(repo)`), `src/lib/api.test.ts`.
   - `useProjectSummary`는 `staleTime` 30초, 창 포커스 때 다시 읽기(TanStack 기본). 실패(`folderNotFound`, `notARepo`)는 화면이 줄 상태로 쓰므로 오류 알림을 띄우지 않는다.
   - 프로젝트를 연 뒤(`openProject`, init 성공) `queryKeys.recent`를 무효화한다. 화면의 모든 프로젝트 열기는 `useOpenProject`를 거친다(ESLint `no-restricted-syntax`나 테스트로 `api.openProject`를 그 훅 밖에서 부르지 않는지 확인).
   - 테스트: api 모킹으로 명령 이름·인자, `ApiError` 변환.
   - 완료 기준: `pnpm typecheck`, `pnpm test`, `cargo test --workspace`, 바인딩 최신.
   - Windows/Ubuntu: 차이 없음(경로 문자열은 core가 정규화한 것을 그대로 주고받는다).

6. 단축키 표와 Ctrl+O (`ui: shortcut table and open folder shortcut`)
   - 변경 파일: `src/lib/shortcuts.ts`(신규), `src/lib/useShortcut.ts`(신규), `src/lib/shortcuts.test.ts`(신규), `src/features/open-project/useOpenFolder.ts`(신규: `pickFolder` → `useOpenProject`의 `open(path, "picked")`. 폴더 고르기만 하고 여는 일은 `useOpenProject`에 맡긴다), `src/screens/Home/Home.tsx`(훅 사용), `src/ui/Kbd.tsx` + `Kbd.module.css`(신규, 단축키 글자 `--text-muted`, 12px, `--font-mono`), `src/features/commit/CommitBox.tsx`(Ctrl+Enter를 표의 항목으로 판정하도록 바꿈, 동작은 그대로).
   - 표(D-033):
     ```ts
     export const SHORTCUTS = {
       commit: { code: "Enter", ctrl: true, shift: false, label: "Ctrl+Enter", scope: "commitBox" },
       openFolder: { code: "KeyO", ctrl: true, shift: false, label: "Ctrl+O", scope: "global" },
       // 반복 4: clone: { code: "KeyO", ctrl: true, shift: true, label: "Ctrl+Shift+O", scope: "global" },
       // 계획: commandPalette: { code: "KeyK", ctrl: true, shift: false, label: "Ctrl+K", scope: "global" },
     } as const;
     ```
     `label`은 화면 문구가 아니라 키 이름이라 messages에 두지 않는다.
   - `useShortcut(id, handler)`: `window`의 `keydown`에서 `e.code`·`ctrlKey`·`shiftKey`·`altKey`로 판정(한글 입력 상태에서는 `e.key`가 `ㅐ`가 된다). `e.isComposing` 또는 `keyCode === 229`이면 무시. 맞으면 `preventDefault()`. 입력칸 안에서도 Ctrl 조합은 동작한다. 폴더 고르기 창이 이미 열려 있으면(mutation 진행 중) 다시 열지 않는다.
   - Ctrl+O는 시작 화면과 작업 공간 모두에서 동작한다(Q4). 작업 공간에서 고른 폴더가 저장소면 그 프로젝트로 바꾸고, 아니면 시작 화면으로 가서 저장소 아님 안내를 보여준다.
   - 단축키 겹침 점검 결과(사용자 지시 8):

     | 키 | 앱 안(기존·계획) | WebView2 | WebKitGTK | Windows 시스템 | Ubuntu GNOME·IBus | 브라우저·다른 앱 | 판단 |
     | --- | --- | --- | --- | --- | --- | --- | --- |
     | Ctrl+O | 없음 | 문서의 브라우저 단축키 목록(찾기·인쇄·새로 고침·확대·개발자 도구)에 없음 | 기본 단축키 없음 | 없음 | 없음 | 브라우저·VS Code "파일 열기", GitHub Desktop "로컬 저장소 추가" | 씀. 뜻이 같아 익숙함 |
     | Ctrl+Shift+O | 없음(Ctrl+Enter, Ctrl+K와 다름) | 목록에 없음. 개발자 도구는 Ctrl+Shift+I/J/C | 기본 없음. 개발자 도구는 Ctrl+Shift+I(개발 빌드) | 없음. 입력 언어 바꾸기가 Ctrl+Shift로 설정된 경우도 다른 키와 함께 누르면 발동하지 않음(손으로 확인) | 기본 없음. IBus 유니코드 입력은 Ctrl+Shift+U | 크롬·엣지 북마크 관리자, 파이어폭스 라이브러리(웹뷰에는 북마크 화면 없음), VS Code "기호로 이동", GitHub Desktop(Windows) "저장소 복제" | 씀(반복 4). 대안 불필요 |
     | 대안 후보 Ctrl+Shift+N | 없음 | 없음 | 없음 | 없음 | 없음 | 크롬 시크릿 창, VS Code 새 창 | 쓰지 않음(익숙한 뜻과 다름) |
     | 대안 후보 Ctrl+Shift+C | 없음 | 개발자 도구 요소 검사 | 없음 | 없음 | 없음 | 브라우저 요소 검사 | 쓰지 않음(겹침) |

     WebView2 목록은 "예시이며 전부가 아님"이라고 문서에 적혀 있으므로, 완료 기준에 두 OS 실기기 확인을 넣는다.
   - 테스트(`shortcuts.test.ts`): 표 안에 같은 `code+ctrl+shift+alt` 조합이 둘 이상 없다. `useShortcut`이 `code: "KeyO"`, `key: "ㅐ"`인 이벤트에 반응하고, `isComposing: true`면 반응하지 않고, Shift가 더해진 Ctrl+Shift+O에는 openFolder가 반응하지 않는다. `CommitBox` 기존 테스트가 그대로 통과.
   - 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 한글 입력 상태에서 Ctrl+O로 폴더 고르기 창이 열림, Ctrl+Shift+O를 눌러도 웹뷰가 아무것도 하지 않음(창이 뜨거나 화면이 바뀌지 않음), 릴리스 빌드에서도 같음.
   - Windows/Ubuntu: WebView2가 Ctrl+O를 앱보다 먼저 가로채면(문서에 없지만 가능성) Tauri 설정으로 브라우저 단축키를 끌 수 있는지 확인하고, 필요하면 별도 단계로 다룬다(OS별 설정이면 platform 또는 Tauri 설정 파일). Ubuntu IBus 한글 입력기에서 `e.code`가 `KeyO`로 오는지 확인한다.
   - 위험: macOS는 검증 대상이 아니다(D-002). `metaKey`는 받지 않는다.

7. 앱을 켜면 마지막 프로젝트 열기, 프로젝트 메뉴 (`screens: reopen last project and project menu`)
   - 변경 파일: `src/App.tsx`, `src/state/useStartup.ts`(신규), `src/state/repoStore.ts`, `src/screens/Workspace/TopBar.tsx` + CSS, `src/screens/Workspace/ProjectMenu.tsx`(신규), `src/screens/Workspace/TopBar.test.tsx`, `src/App.test.tsx`, `src/messages/ko.ts`·`en.ts`, `package.json`(`@radix-ui/react-dropdown-menu`, D-004. 001 19단계가 popover를 넣으면 같은 계열).
   - 시작 흐름(`useStartup`, 앱을 켤 때 한 번만):
     1. `appStatus`가 실패(git 없음·오래됨)면 다시 열기를 하지 않고 시작 화면.
     2. `recentProjects()`의 맨 앞 하나로 `useOpenProject`의 `open(path, "recent")`(같은 열기 함수).
     3. `opened` → 작업 공간. `missing` → 시작 화면 + `startNotice = { kind: "lastMissing", name }`. `notARepo` → 시작 화면 + `{ kind: "lastNotRepo", name }`. 목록이 비었으면 시작 화면.
     4. 확인하는 동안은 빈 `--bg` 화면(시작 화면이 잠깐 보였다 사라지지 않게). 1초가 넘으면 "불러오는 중…"(`m.loading`)을 보여준다.
   - `repoStore`: `startNotice: StartNotice | null`, `openRepo(repo, opts?: { selectPath?: string })`(바뀐 파일 보기용), `closeRepo()`는 `startNotice`를 지운다. "다른 프로젝트 열기"로 올 때는 다시 열기를 하지 않는다.
   - 상단 바: 프로젝트 이름이 메뉴 단추가 된다(경로는 툴팁 그대로). 메뉴 항목 "다른 프로젝트 열기"(→ `closeRepo`). 오른쪽의 "다른 폴더 열기" 글자 버튼은 없앤다(컨트롤 수가 하나 줄어 1b 20단계의 새로 고침·가져오기·올리기 자리가 넉넉해짐, D-025 5개 이하).
   - 문구(ko / en): `otherProjects` 메뉴 "다른 프로젝트 열기" / "Open another project". `projectMenu` 접근성 이름 "프로젝트 메뉴" / "Project menu". `openOtherFolder`는 쓰는 곳이 없어지면 지운다.
   - 테스트: api 모킹으로 시작 흐름 네 경우(열림, 사라짐, 저장소 아님, 목록 없음)와 git 없음에서 `openProject`를 부르지 않음. 프로젝트 메뉴를 키보드(Enter, 화살표)로 열고 "다른 프로젝트 열기"를 누르면 시작 화면. `TopBar.test.tsx`의 "다른 폴더 열기" 기대를 메뉴로 바꿈.
   - 완료 기준: `pnpm test`, `pnpm lint`. 두 OS 수동: 앱을 다시 켜면 마지막 프로젝트가 바로 열림, 폴더를 지운 뒤 다시 켜면 시작 화면과 안내 줄.
   - Windows/Ubuntu: Radix 메뉴가 WebKitGTK에서 포커스 이동·Esc 닫기가 되는지, WebView2 고대비(`forced-colors`)에서 메뉴 테두리가 보이는지 확인. 연결이 끊긴 네트워크 드라이브(Windows `\\server\share`, Ubuntu `/media/…` 분리된 USB)가 마지막 프로젝트이면 `openProject`가 수 초 걸릴 수 있다 → 4번의 "불러오는 중…"으로 버티고, 시간 제한은 위험에 기록.
   - 위험: 마지막 프로젝트가 아주 큰 저장소면 작업 공간이 열린 뒤 status가 늦게 온다(지금도 같음).

8. 시작 화면 (`screens: start screen with last project and recent list`)
   - 변경 파일: `src/screens/Home/` → `src/screens/Start/`로 이름 바꿈(`Start.tsx`, `Start.module.css`, `StartTopBar.tsx`, `LastProject.tsx`, `ProjectList.tsx`, `Start.test.tsx`), `src/features/start/summary.ts` + `summary.test.ts`(신규, 상태 문장·짧은 상태를 고르는 순수 함수), `src/lib/time.ts`(`formatDate(unix, locale)` "10월 7일", 해가 다르면 "2025년 10월 7일"), `src/styles/tokens.css`(`--font-size-display: 26px`), `src/messages/ko.ts`·`en.ts`, `src/App.tsx`(import 경로), `src/App.test.tsx`.
   - 배치(D-031, 그림 `start-normal.png`):
     - 상단 바(`--bg`, 아래 `--border` 1px): 왼쪽 "프로젝트"(600). 오른쪽 굵은 글자 버튼 "폴더 열기"(`variant="text"` + 600) 옆에 `<Kbd>Ctrl+O</Kbd>`. 앞의 `+` 기호는 쓰지 않는다(Q10). 복제·설정은 이번에 없다.
     - 마지막으로 하던 곳(`--bg`): 작은 글자 `m.lastPlace(시각)` "마지막으로 하던 곳 · 10분 전"(`--text-muted` 12px). 이름(`--font-size-display`, 600) + 경로(`--font-mono` 12px `--text-muted`). 상태 문장(15px, `--text`). 노선도 자리는 이번에 없음(N2에서 이 자리에). 주 버튼 "이어서 하기"(`--primary`, 화면의 유일한 주 버튼) + 글자 버튼 "바뀐 파일 N개 보기"(바뀐 파일이 있을 때만). 상태를 불러오는 동안 상태 문장 자리는 빈 줄 높이를 유지한다(밀림 방지).
     - 다른 프로젝트(`--surface`): 머리 "다른 프로젝트"(`--text-muted` 12px). 줄(높이 48px 또는 `--row-height` 두 배, Q11)마다 이름(600) · 짧은 상태 · 마지막으로 연 때(오른쪽 정렬, `--text-muted`). 줄 전체가 버튼(누르면 `useOpenProject`의 `open(path, "recent")`). 작은 노선도 칸은 이번에 없음(N3).
     - 사라진 줄: 이름과 상태를 `--text-muted`로(투명도 금지), 상태 "폴더를 찾을 수 없어요" 또는 "버전 관리를 하지 않는 폴더예요", 오른쪽에 글자 버튼 "목록에서 빼기"(시각 대신). 저장소 아닌 줄을 누르면 지금의 "버전 관리 시작" 안내 흐름으로(Q8 답에 따라).
     - 맨 위 안내 줄(`startNotice`): "마지막으로 연 {이름} 폴더를 찾지 못했어요." / "마지막으로 연 {이름} 폴더는 버전 관리를 하지 않는 폴더예요." `role="status"`.
     - 위쪽 자리 고르기: 목록의 맨 앞이 사라진 폴더면 그다음 열 수 있는 프로젝트를 위쪽에 보여준다(조사 문서 "이럴 때"). 판정은 `useProjectSummary` 결과로 하고, 결과가 오기 전에는 맨 앞을 보여준다.
     - 빈 상태(그림 `start-first.png`의 아래 부분, 이름·이메일 줄은 Q6에 따라 이번에 없음): 위쪽 자리에 "✓ Git {버전}을 찾았어요"(`appStatus`) 한 줄. 아래 머리 "프로젝트", 본문 "아직 연 프로젝트가 없어요. 오른쪽 위의 폴더 열기로 시작해요." 주 버튼은 없다(빈 상태 규칙: 한 줄 설명, 버튼은 상단 바의 폴더 열기). git이 없으면 그 줄 대신 지금의 `ErrorNotice`(gitMissing)를 보여주고 폴더 열기를 끈다.
     - 저장소가 아닌 폴더를 열었을 때: 지금 `Home`의 안내와 `ActionButton({ id: "init" })`을 위쪽 자리에 그대로 옮긴다(Q8 답이 "작업 공간 위"면 나중 단계로).
   - 상태 문장(`summary.ts`가 경우를 고르고 messages가 문장을 만든다. 문장 속 용어는 `t(id)`로 꺼내고 `<Term>`을 붙이지 않는다, D-029):
     | 경우 | ko |
     | --- | --- |
     | 충돌 파일 있음 | `{t("conflict")} 난 파일이 {n}개 있어요.` |
     | 바뀐 파일 있음 | `{branch} {t("branch")}에서 파일 {n}개를 고치다 멈췄어요.` |
     | 바뀐 파일 없음 | `{branch} {t("branch")}에 있어요. 바뀐 파일은 없어요.` |
     | 특정 버전을 보는 중 | `특정 버전을 보는 중이에요.` 기존 `detached` 문구에 이어 붙임 |
     | 뒤에 붙는 문장: 마지막 버전 | `마지막 버전은 {날짜}에 저장한 "{요약}"{josa(요약, "이에요/예요")의 조사}.` |
     | 저장한 버전 없음 | 기존 `noCommits` "아직 저장한 버전이 없어요." |
     목록 줄의 짧은 상태(우선순위 순서로 하나만): 충돌 `{t("conflict")} 난 파일 {n}개`(`--deleted`) → 바뀐 파일 `바뀐 파일 {n}개` → 올릴 버전 `올릴 버전 {n}개` → 가져올 버전 `가져올 버전 {n}개` → 연결 없음 기존 `notUploaded` "아직 올린 적 없음" → `최신 상태`. 브랜치가 `main`·`master`가 아니면 앞에 `{branch} · `를 붙인다(Q12). 조사 문서의 "합치다 멈춤"(반복 2), "받을 버전"(옛 동사, D-024 "가져오기"와 어긋남), "변경 사항 12"는 쓰지 않는다.
   - en 문구는 같은 뜻으로: "Last worked on · 10 minutes ago", "Continue", "View 3 changed files", "Other projects", "Remove from list", "Folder not found", "Not under version control", "No projects yet. Start with Open folder at the top right.", "Found Git 2.43" 등. 문장 속 용어는 en glossary에서.
   - 테스트:
     - `summary.test.ts`: 경우 고르기 전부(충돌 우선, 바뀐 파일, 깨끗함, detached, 저장 없음, ahead/behind/연결 없음 우선순위, main이면 브랜치 이름 생략).
     - `Start.test.tsx`(api 모킹): 목록 3개 → 위쪽 1개 + 아래 2개, "이어서 하기" → 작업 공간, "바뀐 파일 N개 보기" → 변경 사항 탭과 첫 파일 선택, 맨 앞 폴더가 `folderNotFound` → 위쪽에 둘째 프로젝트와 흐린 첫 줄, "목록에서 빼기" → `removeRecent` 호출, 빈 목록 → 빈 상태 문구, git 없음 → 폴더 열기 꺼짐, 주 버튼이 하나뿐.
     - 기존 `copy-rules.test.ts`가 새 문구의 괄호·옛 용어·합쇼체를 검사한다. 새 함수형 문구는 `("예시", 3)`으로 불려도 예외가 나지 않게 쓴다(객체 인자는 기본값). `terms-in-messages.test.ts`에 "충돌", "브랜치"가 든 새 문장을 더한다.
     - `contrast.test.ts`: 이번 단계에 새 색 쌍은 없다. 쓰는 쌍(`--text`·`--text-muted`·`--deleted` on `--bg`·`--surface`·`--surface-hover`, `--primary-text`/`--primary`)은 이미 검사 중이다. 흐린 줄을 투명도로 만들지 않는 것이 전제다.
   - 완료 기준: `pnpm test`, `pnpm lint`(stylelint: tokens.css 밖 색 값 없음). 두 OS 수동: 800px·1200px 너비, 라이트·다크, 긴 프로젝트 이름과 긴 경로(줄임표), 한글 경로, Tab으로 줄 이동과 Enter로 열기, 그림 `start-normal.png`와 배치 비교.
   - Windows/Ubuntu: 경로 표시는 core가 준 문자열 그대로(Windows `C:\Users\…`, Ubuntu `/home/…`). 홈 폴더를 `~`로 줄이는 것(그림의 `~/work/my-shop`)은 홈 경로가 OS별이라 platform 함수가 필요하다 → 이번에는 줄이지 않고, 원하면 Q13. 26px 굵은 Pretendard가 두 웹뷰에서 같은 높이로 보이는지(번들 글꼴이라 같아야 함), WebKitGTK의 `text-overflow: ellipsis`가 flex 안에서 동작하는지 확인.
   - 위험: 최근 프로젝트 20개의 상태 요약이 동시에 git을 20번×2 부른다. Windows는 프로세스 시작이 느려 처음 1~2초 동안 줄이 차례로 채워진다. 문제가 되면 `useProjectSummary`에 동시 4개 제한을 둔다(별도 커밋).

### 나중 단계 (이 계획에서 하지 않음, 순서와 조건만)
- N1. 노선도 미리보기 데이터 (`git: log from several starting points with ref labels` → `core: add graph preview query`) — D-017 확정 + 노선도 반복의 배치 계산 층이 생긴 뒤.
  - git: `LogQuery.rev: Option<String>`을 `revs: Vec<String>`(비면 HEAD)으로 넓히고 `--topo-order`. 1b 6단계의 `Branch`에 `oid`를 더해 브랜치 이름표를 붙인다(`for-each-ref`의 `%(objectname)`은 이미 읽음). 출력은 기존처럼 기계용 형식만(D-003).
  - core: `Queries::graph_preview(repo, GraphPreviewQuery { limit: 10 }) -> GraphPreviewView { commits: Vec<CommitRow>, labels: Vec<RefLabelView>, head: Option<String>, has_worktree_changes: bool }`. 시작점은 HEAD와 "갈라진 브랜치 하나"(HEAD가 기본 브랜치가 아니면 기본 브랜치, 기본 브랜치는 `refs/remotes/origin/HEAD` → 없으면 `main`·`master` 중 있는 것). 명령 `query_graph_preview`.
- N2. 큰 노선도 (`features: start screen route preview`) — `features/history/model.ts`(`HistoryRow`에 이름표 추가) → `features/history/layout/`(노선도 반복이 만든 순수 함수에 `density: "full" | "preview" | "mini"` 옵션) → `features/history/render/SvgRenderer`(`palette: "branch" | "mono"`). 시작 화면은 `<RoutePreview data palette="branch" />`만 쓰고 배치 계산을 따로 갖지 않는다(D-031, architecture.md "버전 기록 그리기 계층"). 선·점은 `--graph-*`, "지금 여기"·버전 이름·브랜치 이름 글자는 `--text`·`--text-muted`.
  - 대비 테스트 추가: `--graph-1`~`--graph-6` / `--bg`·`--surface` 3:1(그래픽 기준, 라이트·다크). 지금 다크 블록에는 그래프 색이 없으므로 테스트가 라이트 값을 이어 받아 계산해야 한다(손으로 계산한 다크 `--bg` 대비: graph-6 약 3.1, graph-1 약 3.4, graph-2 약 3.5로 3:1은 넘지만 4.5:1은 못 넘음 → 글자에 쓰지 않는 근거). 다크 그래프 색을 따로 둘지는 002 Q10대로 노선도 반복에서 정한다.
- N3. 작은 노선도 (`features: mini route in project list`) — 같은 컴포넌트에 `palette="mono"`, `density="mini"`, 버전 10개 + 갈라진 브랜치 하나. 선·점 `--border-strong`(라이트 `--surface`에서 3.21), 갈라진 브랜치 `--text-muted`. 이미 검사 중인 쌍이라 대비 테스트 추가 없음. 목록 줄이 보일 때만 불러온다(`IntersectionObserver`).
- N4. 복제 (반복 4): 시작 화면 상단 바에 `<Term id="clone" />` 굵은 글자 + `<Kbd>Ctrl+Shift+O</Kbd>`, 단축키 표에 `clone` 추가(6단계의 겹침 테스트가 확인). 복제 액션은 새 액션 추가 절차대로(용어 `clone`은 표에 이미 있음).
- N5. 폴더 끌어다 놓기(Q7 답 이후): `src/lib/events.ts`에 `onFolderDrop`(Tauri `onDragDropEvent`는 `src/lib/` 안에서만), 여러 개면 첫 번째, 폴더가 아니면 무시. 머리 오른쪽 "폴더를 창에 끌어다 놓아도 열려요"는 이 단계에서 넣는다.
- N6. Git이 없을 때 OS별 안내: `platform::os_kind()`를 `AppStatus`에 실어 화면이 Windows면 내려받기 링크, Ubuntu면 `sudo apt install git`과 복사 버튼, "다시 확인"(앱 재시작 없이 `init` 다시 시도하는 명령 필요). 외부 링크는 `tauri-plugin-opener`와 capability 추가.
- N7. 처음 실행 이름·이메일(Q6): 새 GitOp `SetIdentity { name, email }`(`git config --global user.name`/`user.email`), Action `identity-set`(safe), 조회 `query_identity`. 새 용어가 필요하면 spec.md 표·`ko.ts`·`en.ts`(사람 확인). 시작 화면 위쪽 자리에 "처음 한 번만 하면 돼요 / 버전에 남길 이름부터 정해요"(20px).

## 용어 변경
- spec.md 용어표·glossary: 이번 1b 범위에는 없음. "프로젝트", "폴더 열기", "이어서 하기", "마지막으로 하던 곳", "목록에서 빼기"는 Git 용어가 아니므로 messages에 둔다. 문장 속 "브랜치", "충돌"은 `t(id)`로 꺼낸다(D-029).
- 나중 단계: N4 복제는 기존 `clone` 행을 쓴다. N7 이름·이메일은 새 행이 필요할 수 있다(예: id `identity`, 쉬운 이름 "버전에 남길 이름", 원래 이름 `user.name · git config user.name`). 사람 확인 후 그 단계에서 spec.md 표·`ko.ts`·`en.ts`를 함께 고친다(D-015, D-024).

## 위험과 열린 질문
- 상태 요약은 마지막 새로 고침 기준이라 "최신 상태"가 원격과 다를 수 있다(Q9).
- 네트워크 드라이브·분리된 USB에 있는 프로젝트는 폴더 확인 자체가 수 초 걸릴 수 있다. 목록 읽기에서는 폴더를 확인하지 않고, 줄마다 Query에서 확인해 화면이 멈추지 않게 했다. 앱을 켤 때의 다시 열기는 이 시간을 기다린다(시간 제한 없음).
- 단일 인스턴스 전까지 앱 두 개가 `recent.json`을 동시에 쓰면 한쪽 기록이 사라진다.
- Windows에서 Tauri 파일 끌어놓기를 켜면 웹뷰 안 HTML5 끌어놓기가 동작하지 않는다(Tauri `dragDropEnabled` 설명). 노선도의 끌어놓기 합치기를 HTML5 끌어놓기로 만들면 폴더 끌어놓기와 함께 쓸 수 없다(Q7).
- WebView2의 브라우저 단축키 목록은 "예시"라서 Ctrl+O·Ctrl+Shift+O가 앱에 오는지는 실기기로만 확정된다.
- 다크 테마 그래프 색이 글자 대비에 못 미친다. 노선도 글자에 브랜치 색을 쓰지 않는 규칙으로 피하고, 다크 그래프 값은 노선도 반복에서 정한다.
- 조사 문서와 기존 결정·계획이 어긋나는 곳은 아래 "사람 확인 필요"에 모았다.

## 결정 결과 (2026-10-15)
- Q1~Q15는 위 "사람 확인 필요"의 추천안대로 확정(사용자). D-031~D-033 확정, 노선도는 D-017 확정 전까지 구현하지 않음. Q3 복제는 반복 4 전까지 글자·단축키 모두 숨김. Q8 저장소 아닌 폴더 안내는 1b에서 시작 화면 위쪽, 문구 "이 폴더는 아직 버전 관리를 하지 않아요". Q14 "충돌 난 파일 N개", "가져올 버전", "바뀐 파일 N개". Q15 `screens/Home` → `screens/Start`.
- 추가 1: 상태 요약 Query는 `GIT_OPTIONAL_LOCKS=0` 경로만 쓰고, 다른 작업이 `index.lock`을 잡고 있어도 성공하며 잠금·index를 건드리지 않음을 테스트로 고정(4단계).
- 추가 2: 앱을 켤 때 열기, 최근 목록에서 열기, 폴더 고르기, 프로젝트 메뉴의 열기는 모두 core `Core::open_project` 하나와 화면 `useOpenProject` 하나를 거친다. 저장소 신뢰 확인은 나중에 그 한 곳에 넣는다(3·5·6·7·8단계).
- 다크 모드 그림 두 장과 `docs/research/security.md`는 이 계획에서 커밋하지 않는다.

## 사람 확인 필요
추천안을 먼저 적는다.
- Q1. D-031~D-033을 확정할까요? 추천: 셋 다 확정. D-031의 노선도 부분은 D-017 확정 전까지 구현하지 않는다는 조건으로.
- Q2. 최근 목록 위치: 추천 로컬 앱 데이터(`%LOCALAPPDATA%`, `~/.local/share`) 아래 `dev.higit.desktop`. 대안: 로밍 `%APPDATA%`(Tauri `app_data_dir`와 같음, 다른 컴퓨터로 경로가 따라감).
- Q3. 시작 화면의 복제(반복 4): 추천 (가) 반복 4 전에는 글자와 단축키 모두 숨김. 대안 (나) 누를 수 없는 글자로 미리 보여주기(툴팁 "아직 준비 중이에요"), (다) 복제를 1b로 당기기(로그인 없이 공개 저장소 HTTPS 복제만, 범위가 커짐).
- Q4. Ctrl+O를 작업 공간에서도 쓸까요? 추천: 예(어디서나 폴더 열기). 아니면 시작 화면에서만.
- Q5. 최근 목록 최대 개수: 추천 20개.
- Q6. 처음 실행의 이름·이메일 설정: 추천 1b 뒤 별도 계획(N7, 새 GitOp·Action·용어 필요). 대안: 1b에 넣기. 그 전까지는 버전 저장 때 `identityMissing` 안내가 뜬다.
- Q7. 폴더 끌어다 놓기: 추천 노선도 반복에서 "앱 안 끌어놓기는 포인터 이벤트로 만든다"를 함께 정하고 그 뒤에 넣기(N5). 대안: 1b에 넣고 Windows에서 HTML5 끌어놓기를 포기.
- Q8. 저장소 아닌 폴더 안내 자리: 조사 문서는 "작업 공간 위 안내 줄 + 버전 관리 시작", 지금 코드는 홈 화면 안 안내. 추천 1b는 시작 화면 위쪽 자리에 두고(작업 공간이 저장소 없는 상태를 다루지 않아도 됨), 작업 공간 안내는 나중. spec.md 진입점의 옛 문구 "이 폴더를 Git으로 관리할까요?"는 이번에 "이 폴더는 아직 버전 관리를 하지 않아요"로 바꿨다. 기존 messages `notARepo` "이 폴더는 아직 Git으로 관리하지 않는 폴더예요."도 8단계에서 같은 문장으로 바꿀까요? 추천: 예.
- Q9. 시작 화면에서 새로 고침(fetch)을 할까요? 추천: 하지 않는다(프로젝트마다 네트워크, SSH 실패 안내가 여러 개 뜰 수 있음). "가져올 버전"은 마지막 새로 고침 기준이라고 받아들인다.
- Q10. 조사 그림의 "+ 폴더 열기"의 `+`: 추천 빼기(design.md "의미 없는 아이콘 남발" 피하기, 문구 규칙상 버튼은 동사). 대안: 그림대로 둠.
- Q11. 목록 줄 높이: 그림은 약 76px(1920px 그림 기준). 추천 48px(design.md 목록 행 32px보다 크게, 작은 노선도 칸이 들어갈 높이).
- Q12. 목록 줄의 브랜치 이름: 추천 `main`·`master`가 아닐 때만 "브랜치 이름 · 상태". 대안: 늘 보이기, 또는 보이지 않기.
- Q13. 경로를 `~/…`로 줄여 보일까요? 추천: 1b는 전체 경로(줄임표), 원하면 platform에 홈 폴더 함수를 더하는 작은 단계 추가.
- Q14. 조사 문구 조정: "합치다 멈춤 · 충돌 2곳" → 1b는 "충돌 난 파일 2개"(곳 수는 파일 안 충돌 개수라 반복 2에서), "받을 버전 5개" → "가져올 버전 5개"(D-024), "디자인 개편 브랜치 · 변경 사항 12" → "디자인-개편 · 바뀐 파일 12개". 추천: 이대로. 참고: "올릴 버전", "가져올 버전"은 `push`·`pull` 쉬운 이름의 활용형이라 D-029처럼 glossary에서 꺼낼 수 없다. 지금 있는 `notUploaded` "아직 올린 적 없음"과 같은 처지다. 용어표가 바뀌면 손으로 함께 고쳐야 한다는 점을 messages 주석에 적는다.
- Q15. 화면 이름: spec.md 표의 "홈"을 "시작 화면"으로 바꿨다. 코드 폴더도 `src/screens/Home` → `src/screens/Start`로 바꿀까요? 추천: 예(8단계).
