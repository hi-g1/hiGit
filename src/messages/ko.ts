// 화면 문구(한국어, 존댓말, 짧게). Git 용어 자체는 여기 두지 않고 <Term>으로 쓴다.
import type { ErrorCode, ChangeKind } from "../lib/api";

export const ko = {
  appTagline: "Git을 모르는 사람도 쉽게 쓰는 Git 앱",
  openFolder: "폴더 열기",
  openOtherFolder: "다른 폴더 열기",
  refresh: "새로 고침",
  loading: "불러오는 중…",
  showDetail: "자세히 보기",
  notARepo: "이 폴더는 아직 Git으로 관리하지 않는 폴더예요.",
  noChanges: "마지막 저장 이후 바뀐 파일이 없어요.",
  noCommits: "아직 저장한 기록이 없어요.",
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
    gitFailed: "Git 작업이 실패했어요.",
    internal: "예상하지 못한 문제가 생겼어요.",
  } satisfies Record<ErrorCode, string>,
  changeKind: {
    added: "추가",
    modified: "수정",
    deleted: "삭제",
    renamed: "이름 바꿈",
    typeChanged: "종류 바뀜",
    conflicted: "겹침",
    untracked: "새 파일",
  } satisfies Record<ChangeKind, string>,
};

export type Messages = typeof ko;
