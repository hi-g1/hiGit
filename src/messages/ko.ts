// 화면 문구(한국어, 존댓말, 짧게). 화면 요소의 Git 용어는 <Term>으로 쓴다.
// 문장 안의 용어는 호버 없이 쉬운 이름만 쓰되, 글자를 직접 쓰지 않고 glossary에서 꺼낸다(D-029).
// 그래서 용어표가 바뀌면 이 문장들도 함께 바뀐다.
import { ko as glossary } from "../glossary/ko";
import type { TermId } from "../glossary/types";
import type { LastVersion, ShortCase, StatusCase } from "../features/start/summary";
import type { ErrorCode, ChangeKind } from "../lib/api";
import { josa } from "./josa";

/** 문장에 끼울 쉬운 이름 */
const t = (id: TermId) => glossary[id].friendly;

export const ko = {
  appTagline: "Git을 모르는 사람도 쉽게 쓰는 Git 앱",
  openFolder: "폴더 열기",
  otherProjects: "다른 프로젝트 열기",
  // 시작 화면(D-031)
  gitUpdate: "Git을 업데이트해 주세요. 보안 수정이 빠진 버전이에요.",
  gitDownload: "내려받기",
  startTitle: "프로젝트",
  lastPlace: (when: string) => `마지막으로 하던 곳 · ${when}`,
  continueWork: "이어서 하기",
  viewChangedFiles: (n: number) => `바뀐 파일 ${n}개 보기`,
  otherProjectsHeader: "다른 프로젝트",
  removeFromList: "목록에서 빼기",
  folderMissing: "폴더를 찾을 수 없어요",
  folderNotRepo: "버전 관리를 하지 않는 폴더예요",
  noProjects: "아직 연 프로젝트가 없어요. 오른쪽 위의 폴더 열기로 시작해요.",
  foundGit: (version: string) => `Git을 찾았어요 · 버전 ${version}`,
  statusSentence: (c: StatusCase) => {
    switch (c.kind) {
      case "conflict":
        return `${t("conflict")} 난 파일이 ${c.files}개 있어요.`;
      case "changed":
        return `${c.branch} ${t("branch")}에서 파일 ${c.files}개를 고치다 멈췄어요.`;
      case "clean":
        return `${c.branch} ${t("branch")}에 있어요. 바뀐 파일은 없어요.`;
      case "detached":
        return "특정 버전을 보는 중이에요.";
    }
  },
  lastVersionSentence: (v: LastVersion, date: string) =>
    v.kind === "none"
      ? "아직 저장한 버전이 없어요."
      : `마지막 버전은 ${date}에 저장한 "${v.summary}"${josa(v.summary, "이에요/예요").slice(v.summary.length)}.`,
  shortStatus: (c: ShortCase) => {
    switch (c.kind) {
      case "conflict":
        return `${t("conflict")} 난 파일 ${c.files}개`;
      case "changed":
        return `바뀐 파일 ${c.files}개`;
      case "ahead":
        return `올릴 버전 ${c.versions}개`;
      case "behind":
        return `가져올 버전 ${c.versions}개`;
      case "notUploaded":
        return "아직 올린 적 없음";
      case "upToDate":
        return "최신 상태";
    }
  },
  projectMenu: "프로젝트 메뉴",
  lastMissing: (name: string) => `마지막으로 연 ${name} 폴더를 찾지 못했어요.`,
  lastNotRepo: (name: string) => `마지막으로 연 ${name} 폴더는 버전 관리를 하지 않는 폴더예요.`,
  reload: "다시 읽기",
  detached: "특정 버전을 보는 중",
  notUploaded: "아직 올린 적 없음",
  workspaceTabs: "작업 공간 보기",
  compareArea: "비교 보기",
  pickFileToCompare: "파일을 고르면 비교가 보여요.",
  allFiles: "모든 파일",
  diffSummary: ({ added, removed }: { added: number; removed: number }) =>
    `마지막 버전과 지금 파일을 비교해요 · ${added}줄 추가, ${removed}줄 삭제`,
  binaryFile: "이진 파일이라 내용을 비교할 수 없어요.",
  noDiff: "바뀐 줄이 없어요.",
  diffTruncated: "파일이 커서 앞부분만 보여요.",
  noNewlineAtEnd: "파일 끝 줄바꿈 없음",
  commitPlaceholder: "무엇을 바꿨나요?",
  commitNeedsFiles: "넣은 파일이 없어요. 파일 앞 체크 상자를 눌러 주세요.",
  commitNeedsMessage: "무엇을 바꿨는지 적어 주세요.",
  committed: (shortOid: string) => `버전을 저장했어요 · ${shortOid}`,
  loading: "불러오는 중…",
  showDetail: "자세히 보기",
  notARepo: "이 폴더는 아직 Git으로 관리하지 않는 폴더예요.",
  noChanges: "마지막 버전 이후 바뀐 파일이 없어요.",
  noCommits: "아직 저장한 버전이 없어요.",
  moreCommits: (limit: number) => `최근 ${limit}개만 보여요.`,
  renamedFrom: (path: string) => `${path}에서 이름 바꿈`,
  errors: {
    gitMissing: "Git이 설치되어 있지 않아요. Git을 설치한 뒤 다시 실행해 주세요.",
    gitTooOld: "Git 2.30 이상이 필요해요. Git을 업데이트해 주세요.",
    folderNotFound: "폴더를 열 수 없어요.",
    notARepo: "이 폴더는 아직 Git으로 관리하지 않는 폴더예요.",
    alreadyRepo: "이미 Git으로 관리하는 폴더예요.",
    unknownAction: "알 수 없는 작업이에요.",
    previewRequired: "먼저 미리보기를 확인해 주세요.",
    previewMismatch: "미리보기가 만료됐어요. 다시 확인해 주세요.",
    notYetSupported: "아직 지원하지 않는 작업이에요.",
    invalidInput: "입력한 내용을 확인해 주세요.",
    busy: "다른 작업이 끝날 때까지 기다려 주세요.",
    notOpened: "열지 않은 프로젝트예요. 프로젝트를 다시 열어 주세요.",
    watchUnavailable: "파일 변경을 자동으로 알아채지 못해요. 창으로 돌아오면 다시 읽어요.",
    nothingToCommit: "넣은 파일이 없어요. 파일 앞 체크 상자를 눌러 주세요.",
    identityMissing: "버전에 남길 이름과 이메일이 설정되어 있지 않아요.",
    invalidBranchName: `${t("branch")} 이름으로 쓸 수 없는 글자가 있어요.`,
    branchExists: `같은 이름의 ${josa(t("branch"), "이/가")} 이미 있어요.`,
    wouldOverwrite: `버전으로 저장하지 않은 변경이 덮어써져요. 먼저 ${t("commit")}부터 해 주세요.`,
    notFastForward: `내 버전과 ${t("remote")}의 버전이 갈라져 있어요. ${josa(t("merge"), "은/는")} 아직 준비 중이에요.`,
    pushRejected: `${t("remote")}에 새 버전이 있어요. ${t("push")} 전에 ${t("pull")}부터 해 주세요.`,
    noUpstream: `아직 ${t("remote")}에 올린 적이 없는 ${josa(t("branch"), "이에요/예요")}.`,
    sshHostUnverified: "처음 접속하는 서버라 확인이 필요해요. 터미널에서 한 번 접속해 서버를 등록해 주세요.",
    sshAuthFailed: "SSH 인증에 실패했어요. 키가 등록되어 있는지 확인하고, 키에 비밀번호가 있으면 ssh-agent에 올려 주세요.",
    httpAuthFailed: `${t("remote")}에 로그인하지 못했어요.`,
    remoteUnreachable: `${t("remote")}에 연결하지 못했어요. 주소와 인터넷 연결을 확인해 주세요.`,
    gitFailed: "Git 작업이 실패했어요.",
    internal: "예상하지 못한 문제가 생겼어요.",
  } satisfies Record<ErrorCode, string>,
  changeKind: {
    added: "추가",
    modified: "수정",
    deleted: "삭제",
    renamed: "이름 바꿈",
    typeChanged: "종류 바뀜",
    conflicted: t("conflict"),
    untracked: "새 파일",
  } satisfies Record<ChangeKind, string>,
};

export type Messages = typeof ko;
