# 진행 상황

이 파일만 읽고 이어서 할 수 있게 단계마다 갱신한다.

## 지금
- 반복: **다듬기**(계획 011, 사용자 결정 D-138~D-145, 근거 `docs/research/polish.md`). 반복 4b는 010-H2까지 하고 멈춤(다듬기 뒤 010-H4부터, H3은 앞당겨 끝남).
- 단계: 011-P12 끝. UX 다듬기 계획 012 확정(code-analyst 검토 15건 반영), 로드맵은 4c 뒤·반복 5 앞
- 마지막 커밋: (이 갱신과 같은 커밋) docs: add ux polish plan 012
- 다음 할 일: 011-P13 홈 노선도 연결("이전 N개" 단추·역 툴팁) → P14 → … → P20 → 010-H4 나머지부터

## 진행 방식 (사용자 지시, 2026-10-15)
- 사람에게 묻지 않고 로드맵 끝까지 진행한다. 순서: 계획 005의 1b-1(1~26) → 1b-2(27~42) → 반복 2 → 반복 2b(버전 기록 노선도, 2026-10-16 사용자 추가) → 3 → 4 → 4b(010-H2까지) → **다듬기**(2026-10-16 사용자 추가) → 4b 나머지 → 4c → **UX 다듬기**(계획 012, 2026-10-16 사용자 추가) → 5.
- 사용자가 직접 정한 것(polish.md "정할 것" 등)은 새 결정 번호로 올리되 "(자동 결정)"을 붙이지 않는다.
- 다듬기의 글자 크기·밑줄을 고친 뒤 Ubuntu에서 홈·버전 기록·브랜치 목록·설정 스크린샷을 `docs/research/img/polish/`에 넣는다(사용자가 보고 확인).
- 새 반복은 planner가 계획을 쓰고 code-analyst가 한 번 검토한 뒤 바로 구현한다(사람 확인 생략).
- 반복이 끝날 때마다 code-analyst로 규칙·보안 점검표를 돌리고, 나온 것을 고친 뒤 다음 반복으로.
- 단계마다 커밋. 여러 단계가 쌓이면 push하고 CI 확인. CI가 실패하면 고칠 때까지 다음 단계로 가지 않는다.
- 정해지지 않은 것은 추천안으로 정해 decision.md에 올리고 제목 끝에 "(자동 결정)".
- 화면은 docs/research 그림과 design.md를 따른다. 그림이 없으면 같은 느낌으로. 문구는 이름과 숫자만(D-049, D-057).
- 금지: force push, `--no-verify`, 테스트 건너뛰기·끄기. main에 바로, 한 커밋에 한 가지. 사용자 컴퓨터 설정과 저장소 밖 파일은 건드리지 않는다. 테스트용 파일은 쓴 뒤 지운다.
- 멈출 때: 아래 "사람이 할 일"에 적고 다른 단계를 먼저 한다. 할 게 없을 때만 멈춘다. 같은 CI 실패를 세 번 고쳐도 못 풀면 적고 멈춘다.

## 작업 환경 메모
- **화면을 찍을 때 OS 입력(xclick·xkey)을 쓰지 않는다.** 사용자가 이 컴퓨터를 함께 쓰고 있어 hiGit 창이 다른 창 아래에 있으면 클릭·키가 사용자 창으로 간다(2026-10-16에 Chrome에 Ctrl+H 등이 간 적 있음). 장면은 웹뷰 안 임시 코드(`VITE_SHOT=<장면>`으로 켜는 커밋하지 않는 `src/dev-shot.ts`, 찍은 뒤 지움)로 만들고, 창 크기는 `xresize.py`(그 창에만 보내는 요청)로 바꾼다.
- 셸마다 `. <scratchpad>/env.sh`(cargo, nvm, 저장소 루트로 cd). cargo는 `src-tauri`에서.
- 커밋은 경로를 지정한다(`git commit -- 경로`). 새 파일은 먼저 `git add`. Rust를 고친 커밋은 먼저 `pnpm check:full`.
- 커밋하지 않는 파일: 저장소 루트 `home.zip`(사용자 파일).
- cargo-deny는 사용자 `~/.cargo`가 아니라 scratchpad `tools/bin`에 설치(세션이 바뀌면 다시 설치하거나 CI에 맡김).
- CI: push마다 이전 실행 취소. `gh run list`, `gh api repos/hi-g1/hiGit/actions/runs/<id>/jobs`.

## 1b-1 단계 (계획 005 "합친 순서")
- [x] 1 004-12 forbid unsafe
- [x] 2 004-13 clippy unwrap·expect·panic
- [x] 3 004-11 ESLint innerHTML·eval 금지
- [x] 4 004-16 Actions 권한과 SHA 고정
- [x] 5 004-15 pnpm 설치 스크립트 제한
- [x] 6 004-14 cargo-deny
- [x] 7 004-1 git 실행 환경
- [x] 8 004-2 git 절대 경로
- [x] 9 004-3 `--end-of-options`
- [x] 10 004-15b Ubuntu 22.04 git CI
- [x] 11 001-6 브랜치·원격 목록
- [x] 12 004-4 옵션 같은 이름 막기
- [x] 13 001-8 브랜치 GitOp
- [x] 14 H1 ref 읽기·쓰기
- [x] 15 H8 맡겨 두기 GitOp
- [x] 16 H11 겹침 검사
- [x] 17 004-9 CSP·capability
- [x] 18 004-10 열린 저장소만
- [x] 19 004-6 보안 기준 판정
- [x] 20 004-7 Git 업데이트 안내 (D-053: 작업 공간에도)
- [x] 21 001-10 브랜치 조회와 오류 코드
- [x] 22 004-5 링크 비교
- [x] 23 001-10b 큰 파일 비교 상한
- [x] 24 H12 시간 제한·백그라운드 모드
- [x] 25 001-9 원격 GitOp
- [x] 26 H16 오류 원문 가리기

## 1b-2 단계
- [x] 27 H2 UndoKind·journal(HMAC, D-063)·백업 ref
- [x] 28 H3 버전 저장 되돌리기·다시 하기
- [x] 29 H4 용어 sync·park, fetch "원격 확인"(D-059)
- [x] 30 H5 단축키 표(D-056)
- [x] 31 H6 토스트·Ctrl+Z
- [x] 32 H7 문서 제목형 이름(D-062)
- [x] 33 H9 맡겨 두는 갈아타기·새 브랜치
- [x] 34 H10 브랜치 목록
- [x] 35 T1~T4 저장소 신뢰 확인
- [x] 36 H13 fetch·pull·push·sync 액션
- [x] 37 H14 상단 바·버전 기록 화면(D-058)
- [x] 38 H15 동기화 버튼·자동 원격 확인(D-055)
- [x] 39 H17 홈 노선도 조회
- [x] 40 H18 두 줄 노선도
- [x] 41 (하지 않음, D-050)
- [x] 42 마무리 + code-analyst 점검(7981794, 고침 ae1094c~1348a21)

## 1b 끝 점검에서 반복 2로 넘긴 것
- journal 서명을 저장소·줄 순번에 묶기, 키를 못 쓸 때 알리기(journal 형식을 바꿀 때 같이)
- 맡긴 ref가 앱이 만든 stash 모양인지 확인하고 꺼내기
- 이미 맡긴 것이 있을 때 덮어쓰며 백업 ref로 옮긴 것을 작업 내역에서 보이게(D-054)
- 받은 저장소의 `remote.<n>.pushurl`·`url.*.pushInsteadOf`·`core.worktree`를 신뢰 확인에 넣을지
- H13 빠진 테스트: commit.gpgsign, 이름 없음, upstream 강제 갱신·사라짐, push 거부 뒤 재시도
- 목록 안 방향키·Enter·Esc는 부품 안 조작으로 보고 `shortcuts.ts` 밖에 둠(Ctrl 조합만 표에)

## 반복 2 단계 (계획 006, 바뀐 순서)
- [x] R1 결정 D-069~D-086·용어(047ecf7)
- [x] R2 caution 실행과 백업(e8d0040)
- [x] R3a 넣기·빼기 되돌리기, journal 호환(e434f4c)
- [x] R3b RestoreTree·refs·Barrier(e68d582)
- [x] R4 올린 뒤 되돌리기(revert)(f0266c5)
- [x] R5 로그 파일(44bf63c)
- [x] R6 비밀 파일 검사·ignore(59ce686, 5ba6e1a)
- [x] R7 비밀 파일 알림 줄(aad1b30)
- [x] R8 충돌 git 조각(+SnapshotFiles)(0a9b820)
- [x] R9a 합치기 제안 조회·merge(ceaaa83)
- [x] R9b merge-into(앞으로 감기)(ceaaa83에 포함)
- [x] R10 충돌 조회(fc36707)
- [x] R11a 조립 함수 / R11b 마치기 / R11c 그만두기(8a23291, 셋을 한 커밋)
- [x] R13 충돌 모델·단축키(e18d5e0)
- [x] R14 충돌 해결 화면(4f42ab2)
- [x] R12 합치기 제안 화면(355a0c9)
- [x] R9c 동기화 겹침 → 합치기 시작(013c5d0)
- [x] R15 임시 보관(a9e40ac)
- [x] R16 겹친 맡긴 파일 해결(5773a98)
- [x] R17 고친 것 버리기(3812d47)
- [x] R19a 백업 정리(d2ae5c6) / R19b 작업 내역 조회(8d231b4) / R19c 되살리기(9453106)
- [x] R20 작업 내역 화면(20330d7)
- [x] R18 임시 보관·버리기 화면(8a09ee4)
- [x] R21 진행 감시·취소(git·platform)(f6601ce)
- [x] R22 op-progress·취소 명령(d9727b7, Windows 테스트 고침 986daf0)
- [x] R23 진행·취소 화면(df01573)
- [x] R24a 멈춘 작업 조회(b83df94) / R24b 잠금 정리·맡긴 항목 복구(87dc3c3)
- [x] R25 끝나지 않은 작업 알림(9bb7f1c)
- [x] R26 마무리 + code-analyst 점검(위반 5건 고침: 5388167 링크 따라 지우기, e532452 가짜 맡긴 항목, 8cf95a6 worktree 확인, 9f4797d 문구 용어. 더해서 2fffa5a 확인 대화상자 포커스, 80222aa 강제로 멈추기 core 확인·되돌리기 진행, ebdaf2d·7b838b1·5cf1781, 그 전에 8c36669·81e5b12)

## 반복 2 끝 점검에서 다음으로 넘긴 것
- `<git 폴더>/higit/unpark.json` 표식 서명(지금은 형식만 본다. 지우기는 링크를 따라가지 않게 막았다)
- 작업 내역: 서명 안 된 journal 줄의 해시·시각으로 reflog 줄을 숨기지 않기, 서명 안 된 줄 수 제한. 백업 ref·맡긴 항목을 200개로 자른 뒤에 `git log`·`parked_paths`를 부르기(성능)
- `take-out`도 `safe_paths` 검사를 거치기(지금은 git의 `.git` 경로 거부에 기댐)
- 충돌 선택의 "직접 고치기" 글이 마치기 전까지 `localStorage`에 평문으로 남음
- 취소 표시가 저장소 단위라, 강제로 멈추기가 같은 저장소의 저절로 하는 원격 확인도 멈춤
- 진행 알림은 1초부터 보낸다(계획은 3초 뒤부터). 화면이 3초 뒤에 "취소"를 보인다
- 홈 두 줄 노선도 첫 역 이름이 왼쪽 끝에 붙음 → 반복 2b 노선도에서 같이

## 반복 2b 단계 (계획 007, 바뀐 순서)
- [x] 계획 b05e571(planner + code-analyst 검토 반영)
- [x] M1 결정 D-087~D-096·다크 그래프 색(68c82dc)
- [x] M2 여러 끝점 기록·태그(252ec95, 태그 테스트 고침 6221627·d70ad2b)
- [x] M3 버전별 줄 수(5af21bc)
- [x] M4 기록 조회 확장(323b4d5)
- [x] M5 줄 수 조회(d785c87)
- [x] M6 시작 화면 노선도 조회(d49aca5)
- [x] M7 공용 장면·Renderer(07ab6f9)
- [x] M8 화면 모델·레인 배치(e41645c)
- [x] M9 확대·축소 접기(ca03831)
- [x] M10a 세로 장면·굵기·팔레트·판정(861fc61)
- [x] M10b 홈·시작 화면을 같은 배치로, 첫 역 이름 고침(4ec4c14)
- [x] M11 목록·가상 스크롤·키보드(d7cebb6)
- [x] M12a 쪽 넘기기·줄 수·기록 무효화 도우미(dbcc710, 테스트 고침 e0afe11)
- [x] M12b 버전 기록 화면(4561ea9)
- [x] M13 역 메뉴 틀(134fcaf)
- [x] M14 시작 화면 큰 노선도(40bffb6) / M15 작은 노선도(3015dc4)
- [x] M16 마무리: 성능 측정(0b3ebb1), 문서(69fb985), code-analyst 점검(위반 3건 고침: 9cceec9 메뉴 안 이벤트가 목록 선택을 바꿈, 942d122 목록 키 판정을 src/lib로, d29a4fc 브랜치 표시 용어. 더해서 57c889e)

## 반복 2b 끝 점검에서 반복 3으로 넘긴 것
- 역 메뉴 틀: `MenuEnv.merging`을 실제 상태와 잇기, `ctx`에 되돌릴 수 없는 액션의 확인 창(D-043) 여는 방법 넣기, 흐린 항목의 까닭을 네이티브 `title` 대신 디자인 시스템 Tooltip으로, 마우스 오른쪽 클릭(contextmenu)으로도 열지 정하기
- 접힌 구간을 펼치면 선택(접힌 구간 id)이 사라짐: 펼친 첫 버전을 고르게
- 시작 화면 상태 요약(`project_summary`)도 동시 실행 큐에 넣을지(지금은 노선도만 3개)
- `hitTest`는 아직 화면에서 쓰지 않음(행 단위 `rowAt`만). 노선도 위에서 역을 직접 끄는 기능이 생기면 씀

### 반복 2b 성능(1만 버전, Ubuntu 릴리스)
- 첫 쪽 200개 40ms, 깊은 쪽 36ms, 25쪽 906ms, 줄 수 100개 10ms. 화면 배치 1만 행: buildRoute 23ms, 장면 10ms(주요 버전 단계 2ms·2ms).

## 반복 3 단계 (계획 008, 바뀐 순서)
- [x] 계획 a255841(planner + code-analyst 검토 반영)
- [x] E1 결정·용어
- [x] E2 올림 판정·메시지 / E3 버전 파일·비교 / E4 amend·cherry-pick·revert git / E5 태그·브랜치 git / E6 rebase -i git
- [x] E7 guard / E8 서명 표식 / E9 작업 맥락 / E10 amend / E11 revert·cherry-pick / E12 마치기·그만두기 / E13 reset / E14a 계획 조회
- [x] E14b rebase-i / E15 정리 충돌 / E16 태그 / E17 브랜치 지우기·이름 / E18 새 브랜치 from / E19 버전 상세 / E20 작업 내역 신뢰 경계
- [x] E21 메뉴 틀 보완 / E26 충돌 화면 새 맥락 / E22 되돌리기·복사·돌아가기 / E23 태그·새 브랜치 / E24 고치기 / E25 파일 보기 / E27 여러 역 선택 / E28 카드 모델 / E29a 카드 지도 / E29b 끌기 / E30 브랜치 목록 / E31 작업 내역
- [x] 실제 앱(Ubuntu dev)에서 확인: 역 메뉴를 마우스로 열면 바로 닫히던 문제(2b부터) → e5fd2a6, 대화상자를 닫은 뒤 목록 포커스 → 9ecba1d. 기록 정리(옮기기·묶기·정리하기·Ctrl+Z로 원래 해시), 파일 보기, 고치기 대화상자, 구간 선택 화면을 눈으로 봄.
- [x] E32 문서
- [x] E32 마무리·점검: code-analyst 반복 끝 점검(보안 핵심 위반 없음, 위반 5건 고침: ede477a 테스트 밖 unreachable·이름 바꾸기 전 맡긴 ref 이름 길이·정리 이어가기마다 할 일 검증, 06a38b7 브랜치 목록 포커스·목록 구조, 702fa76 Shift+클릭 판정을 src/lib로·카드 알림·정리 중 나가기 막기·보호 브랜치 문구, 문서 쓰기 위치 정리)

## 반복 3 끝 점검에서 다음으로 넘긴 것
- Esc·Enter 판정이 화면 부품 여러 곳에 있다(History·Activity·Rewrite·Card·BranchList·대화상자). `src/lib`에 닫기·확정 판정 함수를 두고 모을지(D-056 취지)
- `commit_messages`·`log` 파서가 0x1e 구분자를 믿는다: 메시지 안의 0x1e와 가짜 해시로 다른 버전의 메시지를 보이게 할 수 있다(표시만, 추측). 레코드 검증 강화
- `rebase_state`가 `rebase-merge` 폴더 링크를 따라간다(읽은 값은 서명 표식과 맞을 때만 씀)
- `marker.rs` 링크 거절 테스트가 Unix에서만 돈다(Windows는 링크 만들기에 권한 필요)
- 반복 4 이후로 넘김(D-097): 설정 화면 + 덮어쓰며 올리기 + 올린 기록 바꾸기, 원격 브랜치 지우기·이름 바꾸기, 태그 올리기

## 반복 4 단계 (계획 009, 바뀐 순서)
- [x] 계획 65ad359(planner + code-analyst 검토 반영)
- [x] G1 결정·용어
- [x] G2 의존성(CI fd35f66 두 OS·기본 git·deny 초록, Windows VC++ 런타임 없이 뜸): ureq 3.4(rustls-no-provider·platform-verifier) + rustls 0.23(ring), secrecy 0.10, keyring-core 1.0 + Windows·zbus 스토어. deny 허용 목록 변경 없음, webpki-roots 없음. 새 중복 버전: ring의 windows-sys 0.52·getrandom 0.2, secret-service의 digest·hmac·sha2 0.11 계열(맞출 수 없음). 실행 파일 크기는 코드가 쓰기 시작하는 G20에서 잰다
- [x] G3 platform 키체인(`Keychain` get/set/delete, Windows는 Local 보관, 실패는 Unavailable·Denied·Other)·`current_exe_path`
- [x] G4 hosting `parse_remote`(호스트만 보고 GitHub 판정)·`HostingError`(종류+앞 200자 설명)·`HttpTransport`·`GitHubConfig`(client id 영숫자 8~64자)
- [x] G5 `HostingProvider`(id·detect·begin_login·poll_login·current_user)·`GitHub`, 응답 검사(verification_uri 정확 일치, user_code 모양, 시간 보정), 시험용 `hosting::testing::FakeTransport`
- [x] G6 `list_prs`·`pr_detail`(check-runs 403·404면 시험 결과 없음)·`create_pr`(serde로만 JSON), 422 나눔, 같은 API 주소 안 리디렉트 3번까지
- [x] G7 `UreqTransport`(rustls ring + OS 인증서, 환경 변수 프록시, Authorization은 bearer 칸에서만, 오류에 URL 없음), 로컬 TcpListener 시험
- [x] G8a 실패 종류 3개(원격 연결·올리기 거절보다 먼저 봄)·`ErrorCode`·ko/en 문구·바인딩
- [x] G8b 취소 깃발 키를 (저장소, User|Background)로. `request_cancel`·`clear_cancel`·로컬 쓰기는 User, 저절로 하는 원격 확인은 Background
- [x] G8c `remote_urls`(get-url --all [--push]), `AuthMode`, `set_helper_program`, Helper 호출에만 `-c credential.helper=` + 상수 helper 설정 + `HIGIT_CREDENTIAL_HELPER`. 로컬 http.* 조회는 기존 `local_config`(local·worktree, include 포함)를 core가 거른다
- [x] G9 `GitOp::Clone`(부모 폴더 cwd, `--no-recurse-submodules --origin origin --end-of-options`), 실패 종류 `TargetExists`→`CloneTargetExists`
- [x] G10a `core::github::GitHubService`(github.json·키체인·메모리 토큰, 간격 안 폴링은 네트워크 없음, 401은 같은 세대만 끊음, 비밀 글자 지움), `CoreError::Github`·`ErrorCode` 15개·문구, `remote_host`→`hosting::GitHub::detect`
- [x] G10b 명령 6개(모두 async + blocking, 네 곳 등록)·`api.ts`, `Core::hosting_page_url`(종류만 받고 주소는 core가, 검토 요청은 열린 저장소의 GitHub 원격), D-122
- [x] G11 `git_credential_main`(get만, github.com https·사용자 이름 없음일 때만 키체인, 10초), `main`이 먼저 갈라짐, `auth_for`(주소 모두 github https·로컬 http.*/proxy 없음·이번 실행 키체인), helper 인증 실패 뒤 401이면 연결 끊김, `useAutoFetch` 인증 실패 목록
- [x] G12a 액션 `clone`(31번째, safe/Nothing), `validate_clone_url`·`validate_clone_name`, Core가 열린 경로 대신 대상 검사(부모 폴더·링크 아님·.git 안 아님·대상 200자·없거나 빈 폴더·Git 보안 기준), 대상 경로로 잠금·진행·취소(정확히 같을 때만), 실패 때 이번에 만든 폴더만 지움, 오류 코드 3개
- [x] G12b `query_clone_defaults`(네 곳, `api.cloneDefaults`)
- [x] G13a 대상 원격(upstream 원격, 없으면 하나뿐·origin)의 올리기 주소가 github.com일 때만, head는 upstream 원격 브랜치 이름. 조회 4개(hosting_context·pr_draft는 네트워크 없음), `NotGithubRemote`
- [x] G13b 액션 `pr-create`(32번째, safe/Nothing, 원격 잠금만, journal 남기지 않음), `NotPushed`·`PrSameBranch`, 보내기 전 검사, D-123
- [x] G14 `ui/DialogFrame`(부제 선택, 폭 narrow/wide, 키 차단, `composing`)·`lib/copy`, 역 메뉴 대화상자 4개가 사용
- [x] G15 `features/settings`(SettingsDialog·GitHubSection·ConnectPanel·settingsStore), `features/github/useConnect`(간격 연쇄·창 포커스 때 한 번·닫으면 취소), `useGitHubStatus`, 시작 화면·상단 바 "설정", 원격 없고 미연결이면 "GitHub 연결"
- [x] G16 `features/clone`(CloneDialog·cloneStore·suggestName·joinPath), 단축키 표 `clone`(PLANNED에서 이동), 시작 화면 `<Term clone>` Ctrl+Shift+O, 진행·3초 뒤 취소, 성공하면 프로젝트 열기
- [x] G17 `screens/PullRequests`(listbox, 열림·닫힘·전체, 더 보기, 줄 아래 상세: 시험 결과·겹침·뒤처짐·GitHub에서 열기), `features/pr`(model·PrRow), `view: "pr"`·`selectedPr`, `useHostingContext`·`usePrPages`(60초)·`usePrDetail`·`usePrForBranch`
- [x] G18 `features/pr/CreatePrDialog`·`prDialogStore`, `usePrDraft`, 올리지 않은 버전이 있으면 `push` → `pr-create`, 이미 열린 요청·차이 없음이면 만들기 꺼짐, 성공 토스트 "열기"
- [x] G19 `features/pr/PrSlot`(미연결 → 연결 패널, 있으면 번호·상태 → 검토 요청 화면의 그 줄, 없으면 새 검토 요청), 브랜치 목록의 "main에 합치기" 자리
- [x] G20 문서: architecture(HostingProvider·HttpTransport·GitHub 연결 절, 셸 두 곳, 시간 제한 표, 취소 깃발, 액션 32개, 명령 11개, 보안 표 세 줄, 입력 상한), spec 화면 행, roadmap, code-analyst 점검 줄 6개
- [x] 반복 4 CI 4774d69 두 OS·기본 git·deny 초록
- [x] 반복 4 끝 점검(code-analyst 규칙·보안): 핵심 보안 항목 위반 없음. 고침: 검토 요청 문구의 "브랜치" 직접 사용(34d215c, 합칠 대상은 "대상 브랜치"로), 로그인 폴링 오류 뒤 간격(759a680), 복제 실패 정리는 git이 만든 폴더만(08ae587), 점검표 문구(trust.rs 예외, 복제의 감시 실행)

### 반복 4 끝 점검에서 다음으로 넘긴 것
- 화면의 `e.key === "Escape"`·`"Enter"` 판정이 `src/lib` 밖에 있다(PullRequests·CloneDialog, 기존 Activity·Rewrite·이름 대화상자와 같은 방식). D-056을 엄격히 읽으면 `src/lib`로 옮길 후보 → 반복 5-0
- Windows 릴리스 exe(GUI 하위 시스템)에서 helper 표준 입출력과 Git for Windows `sh`의 공백 경로 풀기는 손 확인("사람이 할 일")

## 다듬기 단계 (계획 011, 끝 절 "검토 반영" R1~R14가 우선)
- [x] 계획 011(planner + code-analyst 검토 14건 반영)
- [x] 011-P1 결정 D-146~D-161·문서
- [x] 011-P2 rem 토큰·CSS 단위·린트(인라인 `code`는 둘레 글자 크기를 따르고, 코드 본문(비교·충돌·파일 목록)만 `--font-size-code`. 시험 `mono-size.test.ts`)
- [x] 011-P3 노선도·목록 배율(`uiStore.uiScale`, `SvgRenderer scale`, `SCENE_FONT`, 이름표 폭은 글자 폭 + 16. `hitTest`는 화면에서 부르는 곳이 없어 바꿀 것 없음)
- [x] 011-P4 OS 글자 배율·다크 읽기(platform, 이 컴퓨터는 `text-scaling-factor` 1.25라 Ubuntu 본문이 15×1.25px가 된다)
- [x] 011-P5 설정 칸 다섯·`app_display_hints`(Core 없이 도는 `higit_core::settings::display_hints`)
- [x] 011-P6 글자 크기 적용·설정 "일반" 구역·Ctrl+=·−·0(저장은 300ms 모아 한 번, 대화상자는 넘치면 스크롤)
- [x] (D-162) Ubuntu는 WebKitGTK가 Xft.dpi로 이미 배율을 반영해 앱이 곱하지 않음(역 간격 측정으로 확인, Ctrl+=·−·0이 WebKitGTK에서 앱에 오는 것도 확인)
- [x] 011-P7 상단 바 테두리 맞춤·줄 바꿈(800px·아주 크게 손 확인: 원격 없는 데모에서 넘침 없음, 왼쪽 칸은 창 폭 45%까지)
- [x] 011-P8 밑줄 줄이기·용어 툴팁 600ms·"용어 설명 보기"·링크 모양 한 파일·stylelint
- [x] 011-P9 Ubuntu 스크린샷(`docs/research/img/polish/p9-*.png`, 아래 "사람이 할 일" 첫 항목)
- [x] 011-P10 언어 설정·시각 표시 규칙(`formatTime`·`<Time>`, ESLint가 `Intl`·`toLocale*`·`navigator.language(s)`를 막음, ko·en 같은 모양 시험)
- [x] 011-P11 화면 모드(시스템 따라가기는 OS 다크를 알면 `data-theme`, 모르면 CSS에 맡김)
- [x] (사용자 추가) 브랜드 D-163: assets/brand, 아이콘 다시 만들기, 시작 화면·설정 "정보" 로고
- [x] (사용자 추가) 칸 나눔선 D-164: 홈·충돌 화면·이때 파일 보기
- [x] (사용자 추가, ux.md 높음 4건) 확인 창 중 전역 단축키 막기·공용 `ConfirmDialog`(67273ba, 010-H6 앞당김) / 오류 토스트 D-166(4a0a247) / 문구 속 버튼 이름(32c50f0) / 이름·이메일(152d586 git, 9b245af core, 화면: 버전 저장 오류 자리 입력과 설정 "일반" 줄)
- [x] 011-P12 노선도 배치(점선·"이전 N개"·홈 5역·이름표 폭 간격(최소 72에서 더 접음)·홈 "지금 여기" 이름표)
- [ ] 011-P13 홈 노선도 연결
- [ ] 011-P14 비교 전체 문맥 / 011-P15 접힌 줄·펼치기·줄 바꿈
- [ ] 011-P16 파일 줄 두 줄·"고침" / 011-P17 체크 모양 / 011-P17b 파일 줄 오른쪽 메뉴
- [ ] 011-P18 압축·실행 파일 알림 / 011-P19 디테일·고급 구역 / 011-P20 마무리·재촬영·점검

## 반복 4b 단계 (계획 010, 검토 반영 순서)
- [x] 계획 0dcaa22(planner + code-analyst 검토 17건 반영)
- [x] 010-H1 결정·용어
- [x] 010-H2 앱 설정(`core::settings`, 쓰기 실패는 메모리, `ExecCtx::new`로 시험 생성부 정리)
- [x] 010-H3 전역 이름·이메일(git, 다듬기에서 앞당김 152d586)
- [ ] 010-H4 identity-set·query_identity(다듬기에서 앞당김 9b245af, 남은 것: `noreplyEmail`) / 010-H5 처음 실행 패널(설정 "일반"의 이름·이메일 줄과 `IdentityForm`은 이미 있음)
- [ ] 010-H6 공용 확인 대화상자 / 010-H7a 원격 ref 실패 분류 / 010-H7b lease push·ref 삭제·태그·ls-remote
- [ ] 010-H8 D-097 완화·rewrote 표식 / 010-H9 force-push / 010-H10 고급 설정·덮어쓰며 올리기 화면
- [ ] 010-H11 원격 브랜치 지우기·이름 바꾸기 / 010-H12 tag-push / 010-H13 원격 브랜치 메뉴·태그 올리기 화면
- [ ] 010-H14 hosting 저장소·리뷰·권한 / 010-H15 publish / 010-H16 저장소 목록·권한·리뷰 / 010-H17~H19 화면 / 010-H20 마무리

## 다음 반복
- [ ] 반복 4c: GitHub 쓰기(D-124)
- [ ] UX 다듬기(계획 012, 2026-10-16 사용자 추가, 검토 반영 R1~R15가 우선): 012-U1~U31
- [ ] 반복 5: OS 통합과 배포(5-0 배포 전 보안 묶음부터)

## 사람이 할 일
### 실제 컴퓨터에서 확인 (모아서 한 번에)
- **칸 나눔선(D-164)**: 홈·충돌 화면·"이때 파일 보기"에서 선을 끌 때 커서가 좌우 화살표로 바뀌고 선이 진해지는지, 끄는 동안 글자가 선택되지 않는지, 더블클릭하면 기본 폭, Tab으로 닿아 ←·→, 창을 줄이면 50%로 맞춰지는지, 껐다 켜도 폭이 남는지(두 OS).
- **브랜드 아이콘(D-163)**: Windows 작업 표시줄(16·24·32·48px, 100%·150% 배율)과 Ubuntu 앱 목록(설치한 .deb/AppImage)에서 아이콘이 흐리지 않은지. 이 컴퓨터에서는 창을 앞으로 가져오거나 앱을 설치하지 않아(사용자 창·설정을 건드리지 않으려고) 생성된 16·24·32·48px 파일을 1:1·8배로 늘린 그림만 만들었다(`docs/research/img/brand/icon-sizes.png`). 로고는 `logo-start(-dark).png`, `logo-settings(-dark).png`.
- 이 컴퓨터는 OS 언어가 English(`LANG=en_US.UTF-8`)라 "시스템 따라가기"에서 앱이 English로 나온다(D-139대로). 한국어로 보려면 설정 → 언어 → 한국어.
- **다듬기 P9 그림 확인(가장 먼저)**: `docs/research/img/polish/`의 그림으로 글자 크기와 밑줄을 봐 주세요. Ubuntu 22.04(X11), 창 1200×780, 이 컴퓨터의 GNOME 글자 배율 1.25(WebKitGTK가 반영, D-162)에서 찍었다.
  - `p9-home.png`: 홈, 글자 크기 "보통"(본문 15px × 1.25). 상단 바·버튼에 밑줄이 없는지, 브랜치 단추가 프로젝트 이름처럼 테두리 없는지.
  - `p9-home-xlarge.png`: 같은 홈, "아주 크게".
  - `p9-home-800-xlarge.png`: 창 800×600, "아주 크게". 상단 바 오른쪽 묶음이 아랫줄로 내려가고 가로 스크롤이 없다. 노선도가 오른쪽에서 잘리는 것은 011-P12(간격 규칙)에서 고친다.
  - `p9-log.png`: 버전 기록, 역 메뉴를 연 모습(r-log.png와 같은 장면).
  - `p9-branches.png`: 브랜치 목록(묶음 제목이 줄 옆에 끼던 문제는 11b8c5e에서 고침).
  - `p9-settings.png`: 설정, "일반"(글자 크기·용어 설명 보기)과 "GitHub" 구역.
  - 아직 바뀌지 않은 것(뒤 단계): 노선도 "…"(P12·P13), 파일 줄 한 줄·"수정"(P16), 모두 빈 체크(P17), 비교 칸 "…"(P15), 언어·화면 모드 설정(P10·P11).
  - 동기화 옆 주황 점은 데모 원격이 로컬 폴더라 앱 보안 규칙(file 프로토콜 막음)으로 원격 확인이 실패한 표시다. 실제 원격에서는 나오지 않는다.
  - 크기가 마음에 들지 않으면 알려 주세요. `src/styles/tokens.css`(크기 토큰)와 `src/lib/fontScale.ts`(단계 값)만 고치면 된다.
- 반복 4 검토 요청(G17·G18): 연결한 뒤 GitHub 저장소에서 상단 "검토 요청" 목록·필터·상세(시험 결과·겹침)·"GitHub에서 열기", 기능 브랜치에서 "새 검토 요청"으로 실제로 만들기(안 올린 버전이 있을 때 올리기 먼저), 만든 뒤 토스트 "열기". 800px·다크, 두 웹뷰의 `select` 모양, 글상자 IME와 Ctrl+Enter.
- 반복 4 복제(G16): 시작 화면·작업 공간에서 Ctrl+Shift+O가 두 웹뷰(WebView2·WebKitGTK)에서 앱에 오는지, 폴더 고르기 창, 한글 폴더 이름·긴 경로, 공개 저장소 https 복제와 GitHub 비공개 저장소(연결됨/안 됨), 3초 뒤 취소하면 만든 폴더가 지워지는지, 800px·다크.
- 반복 4 설정(G15): client id를 넣은 빌드(`HIGIT_GITHUB_CLIENT_ID=<id> pnpm tauri dev`)에서 설정 → GitHub 연결 → 기본 브라우저에 코드 페이지가 열리고, 코드를 넣으면 앱이 "GitHub에 연결했어요"·`@로그인`·"이 컴퓨터에 저장됨"을 보이는지. 껐다 켜도 연결돼 있는지. Ubuntu 키링 잠금 해제 창, 연결 끊기 뒤 "GitHub에서 권한 지우기". 800px·다크.
- 반복 4 helper(G11): **Windows 릴리스 exe(GUI 하위 시스템)**에서 git이 준 표준 입출력 파이프로 `--git-credential get`이 답하는지(연결한 뒤 GitHub 비공개 저장소 https 동기화가 GCM 창 없이 되는지). 설치 경로에 공백이 있는 `C:\Program Files\…`에서도. Ubuntu에서 같은 동기화.
- 반복 4 키체인(G3): 두 OS에서 `cd src-tauri && cargo test -p platform -- --ignored os_keychain_round_trip`이 통과하는지(Ubuntu는 GNOME 키링 잠금 해제 창이 뜰 수 있음, Windows는 자격 증명 관리자에 `dev.higit.test-only.round-trip`이 잠깐 생겼다 지워짐).
- 반복 2 충돌 해결 화면: 실제로 겹치는 합치기를 만들어 끝까지(곳마다 고르기, 직접 고치기 글상자의 한글 IME, 1·2·3·Ctrl+↑↓가 두 웹뷰에서 오는지, 마치기·그만두기), 앱을 중간에 껐다 켜도 고른 것이 이어지는지, 800px·다크.
- 합치기 제안(main 가져와 합치기·main에 합치기)과 동기화 중 겹침이 충돌 화면으로 가는지, 브랜치 목록 "겹침 해결".
- 비밀 파일 알림 줄: `.env`를 만들고 저장하면 알림 줄, 다시 저장하면 저장되는지.
- 시작 화면: Ctrl+O, Ctrl+Shift+O가 두 OS 웹뷰에서 앱에 오는지. 한글 IME. 800px 너비.
- 릴리스 실행 파일(`pnpm tauri build --no-bundle`)로 두 OS에서 시작 화면·폴더 열기·작업 공간·비교 보기를 쓰고, 웹 검사기 콘솔에 CSP·권한 오류가 없는지(dev에는 CSP가 없다). 릴리스에서 개발자 도구가 열리지 않는지.
- Windows에서 Git for Windows가 보안 기준(2.43.7 등)보다 낮을 때 시작 화면 "내려받기"가 기본 브라우저를 여는지.
- 홈(1b-2): Ctrl+B 브랜치 목록, Ctrl+H 버전 기록, Ctrl+Shift+S 동기화, Ctrl+Z·Ctrl+Shift+Z가 두 OS 웹뷰에서 앱에 오는지(입력칸에 포커스가 있을 때 포함).
- 브랜치 목록 찾기 칸에 한글 IME로 브랜치 이름을 쓰고 Enter: 조합 중 Enter는 무시하고 조합이 끝난 뒤에만 새 브랜치를 만드는지.
- 동기화: GitHub 비공개 저장소(HTTPS·SSH 각각)에서 동기화 버튼과 자동 원격 확인. 자동 원격 확인 때 자격 증명 창(GCM)이나 SSH 질문이 뜨지 않는지, 사용자가 누른 동기화에서는 GCM 로그인 창이 뜨는지.
- 토스트 위치(화면 아래 가운데)와 8초, 마우스를 올리면 멈추는지. 800px 너비와 다크 모드에서 홈·브랜치 목록·버전 기록.
- 신뢰 확인 대화상자: hooks나 실행 설정이 있는 저장소(받은 zip)를 열면 뜨는지, "끄고 열기"로 연 뒤 hook이 돌지 않는지. Windows에서 다른 사용자 소유 폴더(dubious ownership) 안내.
- 비교 칸 아래 줄: 임시 보관·보관 목록에서 꺼내기, "고친 것 버리기" 확인 대화상자(파일 이름)와 버린 뒤 토스트의 "작업 내역" 단추.
- 작업 내역 화면(Ctrl+Shift+H): 버전 저장·되돌리기·버리기를 한 뒤 카드가 보이는지, "이 버전으로"·"꺼내기"가 되는지, 800px·다크.
- 동기화 진행: 느린 원격(큰 저장소 복제 직후 등)에서 "동기화 중 · N초", 3초 뒤 "취소", 취소하면 토스트만 뜨는지. 네트워크를 끊어 "응답 없음 · N초"와 90초 뒤 "진행이 멈춰 그만뒀어요" + [동기화].
- 로컬 쓰기 강제로 멈추기: 서명 핀 입력을 기다리는 버전 저장(gpg)이나 오래 도는 pre-commit hook으로 60초를 넘겨 "응답 없음 · 강제로 멈추기" → 확인 → 멈추는지. 두 OS(Windows는 taskkill).
- 멈춘 작업 정리: 강제로 멈춘 뒤 남은 `index.lock`이 60초 뒤 "멈춘 작업 파일 N개" 알림 줄로 보이고 [정리하기] 확인 뒤 지워지는지. 다른 도구(IDE)의 git이 도는 중에는 안 보이는지.
- 남은 맡겨 둔 파일: 브랜치 갈아타기 도중 앱을 강제로 끝내 stash 목록에 "higit parked …"를 남긴 뒤 다시 열면 [복구하기]가 보이는지.
- 반복 2b 버전 기록 노선도: 머리의 "−"·"+"와 `+`·`-`(숫자패드 포함), Ctrl+휠·터치패드 핀치가 WebView2·WebKitGTK에서 페이지 확대가 아니라 앱에 오는지. 범위 전환, "지금 여기", 접힌 구간 펼치기. 800px·다크·고대비에서 선·점·글자가 겹치거나 흐리지 않는지(2px 선, SVG 글자 기준선).
- 역 메뉴: 역을 누르거나 Enter로 열고 "버전 번호 복사"가 두 웹뷰에서 클립보드에 들어가는지(navigator.clipboard가 안 되면 옛 방법). 안 되면 clipboard 플러그인(새 의존성) 결정이 필요. Radix 메뉴가 가상 스크롤 위에서 제자리에 뜨고 Esc·포커스가 맞는지.
- 시작 화면: 큰 노선도가 start-normal.png(라이트)·start-normal-dark.png와 비슷한지, 프로젝트 20개쯤에서 작은 노선도가 차례로 채워지며 Windows에서 느려지지 않는지, 한글 경로·긴 이름.
- 홈 두 줄 노선도: 저장 1~2개짜리 프로젝트의 한 줄 보기에서 첫 역 이름이 잘리지 않는지, r-home.png와 같은 모양인지.
- 버전 1만 개 저장소의 버전 기록에서 WebKitGTK(Ubuntu)가 키가 아주 큰 SVG(48만 px)를 문제없이 그리는지(문제면 화면 높이 SVG를 옮기는 방식으로 바꾼다). 노선도 위 마우스 오른쪽 클릭에 웹뷰 기본 메뉴가 뜨는지.
- 반복 3 역 메뉴: 오른쪽 클릭·터치패드 두 손가락 클릭이 WebView2·WebKitGTK에서 역을 골라 메뉴를 여는지(웹뷰 기본 메뉴가 안 뜨는지), 흐린 항목의 툴팁이 마우스·키보드로 뜨는지, "더 보기" 하위 메뉴.
- 반복 3 Windows: 기록 정리(`GIT_SEQUENCE_EDITOR`가 Git for Windows의 `sh`로 `printf %s "$HIGIT_TODO" >`를 도는지, 한글 메시지 고치기), 복사·되돌리기·기록 정리가 겹쳐 충돌 화면에서 끝까지(앱을 중간에 껐다 켜기 포함), 편집기가 파일을 잡고 있을 때 이 버전으로 돌아가기(`reset --keep`)가 오류로 알리고 상태가 그대로인지.
- 반복 3 카드 지도: 끌기(포인터 캡처)가 두 웹뷰·고DPI Windows에서 되는지(안 되면 키·단추만 남긴다), 끄는 중 글자 선택이 번지지 않는지, Alt+↑↓가 웹뷰 메뉴 단축키와 겹치지 않는지, 800px·다크.
- 반복 3 이름 대화상자: 태그·새 브랜치·브랜치 이름 바꾸기 입력의 한글 IME(조합 중 Enter 무시), 마지막 버전 고치기 글상자의 Ctrl+Enter, 대화상자가 열린 동안 Ctrl+Z·Ctrl+H·Ctrl+B가 뒤에서 돌지 않는지.

### 사용자에게 물을 것
- UX 다듬기(계획 012) planner가 남긴 질문: ① 10초 넘는 작업의 진행 막대나 1분 넘는 작업의 OS 알림이 필요한가(D-080 변경·새 의존성이 든다, 지금은 계획에서 뺐다). ② "가져가기"와 "가져오기"가 한 글자 차이인데 헷갈리지 않는지, 고른 것을 기억하는 설정이 필요한지(D-165 그대로 썼다). ③ Windows에서 브라우저 단축키가 JS로 막히지 않고 새면 platform에 WebView2 설정(`AreBrowserAcceleratorKeysEnabled`)을 허용할지. ④ 코드 글꼴 D2Coding을 번들하면 크기 상한 2MB로 괜찮은지.
- 브랜드(D-163): 앱 아이콘을 `pnpm tauri icon`으로 줄이면 32·48px은 또렷하지만 16px은 흐리다(역 고리가 회색 점이 되고 어두운 작업 표시줄에서는 먹색 바탕이 묻힌다, `docs/research/img/brand/icon-sizes.png`). 24px은 읽히지만 부드럽다. 16·24px용으로 선을 굵게 하고 역 고리를 뺀 단순한 아이콘을 따로 둘지 정해 주세요(정하면 `.ico`의 16·24 칸만 그 그림으로 바꾼다).
- 다듬기(D-162): Ubuntu에서는 WebKitGTK가 GNOME 글자 배율(이 컴퓨터 1.25)로 이미 화면 전체를 키우고 있어서, D-138의 "OS 배율을 앱이 곱한다"를 Ubuntu에서는 하지 않게 했다(한 번만 커짐). OS 배율을 따르는 결과는 같다. 다르게 원하면 알려 주세요.
- 다듬기(D-161): polish.md 6장의 "창 최소 크기 960×600"은 D-138의 "800px 너비에서 상단 바가 넘치지 않는지 시험"과 맞지 않아 지금 800×560을 유지했다. 최소 크기를 960×600으로 올릴지(그러면 800px 시험은 960px 시험으로 바뀜) 알려 주면 따른다.

### 사람만 줄 수 있는 것
- GitHub OAuth App 등록: GitHub 설정 → Developer settings → OAuth Apps → New OAuth App, 이름 `hiGit`(D-009 확인과 같이), Homepage는 저장소 주소, callback은 아무 주소(device flow는 쓰지 않음), **"Enable Device Flow" 체크**. client secret은 만들지 않는다. 나온 client id를 개발은 환경 변수 `HIGIT_GITHUB_CLIENT_ID`로, 릴리스 빌드는 CI 저장소 변수(비밀 아님)로 넣는다. 그 전에는 설정의 GitHub 연결이 "준비 안 됨"으로 보인다.
- 실제 GitHub 계정으로 연결·비공개 저장소 동기화·검토 요청 만들기 확인(반복 4 끝에 손 확인 목록으로 더 적는다).

## 기록
- 2026-10-16: R22 Windows CI 실패 한 번(테스트가 짧은 경로 `RUNNER~1`을 썼는데 core는 정규화한 경로로 기록 → 986daf0). 테스트는 열기 결과의 경로를 쓴다.
- 2026-10-16: CI 실패 두 번(테스트 저장소 이름 없음 → c2316a7, Windows 파일 이름 `*` → 7057fe6). 둘 다 테스트 쪽 문제였고 고쳤다. 새 테스트 저장소는 `user.name`·`user.email`을 저장소에 넣고, 파일 이름에 Windows 금지 글자(`*?:<>|"`)를 쓰지 않는다.
- 2026-10-15: 1b 시작 화면(003) 완료, CI 초록 7b1c1d4. 보안·홈 결정 D-034~D-049, 자동 결정 D-050~D-063. 계획 004, 005.
