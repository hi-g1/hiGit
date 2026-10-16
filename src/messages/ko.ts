// 화면 문구(한국어, 존댓말, 짧게). 화면 요소의 Git 용어는 <Term>으로 쓴다.
// 문장 안의 용어는 호버 없이 쉬운 이름만 쓰되, 글자를 직접 쓰지 않고 glossary에서 꺼낸다(D-029).
// 그래서 용어표가 바뀌면 이 문장들도 함께 바뀐다.
import { ko as glossary } from "../glossary/ko";
import type { TermId } from "../glossary/types";
import type { LastVersion, ShortCase, StatusCase } from "../features/start/summary";
import type { ErrorCode, ChangeKind, RecoverySource, SecretRule } from "../lib/api";
import { josa } from "./josa";

/** 버전 기록 노선도 행의 읽기 이름에 들어가는 것. 저장소 글자는 부르는 쪽이 자른다. */
export type RouteRowParts = {
  summary: string;
  author: string;
  date: string;
  branches: string[];
  tags: string[];
  head: boolean;
  merge: boolean;
  unpushed: boolean;
  position: number;
};

/** 문장에 끼울 쉬운 이름 */
const t = (id: TermId) => glossary[id].friendly;

export const ko = {
  appTagline: "Git을 모르는 사람도 쉽게 쓰는 Git 앱",
  openFolder: "폴더 열기",
  otherProjects: "다른 프로젝트 열기",
  // 시작 화면(D-031)
  gitUpdate: "Git을 업데이트해 주세요. 보안 수정이 빠진 버전이에요.",
  gitDownload: "내려받기",
  undo: "되돌리기",
  redo: "다시 하기",
  undone: "되돌렸어요",
  redone: "다시 했어요",
  reverted: "되돌리는 새 버전을 만들었어요",
  // 충돌 해결 화면(D-028, D-086)
  conflictsLeft: (total: number, left: number) => `${t("conflict")} ${total}곳 중 ${left}곳 남음`,
  conflictFiles: "겹친 파일",
  cleanMerged: (n: number) => `문제없이 합쳐진 ${n}개`,
  atLine: (n: number) => `${n}번째 줄`,
  conflictQuestion: `두 ${josa(t("branch"), "이/가")} 같은 줄을 다르게 고쳤어요. 어느 쪽을 남길까요?`,
  wholeQuestion: "어느 쪽 파일을 남길까요?",
  useThis: "이걸로",
  useBoth: "둘 다 쓰기",
  editByHand: "직접 고치기",
  showBase: `${t("base")} 보기`,
  savedLooks: "저장될 모습",
  prevPlace: "이전 곳",
  nextPlace: "다음 곳",
  finishMerge: `${t("merge")} 마치기`,
  abortMerge: `${t("merge")} 그만두기`,
  allChosen: "모두 골랐어요",
  sideDeleted: "이 쪽에서는 지웠어요",
  sideBinary: "이진 파일",
  sideLarge: "1MB가 넘는 파일",
  sideSubmodule: "하위 저장소",
  sideLink: (target: string) => `링크 → ${target}`,
  merged: "합쳤어요",
  // 작업 내역(D-085)
  activityArea: t("reflog"),
  activityNote: "되살려도 지금 버전은 남아요",
  activityEmpty: "아직 기록이 없어요",
  activitySource: {
    journal: "앱 작업",
    reflog: "Git 기록",
    backup: "백업",
    parked: "맡겨 둔 파일",
    stash: t("stash"),
  } satisfies Record<RecoverySource, string>,
  restoreHere: "이 시점으로 되돌리기",
  stashed: "보관했어요",
  stashCount: (n: number) => `${t("stash")} ${n}개`,
  filesCount: (n: number) => `파일 ${n}개`,
  takeOutNeedsClean: "고친 파일이 없을 때 꺼낼 수 있어요",
  discardTitle: (name: string) => `${name}의 고친 내용을 버려요.`,
  discardConfirm: "버리기",
  discarded: "버렸어요",
  takeOut: "꺼내기",
  restored: "되돌렸어요",
  tookOutToast: "꺼냈어요",
  openLogs: "로그 폴더 열기",
  openActivity: t("reflog"),
  // 꺼내다 겹친 맡긴 파일(D-083)
  resolveOverlap: "겹침 해결",
  parkedSide: "맡겨 둔 파일",
  finishUnpark: "꺼내기 마치기",
  abortUnpark: "꺼내기 그만두기",
  pickSide: "복사할 버전",
  revertSide: "되돌린 모습",
  movingSide: "옮기는 버전",
  rewrittenSide: "앞서 정리한 기록",
  finishPick: "복사 마치기",
  abortPick: "복사 그만두기",
  finishRevert: `${t("revert")} 마치기`,
  abortRevert: `${t("revert")} 그만두기`,
  finishRewrite: `${t("rebase-i")} 마치기`,
  abortRewrite: `${t("rebase-i")} 그만두기`,
  picked: "버전을 복사했어요",
  rewritten: `${josa(t("rebase-i"), "을/를")} 마쳤어요`,
  pickAborted: "복사를 그만뒀어요",
  revertAborted: `${josa(t("revert"), "을/를")} 그만뒀어요`,
  rewriteAborted: `${josa(t("rebase-i"), "을/를")} 그만뒀어요`,
  parkedResolved: "맡겨 둔 파일을 꺼냈어요",
  mainNewVersions: (branch: string, n: number) => `${branch}에 새 버전 ${n}개`,
  bringMerge: (branch: string) => `${branch} 가져와 ${t("merge")}`,
  mergeIntoBranch: (branch: string) => `${branch}에 ${t("merge")}`,
  mergeAborted: `${josa(t("merge"), "을/를")} 그만뒀어요`,
  otherOperation: "다른 도구에서 시작한 작업이 멈춰 있어요.",
  // 비밀 파일 알림 줄(D-077)
  secretTitle: (n: number) => `키나 비밀번호로 보이는 파일 ${n}개`,
  secretRule: {
    envFile: "환경 설정 파일",
    sshKey: "SSH 키",
    keyFile: "키 파일",
    credentials: "인증 정보 파일",
    privateKey: "개인 키",
    githubToken: "GitHub 토큰",
    awsKey: "AWS 키",
    slackToken: "Slack 토큰",
    apiKey: "API 키",
    largeFile: "50MB 넘는 파일",
  } satisfies Record<SecretRule, string>,
  moreFiles: (n: number) => `외 ${n}개`,
  addToIgnore: `${t("ignore")}에 추가`,
  saveAnyway: "그래도 저장",
  ignored: (n: number) => `${t("ignore")}에 넣었어요 · ${n}개`,
  saved: "저장했어요",
  suggest: {
    one: (kind: "update" | "add" | "delete" | "rename", name: string) =>
      ({ update: `${name} 고침`, add: `${name} 추가`, delete: `${name} 삭제`, rename: `${name} 이름 바꿈` })[kind],
    many: (kind: "update" | "add" | "delete" | "rename", name: string, more: number) =>
      `${name} 외 ${more}개 ${({ update: "고침", add: "추가", delete: "삭제", rename: "고침" })[kind]}`,
  },
  versionName: "버전 이름",
  descriptionPlaceholder: "설명 추가",
  noChangedFiles: "고친 파일 없음",
  branchSearch: "찾기 또는 새로 만들기",
  currentSection: "지금",
  recentSection: `최근에 쓴 ${t("branch")}`,
  moreBranches: (n: number) => `${t("branch")} ${n}개 더`,
  createBranch: (name: string) => `+ "${name}" 새로 만들기`,
  badBranchName: "쓸 수 없는 이름",
  sameBranchName: "같은 이름 있음",
  parkedFiles: (n: number) => `맡겨 둔 파일 ${n}개`,
  parkedBlocked: (n: number) => `맡겨 둔 파일 ${n}개 · 겹침`,
  parkedOnly: "맡겨 둔 파일만 남음",
  newVersions: (n: number) => `새 버전 ${n}개`,
  // 브랜치 이름이 영어일 수 있어 조사는 용어("브랜치") 뒤에 붙인다.
  switchedTo: (branch: string) => `${branch} ${josa(t("branch"), "으로/로")} 갈아탔어요`,
  createdBranch: (branch: string) => `${branch} ${josa(t("branch"), "을/를")} 만들었어요`,
  trustTitle: "이 프로젝트에는 열 때 실행되는 설정이 있어요.",
  trustOpen: "믿고 열기",
  trustDisable: "끄고 열기",
  trustNeeded: "확인 필요",
  ownerTitle: "이 폴더의 주인이 다른 사용자예요.",
  ownerAllow: "이 폴더 믿기",
  close: "닫기",
  changedFilesCount: (n: number) => `고친 파일 ${n}개`,
  unpushed: (n: number) => `안 올린 버전 ${n}개`,
  historyArea: t("log"),
  synced: (sent: number, received: number) => `${t("sync")}했어요 · ↑${sent} ↓${received}`,
  overlapFiles: (n: number) => `겹친 파일 ${n}개`,
  fetchFailed: (when: string) => `${t("fetch")} 못 함 · ${when}`,
  /** 동기화 버튼 이름 뒤에 붙는다(D-080). 첫 진행 알림 전에는 초가 없다. */
  syncRunning: (secs: number | null) => (secs === null ? "중" : `중 · ${secs}초`),
  cancel: "취소",
  noResponse: (secs: number) => `응답 없음 · ${secs}초`,
  forceStop: "강제로 멈추기",
  forceStopTitle: "강제로 멈출까요?",
  forceStopDetail: "하던 작업이 반쯤 남을 수 있어요.",
  routePreviewLabel: (branch: string, n: number) => `${t("log")} · ${branch} · 버전 ${n}개`,
  /** 버전 기록 노선도(D-087~D-092). */
  scopeAll: `모든 ${t("branch")}`,
  scopeCurrent: `지금 ${t("branch")}만`,
  zoomNames: { all: "모든 버전", key: "주요 버전", tags: `${t("tag")}만` },
  zoomIn: "늘리기",
  zoomOut: "줄이기",
  zoomStatus: (zoom: string, n: number) => `${zoom} · ${n}개`,
  /** `<Term id="branch" />` 뒤에 붙는다. */
  tipsCut: (n: number) => `200개까지 · ${n}개 더`,
  notPushedMark: "안 올림",
  stationMenu: "버전 메뉴",
  copyVersionId: "버전 번호 복사",
  copied: "복사했어요",
  copyFailed: "복사하지 못했어요",
  moreItems: "더 보기",
  routeFold: (n: number) => `버전 ${n}개 · 펼치기`,
  routeFoldLabel: (n: number) => `접힌 버전 ${n}개`,
  routeRowLabel: (r: RouteRowParts) =>
    [
      r.summary,
      r.author,
      r.date,
      ...r.branches.map((b) => `${t("branch")} ${b}`),
      ...r.tags.map((x) => `${t("tag")} ${x}`),
      r.head ? "지금 여기" : null,
      r.merge ? t("merge") : null,
      r.unpushed ? "안 올림" : null,
      `${r.position}번째`,
    ]
      .filter(Boolean)
      .join(", "),
  /** 끝나지 않은 작업 알림 줄(D-081, D-082). */
  staleLocks: (n: number) => `멈춘 작업 파일 ${n}개`,
  clearLocks: "정리하기",
  clearLocksTitle: (n: number) => `멈춘 작업 파일 ${n}개를 지울까요?`,
  locksCleared: (n: number) => `멈춘 작업 파일 ${n}개를 지웠어요`,
  leftoverParked: (n: number) => `남은 맡겨 둔 파일 ${n}건`,
  recoverParked: "복구하기",
  parkedRecovered: (n: number) => `맡겨 둔 파일 ${n}건을 복구했어요`,
  byOtherTool: "다른 도구에서 시작함",
  hereNow: "지금 여기",
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
  diffAdded: (n: number) => `${n}줄 추가`,
  diffRemoved: (n: number) => `${n}줄 삭제`,
  linkTarget: (target: string) => `링크: ${target}`,
  binaryFile: "이진 파일이라 내용을 비교할 수 없어요.",
  noDiff: "바뀐 줄이 없어요.",
  diffTruncated: "파일이 커서 앞부분만 보여요.",
  noNewlineAtEnd: "파일 끝 줄바꿈 없음",
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
    invalidInput: "입력한 내용을 확인해 주세요.",
    busy: "다른 작업이 끝날 때까지 기다려 주세요.",
    notOpened: "열지 않은 프로젝트예요. 프로젝트를 다시 열어 주세요.",
    nothingToUndo: "되돌릴 작업이 없어요.",
    undoStale: "앱 밖에서 바뀌어서 되돌리지 않았어요.",
    undoPushed: "올린 것은 되돌릴 수 없어요.",
    backupFailed: `버렸지만 백업을 남기지 못했어요. 버린 내용은 ${t("stash")}에 있어요.`,
    conflictChanged: `${t("conflict")} 파일이 바뀌었어요. 다시 골라 주세요.`,
    stagedChanges: `이번 버전에 넣은 파일이 있어요. 먼저 ${josa(t("commit"), "을/를")} 하거나 파일을 빼 주세요.`,
    operationInProgress: `${josa(t("merge"), "을/를")} 마치거나 그만둔 뒤에 할 수 있어요.`,
    parkBlocked: "맡긴 파일을 원래대로 꺼내지 못했어요. 파일은 안전하게 남아 있어요.",
    needsTrust: "믿고 열지 정해야 해요.",
    dubiousOwnership: "폴더 주인이 다른 사용자예요.",
    noRemote: `연결된 ${josa(t("remote"), "이/가")} 없어요.`,
    watchUnavailable: "파일 변경을 자동으로 알아채지 못해요. 창으로 돌아오면 다시 읽어요.",
    publishedHistory: `이미 올린 버전이 있어서 기록을 바꿀 수 없어요. ${josa(t("revert"), "을/를")} 써 보세요.`,
    hasChanges: `고친 파일이 있어요. 먼저 ${josa(t("commit"), "을/를")} 하거나 ${josa(t("stash"), "을/를")} 해 주세요.`,
    tagExists: `같은 이름의 ${josa(t("tag"), "이/가")} 이미 있어요.`,
    branchProtected: `지금 ${josa(t("branch"), "과/와")} 기본 ${josa(t("branch"), "은/는")} 지울 수 없어요.`,
    noChange: "이미 같은 내용이에요.",
    annotatedTag: `설명이 달린 ${josa(t("tag"), "은/는")} 앱에서 지우지 않아요.`,
    nothingToCommit: "넣은 파일이 없어요. 파일 앞 체크 상자를 눌러 주세요.",
    identityMissing: "버전에 남길 이름과 이메일이 설정되어 있지 않아요.",
    invalidBranchName: `${t("branch")} 이름으로 쓸 수 없는 글자가 있어요.`,
    branchExists: `같은 이름의 ${josa(t("branch"), "이/가")} 이미 있어요.`,
    wouldOverwrite: `버전으로 저장하지 않은 변경이 덮어써져요. 먼저 ${t("commit")}부터 해 주세요.`,
    notFastForward: `두 쪽 버전이 갈라져 있어요. 먼저 ${josa(t("merge"), "을/를")} 해 주세요.`,
    pushRejected: `${t("remote")}에 새 버전이 있어요. ${t("push")} 전에 ${t("pull")}부터 해 주세요.`,
    noUpstream: `아직 ${t("remote")}에 올린 적이 없는 ${josa(t("branch"), "이에요/예요")}.`,
    sshHostUnverified: "처음 접속하는 서버라 확인이 필요해요. 터미널에서 한 번 접속해 서버를 등록해 주세요.",
    sshAuthFailed: "SSH 인증에 실패했어요. 키가 등록되어 있는지 확인하고, 키에 비밀번호가 있으면 ssh-agent에 올려 주세요.",
    httpAuthFailed: `${t("remote")}에 로그인하지 못했어요.`,
    remoteUnreachable: `${t("remote")}에 연결하지 못했어요. 주소와 인터넷 연결을 확인해 주세요.`,
    protocolNotAllowed: "이 주소 방식은 쓸 수 없어요. https나 ssh 주소를 써 주세요.",
    unparkConflict: "맡겨 둔 파일이 지금 버전과 겹쳐서 꺼내지 않았어요.",
    timedOut: `${josa(t("remote"), "이/가")} 응답하지 않아 그만뒀어요.`,
    mergeConflict: "같은 곳을 서로 다르게 고쳐서 합치지 못했어요.",
    cancelled: "취소했어요.",
    stalled: "진행이 멈춰 그만뒀어요.",
    indexLocked: "다른 Git 작업이 진행 중이에요.",
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
