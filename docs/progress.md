# 진행 상황

이 파일만 읽고 이어서 할 수 있게 단계마다 갱신한다.

## 지금
- 반복: 4 (계획 009, 끝의 "검토 반영" 절이 단계 설명보다 우선)
- 단계: G7 끝 (hosting `UreqTransport`: https만·리디렉트 안 따름·연결 5초·전체 15초·본문 2MB)
- 마지막 커밋: (이 갱신과 같은 커밋) hosting: add ureq transport with limits
- 다음 할 일: G8a 실패 종류·G8b 취소 표시 분리(작업 트리에 준비됨) → G8c 원격 주소·인증 방식 → G9 복제 … (바뀐 순서는 계획 009 검토 반영 절)

## 진행 방식 (사용자 지시, 2026-10-15)
- 사람에게 묻지 않고 로드맵 끝까지 진행한다. 순서: 계획 005의 1b-1(1~26) → 1b-2(27~42) → 반복 2 → 반복 2b(버전 기록 노선도, 2026-10-16 사용자 추가) → 3 → 4 → 5.
- 새 반복은 planner가 계획을 쓰고 code-analyst가 한 번 검토한 뒤 바로 구현한다(사람 확인 생략).
- 반복이 끝날 때마다 code-analyst로 규칙·보안 점검표를 돌리고, 나온 것을 고친 뒤 다음 반복으로.
- 단계마다 커밋. 여러 단계가 쌓이면 push하고 CI 확인. CI가 실패하면 고칠 때까지 다음 단계로 가지 않는다.
- 정해지지 않은 것은 추천안으로 정해 decision.md에 올리고 제목 끝에 "(자동 결정)".
- 화면은 docs/research 그림과 design.md를 따른다. 그림이 없으면 같은 느낌으로. 문구는 이름과 숫자만(D-049, D-057).
- 금지: force push, `--no-verify`, 테스트 건너뛰기·끄기. main에 바로, 한 커밋에 한 가지. 사용자 컴퓨터 설정과 저장소 밖 파일은 건드리지 않는다. 테스트용 파일은 쓴 뒤 지운다.
- 멈출 때: 아래 "사람이 할 일"에 적고 다른 단계를 먼저 한다. 할 게 없을 때만 멈춘다. 같은 CI 실패를 세 번 고쳐도 못 풀면 적고 멈춘다.

## 작업 환경 메모
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
- [ ] G8a 실패 종류 / G8b 취소 표시 분리 / G8c 원격 주소·인증 방식 / G9 복제(git)
- [ ] G10a 연결 서비스 / G10b 연결 명령 / G11 helper / G12a 복제 액션 / G12b 복제 기본값 / G13a 검토 요청 조회 / G13b pr-create
- [ ] G14 대화상자 공용 / G15 설정 / G16 복제 화면 / G17 검토 요청 화면 / G18 새 검토 요청 / G19 브랜치 목록 "검토 요청" / G20 마무리

## 다음 반복
- [ ] 반복 4b: 설정과 GitHub 넓히기
- [ ] 반복 5: OS 통합과 배포(5-0 배포 전 보안 묶음부터)

## 사람이 할 일
### 실제 컴퓨터에서 확인 (모아서 한 번에)
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

### 사람만 줄 수 있는 것
- GitHub OAuth App 등록: GitHub 설정 → Developer settings → OAuth Apps → New OAuth App, 이름 `hiGit`(D-009 확인과 같이), Homepage는 저장소 주소, callback은 아무 주소(device flow는 쓰지 않음), **"Enable Device Flow" 체크**. client secret은 만들지 않는다. 나온 client id를 개발은 환경 변수 `HIGIT_GITHUB_CLIENT_ID`로, 릴리스 빌드는 CI 저장소 변수(비밀 아님)로 넣는다. 그 전에는 설정의 GitHub 연결이 "준비 안 됨"으로 보인다.
- 실제 GitHub 계정으로 연결·비공개 저장소 동기화·검토 요청 만들기 확인(반복 4 끝에 손 확인 목록으로 더 적는다).

## 기록
- 2026-10-16: R22 Windows CI 실패 한 번(테스트가 짧은 경로 `RUNNER~1`을 썼는데 core는 정규화한 경로로 기록 → 986daf0). 테스트는 열기 결과의 경로를 쓴다.
- 2026-10-16: CI 실패 두 번(테스트 저장소 이름 없음 → c2316a7, Windows 파일 이름 `*` → 7057fe6). 둘 다 테스트 쪽 문제였고 고쳤다. 새 테스트 저장소는 `user.name`·`user.email`을 저장소에 넣고, 파일 이름에 Windows 금지 글자(`*?:<>|"`)를 쓰지 않는다.
- 2026-10-15: 1b 시작 화면(003) 완료, CI 초록 7b1c1d4. 보안·홈 결정 D-034~D-049, 자동 결정 D-050~D-063. 계획 004, 005.
