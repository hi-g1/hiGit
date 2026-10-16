# 013 GitHub 쓰기 (반복 4c: 댓글·합치기·닫기·포크·원격 태그 지우기·조직 저장소·저장소 찾기·이메일 비교·로그인 설정·원격 기본 브랜치)
- 관련 결정: D-003, D-006, D-010, D-015, D-020, D-021, D-024, D-027, D-029, D-034, D-035, D-037, D-040, D-042, D-043, D-049, D-057, D-063, D-068, D-080, D-097, D-099, D-104, D-109, D-111, D-112, D-113, D-115, D-118, D-120, D-121, D-122, D-123, **D-124**(4c로 넘긴 목록이 이 계획의 범위), D-125~D-136(반복 4b 결과), **D-137**(5-0 이월은 그대로), D-148, D-160, D-166. 새로 정할 것은 아래 "결정 후보" C1~C14이고, **첫 단계(013-W1)가 `docs/decision.md` 맨 아래의 다음 빈 번호부터 "(자동 결정)"으로 올린다.** 번호를 이 계획에 미리 적지 않는다(그 사이 다른 세션이 결정을 더하고 있다). 사람에게 묻지 않고 추천안으로 진행한다.
- 관련 명세: spec.md "화면 구성"의 검토 요청·설정·버전 기록(태그)·작업 공간(GitHub에 올리기) 행, "진입점"(복제 목록·GitHub에 올리기), 용어표(`pr`·`review`·`force-push`·`remote-branch-delete`·`remote-branch-rename`·`tag-push`·`publish`·`identity`)와 "위험도" 문단의 "실행 전 확인" 줄, "오류 메시지 원칙". architecture.md "HostingProvider"·"GitHub 연결"(`call`·`auth_for`), "Action"(`Irreversible`·확인 토큰·잠금 둘·진행과 취소), "기록을 바꾸는 액션의 검사"(`PublishedRule`), "보안"(위협 표·입력 상한), "프론트와 백엔드 계약"(명령 네 곳), "새 액션 추가 절차". design.md "확인 대화상자"·"설정 대화상자"·"GitHub 연결 패널"·"검토 요청 화면"·"새 검토 요청 대화상자"·"복제 대화상자"·"이름 대화상자"·"토스트"·"오류 토스트"·"알림 줄".
- 근거 자료(결정 아님): GitHub REST API 문서. [조직 목록은 OAuth 토큰에 `read:org`나 `user` 권한이 있어야 한다](https://docs.github.com/en/rest/orgs/orgs)(C10의 근거). 나머지 끝점(`PUT pulls/{n}/merge`의 `sha`·405·409, `PATCH pulls/{n}`, `POST/GET issues/{n}/comments`, `POST repos/{o}/{r}/forks`의 202, `POST orgs/{org}/repos`, `GET repos/{o}/{r}`의 `permissions`·`allow_*_merge`·`parent`, `GET repos/{o}/{r}/branches/{b}`의 `protected`, `GET user/emails`와 `user:email` 권한, 푸시 거절 GH007)은 구현하는 단계가 문서를 다시 열어 표본 응답을 시험에 옮긴다. 그림은 없다: 검토 요청 화면·확인 창의 지금 모양(`--surface` 판, 얇은 선, 먹색 모노, 글자 버튼, 위험 버튼은 확인 창에서만)을 따른다.
- 앞선 계획: 009(GitHub), **010(반복 4b)**, 011(다듬기). **이 계획은 010의 H8~H20이 끝난 코드 위에서 시작한다.** 지금 코드는 커밋 f11b0b0(010-H8 앞부분까지)이고, 010이 만들 것은 아래 "현재 상태"의 "010이 만들 것"에 적은 이름으로 기댄다. 그것을 다시 만들지 않는다. 구현 첫날(W1 전) `rg`로 그 이름들이 실제로 있는지 확인하고 다르면 `docs/progress.md`에 적고 단계 설명을 실제 이름에 맞춘다. 012(UX 다듬기)는 이 계획 **뒤**에 온다.
- 단계 이름은 W1~W21이고 `docs/progress.md`와 커밋 본문에는 "013-W3"처럼 계획 번호를 붙인다(005·010·011·012의 단계 이름과 겹치지 않게). 진행: 사람 확인 없이 Claude가 이 계획 → **code-analyst 한 번 검토**(결과는 이 문서 끝 "검토 반영"에 더하고 단계 설명보다 우선) → 아래 순서대로 구현, 한 단계 = 한 커밋(메시지 `<영역>: <요약>`).

## 목표
GitHub에 연결하면 앱 안에서 검토 요청에 댓글을 달고, 합치고(확인 창), 닫고 다시 연다. 올릴 권한이 없는 저장소는 포크를 만들어 거기서 요청한다. 원격 태그를 지우고(확인 창), 원격 기본 브랜치도 브랜치 이름을 입력하면 덮어쓰며 올리거나 지울 수 있다. 조직 저장소로도 올리고, 복제할 저장소를 찾기 칸으로 찾고, 내 이메일이 GitHub 계정의 것인지 보고, 올리기·가져오기에 hiGit 로그인을 쓸지 고른다.

## 완료 기준 (반복 전체)
두 OS(Ubuntu 22.04 git 2.34.1, Windows 11)에서 손으로, 가능한 것은 자동 테스트로 확인한다. GitHub는 시험에서 가짜 `HttpTransport`, 원격은 로컬 bare 저장소, 키체인은 모의 저장소만 쓴다. 실제 계정이 필요한 것은 `docs/progress.md` "사람이 할 일"(W21)에 모은다.
- 댓글: 검토 요청 상세에 최근 댓글 20개가 글자로만(마크다운·링크·아바타 없이) 보이고, 아래 칸에 쓰고 "댓글 달기"(또는 Ctrl+Enter, 한글 조합 중 무시)를 누르면 GitHub에 달리고 목록이 다시 읽힌다. 빈 글·64KB 넘는 글은 보내기 전에 막힌다.
- 합치기: 열린 검토 요청에 "합치기"가 있고 누르면 확인 창이 뜬다(포커스는 "닫기", Enter가 위험 버튼을 누르지 않음). 방법 "합친 버전 만들기"·"버전 하나로 묶기" 중 저장소가 허용하는 것만 보인다. 확인하면 화면이 본 head 해시(`sha`)로만 합쳐지고, 그사이 새 버전이 올라왔으면 "그사이 새 버전이 올라왔어요"로 거절된다. 겹치는 곳이 있거나 초안이면 까닭과 함께 꺼져 있다. 합친 뒤 원격 확인이 한 번 돌고, 원격 브랜치가 남아 있으면 토스트에 "브랜치 지우기"가 나와 010의 원격 브랜치 지우기 확인 창을 연다. 확인 토큰 없이 실행하면 `PreviewRequired`.
- 닫기·다시 열기: 열린 요청에 "검토 요청 닫기"(확인 창 없음), 닫으면 토스트에 "다시 열기". 닫힌(합쳐지지 않은) 요청의 상세에 "다시 열기". 권한이 없으면(작성자도 아니고 올릴 권한도 없음) 단추가 없다.
- 포크: 올릴 권한이 없는 GitHub 저장소에서 "새 검토 요청"을 열면 "이 저장소에 올릴 권한이 없어요" 안내와 주 버튼 "포크 만들고 올리기"가 보이고, 누르면 내 계정에 포크가 생기고(이미 있으면 그것을 씀) 원격 `fork`가 이어지고 지금 브랜치가 그리로 올라가며, 이어서 원래 저장소에 검토 요청이 만들어진다(head는 `<내 로그인>:<브랜치>`). 그 뒤 검토 요청 목록·브랜치 목록의 "검토 요청 #N"이 원래 저장소 기준으로 보인다.
- 원격 태그 지우기: 태그 대화상자 줄마다 "원격에서 지우기"가 있고 확인 창을 거쳐 그 태그만 원격에서 지운다(내 컴퓨터의 태그는 그대로). 원격 태그가 내 태그와 다른 버전이면 지우지 않고 알린다. 원격에 없으면 "원격에 이 태그가 없어요".
- 원격 기본 브랜치: 설정 "올린 버전도 바꾸기"가 켜져 있으면 기본 브랜치에서도 올린 버전을 고치고 덮어쓰며 올릴 수 있고, 그 확인 창에는 브랜치 이름 입력칸이 있어 똑같이 입력해야 위험 버튼이 켜진다(core도 입력을 다시 확인). GitHub가 보호하는 브랜치면 확인 창에 "GitHub가 보호하는 브랜치예요. 서버가 거절할 수 있어요."가 보이고, 거절되면 "GitHub에서 보기" 단추가 나온다. 이름만 기본 브랜치 같은 원격 브랜치(`main`·`master` 등이지만 서버 기본은 아님)는 이름 입력 뒤 지우기·이름 바꾸기가 되고, 서버가 알려 준 진짜 기본 브랜치는 지우기·이름 바꾸기가 여전히 막힌다.
- 조직 저장소: "GitHub에 올리기" 대화상자에 "만들 곳"이 있고 내 계정(기본)이나 조직을 고르면 그곳에 저장소가 만들어지고 `origin`이 이어진다. 조직이 앱을 막았으면 쉬운 문구로 알린다.
- 저장소 찾기: 복제 대화상자 "내 저장소" 목록 위 찾기 칸에 글자를 치면 이름에 그 글자가 든 저장소만 남고, 받은 것이 50개를 넘으면 500개까지 저절로 더 읽어 찾는다.
- 이메일: 설정 "일반"의 이름과 이메일 아래에 "GitHub 계정의 이메일이에요"·"이 이메일은 GitHub 계정에 없어요" 줄이 보이고, 권한이 없으면 "이메일 확인 권한 추가"가 같은 연결 패널로 `user:email`을 더한다(워크플로 권한이 있었으면 함께 유지). 화면·로그 어디에도 GitHub 계정의 이메일 목록이 넘어가지 않는다. GitHub가 비공개 이메일 때문에 올리기를 막으면(GH007) 그 뜻의 문구와 "이름과 이메일 설정" 단추가 나온다.
- 로그인 설정: 설정 "GitHub"에 "올리기와 가져오기에 이 연결 쓰기"(기본 켬)가 있고 끄면 모든 원격 작업이 이 컴퓨터의 Git 로그인을 쓴다(`AuthMode::UserDefault`, 저절로 하는 원격 확인 포함).
- 새 액션 6개(`pr-comment`, `pr-merge`, `pr-close`, `pr-reopen`, `fork`, `remote-tag-delete`)의 위험도가 용어표와 같다(`registry-glossary`). `Irreversible` 액션 집합 시험이 `pr-merge`·`remote-tag-delete`를 더한 값이다. 새 용어 5개가 표·glossary ko·en에 있다. 새 오류 코드마다 ko·en 문구가 있다. 새 명령이 네 곳에 있다(`tests/security_config.rs`).
- 토큰·`device_code`·이메일 목록은 어떤 새 화면 타입·로그·오류·명령줄·주소에도 없다. 서버가 준 글(댓글·저장소 이름)은 글자로만 그린다.
- 800px 너비와 다크 모드에서 댓글 목록·합치기 확인 창·포크 안내·이름 입력 확인 창·만들 곳 고르기·찾기 칸이 겹치거나 잘리지 않는다.
- `pnpm check:full`, 두 OS CI(`git-min` 포함)·`cargo deny check` 통과. code-analyst 점검 위반 0건.

## 범위
- **이번에 한다**: D-124가 4c로 넘긴 9가지 전부(검토 요청 댓글 쓰기·포크해서 요청하기·합치기·닫기, 원격 태그 지우기, 조직 저장소로 올리기, 저장소 목록 검색, GitHub 이메일과 앱 이름·이메일 비교, "git에도 hiGit 로그인을 쓰기" 설정, 원격 기본 브랜치에 덮어쓰며 올리기·지우기)와 그 바탕(hosting PUT·PATCH, 검토 요청 저장소 정보 조회, GH007 분류, 닫은 요청 다시 열기, 최근 댓글 읽기).
- **뺀다**(까닭과 갈 곳은 C14):
  - 리뷰 쓰기(승인·변경 요청), 줄 댓글, 검토 요청 제목·본문 고치기, 라벨·담당자·리뷰어 지정, 릴리스: 남에게 보이는 쓰기가 더 늘고 화면이 커진다 → 이후 후보.
  - "다시 쌓아 합치기"(rebase merge): 용어 `rebase`와 뜻이 겹쳐 설명이 길어진다. 저장소가 그것만 허용하면 "GitHub에서 합쳐 주세요"와 "GitHub에서 열기"(C5).
  - 원격에만 있는 태그 목록·지우기: 태그 대화상자는 로컬 태그 목록이다. 원격 태그를 읽으려면 네트워크 조회가 하나 더 든다(C8).
  - `read:org` 권한과 조직 목록 API: 권한이 또 늘어난다. 조직 후보는 이미 읽는 저장소 목록의 소유자에서 얻는다(C10).
  - GitHub 검색 API: 레이트 한도가 따로이고 "내가 볼 수 있는 저장소" 조건을 표현할 수 없다(C11).
  - D-137의 5-0 묶음(journal 줄 묶기·키 알림, `localStorage` 평문, `pushurl`·`pushInsteadOf`·`core.worktree`·`http.*` 신뢰 확인, Escape·Enter 판정 모으기): 그대로 반복 5-0.
- **010이 만들 것**(이 계획이 기대는 결과물, 다시 만들지 않는다. 구현 첫날 이름을 `rg`로 확인한다):

| 010 단계 | 결과물 | 이 계획에서 쓰는 곳 |
| --- | --- | --- |
| H8 | `PublishedRule::{Refuse, Allow}`와 "upstream이 원격 기본 브랜치면 `Refuse`" 판정, `rewrote.json` 표식, `RewrotePublished` | W16이 기본 브랜치 판정을 바꾼다 |
| H9 | 액션 `force-push`(입력 `{ remote, branch, expect_remote, expect_head }`), `query_force_push_plan`·`ForcePushPlanView`(`blocked: ForcePushBlock::DefaultBranch` 포함), 원격 기본 브랜치 판정(010 검토 반영 R1: ① 서버 `ls-remote --symref` ② 로컬 원격 HEAD ③ `main`·`master`·`init.defaultBranch`), `Irreversible` 집합 시험 | W10·W14가 집합을 넓히고 W16이 입력·판정을 넓힌다 |
| H10 | `features/force-push/ForcePushDialog`·`forcePushStore`, 설정 "고급" 스위치, 역 메뉴 `force-push` 항목 | W17 |
| H11 | 액션 `remote-branch-delete`·`remote-branch-rename`, `query_remote_branch_plan`·`RemoteBranchPlanView`(`RemoteBranchBlock::Protected`) | W16 |
| H12·H13 | 액션 `tag-push`, `features/remote-branch/{RemoteBranchMenu, RemoteDeleteDialog, RemoteRenameDialog, remoteBranchStore}`, `TagDialog` 줄의 "올리기" | W11(합친 뒤 지우기)·W15·W17 |
| H14 | `crates/hosting/src/repo.rs`(`create_repo`·`list_repos`·`RepoSummary`), `LoginScope::{Repo, RepoWorkflow}`, `LoginPoll::Granted { token, workflow }`, `PrDetail.reviews`·`comments`(issue+review 합) | W2~W4 |
| H15 | 액션 `publish`(입력 `{ name, description, visibility, acknowledgePublic }`), `GitOp::RemoteAdd`, `HostingPage::Repository`, `ErrorCode::GithubRepoExists`·`GithubRepoInvalid` | W12(`RemoteAdd`)·W18 |
| H16 | `connect_start(workflow: bool)`, `github.json`의 `workflow`, `GitHubStatusView.workflow`, `query_github_repos(page)`·`GitHubRepoListView` | W7·W18·W20 |
| H17·H18·H19 | `features/publish/PublishDialog`, `features/clone/{RepoList, useRepoList}`·`useGithubRepos`, 설정의 "워크플로 권한 추가"·`useConnect(…, workflow)` | W8·W19·W20 |
| H7·R7 | 프런트 `useProgress.ts`의 `REMOTE_ACTIONS`와 core의 `REMOTE_ACTIONS`가 같은지 보는 시험 | W12·W14가 항목을 더한다 |

- 새 액션 6개(`pr-comment`, `pr-merge`, `pr-close`, `pr-reopen`, `fork`, `remote-tag-delete`). `Irreversible`이 둘 늘어난다(`pr-merge`, `remote-tag-delete`). 바뀌는 액션 입력: `force-push`·`remote-branch-delete`·`remote-branch-rename`에 `confirm_branch`, `publish`에 `owner`. 새 Tauri 명령 4개(`query_pr_repo`, `query_pr_comments`, `query_branch_protection`, `query_github_email_check`)와 `github_connect_start` 인자 모양 변경(bool → 더할 권한). 새 의존성 없음.

## 현재 상태 (코드로 확인한 사실, 커밋 f11b0b0)
- 진행: `docs/progress.md:244-247`이 010-H8 뒷부분(표식·동기화 보호)과 H9~H20을 남은 일로 적는다. 아래 "010이 만들 것"은 아직 코드에 없다.
- hosting: `HostingProvider`(`crates/hosting/src/lib.rs:34-66`)는 `id`·`detect`·`begin_login`·`poll_login`·`current_user`·`list_prs`·`pr_detail`·`create_pr` 여덟 개다. `Method`는 `Get`·`Post`뿐(`http.rs:11-14`)이고 `UreqTransport::send`(`ureq_transport.rs:67-93`)도 둘만 다룬다. `api::status_error`(`api.rs:19-44`)는 405·409를 `BadResponse`로 떨어뜨리고 422는 `pr::validation_error`가 `ValidationKind::{AlreadyExists, NoDifference, Invalid}`로 나눈다. 요청은 모두 `GitHub::api_request`(검증된 `api_base`, 베어러 칸)와 `api_send`(같은 API 주소 리디렉트만)를 거친다(`github.rs:92-141`). 소유자·이름 조각은 `repo_path`가 `valid_owner`·`valid_name` 뒤 퍼센트 인코딩한다(`github.rs:23-32`).
- 검토 요청: `PrDetail.head_sha`는 `pub(crate)`이고 16진수 40·64자 검사를 거친다(`pr.rs:100-106`, `github.rs:216-221`). 화면 타입 `PrDetailView { row, mergeable, checks }`(`core/src/view/hosting.rs:105-110`)에 head 해시가 없다. `NewPr { title, body, head, base, draft }`(`pr.rs:109-117`)의 `head`는 같은 저장소 브랜치 이름뿐이고 `valid_branch`가 `:`를 받는지 확인이 필요하다. `PullRequest.head_owner`는 포크에서 낸 것이면 그 소유자이고 `row().is_current_branch`는 `head_owner`가 없을 때만 참이다(`core/src/github/pr.rs:109-127`).
- 검토 요청 대상: `pr_target`(`core/src/github/pr.rs:32-92`)은 지금 브랜치 upstream의 원격(없으면 하나뿐·`origin`)을 고르고, 그 원격의 올리기 주소가 모두 같은 github.com 저장소일 때만 `repo`를 준다. 포크·부모 개념이 없다(D-113 "같은 저장소 안의 검토 요청만").
- `pr-create`(`core/src/actions/pr_create.rs`)는 `Safe`/`Nothing`, 용어 `pr`, 원격 잠금만, journal 없음(D-123). 같은 꼴로 HTTP 하나만 쓰는 쓰기 액션을 더할 수 있다. GitHub 호출은 `ctx.github.call(|provider, token| …)`(`core/src/github/mod.rs:431-445`)이 토큰 읽기·401 처리·오류 변환(`from_hosting`)을 한 곳에서 한다. 오류 종류는 `GithubFailure`(`core/src/error.rs:167-196`).
- 인증: `auth_for(git, repo, remote, background)`(`core/src/github/mod.rs:451-479`)는 설정을 받지 않는다. 부르는 곳은 `actions/remote.rs:206`(`Net::fetch`)·`:219`(`Net::push`) 둘이다(010의 원격 쓰기도 `Net`에 메서드로 더한다, 010 검토 반영 R4).
- 연결 상태 파일 `github.json`은 `StatusFile { v, login, id }`(`core/src/github/store.rs:15-20`, 4KB, `v=1`). `LoginPoll::Granted(SecretString)`(`hosting/src/login.rs:54`)은 받은 권한을 버린다(010-H14가 `workflow`만 읽게 바꾼다). 권한 문자열은 `GitHubConfig.scope`(`GITHUB_SCOPE`).
- 앱 설정: `AppSettings`(`core/src/settings.rs:29-42`)는 `Copy`, 파일은 `deny_unknown_fields`이고 새 칸은 `#[serde(default)]`로 옛 파일과 호환된다(D-148). 로그인 관련 칸은 없다.
- 올리기: `push`는 입력이 없고(`ActionInput::Push`), upstream이 없으면 `pick_remote`(원격이 여럿이면 `origin`)로 같은 이름에 `-u`로 올린다(`actions/remote.rs:175-191`, `:268-`). 그래서 포크 원격을 더해도 기존 `push`는 포크로 가지 않는다.
- 원격 ref 쓰기: `GitOp::{Push, PushForce, PushDelete, PushNewRef, PushTag}`(`git/src/types.rs:507-541`)와 `cli/refpush.rs`가 있다. 지우기는 `refs/heads/`만 다루고 태그 지우기는 없다. `ls_remote`·`ls_remote_head`(`git/src/backend.rs:91-104`)가 있다.
- 실패 분류: `failure.rs`(`git/src/failure.rs:116-167`)는 GH006·"protected branch"·"[remote rejected]"를 `RemoteRefused`로, "Permission to … denied"+403을 `HttpForbidden`으로 본다. GH007(비공개 이메일 거절)은 `RemoteRefused`로 뭉개진다.
- 화면: `screens/PullRequests/PullRequests.tsx`는 목록·필터·"새 검토 요청"만 있고 쓰기 단추가 없다. 상세는 `features/pr/PrRow.tsx`가 줄 아래에 펼친다. 새 요청은 `features/pr/CreatePrDialog.tsx`(먼저 `push`, 끝나면 `pr-create`). 설정 구역은 `features/settings/{GeneralSection, GitHubSection, AdvancedSection, InfoSection}.tsx`, 이름·이메일 입력은 `features/identity/IdentityForm.tsx`. 태그 대화상자 `features/history/menu/dialogs/TagDialog.tsx:40`은 `tag-delete`만 부른다. 프런트 `REMOTE_ACTIONS`는 `fetch·pull·push·sync`(`features/progress/useProgress.ts:6`). 조회 훅은 `lib/queries.ts`의 `useHostingContext`·`usePrPages`·`usePrForBranch`·`usePrDraft`·`usePrDetail`·`useGitHubStatus`.
- 명세: spec.md "검토 요청" 행이 "댓글 쓰기·합치기·닫기는 GitHub에서(D-113, D-120)"라고 적는다(`spec.md:25`). "위험도" 문단의 확인 목록(`spec.md:127`)에 검토 요청 합치기·원격 태그 지우기가 없다.

## 결정 후보 (W1이 decision.md 맨 아래 다음 빈 번호부터 "(자동 결정)"으로 올린다)
추천안을 적었다. 기존 확정 결정을 바꾸는 곳은 근거 열에 적었다(C5·C8이 D-043·D-136의 `Irreversible` 목록을 넓히고, C7이 D-113의 "같은 저장소만"을 넓히고, C9가 D-127·D-128·D-130의 원격 기본 브랜치 규칙을 바꾸고, C10이 D-132의 "내 계정만"을 넓히고, C12가 D-135의 권한 선택을 넓힌다).

| 번호 | 결정(추천안) | 까닭 | 대안 | 근거·부딪힘 |
| --- | --- | --- | --- | --- |
| C1 | **4c 범위와 순서.** D-124가 넘긴 9가지를 모두 넣는다. 순서: hosting 바탕 → 설정·이메일 → 검토 요청 쓰기(댓글·합치기·닫기) → 포크 → 원격 태그 → 원격 기본 브랜치 → 조직 저장소·찾기. 하지 않는 것은 C14. | 이번 반복은 "남에게 보이는 쓰기"의 위험·확인 설계가 주제다(D-124 이유). 바탕(HTTP 메서드·저장소 정보)을 먼저 깔면 나머지가 작아진다. | 일부를 반복 5 뒤로 | D-124, D-137 |
| C2 | **HTTP 쓰기 공통 규칙.** hosting `Method`에 `Put`·`Patch`를 더하고 전송·가짜 전송이 같은 방식으로 본문을 보낸다. 상태 표에 405 → `NotAllowed`, 409 → `Conflict`를 더한다(지금은 `BadResponse`). GitHub HTTP만 쓰는 쓰기 액션(`pr-comment`·`pr-merge`·`pr-close`·`pr-reopen`)은 `pr-create`와 같이 **원격 잠금만, 취소 대상(`REMOTE_ACTIONS`) 아님, journal 없음**이고 입력 검사는 보내기 전에 core가 한다. 본문 JSON은 `serde_json`으로만, 응답의 번호·해시·이름은 형식을 검사한 뒤 쓰고 서버가 준 URL은 읽지 않는다. 토큰은 `call` 밖으로 나가지 않는다. | D-112·D-123의 틀을 그대로 넓힌다. 405·409를 따로 두어야 "합칠 수 없어요"와 "그사이 새 버전이 올라왔어요"를 나눈다. | 쓰기마다 따로 요청 함수, 405·409를 `BadResponse`로 두기(안내가 뭉개짐) | D-112, D-122, D-123 |
| C3 | **검토 요청 저장소 정보는 조회 하나로.** 새 명령 `query_pr_repo(repo)`가 `GET repos/{o}/{r}` 한 번으로 `PrRepoView { canPush, canMerge, mergeMethods: Vec<Merge|Squash>, archived, viewer }`를 준다(`canMerge`는 `permissions.push`, `viewer`는 연결된 로그인). 검토 요청 화면·새 검토 요청 대화상자가 열릴 때만 읽고 5분 캐시, 폴링 없음, 401은 연결 끊김(D-120). 응답에 `allow_merge_commit`·`allow_squash_merge`가 없으면 `Merge` 하나만 준다(서버가 마지막에 결정). | 포크가 필요한지(올릴 권한), 합치기·닫기 단추를 보일지, 어떤 방법을 보일지가 모두 이 한 응답에 있다. | 단추마다 따로 묻기, 누른 뒤 403으로 알기(권한 없는 사람에게 단추가 보임) | D-113, D-120 |
| C4 | **댓글.** 새 용어 `comment` "댓글"(`safe`). 액션 `pr-comment`(`Safe`/`Nothing`, 확인 없음): 입력 `{ number, body }`, 본문은 앞뒤 공백을 뗀 뒤 1바이트~64KB, NUL 없음, `POST issues/{n}/comments`. 읽기는 새 명령 `query_pr_comments(repo, number)`: 검토 요청의 issue 댓글 수(`comments`, 010-H14가 합으로 읽던 값을 issue·review로 나눔)로 마지막 쪽을 계산해 **최근 20개**만 `{ author, createdAt, body(앞 4000자), mine }`로, 줄 댓글(review comment)은 읽지 않는다. 글은 React 글자로만(`white-space: pre-wrap`, 마크다운·링크·아바타 없음), 더 보려면 "GitHub에서 열기". 상세를 열 때만 읽고 쓴 뒤 다시 읽는다. | 쓰는 칸만 있고 앞 댓글을 못 보면 대화가 안 된다. 20개는 응답 2MB와 레이트를 넉넉히 지킨다. 댓글 달기는 사람이 글을 쓰고 누른 일이라 확인 창이 의미 없다(D-043은 되돌릴 수 없는 일만, 검토 요청 만들기와 같은 판단). | 쓰기만(맥락 없음), 모든 댓글(응답이 큼), 마크다운 그리기(D-112, `security.md`의 DOMPurify 검토가 필요) | D-043, D-112, D-113, D-120, D-123 |
| C5 | **검토 요청 합치기.** 새 용어 `pr-merge` "검토 요청 합치기"(`normal`). 액션 `pr-merge`(`Normal`/**`Irreversible`**, 항상 확인 창): 원격 기본 브랜치가 바뀌고 다른 사람이 곧 받아 가며 앱이 되돌릴 수 없다(GitHub의 되돌리기는 새 검토 요청). 입력 `{ number, method: Merge|Squash, expect_head }`이고 `PUT pulls/{n}/merge`에 `sha = expect_head`를 실어 **화면이 보여 준 head일 때만** 합친다(409 → `PrHeadChanged`, 405 → `PrNotMergeable`). 방법은 `query_pr_repo`의 `mergeMethods`에 있는 것만(문구 "합친 버전 만들기"·"버전 하나로 묶기"), 기본은 첫 번째. "다시 쌓아 합치기"는 보이지 않고 저장소가 그것만 허용하면 확인 창 대신 "GitHub에서 합쳐 주세요"와 "GitHub에서 열기". 꺼짐: 초안, 겹침(`mergeable = Conflicts`), 닫힘·합쳐짐, `canMerge` 거짓(단추 없음). 경고만: 시험 결과 실패·진행 중, 뒤처짐·막힘(서버의 보호 규칙이 결정). 확인 창 본문: `#12 · feature → main`, 방법 고르기, 시험 결과 줄. 성공 뒤 화면이 기존 `fetch`를 한 번 부르고, 그 원격 브랜치의 추적 ref가 아직 있으면 토스트 "합쳤어요"에 "브랜치 지우기"(010-H13의 원격 브랜치 지우기 확인 창을 연다, 지우기는 그 확인 창의 일). PrDetailView에 `headOid`를 더한다. | GitHub도 합치기 전에 확인을 받는다. `sha`를 실으면 "본 것만 합친다"를 서버가 보장한다(D-128의 lease와 같은 생각). 브랜치 지우기를 합치기 안에 넣으면 되돌릴 수 없는 일 두 개가 한 확인에 묶인다. | 확인 없이 바로(D-043 위반), 세 방법 모두(용어 부담), 합친 뒤 저절로 브랜치 지우기(저장소 설정 `delete_branch_on_merge`와 겹침) | D-043, D-113, D-128, D-130, D-136 |
| C6 | **닫기·다시 열기.** 새 용어 `pr-close` "검토 요청 닫기"(`normal`). 액션 `pr-close`(`Normal`/`Nothing`, 확인 없음)와 `pr-reopen`(용어 `pr`, `Safe`/`Nothing`), 둘 다 `PATCH pulls/{n}`의 `state`만 보낸다. 닫으면 토스트 "닫았어요"에 글자 버튼 "다시 열기"(`pr-reopen`, 실행기의 되돌리기·Ctrl+Z와는 다른 단추라 "되돌리기"라 부르지 않는다). 단추는 열린 요청에서 `viewer`가 작성자이거나 `canPush`일 때만, 다시 열기는 합쳐지지 않고 닫힌 요청에서만. 서버가 거절하면(브랜치가 지워져 다시 열 수 없음 등 422) 오류 문구 그대로. | 닫기는 다시 열 수 있으니 묻지 않는다(D-043). 다시 열기를 같은 액션의 입력으로 두면 위험도·용어가 하나에 섞인다. | 확인 창, Ctrl+Z에 묶기(journal 밖 원격 일이라 실행기 되돌리기 모델과 맞지 않음) | D-042, D-043, D-113 |
| C7 | **포크해서 요청하기.** 새 용어 `fork` "포크"(`normal`). 액션 `fork`(`Normal`/`Nothing`, 잠금 둘, `REMOTE_ACTIONS`, journal 없음): ① `POST repos/{o}/{r}/forks`(본문 없음, 이미 있으면 GitHub가 같은 포크를 돌려줌) ② 응답의 소유자가 연결된 로그인과 같고(대소문자 무시) 이름이 `valid_name`인지 검사 ③ 포크가 준비될 때까지 `GET repos/{me}/{name}`를 2초 간격 최대 15번(취소 표시를 매번 봄, 넘으면 `ForkNotReady`, 다시 누르면 이어서 됨) ④ 원격 이름은 상수 `fork`, 주소는 core가 `https://github.com/<로그인>/<이름>.git`으로 만들어 010-H15의 `GitOp::RemoteAdd`로 잇는다(이미 `fork`가 같은 주소면 그대로, 다른 주소면 `ForkRemoteTaken`) ⑤ 지금 브랜치를 `fork`에 같은 이름으로 `-u` 올린다(`Net::push`, `auth_for`). 보이는 조건: 대상 저장소가 GitHub이고 `canPush` 거짓이고 연결됨. 검토 요청 대상(`pr_target`)은 upstream 원격이 `fork`이고 그 주소가 연결된 로그인의 저장소이며 `origin`이 GitHub 저장소이면 **기준 저장소 = `origin`, head = `<로그인>:<브랜치>`** 로 바꾸고, `pr-create`는 보내기 전에 `GET repos/{fork}`의 `parent`가 `origin` 저장소인지 확인한다(아니면 `ForkParentMismatch`). 검토 요청 목록·상세·합치기·닫기·댓글도 같은 기준 저장소를 쓴다. 지금 브랜치 요청 찾기는 head 필터를 `<로그인>:<브랜치>`로. | 올릴 권한이 없는 사람이 기여하는 유일한 길이다. `push`에 원격 입력을 더하면 매일 쓰는 액션이 넓어지므로 첫 올리기는 `fork`가 맡고 이후는 upstream이 `fork`라 기존 `push`·동기화가 그대로 간다. 원격 이름을 상수로 두면 사용자 글자가 원격 이름이 되지 않는다. 주소는 응답 글자가 아니라 검사한 값으로 만든다(D-132와 같음). | `push`에 대상 원격 입력, 포크 이름을 사람이 정하기, 원격 이름을 로그인으로(사용자 글자), 포크 준비를 기다리지 않기(첫 올리기가 자주 실패) | D-035, D-111, D-112, D-113, D-132 |
| C8 | **원격 태그 지우기.** 새 용어 `remote-tag-delete` "원격 태그 지우기"(`danger`). 액션 `remote-tag-delete`(`Danger`/**`Irreversible`**, 항상 확인, `moves_head` 거짓, 원격 잠금, `REMOTE_ACTIONS`, journal 없음): 원격에서 지운 태그는 다른 사람의 저장소·GitHub 릴리스에서 이미 쓰고 있을 수 있어 앱이 되돌릴 수 없다(D-130과 같은 판단). 대상은 **로컬에 있는 태그만**: 입력 `{ name, expect }`이고 core가 `check_tag_name`·`refs/tags/<이름>`의 값(가벼운 태그는 버전, 주석 태그는 태그 객체)이 `expect`인지 확인한 뒤 새 `GitOp::PushDeleteTag { remote, name, expect, auth }` = `push --force-with-lease=refs/tags/<이름>:<expect> --end-of-options <원격> :refs/tags/<이름>`. "(stale info)"면 `ls_remote(refs/tags/<이름>)`으로 한 번 더 보고 없으면 `RemoteRefMissing`("원격에 이 태그가 없어요"), 다른 값이면 `RemoteTagDiffers`("원격의 같은 이름 태그는 다른 버전이에요"). 원격은 `pick_remote`. 로컬 태그는 건드리지 않는다. 진입점: 태그 대화상자 줄마다 글자 버튼 "원격에서 지우기". 네트워크 계획 조회는 두지 않는다(확인 창은 로컬 값만 보이고 서버 일치는 lease가 지킨다). | lease 값을 로컬 태그로 두면 "내가 아는 그 태그일 때만"이 서버에서 보장된다. 원격만 있는 태그까지 다루면 ls-remote 목록 조회와 화면이 하나 더 든다. | `--delete`(lease 없음), 원격 태그 목록 화면, `Normal`/되돌리기로 다시 올리기(남의 저장소에 이미 퍼진 지우기는 못 되돌림) | D-035, D-043, D-104, D-130, D-131 |
| C9 | **원격 기본 브랜치는 브랜치 이름을 입력하면 덮어쓰며 올리기·지우기를 허용한다.** ① `force-push`·`remote-branch-delete`·`remote-branch-rename` 입력에 `confirm_branch: Option<String>`(`skip_serializing_if = "Option::is_none"`로 옛 journal 서명 호환)을 더하고, 대상이 "기본 브랜치로 보이는 것"이면 core가 `confirm_branch == 원격 브랜치 이름`(정확 일치)일 때만 실행한다(아니면 `NameConfirmMismatch`). ② 판정은 010 R1의 셋을 둘로 나눈다: **서버 기본**(① `ls-remote --symref`, 읽지 못하면 서버 기본으로 본다)과 **이름만 기본 같은 것**(②로컬 원격 HEAD ③`main`·`master`·`init.defaultBranch`). 덮어쓰며 올리기는 둘 다 이름 입력으로 허용, 지우기·이름 바꾸기는 **서버 기본만 계속 막고**(GitHub도 기본 브랜치 지우기를 거절함) 이름만 같은 것은 이름 입력으로 허용. ③ D-127의 보정("upstream이 원격 기본 브랜치이면 완화하지 않는다")을 푼다: 스위치가 켜져 있으면 기본 브랜치에서도 고치기·돌아가기·기록 정리를 허용한다. 막다른 길은 이제 이름 입력 덮어쓰기로 풀린다. `rewrote.json` 동기화 보호는 그대로. ④ 서버 보호 규칙 안내: 새 명령 `query_branch_protection(repo, remote, branch)`가 GitHub 원격이고 연결돼 있을 때만 `GET repos/{o}/{r}/branches/{b}`의 `protected`를 `Some(bool)`로(그 밖·403·404는 `None`), 확인 창이 열릴 때 한 번 읽는다. `true`면 확인 창에 "GitHub가 보호하는 브랜치예요. 서버가 거절할 수 있어요." ⑤ `RemoteRefused`가 GitHub 원격에서 나면 오류 줄에 "GitHub에서 보기"(새 `HostingPage::BranchSettings` = `{web}/{o}/{r}/settings/branches`, core가 검사한 값으로 만듦)와 "저장소 관리자만 바꿀 수 있어요" 한 줄. 이름 입력칸은 한글 조합 중 Enter를 무시하고 Enter로 위험 버튼을 누르지 않는다. | 로드맵이 정한 "이름 입력 확인과 서버 보호 규칙 안내"를 구체화한다. 이름 입력은 습관적 확인을 깨는 GitHub의 저장소 지우기 방식과 같다. 기본 브랜치에 올린 비밀을 지우는 일이 초보자가 이 기능을 찾는 가장 흔한 까닭이라 고치기까지 열어야 쓸모가 있다. | 덮어쓰기만 열고 고치기는 계속 막기(앱 안에서 덮어쓸 기록을 만들 길이 없음), 기본 브랜치용 별도 스위치(설정이 늘어남), 보호 규칙 상세(`branches/{b}/protection`은 관리자만 읽음) | D-043, D-097, D-127, D-128, D-129, D-130, D-122 |
| C10 | **조직 저장소로 올리기.** `publish` 입력에 `owner: Option<String>`(없으면 내 계정, `skip_serializing_if`)을 더한다. 있으면 `valid_owner` 뒤 `POST orgs/{owner}/repos`, 응답 소유자가 그 값과 같아야 한다(대소문자 무시, D-132의 "로그인과 같아야 함"을 "고른 곳과 같아야 함"으로). 후보는 새 권한 없이 얻는다: 010-H14의 `RepoSummary`에 `owner_is_org`(`owner.type == "Organization"`)를 더하고 대화상자가 `useGithubRepos` 첫 3쪽의 조직 소유자를 중복 없이 보인다. 목록에 없는 조직은 "다른 조직" 입력칸으로(화면이 같은 규칙으로 즉시 안내, core가 최종 검사). 조직이 OAuth 앱을 막았거나 만들 권한이 없으면 403 → `GithubOrgForbidden`("이 조직에 저장소를 만들 수 없어요. 조직 관리자가 hiGit을 허용해야 해요."). 공개 여부·`acknowledgePublic` 규칙은 그대로. | 조직 목록 API는 `read:org`나 `user` 권한이 필요하다(근거 자료). 이미 읽는 저장소 목록에서 대부분의 조직이 나오고 나머지는 입력으로 덮는다. | `read:org` 권한을 선택해서 더하기(권한 증가), 조직 입력칸만(오타) | D-112, D-132, D-133, D-135 |
| C11 | **저장소 목록 찾기는 화면에서 거른다.** 복제 대화상자 "내 저장소" 목록 위에 찾기 칸(`role="searchbox"`)을 두고 `소유자/이름`에 글자가 든(대소문자 무시) 줄만 보인다. 찾기 칸에 글자가 있고 다음 쪽이 있으면 화면이 기존 `query_github_repos`를 이어 불러 **500개(10쪽)까지** 저절로 더 읽고 "N개 중 찾는 중" 표시, 넘으면 "더 많아요. 이름을 더 쳐 주세요". 새 명령·새 API 없음. | GitHub 검색 API는 레이트 한도가 따로(분당 30)이고 "내가 볼 수 있는 저장소"(협업·조직)를 한 질의로 못 쓴다. 목록 API는 이미 그 조건으로 읽고 60초 캐시가 있다. | 검색 API(`search/repositories`), core에서 전부 읽어 거르기(새 명령) | D-120, D-133 |
| C12 | **이메일 비교와 GH007.** ① 권한: 010-H14의 `LoginScope`를 묶음 `LoginScopes { workflow, email }`로 넓혀 폼 값(`repo`, `repo workflow`, `repo user:email`, `repo workflow user:email`)을 hosting 한 곳에서 만들고 받은 `scope`에서 둘을 읽는다. `github.json`에 `email`(기본 거짓, `v`는 1 그대로). 권한을 더할 때는 **지금 가진 권한과 합친 값**으로 다시 연결한다(이메일을 더하다 워크플로를 잃지 않게). `github_connect_start`의 인자는 bool 대신 `add: Option<ScopeAdd>`(`workflow`·`email`). ② 비교: 새 명령 `query_github_email_check()`(앱 수준, 저장소 없음)가 전역 이메일(D-125 `query_identity`와 같은 값)을 보고 **결과 종류만** 준다: `noIdentity`·`noreply`(이 계정의 `<id>+<로그인>@users.noreply.github.com`이나 `<로그인>@users.noreply.github.com`이면 API 없이)·`noPermission`(이메일 권한 없음)·`onAccount`(확인된 이메일)·`unverified`·`notOnAccount`. `GET user/emails` 응답의 주소 목록은 core 안에서만 비교하고 버린다(화면·로그·오류에 없음). 설정 "일반"이 열릴 때 연결돼 있으면 읽고 60초 캐시. ③ GH007: `failure.rs`에 `PrivateEmailBlocked`("GH007", "push declined due to email privacy restrictions")를 `RemoteRefused` **앞에** 두고 `ErrorCode::PrivateEmailBlocked` 문구 "GitHub가 비공개 이메일이 든 버전을 막았어요. 이메일을 GitHub 주소로 바꾸고 그 버전을 다시 저장해 주세요." + "이름과 이메일 설정" 단추(010-H5의 설정 열기). | 이메일 목록은 개인정보라 화면에 보낼 까닭이 없다(D-040). 이미 있는 "GitHub 주소로 채우기"(D-126)가 해결 단추다. GH007은 이메일 설정 때문에 생기는 대표 실패라 같은 반복에서 문구를 준다. | 이메일 목록을 화면에 보내 고르게 하기(개인정보), 처음 연결부터 `user:email`(권한 증가), 저장소 로컬 이메일까지 비교(저장소마다 다를 수 있어 설정 화면과 맞지 않음) | D-021, D-040, D-109, D-125, D-126, D-135 |
| C13 | **"올리기와 가져오기에 이 연결 쓰기" 설정.** `settings.json`에 `git_helper: bool`(기본 참 = 지금 동작, `#[serde(default = "yes")]`)을 더하고, `auth_for`가 인자로 받은 이 값이 거짓이면 늘 `UserDefault`를 준다(저절로 하는 원격 확인·`fork`·010의 원격 쓰기 모두 `Net`을 거치므로 한 곳). 값은 `ExecCtx.settings`에서 오고 전역 상태를 두지 않는다(D-129와 같은 방식). 설정 "GitHub" 구역의 체크 상자 + `--text-muted` 한 줄 "끄면 이 컴퓨터에 설정된 Git 로그인을 써요." 연결 상태와 무관하게 보이되 연결 안 됐으면 꺼진 모양이 아니라 그대로 둔다(연결하면 따름). GitHub API 호출(검토 요청 등)은 이 설정과 무관하게 연결된 토큰을 쓴다. | 009 C3의 부작용(GCM에 다른 계정을 쓰던 사람이 hiGit 계정으로 올리게 됨)을 사람이 풀 길이다. 화면 낱말은 Git 내부 말 대신 용어표의 "올리기"·"가져오기"(`t("push")`·`t("pull")`). | 저장소마다 설정(받은 저장소가 바꿀 수 있는 곳에 두면 안 됨), 연결 끊기만으로 풀기(검토 요청도 못 씀) | D-040, D-111, D-118, D-127, D-148, D-160 |
| C14 | **이월·하지 않음.** D-137의 5-0 묶음은 그대로 5-0. 새 원격 쓰기(`fork`의 올리기, `remote-tag-delete`)도 `auth_for`를 쓰므로 토큰 위협은 늘지 않고, 받은 저장소의 `pushurl`이 다른 곳으로 올리게 하는 기존 위험은 `fork` 원격에는 없다(앱이 만든 원격, 그러나 `remote.fork.pushurl`을 저장소가 미리 심어 둘 수는 있음 → `fork` 원격이 이미 있으면 주소 일치만이 아니라 `pushurl`도 없어야 재사용, 아니면 `ForkRemoteTaken`). 하지 않음: 리뷰 쓰기·줄 댓글·제목·본문 고치기·라벨·담당자·릴리스·rebase 합치기·원격만 있는 태그·`read:org`·검색 API → 이후 후보로 roadmap에 적는다. | 이번에 만지는 코드와 겹치지 않는 것은 미룬다. 새로 생기는 원격(`fork`)에 대해서는 이번에 막을 수 있는 것을 막는다. | 이번에 모두 | D-118, D-137 |

## 용어 변경
- spec.md 용어표와 `src/glossary`(`types.ts`의 `TERM_IDS`는 표 순서, `ko.ts`·`en.ts`)에 W1에서 추가한다(D-015, spec-sync 시험).
  - `| comment | 댓글 | comment | 검토 요청에 남기는 글. 저장소를 볼 수 있는 사람이 모두 봐요. | safe | 가끔 |` (`review` 다음) — en "Comment"
  - `| pr-merge | 검토 요청 합치기 | merge pull request | 검토 요청의 브랜치를 GitHub에서 대상 브랜치에 합치기. 앱에서 되돌릴 수 없어요. | normal | 가끔 |` (`pr-draft` 다음) — en "Merge pull request"
  - `| pr-close | 검토 요청 닫기 | close pull request | 합치지 않고 검토 요청을 닫기. 다시 열 수 있어요. | normal | 드묾 |` (`pr-merge` 다음) — en "Close pull request"
  - `| fork | 포크 | fork | 남의 저장소를 내 GitHub 계정에 복사한 것. 여기에 올리고 원래 저장소에 검토 요청을 보내요. | normal | 드묾 |` (`clone` 다음) — en "Fork"
  - `| remote-tag-delete | 원격 태그 지우기 | push --delete tag · `git push <원격> --delete <태그>` | 원격 저장소의 태그를 지우기. 내 컴퓨터의 태그는 그대로예요. | danger | 드묾 |` (`tag-push` 다음) — en "Delete remote tag"
- 새 액션은 같은 id의 용어를 쓰고 `pr-comment`는 `comment`, `pr-reopen`은 `pr`을 쓴다(`pr-create`가 `pr`을 쓰는 것과 같음). 위험도는 용어표와 같다(`registry-glossary`).
- spec.md "위험도" 문단: "실행 전 확인" 목록에 검토 요청 합치기·원격 태그 지우기를 더하고, 원격 기본 브랜치에 덮어쓰며 올리기·이름만 기본 같은 원격 브랜치 지우기·이름 바꾸기는 브랜치 이름 입력을 받는다는 문장을 더한다. `danger` 줄의 "원격 기본 브랜치 제외"를 C9에 맞게 고친다.
- 용어표에 넣지 않는 화면 문구(`src/messages` ko·en 같은 커밋): "댓글 달기", "댓글을 달았어요", "합친 버전 만들기", "버전 하나로 묶기", "합쳤어요", "브랜치 지우기", "닫았어요", "다시 열기", "이 저장소에 올릴 권한이 없어요", "포크 만들고 올리기", "포크를 준비하는 중", "원격에서 지우기", "브랜치 이름 {name}을 똑같이 입력해 주세요", "GitHub가 보호하는 브랜치예요. 서버가 거절할 수 있어요.", "GitHub에서 보기", "만들 곳", "내 계정", "다른 조직", "저장소 찾기", "GitHub 계정의 이메일이에요", "이 이메일은 GitHub 계정에 없어요. 올린 버전이 내 계정으로 이어지지 않아요.", "이메일 확인 권한 추가", "올리기와 가져오기에 이 연결 쓰기", "끄면 이 컴퓨터에 설정된 Git 로그인을 써요." 괄호 보충 없이 해요체로, 문장 속 용어는 `t(id)`(D-029). 모두 `copy-rules`·`terms-in-messages`·ko·en 키 같음 시험을 통과해야 한다.

## 단계
각 단계 = 커밋 하나. Rust를 고친 단계는 커밋 전에 `pnpm check:full`. `core::view`·Tauri 명령·액션 목록이 바뀌면 같은 커밋에서 `cargo run -p app --bin gen-bindings`(`src/lib/bindings.ts`·`action-meta.json`). 새 Tauri 명령은 `src-tauri/src/commands.rs`·`src-tauri/src/bindings.rs`·`src-tauri/build.rs`의 `COMMANDS`·`src-tauri/capabilities/default.json` 네 곳과 `src/lib/api.ts`를 같은 커밋에 넣고 모두 `async` + `blocking()`(`tests/security_config.rs`가 확인). 새 액션은 architecture.md "새 액션 추가 절차": 용어(W1) → 필요하면 `GitOp`(git) → `ActionInput`·`actions/<id>.rs`·`register_all`·`Outcome`·잠금(`bootstrap.rs`) → 화면 → 테스트(가짜 전송·로컬 bare 원격으로 실행 → 결과 → 되돌리기 없음 또는 `Irreversible` 토큰 확인). `GitBackend`에 메서드나 `GitOp` 변형을 더하면 가짜 구현 둘(`executor.rs` 시험의 `NoGit`, `crates/core/tests/common/mod.rs`의 `FakeGit`)을 같은 커밋에서. `HostingProvider`에 메서드를 더하면 hosting `testing.rs`와 core 시험의 가짜 provider도 같은 커밋에서. 새 `ErrorCode`는 ko·en `errors` 문구를 같은 커밋에(완전성 시험). 화면은 git도 `invoke`도 직접 부르지 않고 `src/lib/api.ts`·`queries.ts`로만 core를 부른다(D-010). 용어 호버는 `<Term>`, 문장 속 용어는 `t(id)`. 색·간격은 `tokens.css` 변수만. **되돌릴 수 없는 액션의 확인 창은 `ui/ConfirmDialog`로, 포커스는 "닫기", Enter가 위험 버튼을 누르지 않는다(D-043).** 모든 새 입력칸은 한글 조합 중(`isComposing`, WebKitGTK `keyCode 229`) Enter·Ctrl+Enter를 무시한다. **토큰·device code·이메일 목록은 어떤 단계에서도 로그·토스트·명령줄·주소에 넣지 않는다.** OS별 코드는 없다(`#[cfg(windows)]`/`#[cfg(unix)]`는 시험에만). 테스트는 사용자 전역 설정·실제 키체인·실제 네트워크를 건드리지 않는다.

### A. 결정·hosting 바탕

W1. 결정·용어·문서 (`docs: add iteration 4c decisions and terms`)
- 목표: 이후 단계가 쓰는 결정과 새 용어를 먼저 확정한다.
- 바꿀 파일: `docs/decision.md`(C1~C14를 **맨 아래 다음 빈 번호부터** "(자동 결정)", 상태 확정. 기존 결정의 상태 줄만 갱신: D-043 "`Irreversible` 목록에 검토 요청 합치기·원격 태그 지우기", D-113 "포크·댓글·합치기·닫기는 C5~C7 번호로 이행", D-127 "기본 브랜치 보정은 C9 번호로 풀림", D-128·D-130 "원격 기본 브랜치는 이름 입력, C9 번호", D-132 "조직은 C10 번호", D-133 "찾기는 C11 번호", D-135 "권한 묶음은 C12 번호", D-136 "`Irreversible` 집합 7개", D-124 "이행: 계획 013"), `docs/spec.md`(용어표 5행, "위험도" 문단, 화면 구성: 검토 요청 행(댓글·합치기·닫기·다시 열기·포크), 설정 행("GitHub"에 "올리기와 가져오기에 이 연결 쓰기"·"이메일 확인 권한 추가", "일반"의 이메일 줄), 버전 기록 행의 태그 줄("원격에서 지우기"), 작업 공간 행("GitHub에 올리기"의 만들 곳), "진입점"(복제 목록 찾기)), `src/glossary/{types,ko,en}.ts`, `docs/roadmap.md`(반복 4c: "계획 013, 결정 …" 헤더와 하지 않는 것(C14)), `docs/progress.md`(013-W1~W21 체크리스트).
- 내용: 결정 문장은 위 표를 그대로 옮기고 번호는 C 순서대로 매긴다. CLAUDE.md는 고치지 않는다.
- 테스트: `src/glossary/spec-sync.test.ts`, `TERM_IDS` 순서, `copy-rules.test.ts`, `terms-in-messages.test.ts`, `registry-glossary.test.ts`(아직 새 액션 없음).
- 완료 기준: `pnpm test`, `pnpm lint`.
- Windows/Ubuntu: 차이 없음. 보안: 해당 없음.

W2. hosting: PUT·PATCH·저장소 정보·브랜치 보호 (`hosting: add put and patch, repository info and branch protection`)
- 목표: 쓰기 요청 바탕과 C3·C9가 읽을 두 조회를 hosting에 둔다(C2·C3·C9 ④).
- 바꿀 파일: `crates/hosting/src/http.rs`(`Method::{Put, Patch}`), `ureq_transport.rs`(`put`·`patch`에 본문, 헤더·베어러는 기존 두 갈래와 같은 도우미로 묶기), `api.rs`(`status_error`에 405 → `NotAllowed`, 409 → `Conflict`), `error.rs`(`HostingErrorKind::{NotAllowed, Conflict}`), `github.rs`(`api_write(method, token, path, json)` 도우미: `Content-Type: application/json`, `api_send`·`check_status`), 010-H14의 `repo.rs`(`repo_info(token, repo) -> RepoInfo { can_push, archived, allow_merge_commit: Option<bool>, allow_squash_merge: Option<bool>, parent: Option<RemoteRepo>, fork: bool }` = `GET repos/{o}/{r}`, `parent`는 `owner.login`·`name`을 `valid_owner`·`valid_name`로 검사; `branch_protected(token, repo, branch) -> Option<bool>` = `GET repos/{o}/{r}/branches/{encode(branch)}`, 브랜치 이름은 `pr::valid_branch`, 403·404는 `None`), `lib.rs`(`HostingProvider`에 둘, 내보내기), `testing.rs`(가짜 전송이 메서드를 기록), core의 가짜 provider, `crates/core/src/github/errors.rs`(`from_hosting`이 새 종류를 일단 `BadResponse`로 — 실제 코드는 W9·W10이 맥락에 따라 나눔), `docs/architecture.md`("HostingProvider").
- 내용: `UreqTransport`는 `https_only` 그대로, 리디렉트를 따르지 않는다. PUT·PATCH 리디렉트(307·308)는 `api_send`가 같은 API 주소일 때만 같은 메서드·본문으로 따른다(기존 규칙).
- 테스트(가짜 전송·로컬 `ureq` 시험 서버): PUT·PATCH 본문과 베어러가 한 번씩 나감, 405·409 상태 표, `repo_info` 표본(쓰기 권한 있음·없음, `allow_*` 없는 응답, 포크·`parent` 있음, 이상한 `parent` 이름은 `BadResponse`), `branch_protected`(참·거짓·403·404·이상한 브랜치 이름은 보내지 않음), 토큰이 URL·본문·오류에 없음, 기존 hosting 시험 통과.
- 완료 기준: `cargo test -p hosting`(두 OS), `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음(순수 계산·같은 rustls). 보안: URL은 상수 `api_base` + 검사한 조각뿐, 서버가 준 URL을 읽지 않는다.

W3. hosting: 검토 요청 쓰기·포크 (`hosting: add pull request comments, merge, state and fork`)
- 목표: C4~C7의 GitHub 호출을 넣는다.
- 바꿀 파일: 신규 `crates/hosting/src/pr_write.rs`(+시험), `pr.rs`(`PrDetail.issue_comments`·`review_comments`로 나눔(010-H14의 합은 두 값의 합으로 계속 계산), `PrDetail.head_sha`를 `pub`로(화면 `headOid`가 W9에서 씀), `NewPr.head_owner: Option<String>`과 `new_pr_json`이 `"<owner>:<branch>"`를 만듦, `valid_branch`가 `:`를 받지 않음을 시험으로 고정), `github.rs`·`lib.rs`(`HostingProvider`에 `list_comments(token, repo, number, page, per_page) -> Vec<Comment { author: Option<String>, created_at, body }>`, `create_comment(token, repo, number, body) -> CommentId`, `merge_pr(token, repo, number, MergeRequest { method: Merge|Squash, sha }) -> MergeResult { merged: bool, sha }`, `set_pr_state(token, repo, number, open: bool)`, `create_fork(token, repo) -> RemoteRepo`, `repo_exists(token, repo) -> bool`), `error.rs`(필요하면 `ValidationKind`에 `Unprocessable` 대신 기존 `Invalid` 재사용), `testing.rs`, `docs/architecture.md`.
- 내용: 댓글 쓰기 `POST issues/{n}/comments` `{ body }`(64KB·NUL 검사는 보내기 전), 읽기 `GET issues/{n}/comments?per_page=20&page=K`(본문은 앞 4000자, 제어 문자는 줄바꿈만 남김, `user`가 없으면 `None`), 합치기 `PUT pulls/{n}/merge` `{ merge_method, sha }`(sha는 16진수 40·64자만), 상태 `PATCH pulls/{n}` `{ state: "open"|"closed" }`, 포크 `POST repos/{o}/{r}/forks`(본문 없음, 202, 응답 `owner.login`·`name` 검사), 준비 확인 `GET repos/{o}/{r}`의 200/404. 번호는 1 이상만.
- 테스트(가짜 전송): 각 요청의 메서드·경로·JSON 본문(이스케이프·한글·이모지), 댓글 쪽 계산 입력과 응답 읽기(탈퇴 사용자·긴 본문 자르기·제어 문자), 합치기 200·405·409·422 종류, 상태 200·422, 포크 202 응답 검사(이상한 이름 → `BadResponse`), `NewPr` head에 소유자가 붙는 JSON과 이상한 소유자 거절, 토큰이 어디에도 없음.
- 완료 기준: `cargo test -p hosting`, `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음. 보안: 사용자 글은 serde로만, 서버 글은 글자로 잘라 넘긴다(그리기는 화면이 글자로만).

W4. hosting: 조직 저장소·권한 묶음·이메일 (`hosting: org repositories, scope set and account emails`)
- 목표: C10·C12의 GitHub 호출과 권한 문자열을 넣는다.
- 바꿀 파일: 010-H14의 `repo.rs`(`create_repo`에 `owner: RepoOwner::{Me, Org(String)}` → `POST user/repos` 또는 `POST orgs/{org}/repos`, 응답 소유자가 고른 곳과 같아야 함; `RepoSummary.owner_is_org` = `owner.type == "Organization"`), `login.rs`(010-H14의 `LoginScope`를 `LoginScopes { workflow: bool, email: bool }`로, `form_value()`가 네 값 중 하나, `parse_token`이 받은 `scope`에서 `workflow`·`user:email`을 읽어 `Granted { token, workflow, email }`), 신규 `crates/hosting/src/account.rs`(`account_emails(token) -> Vec<AccountEmail { address, verified }>` = `GET user/emails`, 100개까지, 주소는 `Debug`에서 가림), `config.rs`(기본 권한 상수는 `LoginScopes::default()`가 만듦), `github.rs`·`lib.rs`·`testing.rs`, core 가짜 provider, `docs/architecture.md`.
- 내용: 조직 이름은 보내기 전에 `valid_owner`. `AccountEmail`은 `Clone`이지만 `Debug`·`Display`가 주소를 찍지 않는다(토큰 타입과 같은 방식). 403(권한 없음)은 `Forbidden`.
- 테스트: 조직 만들기 경로·본문·응답 소유자 불일치, 403의 종류, `owner_is_org` 읽기, 권한 폼 값 넷과 받은 `scope` 파싱 표(`repo`, `repo,workflow`, `repo user:email`, `user:email,workflow,repo`, 빈 값), 이메일 응답 읽기(확인 안 됨·100개 넘음·이상한 값), `Debug`에 주소 없음, 기존 시험 통과.
- 완료 기준: `cargo test -p hosting`, `cargo test --workspace`.
- Windows/Ubuntu: 차이 없음. 보안: 이메일 주소는 hosting → core 안에서만 다닌다.

### B. 로그인 설정과 이메일

W5. git+core: 비공개 이메일 거절 분류 (`git+core: classify private email push refusal`)
- 목표: GH007을 따로 가려 해결 단추가 있는 문구로 낸다(C12 ③).
- 바꿀 파일: `crates/git/src/failure.rs`(`FailureKind::PrivateEmailBlocked`: "GH007", "push declined due to email privacy restrictions"를 `RemoteRefused` 앞에, 표 시험), `crates/core/src/error.rs`·`view/mod.rs`(`ErrorCode::PrivateEmailBlocked`와 `From`), `src/messages/ko.ts`·`en.ts`(`errors.privateEmailBlocked`), 생성물, `docs/architecture.md`(실패 분류 목록).
- 내용: D-021대로 `LC_ALL=C` 영어 문구만 본다. 일반 `push`·`sync`의 재시도 판정이 `PushRejected`가 아니므로 돌지 않는 것을 시험으로 고정한다.
- 테스트(실제 git): 로컬 bare 원격의 `pre-receive` hook이 GitHub의 GH007 문구를 stderr로 내며 거절 → `PrivateEmailBlocked`(Windows는 Git for Windows `sh`, 안 되면 `#[cfg(unix)]`로 표시), 기존 GH006 표본은 그대로 `RemoteRefused`, `git-min`, ko·en 완전성, 바인딩 최신.
- 완료 기준: `cargo test --workspace`(두 OS)·clippy, 바인딩 최신.
- Windows/Ubuntu: hook 시험의 줄바꿈은 LF 고정(`core.autocrlf false`). 보안: 원문은 "자세히 보기"로만.

W6. core: "올리기와 가져오기에 이 연결 쓰기" (`core: setting to stop using the hiGit login for git`)
- 목표: helper를 붙일지를 사람이 정한다(C13).
- 바꿀 파일: `crates/core/src/settings.rs`(`AppSettings.git_helper`, `SettingsFile.git_helper`(`#[serde(default = "yes")]`)), `view/settings.rs`(`AppSettingsView.gitHelper`, `AppSettingsPatch.gitHelper: Option<bool>`), `github/mod.rs`(`auth_for(git, repo, remote, background, use_helper: bool)`: 거짓이면 다른 검사 전에 `UserDefault`), `actions/remote.rs`(`Net`이 `ctx.settings.git_helper`를 들고 모든 `auth_for` 호출에 넘김; 010이 `Net`에 더한 원격 쓰기 메서드도 같은 값), 저절로 하는 원격 확인을 만드는 곳(`bootstrap.rs`, `rg "background: true"`)이 같은 설정을 읽음, 생성물, `docs/architecture.md`("GitHub 연결"의 `auth_for` 조건).
- 내용: 설정 값은 화면 인자가 아니라 앱 데이터 `settings.json`에서만 온다. 시험용 `ExecCtx::new`의 기본값은 참.
- 테스트: 기본 참·옛 파일(칸 없음)이 참으로 읽힘·패치로 끄고 다시 읽기, `auth_for` 표(꺼짐이면 github.com https·토큰 있음이어도 `UserDefault`, 켜짐이면 지금 표 그대로), 꺼짐 설정으로 가짜 백엔드 `fetch`·`push`가 받은 `GitOp`의 `auth`가 `UserDefault`(저절로 하는 확인 포함), 기존 `auth_for` 시험 통과, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음(사용자 helper는 OS마다 다르지만 앱은 손대지 않음). 보안: 끄면 토큰이 git으로 가는 길이 전부 닫힌다(더 안전한 쪽). 설정 칸은 고정 bool.

W7. core: 권한 묶음과 이메일 비교 (`core: scope set for reconnect and github email check`)
- 목표: 이메일 권한을 더하고 내 이메일이 계정 것인지 결과 종류만 준다(C12 ①②).
- 바꿀 파일: `github/mod.rs`(010-H16의 `connect_start(workflow)`를 `connect_start(add: Option<ScopeAdd>)`로: 지금 `github.json`의 `workflow`·`email`에 더할 것을 합친 `LoginScopes`로 시작, 승인되면 받은 권한으로 두 칸 갱신), `github/store.rs`(`StatusFile.email`, `#[serde(default)]`), 신규 `github/email.rs`(`email_check(identity_email) -> EmailCheckView`: 가림 주소 두 꼴은 API 없이 `noreply`, 권한이 없으면 `noPermission`, 아니면 `call(account_emails)` 뒤 대소문자 무시 비교, 목록은 함수 안에서 버림), `view/github.rs`(`GitHubStatusView.email`, `EmailCheckView { state }`, `ScopeAdd`), `bootstrap.rs`(`Core::github_email_check()`: 전역 이름·이메일 읽기는 010-H4의 `identity` 조회와 같은 함수), `src-tauri`(`query_github_email_check` 네 곳, `github_connect_start(add)` 인자 변경), `api.ts`, 생성물, `docs/architecture.md`(명령 목록·"GitHub 연결").
- 내용: `connect_start(None)`은 처음 연결(`repo`만, 지금과 같음). 이미 연결된 상태에서 `Some(Email)`이면 `workflow`가 참이었으면 그대로 포함한다. 401은 `call`이 연결을 끊는다.
- 테스트(가짜 전송·메모리 키체인): 권한 합치기 표(아무것도 없음+이메일 → `repo user:email`, 워크플로 있음+이메일 → `repo workflow user:email`), 받은 권한으로 상태 파일 두 칸, 옛 `github.json`(칸 없음) 읽힘, 이메일 비교 표(이름 없음, 두 가림 꼴, 권한 없음은 전송 0회, 확인됨·확인 안 됨·없음, 대소문자), 결과·오류·로그에 주소가 없음(가짜 이메일 문자열로 검사), `tests/security_config.rs`, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 연결 패널·키체인은 기존과 같다. 보안: 화면에 가는 것은 상태 종류 하나뿐. 이메일은 `redact`가 가리는 대상이기도 하다(D-040).

W8. 화면: 설정의 로그인 스위치·이메일 줄·GH007 단추 (`screens: git login switch, email check line and private email fix`)
- 목표: C12·C13을 설정과 오류 줄에 보인다.
- 바꿀 파일: `src/features/settings/GitHubSection.tsx`(체크 상자 "올리기와 가져오기에 이 연결 쓰기"(문구는 `t("push")`·`t("pull")`로 짜 맞춤) + 설명 한 줄, `useAppSettings` 패치; 010-H19의 "워크플로 권한 추가" 옆에 "이메일 확인 권한 추가"는 두지 않고 이메일 줄에서 연다), `src/features/identity/IdentityForm.tsx` 또는 `GeneralSection.tsx`(저장된 이메일 아래 `--text-muted` 한 줄: `onAccount`·`noreply` "GitHub 계정의 이메일이에요", `notOnAccount`·`unverified` 문구 + 010-H5의 "GitHub 주소로 채우기", `noPermission` 글자 버튼 "이메일 확인 권한 추가"(연결 패널을 `email`로 엶), 연결 안 됐거나 `noIdentity`면 줄 없음), `src/features/github/useConnect.ts`(`start(add?)`), `src/features/settings/settingsStore.ts`(`openScopeConnect(add)`; 010-H19의 `openWorkflowConnect`를 이 함수로 갈음), `src/lib/queries.ts`(`useEmailCheck`, `staleTime` 60초, 연결·끊기·이름과 이메일 저장 때 무효화), `PrivateEmailBlocked`를 보이는 오류 줄(동기화·올리기·새 검토 요청·포크)의 단추 "이름과 이메일 설정"(010-H5의 설정 열기), `src/messages`, `docs/design.md`("설정 대화상자"·"GitHub 연결 패널").
- 내용: 스위치는 바로 저장(확인 없음, 되돌릴 수 있는 설정). 이메일 줄은 이름·이메일을 저장한 직후 다시 읽는다.
- 테스트: 스위치 켜고 끄기와 `settingsSet({ gitHelper })`, 이메일 줄 상태별 문구·단추, "이메일 확인 권한 추가"가 `githubConnectStart("email")`와 패널, 워크플로 단추 동작은 그대로(010 시험), GH007 오류 줄 단추, 800px·다크, IME 없음(입력 없음).
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`.
- Windows/Ubuntu: 연결 패널·브라우저 열기·키링 잠금 해제 창은 기존과 같다. 보안: 화면은 bool과 상태 종류만 다룬다.

### C. 검토 요청 쓰기

W9. core: 저장소 정보·댓글 (`core: pull request repo info, comments and pr-comment action`)
- 목표: 화면이 단추를 정할 정보와 댓글 읽기·쓰기를 준다(C3·C4).
- 바꿀 파일: `crates/core/src/github/pr.rs`(`repo_view(info, viewer)`, `comments_view`), `view/hosting.rs`(`PrRepoView { canPush, canMerge, mergeMethods, archived, viewer }`, `MergeMethodView::{Merge, Squash}`, `PrCommentsView { items: Vec<PrCommentView { author, createdAt, body, mine }>, total, partial }`, `PrDetailView.headOid`·`author`), `query.rs` 또는 `bootstrap.rs`(조회 둘: 대상 저장소는 `pr_target`의 기준 저장소, 연결 안 됐으면 `GithubNotConnected`), 신규 `actions/pr_comment.rs`(+시험), `actions/mod.rs`(등록), `view/input.rs`(`PrComment(PrCommentInput { number, body })`), `action.rs`(`Outcome::PrCommented`, `journal_mode` = `Skip`), `bootstrap.rs`(잠금 `Network`), `github/errors.rs`(맥락별 변환 정리), `src-tauri`(`query_pr_repo`·`query_pr_comments` 네 곳), `api.ts`, `src/messages`, 생성물, `docs/architecture.md`(액션 목록·명령·입력 상한: 댓글 64KB·읽기 20개·본문 4000자).
- 내용: 액션 `pr-comment`(용어 `comment`/`Safe`/`Nothing`). 순서: 번호 ≥ 1 → 본문 검사(앞뒤 공백 제거 뒤 1바이트 이상·64KB·NUL 없음) → `pr_target`·GitHub 저장소 → `call(create_comment)`. 댓글 쪽은 `ceil(issue_comments / 20)`로 마지막 쪽, 둘째 마지막 쪽까지 읽어 20개를 채우고 `partial`은 더 있음. `mine` = 작성자가 연결된 로그인.
- 테스트(가짜 전송): `query_pr_repo` 값(권한 있음·없음, 방법 목록, `allow_*` 없음 → `[merge]`, 보관된 저장소), 댓글 쪽 계산(0·1·20·21·45개), `mine`, 액션 정상·빈 글·64KB+1·NUL은 전송 0회, 401이 연결을 끊음, `action-meta.json`(`safe`/`nothing`), `tests/security_config.rs`, 가짜 토큰이 결과·오류에 없음.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 화면이 보낸 줄바꿈(`\n`)을 그대로 보낸다(Windows 입력칸도 `\n`). 보안: 새 조회는 열린 저장소만·읽기 전용, 인자에 주소·토큰 없음.

W10. core: 합치기·닫기·다시 열기 (`core: add pr-merge, pr-close and pr-reopen actions`)
- 목표: C5·C6의 액션을 만든다.
- 바꿀 파일: 신규 `actions/pr_merge.rs`·`actions/pr_state.rs`(+시험), `actions/mod.rs`(등록 셋), `view/input.rs`(`PrMerge(PrMergeInput { number, method, expect_head })`, `PrClose(PrNumberInput)`, `PrReopen(PrNumberInput)`), `action.rs`(`Outcome::PrMerged`·`PrClosed`·`PrReopened`, 모두 `Skip`), `bootstrap.rs`(잠금 `Network`), `error.rs`·`view/mod.rs`(`GithubFailure`·`ErrorCode`: `PrNotMergeable`, `PrHeadChanged`, `PrNotOpen`), `github/errors.rs`(합치기 맥락: `NotAllowed` → `PrNotMergeable`, `Conflict` → `PrHeadChanged`), `src/messages`(문구: "지금은 합칠 수 없어요. GitHub에서 까닭을 봐 주세요.", "그사이 새 버전이 올라왔어요. 다시 확인해 주세요.", "열린 검토 요청이 아니에요."), **`Irreversible` 집합 시험**(010-H9/H11의 시험에 `pr-merge` 추가), 생성물, `docs/architecture.md`.
- 내용: `pr-merge`(용어 `pr-merge`/`Normal`/`Irreversible`, `moves_head` 거짓): 번호 ≥ 1, `expect_head` 16진수 40·64자, 방법이 `query_pr_repo`와 같은 규칙으로 허용된 것인지 core가 `repo_info`로 다시 확인(허용 안 되면 `PrNotMergeable`, 확인 실패는 그 오류), `call(merge_pr)`. 응답 `merged`가 거짓이면 `PrNotMergeable`. `pr-close`(용어 `pr-close`/`Normal`/`Nothing`)·`pr-reopen`(용어 `pr`/`Safe`/`Nothing`): `call(set_pr_state)`. 닫힌 것을 닫거나 열린 것을 열면 서버 422 → `PrNotOpen`. 확인 토큰은 실행기가 `pr-merge` 입력(번호·방법·해시)에 묶는다.
- 테스트(가짜 전송 + 실제 `Core`): 토큰 없이 `PreviewRequired`·다른 입력의 토큰 `PreviewMismatch`, 정상 합치기 요청 본문(`sha`·`merge_method`), 허용 안 된 방법은 PUT 0회, 405·409·422 문구 코드, 닫기·다시 열기 본문과 오류, `action-meta.json`(셋의 위험도·`undo`), **`Irreversible` 집합 = 010 값 + `pr-merge`**, 원격 잠금 중 다른 원격 작업은 `Busy`.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음. 보안: 합치기는 화면이 본 해시로만, 확인 토큰이 입력에 묶임.

W11. 화면: 검토 요청 상세의 댓글·합치기·닫기 (`features: comment, merge and close in pull request detail`)
- 목표: 상세에서 C4~C6을 쓴다.
- 바꿀 파일: `src/features/pr/PrRow.tsx`·`PrRow.module.css`(상세 아래: 단추 줄 `[합치기]` 주 버튼·글자 버튼 `<Term pr-close>`·"다시 열기", 그 아래 댓글), 신규 `src/features/pr/{PrComments.tsx, CommentBox.tsx, MergeDialog.tsx, mergeStore.ts, PrComments.test.tsx, CommentBox.test.tsx, MergeDialog.test.tsx}`, `src/lib/queries.ts`(`usePrRepo`(`staleTime` 5분), `usePrComments`(상세가 열릴 때만), 쓰기 뒤 `prDetail`·`prPages`·`prComments` 무효화), `src/App.tsx` 또는 `PullRequests.tsx`(`<MergeDialog />`), `src/messages`, `docs/design.md`("검토 요청 화면"에 단추 줄·댓글, 새 항목 "검토 요청 합치기 확인 대화상자").
- 내용: 댓글 목록은 줄마다 작성자(`<bdi>`)·`<Time>`·본문(`white-space: pre-wrap`, `overflow-wrap: anywhere`, `data-selectable`가 생기기 전이면 기본 선택 허용), 내 댓글은 작성자 옆 `--text-muted` "나". 20개보다 많으면 위에 "그 전 댓글 N개 · GitHub에서 열기". 쓰기 칸은 3줄 `textarea`, 주 버튼 "댓글 달기"(빈 글이면 꺼짐, Ctrl+Enter, 조합 중 무시), 보내는 동안 버튼 진행 표시, 성공하면 칸을 비우고 토스트 "댓글을 달았어요". `MergeDialog`는 `ConfirmDialog`(560px): 제목 `<Term pr-merge>`, 부제 `#12 · feature → main`(코드 글꼴), 방법 고르기(`ui/Choice`, 허용된 것만, 하나면 고르기 없이 한 줄), 시험 결과 실패·진행 중이면 `--deleted` 12px 한 줄, 뒤처짐·막힘이면 `--text-muted` "GitHub 규칙에 따라 거절될 수 있어요", 위험 버튼 "합치기", 주 버튼 "닫기"(포커스). `previewAction` → 토큰 → `executeAction(…, token)`. 성공: 닫고 `executeAction(repo, { id: "fetch" })`를 한 번(실패해도 무시) → 그 브랜치의 원격 추적 줄이 아직 있으면 토스트 "합쳤어요" + 글자 버튼 "브랜치 지우기"(010-H13 `remoteBranchStore`로 지우기 확인 창), 없으면 "합쳤어요"만. 꺼짐 까닭(툴팁·`aria-describedby`): 초안 "초안은 합칠 수 없어요", 겹침 "겹치는 곳을 GitHub에서 풀어야 해요", rebase만 허용 "GitHub에서 합쳐 주세요"(옆에 "GitHub에서 열기"). 닫기: 바로 `executeAction(pr-close)` → 토스트 "닫았어요" + "다시 열기". 다른 저장소(포크)에서 낸 요청도 같은 단추(서버가 권한 결정).
- 테스트: 권한·상태별 단추 표(권한 없음·작성자·초안·겹침·닫힘·합쳐짐), 댓글 목록 모양(긴 글·한글·이모지·HTML 같은 글이 글자로), 쓰기 IME·Ctrl+Enter·빈 글, 합치기 확인 흐름(포커스 "닫기", Enter 안전, 토큰, 입력 JSON에 `expectHead`), 방법 고르기, 409·405 문구가 대화상자 안, 성공 뒤 `fetch`와 "브랜치 지우기" 조건, 닫기·다시 열기 토스트, 800px·다크.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 일회용 저장소에서 댓글·합치기·닫기(사람이 할 일).
- Windows/Ubuntu: 확인 창 포커스(WebKitGTK `onOpenAutoFocus`), textarea의 한글 조합과 Ctrl+Enter, 800px. 보안: 서버 글은 글자로만, `dangerouslySetInnerHTML` 없음.

### D. 포크

W12. core: 포크와 포크 경로의 검토 요청 (`core: add fork action and fork route for pull requests`)
- 목표: C7을 core에 만든다.
- 바꿀 파일: 신규 `actions/fork.rs`(+시험), `actions/mod.rs`, `view/input.rs`(`ActionInput::Fork`, 입력 없음), `action.rs`(`Outcome::Forked { owner, name }`, `Skip`), `bootstrap.rs`(잠금 `Both`, core `REMOTE_ACTIONS`에 `fork`), `actions/remote.rs`(`Net::push`를 `fork.rs`가 쓰게 `pub(crate)`, 구조 변경 없음), `github/pr.rs`(`PrTarget`에 `base_repo`·`head_owner`, 포크 경로 판정: upstream 원격 = `fork`, 그 올리기 주소들이 연결된 로그인의 같은 저장소, `origin`이 GitHub 저장소; 그러면 `base_repo` = `origin`, `remote_branches`·`default_branch`도 `origin` 기준), `actions/pr_create.rs`(포크 경로면 `NewPr.head_owner = 로그인`, 보내기 전 `repo_info(fork).parent == origin 저장소`, 아니면 `ForkParentMismatch`), 목록·상세·댓글·합치기·닫기·지금 브랜치 요청 찾기(`list_prs`의 head 필터 소유자)가 `base_repo`를 씀, `view/hosting.rs`(`HostingContextView.viaFork`, `PrDraftView.viaFork`), `error.rs`·`view/mod.rs`(`ForkNotReady`, `ForkRemoteTaken`, `ForkParentMismatch`), `src/features/progress/useProgress.ts`(`REMOTE_ACTIONS`에 `fork`, 010 R7 시험이 core와 같은지 확인), `src/messages`, 생성물, `docs/architecture.md`(액션·잠금·"GitHub 연결"의 검토 요청 대상).
- 내용: `fork` 순서: `require_branch` → `refuse_during_operation` → 연결 토큰 → 대상 = `pr_target`의 GitHub 저장소(`origin`) → `call(create_fork)` → 소유자 == 로그인·이름 검사 → 준비 확인(2초 × 15, 매번 취소 표시) → 원격 `fork` 확인(없으면 `RemoteAdd { name: "fork", url }`; 있으면 `remote_urls`의 받기·올리기가 모두 그 주소 하나일 때만 재사용, 아니면 `ForkRemoteTaken`) → `Net::push((fork, 지금 브랜치), set_upstream = true)`(`auth_for`, 브랜치 이름 검사). 올리기가 실패하면 원격 `fork`는 남기고 오류(다시 누르면 이어서 됨).
- 테스트(가짜 전송 + 로컬 bare 두 개로 `origin`·`fork` 흉내, `allow_local_remotes_for_tests`): 정상(원격 `fork` 주소가 core가 만든 꼴, 지금 브랜치 upstream이 `fork/<브랜치>`), 이미 포크가 있음(같은 응답) → 원격 재사용, 다른 주소의 `fork` 원격 → `ForkRemoteTaken`·`pushurl`만 다른 경우도, 준비 안 됨 15번 → `ForkNotReady`(가짜 시계), 취소, 응답 소유자 불일치 → 원격 안 생김, 포크 경로 `pr_target`(기준 저장소·head 소유자·기본 브랜치), `pr-create` 포크 경로 본문 `head: "mona:feature"`와 `parent` 불일치 거절, 같은 저장소 경로 시험은 그대로, `action-meta.json`(`normal`/`nothing`), 바인딩 최신.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 로컬 경로 원격 시험은 기존 `remote_tests` 방식. 보안: 원격 이름은 상수, 주소는 검사한 로그인·이름으로 core가 만든다. 새 원격도 `auth_for` 조건을 그대로 거친다.

W13. 화면: 포크해서 요청하기 (`features: fork and open a pull request`)
- 목표: 올릴 권한이 없을 때 새 검토 요청 대화상자에서 포크 흐름으로 간다(C7).
- 바꿀 파일: `src/features/pr/CreatePrDialog.tsx`·`.module.css`·`.test.tsx`(`usePrRepo().canPush === false`이고 `viaFork`가 아니면 위에 안내 "이 저장소에 올릴 권한이 없어요. 내 계정에 `t("fork")`를 만들어 거기서 요청해요." + 주 버튼 "포크 만들고 올리기" → `executeAction(fork)`(진행 "포크를 준비하는 중 · N초", 3초 뒤 "취소") → 성공하면 같은 대화상자가 `prDraft`를 다시 읽고 기존 "만들기" 흐름(`pr-create`)으로), `src/features/pr/PrSlot.tsx`·`model.ts`(지금 브랜치 요청 표시가 포크 경로에서도 맞게), `screens/PullRequests/PullRequests.tsx`(머리의 `소유자/이름`이 기준 저장소, 포크 경로면 옆에 `--text-muted` "포크에서 요청"), `src/messages`, `docs/design.md`("새 검토 요청 대화상자").
- 내용: `ForkNotReady`·`ForkRemoteTaken`·`ForkParentMismatch`·`HttpForbidden`은 대화상자 안 오류 줄. 이미 포크 경로면 안내 없이 평소 대화상자(올리기는 `push`가 `fork`로 감).
- 테스트: 권한별 표시(있음·없음·포크 경로), 실행 순서(fork → prDraft 다시 읽기 → push 없이 pr-create, 안 올린 버전이 있으면 push 먼저), 진행·취소, 오류별 문구, 포크 경로의 PrSlot, 800px·IME.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 두 번째 계정이나 남의 공개 저장소로 포크해서 요청(사람이 할 일).
- Windows/Ubuntu: 진행 표시는 기존 진행 이벤트. 보안: 화면은 입력 없는 액션을 부를 뿐 원격·주소를 다루지 않는다.

### E. 원격 태그

W14. git+core: 원격 태그 지우기 (`git+core: add remote-tag-delete with lease`)
- 목표: C8을 만든다.
- 바꿀 파일: `crates/git/src/types.rs`(`GitOp::PushDeleteTag { remote, name, expect: Oid, auth }`), `cli/refpush.rs`(`push_delete_tag`: `--force-with-lease=refs/tags/<name>:<expect>`, refspec `:refs/tags/<name>`), `cli/mod.rs`(매핑), `cli/refpush_tests.rs`, 가짜 구현 둘, 신규 `crates/core/src/actions/remote_tag.rs`(+시험), `actions/mod.rs`, `view/input.rs`(`RemoteTagDelete(RemoteTagInput { name, expect })`), `action.rs`(`Outcome::RemoteTagDeleted`, `Skip`), `bootstrap.rs`(잠금 `Network`, `REMOTE_ACTIONS`), `src/features/progress/useProgress.ts`(`REMOTE_ACTIONS`), `error.rs`·`view/mod.rs`(`RemoteTagDiffers`; 없음은 010-H7a의 `RemoteRefMissing` 재사용, 문구는 맥락으로 "원격에 이 태그가 없어요"), **`Irreversible` 집합 시험**(`remote-tag-delete` 추가), `src/messages`, 생성물, `docs/architecture.md`(GitOp 목록·액션).
- 내용: 액션 `remote-tag-delete`(용어 같은 id/`Danger`/`Irreversible`, `moves_head` 거짓). 순서: `check_tag_name`(D-104 규칙) → `resolve_ref(refs/tags/<이름>)` == `expect`(아니면 `InvalidInput`) → `pick_remote` → `auth_for` → `PushDeleteTag`. `LeaseStale`이면 `ls_remote(refs/tags/<이름>)`: 없음 → `RemoteRefMissing`, 다른 값 → `RemoteTagDiffers`. 로컬 태그·백업 ref는 만들지 않는다(로컬 태그가 그대로 있어 다시 올릴 수 있음, 확인 창 문구가 그것을 말한다).
- 테스트(실제 git, 로컬 bare): 원격 태그가 같은 값이면 지워지고 로컬 태그 그대로, 원격이 다른 값이면 거절·원격 그대로·`RemoteTagDiffers`, 원격에 없으면 `RemoteRefMissing`, 주석 태그(태그 객체 값)도 같은 규칙, 다른 태그·브랜치는 그대로, `-x`·`a b` 이름 거절, 인자에 `--force` 단독·`+`가 없음, 토큰 없이 `PreviewRequired`, `action-meta.json`(`danger`/`irreversible`), **집합 = 010 값 + `pr-merge` + `remote-tag-delete`**, `git-min`(값 있는 tag lease가 2.34.1에서 되는지).
- 완료 기준: `cargo test --workspace`(두 OS, `git-min`), 바인딩 최신.
- Windows/Ubuntu: 한글 태그 이름의 인자(셸 없음). 보안: 값 있는 lease만, 이름은 core 검사, 해시는 `Oid`.

W15. 화면: 태그 대화상자의 "원격에서 지우기" (`features: delete a tag on the remote from the tag dialog`)
- 목표: 태그 줄에서 원격 태그를 지운다(C8).
- 바꿀 파일: `src/features/history/menu/dialogs/TagDialog.tsx`·`.test.tsx`(010-H13의 "올리기" 옆 글자 버튼 "원격에서 지우기", 원격이 없으면 없음), 신규 `src/features/remote-tag/{RemoteTagDeleteDialog.tsx, RemoteTagDeleteDialog.test.tsx}`, `src/messages`, `docs/design.md`("확인 대화상자"에 원격 태그).
- 내용: `ConfirmDialog`: 제목 `<Term remote-tag-delete>`, 부제 `origin · v1.0`(코드 글꼴), 본문 "원격 저장소의 태그를 지워요. 내 컴퓨터의 태그는 그대로라 다시 올릴 수 있어요." 위험 버튼 "지우기", 주 버튼 "닫기"(포커스). 입력 `{ name, expect: 태그 줄의 값 }`. 성공 토스트 "원격 태그를 지웠어요"(되돌리기 단추 없음). 오류(`RemoteTagDiffers`·`RemoteRefMissing`·`RemoteRefused`)는 대화상자 안.
- 테스트: 단추 표시 조건, 확인 흐름(토큰·입력·포커스·Enter 안전), 오류 문구, 진행·취소, 800px·긴 태그 이름 말줄임.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 일회용 저장소에서 원격 태그 지우기(사람이 할 일).
- Windows/Ubuntu: 대화상자 위 대화상자(태그 대화상자 위 확인 창)의 포커스·Esc. 보안: 화면은 대화상자가 보인 값만 보낸다.

### F. 원격 기본 브랜치

W16. core: 이름 입력으로 기본 브랜치 덮어쓰기·지우기와 보호 규칙 조회 (`core: allow default branch force-push and delete with name confirmation`)
- 목표: C9 ①~⑤를 core에 만든다.
- 바꿀 파일: 010-H9의 원격 기본 브랜치 판정 함수(서버 기본 / 이름만 기본 같은 것으로 나눈 `DefaultKind::{Server, NameOnly}`를 주게), `actions/force_push.rs`·`actions/remote_branch.rs`(입력의 `confirm_branch` 확인, 서버 기본 지우기·이름 바꾸기는 계속 `RemoteBranchProtected`), `view/input.rs`(세 입력에 `confirm_branch: Option<String>`, `skip_serializing_if`), `view/force_push.rs`·`view/remote_branch.rs`(`blocked`의 기본 브랜치 값을 `needsNameConfirm: bool`과 `serverDefault: bool`로; 서버 기본 지우기만 `blocked`로 남김), `guard.rs`(010-H8의 "upstream이 기본 브랜치면 `Refuse`" 분기를 지우고 스위치만 봄), `query.rs`·`bootstrap.rs`(`query_branch_protection(repo, remote, branch) -> Option<bool>`: GitHub 원격·연결됨일 때만 `call(branch_protected)`, 그 밖 `None`), `view/github.rs`(`HostingPage::BranchSettings`)·`bootstrap.rs`(`hosting_page_url`: 열린 저장소의 GitHub 원격 소유자·이름으로 `{web}/{o}/{r}/settings/branches`), `error.rs`·`view/mod.rs`(`NameConfirmMismatch`), `input.rs`의 `journal_copy`(새 칸), `src-tauri`(`query_branch_protection` 네 곳), `api.ts`, `src/messages`, 생성물, `docs/architecture.md`("기록을 바꾸는 액션의 검사"·보안 표 "원격 기본 브랜치를 잘못 덮어씀" 줄).
- 내용: 이름 비교는 바이트 정확 일치(앞뒤 공백도 다름). 서버 기본을 읽지 못하면 서버 기본으로 본다(fail-closed: 덮어쓰기는 이름 입력으로 되지만 지우기는 막힘). 표식 `rewrote.json`과 `RewrotePublished`는 기본 브랜치에서도 같은 규칙.
- 테스트(로컬 bare + 실제 `Core`): 기본 브랜치에서 스위치 켜짐 → 고치기·돌아가기·기록 정리가 됨(꺼짐은 `PublishedHistory` 그대로), 기본 브랜치 덮어쓰기: `confirm_branch` 없음·다름 → `NameConfirmMismatch`·원격 그대로, 같음 → 덮어씀·백업 ref, 서버 기본 지우기·이름 바꾸기 → 여전히 거절, 이름만 `main`(서버 기본은 `trunk`) 지우기 → 이름 입력 뒤 됨, 서버 기본을 못 읽으면 지우기 거절, 옛 journal 줄(새 칸 없음) 서명 검증, 보호 조회(가짜 전송 참·거짓·`None`, GitHub 아님 → 전송 0회), `HostingPage::BranchSettings` 주소가 검사한 값, 확인 토큰이 `confirm_branch`까지 묶음, `tests/security_config.rs`.
- 완료 기준: `cargo test --workspace`, 바인딩·명령 네 곳 최신.
- Windows/Ubuntu: 차이 없음. 보안: 이름 입력은 토큰과 함께 입력에 묶이고 core가 다시 비교한다. 서버 보호 규칙은 서버가 마지막에 지킨다(앱은 안내만).

W17. 화면: 이름 입력 확인과 보호 규칙 안내 (`screens: type the branch name to overwrite or delete the default branch`)
- 목표: C9를 확인 창에 보인다.
- 바꿀 파일: 010-H10의 `src/features/force-push/ForcePushDialog.tsx`, 010-H13의 `src/features/remote-branch/{RemoteDeleteDialog, RemoteRenameDialog}.tsx`(`needsNameConfirm`이면 본문 아래 입력칸 "브랜치 이름 {name}을 똑같이 입력해 주세요", 코드 글꼴, 같을 때만 위험 버튼 켬, Enter는 위험 버튼을 누르지 않음, 조합 중 무시), 신규 `src/features/remote-branch/{NameConfirmField.tsx, useBranchProtection.ts}`(확인 창이 열릴 때 한 번 `query_branch_protection`, 참이면 `--deleted` 12px 한 줄), 010-H10·H13의 메뉴·알림 줄 availability(기본 브랜치에서 덮어쓰며 올리기 항목이 보임, 원격 줄 메뉴의 이름만 기본 같은 브랜치가 흐리지 않음, 서버 기본은 지금처럼 흐림), `RemoteRefused` 오류 줄에 "GitHub에서 보기"(`openHostingPage(repo, { kind: "branchSettings" })`)와 "저장소 관리자만 바꿀 수 있어요", 설정 "고급" 스위치 설명 한 줄의 기본 브랜치 언급 정리, `src/messages`, `docs/design.md`("확인 대화상자"에 이름 입력).
- 테스트: 이름 입력 표(빈 값·앞뒤 공백·대소문자 다름·같음), 위험 버튼 켜짐 조건, Enter·IME, 보호 줄(참·거짓·`None`), 서버 기본 지우기 흐림·이름만 기본 같은 것 허용, `RemoteRefused` 단추, 입력 JSON에 `confirmBranch`, 기존 010 확인 창 시험 통과, 800px.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 일회용 저장소의 `main` 덮어쓰기와 보호 규칙을 켠 저장소에서 거절(사람이 할 일).
- Windows/Ubuntu: 입력칸 한글 조합(브랜치 이름이 한글일 때), 800px. 보안: 화면 비교는 힌트이고 core가 다시 비교한다.

### G. 조직 저장소와 찾기

W18. core: 조직 저장소로 올리기 (`core: publish to an organization`)
- 목표: C10을 core에 만든다.
- 바꿀 파일: 010-H15의 `actions/publish.rs`(입력 `owner`가 있으면 `valid_owner` → `create_repo(RepoOwner::Org)`, 응답 소유자 == 고른 곳, `origin` 주소는 그 소유자로), `view/input.rs`(`PublishInput.owner: Option<String>`, `skip_serializing_if`), 010-H16의 `view/github.rs`(`GitHubRepoRowView.ownerIsOrg`), `github/repo.rs`(행 변환), `github/errors.rs`(만들기 맥락 403 → `GithubOrgForbidden`), `error.rs`·`view/mod.rs`, `src/messages`, 생성물, `docs/architecture.md`(입력 상한: 조직 이름은 `valid_owner` 39자).
- 테스트(가짜 전송 + 실제 git): 내 계정(지금과 같음)·조직 경로와 본문, 소유자 불일치 → `BadResponse`·`origin` 없음, 403 → `GithubOrgForbidden`, 조직 이름 규칙 표(`-a`·빈 값·40자)는 전송 0회, 이름 중복 422는 기존 `GithubRepoExists`, `ownerIsOrg` 행, 바인딩 최신.
- 완료 기준: `cargo test --workspace`, 바인딩 최신.
- Windows/Ubuntu: 차이 없음. 보안: 조직 이름은 검사한 값만 URL 조각이 되고 주소는 core가 만든다.

W19. 화면: GitHub에 올리기의 "만들 곳" (`features: choose where to create the repository when publishing`)
- 목표: 올리기 대화상자에서 내 계정·조직을 고른다(C10).
- 바꿀 파일: 010-H17의 `src/features/publish/PublishDialog.tsx`·`.module.css`·`.test.tsx`(이름 칸 위 "만들 곳": `ui/Choice`나 선택 목록 "내 계정 · {login}"(기본)·조직들·"다른 조직", "다른 조직"이면 입력칸(코드 글꼴, 같은 규칙으로 즉시 안내)), `src/features/publish/useOwners.ts`(`useGithubRepos` 첫 3쪽의 `ownerIsOrg` 소유자, 중복 없이 이름순), `src/messages`, `docs/design.md`("GitHub에 올리기 대화상자").
- 내용: 조직을 고르면 부제에 `조직/이름` 미리보기. 공개 확인 단계·주 버튼 "저장소 만들고 올리기"는 그대로.
- 테스트: 후보 목록(조직 없음·여럿·중복), 입력 JSON의 `owner`(내 계정이면 없음), "다른 조직" 입력 검사·IME, `GithubOrgForbidden` 문구, 기존 대화상자 시험 통과, 800px.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 실제 조직(만들 권한 있음·없음)으로 올리기(사람이 할 일).
- Windows/Ubuntu: 선택 목록 키보드. 보안: 화면은 이름만 보낸다.

W20. 화면: 복제 목록 찾기 (`features: search the github repository list when cloning`)
- 목표: 내 저장소 목록에서 이름으로 찾는다(C11).
- 바꿀 파일: 010-H18의 `src/features/clone/RepoList.tsx`·`useRepoList.ts`·`CloneDialog.test.tsx`(찾기 칸, 거르기, 찾기 중 다음 쪽 저절로 읽기 500개 상한, "N개 중 찾는 중"·"더 많아요. 이름을 더 쳐 주세요"), 신규 `src/lib/search.ts`(대소문자 무시·NFC 정규화 부분 일치, 순수 함수 + 시험), `src/messages`, `docs/design.md`("복제 대화상자").
- 내용: 찾기 칸에서 ↓는 목록 첫 줄로, 목록 키는 기존 `listKeys`. 찾기 글자가 비면 지금 목록 그대로. 저절로 읽기는 찾기 칸에 글자가 있는 동안만, 한 번에 한 쪽, 대화상자가 닫히면 멈춘다.
- 테스트: 거르기 표(대소문자·한글·`/` 포함), 저절로 읽기가 10쪽에서 멈춤, 글자를 지우면 멈춤, 결과 없음 문구, 키보드, 기존 목록 시험 통과.
- 완료 기준: `pnpm test`, `pnpm lint`, `pnpm typecheck`. 손 확인: 저장소가 많은 계정에서 찾기(사람이 할 일).
- Windows/Ubuntu: 한글 조합 중에도 거르기는 조합 결과로(조합 중 Enter 무시). 보안: 새 요청 없음.

### H. 마무리

W21. 문서·점검·사람이 할 일 (`docs: wrap up iteration 4c`)
- 목표: 문서를 구현에 맞추고 규칙·보안·접근성을 점검한다.
- 바꿀 파일: `docs/architecture.md`(HostingProvider 새 함수, `Method`, 액션 목록과 각 `undo`, `Irreversible` 집합 7개, 잠금·`REMOTE_ACTIONS`, 명령 목록 4개 추가와 `github_connect_start` 인자, "GitHub 연결"의 `auth_for` 설정 조건·포크 경로, 보안 표 줄: "남의 검토 요청을 잘못 합침"(확인 창·`sha`), "포크 원격 바꿔치기"(`fork` 원격 재사용 조건), "이메일 목록 노출"(결과 종류만), "원격 기본 브랜치를 잘못 덮어씀"(이름 입력·토큰), 입력 상한(댓글 64KB, 읽기 20개·4000자, 포크 대기 30초, 찾기 500개)), `docs/design.md`(각 화면 최종), `docs/spec.md`(화면 구성 행 최종), `docs/roadmap.md`(반복 4c 완료, 이후 후보: 리뷰 쓰기·줄 댓글·rebase 합치기·원격만 있는 태그·`read:org`), `docs/progress.md`(체크 W1~W21, "사람이 할 일"), `.claude/agents/code-analyst.md`(점검 줄: `Method::Put`·`Patch`는 hosting `github.rs`의 `api_write`만, 새 HTTP 쓰기 액션이 원격 잠금만·journal 없음, `Irreversible` 집합 7개와 화면 `ConfirmDialog`, `PushDeleteTag`가 값 있는 lease만, 원격 이름 `fork`가 상수 하나, 이메일 주소가 hosting·core `email.rs` 밖으로 안 나감, `auth_for` 호출이 모두 설정 값을 넘김, `confirm_branch`가 core에서 비교됨).
- code-analyst 점검: 계층 규칙(화면에서 `invoke`·git·`fetch` 없음, hosting·git이 서로·core를 모름), 용어·문구 규칙, 위험도 = 용어표(새 액션 6개), `dangerouslySetInnerHTML` 없음, 서버 글이 글자로만, 토큰·이메일이 로그·오류·화면 타입에 없음, 새 명령 네 곳, 단축키는 `shortcuts.ts`만(새 단축키 없음, Ctrl+Enter는 입력칸 안 동작), 확인 창 포커스·Enter 안전·IME, `cargo deny check`.
- 완료 기준: 위반 0건, 두 OS 새 clone에서 `pnpm install && pnpm check:full`, "완료 기준(반복 전체)"이 모두 자동 시험이나 "사람이 할 일"에 있다. CI는 `gh run list`로 확인.
- Windows/Ubuntu: 문서만. 보안: 위 점검이 보안 점검이다.

## 위험
- **남에게 보이는 쓰기를 잘못 누름**: 댓글·닫기는 확인 없이 된다(D-043). 댓글은 지울 수 없고(앱에서) 알림 메일이 이미 나간다. 대책은 쓰기 칸과 주 버튼이 분명하고, 빈 글은 꺼지고, 성공 토스트에 "GitHub에서 열기"(지우기는 GitHub에서). 닫기는 "다시 열기"로 푼다.
- **합치기가 기본 브랜치를 바꿈**(가장 큼): 확인 창 + 확인 토큰 + `sha`(본 head만) + 저장소가 허용한 방법만 + 서버 보호 규칙. 남는 위험: 시험 결과가 실패해도 보호 규칙이 없으면 합쳐진다(경고 줄만). 되돌리기는 GitHub의 "Revert" 단추(새 검토 요청)이고 앱은 안내하지 않는다.
- **GitHub 응답 가정**: `allow_*_merge` 칸이 쓰기 권한 없는 사람에게 빠질 수 있고, 포크 준비 시간이 30초를 넘을 수 있고, 422 문구가 바뀔 수 있다. 칸이 없으면 "합친 버전 만들기"만 보이고 서버가 405로 막으면 문구로 안내한다. 포크는 다시 누르면 이어서 된다. 실제 응답은 손 확인으로 맞춘다.
- **포크 경로 판정**: 원격 이름 `fork`와 `origin`에 기댄다. 사람이 원격 이름을 바꾸거나 `origin`이 다른 저장소면 포크 경로가 아닌 것으로 보고 같은 저장소 요청을 시도한다(`pr-create`가 403·404로 실패 → 문구). 포크의 `parent`를 보내기 전에 확인하므로 엉뚱한 저장소로 요청이 가지는 않는다. 포크 경로에서 동기화는 `fork`에서 가져온다(원래 저장소의 새 버전은 "main 가져와 합치기"가 `origin` 기준으로 제안, 010까지의 동작). 포크를 원래 저장소와 맞추는 기능은 이번에 없다.
- **받은 저장소가 심어 둔 `fork` 원격**: `remote.fork.url`·`pushurl`을 미리 넣어 두면 첫 올리기가 엉뚱한 곳으로 갈 수 있다 → 받기·올리기 주소가 모두 core가 만든 주소 하나일 때만 재사용(C14). `pushInsteadOf` 같은 다른 설정은 5-0 신뢰 확인 범위다(D-137).
- **기본 브랜치 고치기를 연 것**(C9 ③): 스위치를 켠 사람은 `main`에서도 올린 버전을 고칠 수 있게 된다. 덮어쓰며 올리기에 이름 입력이 있어 실제 원격 피해 전에 한 번 더 막지만, 고친 뒤 올리지 않고 두면 `RewrotePublished`가 동기화를 멈춘다(010의 동작, 막다른 길은 이름 입력 덮어쓰기로 풀림). 사람이 이 범위를 원치 않으면 ③만 되돌리는 것이 쉽다(가드 한 분기).
- **서버 기본 판정 실패**: `ls-remote --symref`가 안 되면 서버 기본으로 본다(지우기 막힘, 덮어쓰기는 이름 입력). 오프라인에서 계획 조회는 로컬 ②③만 본다.
- **이메일 비교의 한계**: 전역 이메일만 비교한다. 저장소 로컬 `user.email`을 쓰는 사람은 버전에 다른 이메일이 남을 수 있다(표시 안 됨). GH007 문구가 그 경우를 덮는다. 권한 `user:email`은 한 번 더하면 줄일 수 없다(연결 끊고 다시).
- **권한 다시 연결**: 권한을 더할 때 새 토큰이 이전 것을 바꾼다. 합치는 값을 틀리면 워크플로 권한을 잃는다 → W7 시험이 표로 고정.
- **로그인 스위치를 끈 뒤**: GCM 등 사용자 helper가 없으면 https 올리기가 "인증 실패"로 끝난다. 문구는 기존 `HttpAuthFailed`이고 설정 위치를 알려 주는 해결 단추는 012-U9의 `ERROR_FIX`가 맡는다(012가 이 계획 뒤에 오므로 새 오류 코드 `PrNotMergeable`·`PrHeadChanged`·`PrNotOpen`·`ForkNotReady`·`ForkRemoteTaken`·`ForkParentMismatch`·`RemoteTagDiffers`·`NameConfirmMismatch`·`GithubOrgForbidden`·`PrivateEmailBlocked`도 012-U9의 표가 덮어야 한다).
- **찾기의 500개 상한**: 저장소가 더 많은 사람은 다 못 찾는다(문구로 알림). 레이트는 쪽당 1회라 10회가 상한.
- **010 결과물 이름이 다를 수 있음**: 010-H9~H19가 아직 구현 전이라 이 계획의 파일·타입 이름은 010 계획 기준이다. 구현 첫날 `rg`로 맞추고 어긋남은 progress에 적는다. 010이 범위를 바꿨으면(예: H19를 미룸) 이 계획의 해당 단계 앞에 그것을 먼저 한다.
- 구현 중 알게 된 것은 이 문서가 아니라 `docs/progress.md`에 적고, 결정이 필요하면 `docs/decision.md` 맨 아래에 "(자동 결정)"으로 올린 뒤 이어 간다.
- code-analyst 검토 때 먼저 볼 곳: W2(PUT·PATCH 리디렉트가 같은 API 주소만, 405·409 표), W3·W9(서버 글 자르기·글자로만, 댓글 쪽 계산), W7(권한 합치기, 이메일이 화면 타입에 없음), W10(`Irreversible`·토큰이 `sha`까지 묶임, 방법 재확인), W12(`fork` 원격 재사용 조건·주소 생성·`parent` 확인·준비 대기 취소), W14(태그 lease 값·stale 뒤 재확인), W16(이름 비교 위치·fail-closed·가드 분기 제거가 다른 검사를 건너뛰지 않음·옛 journal 호환), W6(`auth_for` 호출부 전부가 설정을 넘김).

## 사람이 할 일
`docs/progress.md`의 "사람이 할 일"에 W21에서 옮긴다. **사람만 줄 수 있는 것은 새로 없다**(client id는 반복 4에서 받은 것). 실제 컴퓨터·계정에서 확인할 것:
- **일회용 비공개 저장소와 가능하면 두 번째 계정이나 공동 작업자**로: 검토 요청에 댓글 달기(한글·이모지·긴 글, GitHub 화면과 같은지), 최근 20개 목록과 "그 전 댓글", 합치기 두 방법(저장소 설정에서 하나만 허용했을 때 하나만 보이는지), 확인 창을 연 채 다른 계정으로 새 버전을 올린 뒤 합치기 → "그사이 새 버전이 올라왔어요", 겹치는 요청·초안의 꺼짐, 합친 뒤 "브랜치 지우기", 저장소의 "합친 뒤 브랜치 자동 삭제"를 켰을 때 단추가 안 뜨는지, 닫기·다시 열기, 권한 없는 계정에서 단추가 안 보이는지.
- **포크**: 두 번째 계정(또는 남의 공개 저장소)을 복제한 뒤 새 검토 요청 → 포크 안내 → "포크 만들고 올리기" → 원래 저장소에 요청이 생기는지(head가 `계정:브랜치`), 이미 포크가 있을 때, 포크 준비가 느릴 때, 그 뒤 동기화·검토 요청 목록·브랜치 목록 표시. 만든 포크는 GitHub에서 직접 지운다.
- **원격 태그**: 태그를 올린 뒤 "원격에서 지우기", 다른 계정이 같은 이름 다른 버전으로 바꿔 둔 경우의 거절, GitHub 릴리스가 달린 태그를 지웠을 때 릴리스가 어떻게 되는지 기록.
- **원격 기본 브랜치**: 일회용 저장소에서 스위치 켜고 `main`의 올린 버전 고치기 → 덮어쓰며 올리기의 이름 입력, GitHub 브랜치 보호 규칙("Allow force pushes" 끔)을 켠 저장소에서 보호 줄과 거절 문구·"GitHub에서 보기", 서버 기본이 `main`이 아닌 저장소에서 `main` 지우기.
- **조직**: 만들 권한이 있는 조직과 OAuth 앱 접근 제한이 켜진 조직에서 "GitHub에 올리기".
- **찾기**: 저장소가 50개 넘는 계정에서 찾기와 500개 상한 문구.
- **이메일**: 이메일 확인 권한 추가(브라우저에 `user:email`이 보이는지, 워크플로 권한이 유지되는지), 계정에 없는 이메일·확인 안 된 이메일·가림 주소일 때 줄, GitHub "Block command line pushes that expose my email"을 켜고 비공개 이메일로 올려 GH007 문구와 단추.
- **로그인 스위치**: 끈 뒤 Windows(GCM 있음)·Ubuntu(helper 없음·`libsecret` helper)에서 동기화가 사용자 Git 로그인을 쓰는지, 다시 켜면 hiGit 로그인을 쓰는지.
- **웹뷰**: 댓글 칸·이름 입력칸·다른 조직 칸의 한글 IME와 Ctrl+Enter·Enter, 확인 창 포커스·Esc(태그 대화상자 위 확인 창 포함), 800px·다크(WebView2·WebKitGTK).
- **사람에게 묻는 것**(진행은 추천안으로 하고 답이 오면 고친다): ① 기본 브랜치에서도 올린 버전 고치기를 연 것(C9 ③)을 유지할지, ② "포크" 낱말(대안 "내 복사본")이 비개발자에게 통하는지, ③ 합치기 방법 두 가지 이름이 알아보기 쉬운지, ④ 댓글을 최근 20개만 보이는 것으로 충분한지.

## 단계 순서 요약
| 단계 | 내용 | 크기 | 크레이트·화면 | 커밋 메시지 제안 |
| --- | --- | --- | --- | --- |
| W1 | 결정·용어 5개·문서 | 소 | docs, glossary | `docs: add iteration 4c decisions and terms` |
| W2 | PUT·PATCH·405·409·저장소 정보·브랜치 보호 | 중 | hosting | `hosting: add put and patch, repository info and branch protection` |
| W3 | 댓글·합치기·상태·포크 호출 | 중 | hosting | `hosting: add pull request comments, merge, state and fork` |
| W4 | 조직 저장소·권한 묶음·계정 이메일 | 중 | hosting | `hosting: org repositories, scope set and account emails` |
| W5 | GH007 분류 | 소 | git, core | `git+core: classify private email push refusal` |
| W6 | 로그인 스위치·`auth_for` | 소 | core | `core: setting to stop using the hiGit login for git` |
| W7 | 권한 합치기·이메일 비교 조회 | 중 | core, app | `core: scope set for reconnect and github email check` |
| W8 | 설정 스위치·이메일 줄·GH007 단추 | 중 | 화면 | `screens: git login switch, email check line and private email fix` |
| W9 | 저장소 정보·댓글 조회·`pr-comment` | 중 | core, app | `core: pull request repo info, comments and pr-comment action` |
| W10 | `pr-merge`·`pr-close`·`pr-reopen` | 중 | core | `core: add pr-merge, pr-close and pr-reopen actions` |
| W11 | 상세의 댓글·합치기 확인 창·닫기 | 대 | 화면 | `features: comment, merge and close in pull request detail` |
| W12 | `fork`·포크 경로 검토 요청 | 대 | core, app | `core: add fork action and fork route for pull requests` |
| W13 | 포크해서 요청하기 화면 | 중 | 화면 | `features: fork and open a pull request` |
| W14 | `PushDeleteTag`·`remote-tag-delete` | 중 | git, core | `git+core: add remote-tag-delete with lease` |
| W15 | 태그 대화상자 "원격에서 지우기" | 소 | 화면 | `features: delete a tag on the remote from the tag dialog` |
| W16 | 기본 브랜치 이름 입력·가드 정리·보호 조회 | 대 | core, app | `core: allow default branch force-push and delete with name confirmation` |
| W17 | 이름 입력칸·보호 안내·"GitHub에서 보기" | 중 | 화면 | `screens: type the branch name to overwrite or delete the default branch` |
| W18 | 조직으로 올리기 | 소 | core | `core: publish to an organization` |
| W19 | "만들 곳" 고르기 | 소 | 화면 | `features: choose where to create the repository when publishing` |
| W20 | 복제 목록 찾기 | 소 | 화면 | `features: search the github repository list when cloning` |
| W21 | 문서·점검·사람이 할 일 | 소 | docs | `docs: wrap up iteration 4c` |

순서의 까닭: W1(결정·용어)이 모든 단계 앞. hosting(W2~W4)이 core 단계의 바탕이고 서로 독립이다. 설정·이메일(W5~W8)은 가장 작고 다른 묶음과 독립이라 먼저 끝내 사람이 일찍 손 확인할 수 있다. 검토 요청 쓰기(W9~W11)가 포크(W12·W13)보다 먼저다: 포크 경로가 목록·합치기·댓글의 기준 저장소를 바꾸므로 같은 저장소 흐름을 먼저 시험으로 굳힌다. 원격 태그(W14·W15)는 010의 원격 ref 쓰기 틀만 쓰므로 독립이다. 기본 브랜치(W16·W17)는 010-H8~H13 코드를 고치므로 그 위에서 마지막 큰 단계로 둔다. 조직·찾기(W18~W20)는 작고 독립이라 끝에 둔다. 각 단계는 CI가 초록이어야 다음으로 간다. 시간이 모자라면 뒤에서부터(W20 → W19·W18 → W17·W16)를 이후로 넘기고 W21에 적는다.

## 검토 반영 (code-analyst, 구현 전)
code-analyst 검토 결과를 이 절에 더한다. 구현은 이 절을 위 단계 설명보다 우선한다. W1이 결정을 올릴 때도 이 절로 고친 값을 쓴다.

## 사용자 답 (2026-10-16, 위 단계 설명보다 우선)
W1은 아래를 사용자 결정으로 올린다("(자동 결정)"을 붙이지 않는다). 나머지 결정 후보는 W1이 맨 아래 다음 빈 번호부터 "(자동 결정)"으로 올린다. 이 계획은 code-analyst 검토 없이 구현하고, 지금 코드와 다른 곳은 그 자리에서 맞추며 커밋 본문에 한 줄 적는다.
- **원격 기본 브랜치(C9 ③ 바뀜)**: 원격 기본 브랜치에서는 올린 버전 고치기를 계속 막는다(D-127 그대로). 덮어쓰며 올리기와 원격 브랜치 지우기만 이름 입력 확인으로 연다.
- **포크**: 낱말 "포크"를 그대로 쓰고 `<Term>` 툴팁으로 "내 GitHub 계정에 복사본 만들기"라고 설명한다.
- **합치기 방법 두 가지와 댓글 최근 20개**: 계획대로.
- 순서: 4b → 4c(이 계획) → UX 다듬기(012) → 반복 5(014).
