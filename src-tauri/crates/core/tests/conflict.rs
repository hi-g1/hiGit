//! 충돌 해결 화면 조회를 실제 git으로 시험한다(D-069, D-086).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{ConflictFileKind, ErrorCode, ErrorView, OpenHow, SideState};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "core.quotepath=off"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn commit_all(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
}

const BASE: &str = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n";

/// main과 "기능"이 a.txt 세 곳과 이진 파일 b.bin을 다르게 고친 뒤 main에서 합치기를 시작한다.
/// c.txt는 기능 쪽에서만 고쳐 문제없이 합쳐진다.
fn merging() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
        ("commit.gpgsign", "false"),
    ] {
        git(dir, &["config", k, v]);
    }
    std::fs::write(dir.join("a.txt"), BASE).unwrap();
    std::fs::write(dir.join("b.bin"), b"\0base").unwrap();
    std::fs::write(dir.join("c.txt"), "c\n").unwrap();
    std::fs::write(dir.join("d.txt"), "d\n").unwrap();
    commit_all(dir, "처음");
    git(dir, &["switch", "-q", "-c", "기능"]);
    std::fs::write(
        dir.join("a.txt"),
        BASE.replace("2\n", "기능2\n")
            .replace("6\n", "기능6\n")
            .replace("11\n", "기능11\n"),
    )
    .unwrap();
    std::fs::write(dir.join("b.bin"), b"\0theirs").unwrap();
    std::fs::write(dir.join("c.txt"), "c 기능\n").unwrap();
    std::fs::remove_file(dir.join("d.txt")).unwrap();
    commit_all(dir, "기능 쪽 고침");
    git(dir, &["switch", "-q", "main"]);
    std::fs::write(
        dir.join("a.txt"),
        BASE.replace("2\n", "내2\n")
            .replace("6\n", "내6\n")
            .replace("11\n", "내11\n"),
    )
    .unwrap();
    std::fs::write(dir.join("b.bin"), b"\0ours").unwrap();
    std::fs::write(dir.join("d.txt"), "d 내 고침\n").unwrap();
    commit_all(dir, "내 쪽 고침");
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    repo
}

fn core(dir: &Path) -> Core {
    let core = Core::with_backend(
        Arc::new(git::CliBackend::new("git", Arc::new(|_| {}))) as Arc<dyn GitBackend>
    )
    .unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

#[test]
fn state_lists_conflicted_files_and_both_sides() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(state.active && !state.other_operation);
    let ours = state.ours.unwrap();
    let theirs = state.theirs.unwrap();
    assert_eq!(
        (ours.label.as_str(), ours.summary.as_str()),
        ("main", "내 쪽 고침")
    );
    assert_eq!(
        (theirs.label.as_str(), theirs.summary.as_str()),
        ("기능", "기능 쪽 고침")
    );
    let files: Vec<(String, ConflictFileKind, u32)> = state
        .files
        .iter()
        .map(|f| (f.path.clone(), f.kind, f.hunks))
        .collect();
    assert_eq!(
        files,
        [
            ("a.txt".to_string(), ConflictFileKind::Text, 3),
            ("b.bin".to_string(), ConflictFileKind::Whole, 1),
            ("d.txt".to_string(), ConflictFileKind::Whole, 1),
        ]
    );
    assert_eq!(state.clean_files, 1, "c.txt");
}

#[test]
fn text_file_hunks_carry_no_markers() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let file = core.queries().conflict_file(dir, "a.txt").unwrap();
    assert_eq!(file.kind, ConflictFileKind::Text);
    let lines: Vec<u32> = file.hunks.iter().map(|h| h.line).collect();
    assert_eq!(lines, [2, 6, 11]);
    assert_eq!(file.hunks[0].ours, ["내2"]);
    assert_eq!(file.hunks[0].theirs, ["기능2"]);
    assert_eq!(file.hunks[0].base, ["2"]);
    assert_eq!(file.hunks[0].before, ["1"]);
    assert_eq!(file.hunks[0].after, ["3", "4", "5"]);
    let json = serde_json::to_string(&file).unwrap();
    for marker in ["<<<<<<<", "=======", ">>>>>>>", "|||||||"] {
        assert!(!json.contains(marker), "{marker} in {json}");
    }
}

#[test]
fn binary_and_deleted_sides_are_whole_files() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    let bin = core.queries().conflict_file(dir, "b.bin").unwrap();
    assert_eq!(
        (bin.ours_side, bin.theirs_side),
        (SideState::Binary, SideState::Binary)
    );
    let deleted = core.queries().conflict_file(dir, "d.txt").unwrap();
    assert_eq!(deleted.kind, ConflictFileKind::Whole);
    assert_eq!(deleted.ours_side, SideState::Text);
    assert_eq!(deleted.theirs_side, SideState::Deleted);
    assert_eq!(deleted.ours_preview, ["d 내 고침"]);
    // 충돌이 아닌 파일·이상한 경로는 받지 않는다.
    for bad in ["c.txt", "../x"] {
        let err = core.queries().conflict_file(dir, bad).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput, "{bad}");
    }
}

#[test]
fn not_merging_and_other_operations() {
    let repo = merging();
    let dir = repo.path();
    let core = core(dir);
    git(dir, &["merge", "--abort"]);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(!state.active && !state.other_operation && state.files.is_empty());
    // 다른 도구로 시작한 리베이스가 충돌로 멈췄다.
    git(dir, &["switch", "-q", "기능"]);
    git(dir, &["rebase", "main"]);
    let state = core.queries().conflict_state(dir).unwrap();
    assert!(!state.active);
    assert!(state.other_operation);
}

mod finish {
    use super::*;
    use higit_core::view::{
        ActionInput, ActionResult, ConflictFinishInput, FileChoice, FileResolution, HunkChoice,
    };

    fn finish(
        core: &Core,
        dir: &Path,
        resolutions: Vec<FileResolution>,
    ) -> Result<ActionResult, higit_core::CoreError> {
        core.execute(
            dir,
            &ActionInput::ConflictFinish(ConflictFinishInput {
                resolutions,
                locale: "ko".into(),
            }),
            None,
        )
    }

    fn all_choices(a: Vec<HunkChoice>) -> Vec<FileResolution> {
        vec![
            FileResolution {
                path: "a.txt".into(),
                choice: FileChoice::Hunks { choices: a },
            },
            FileResolution {
                path: "b.bin".into(),
                choice: FileChoice::Theirs,
            },
            FileResolution {
                path: "d.txt".into(),
                choice: FileChoice::Theirs,
            },
        ]
    }

    fn backups(dir: &Path) -> String {
        git(
            dir,
            &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
        )
    }

    #[test]
    fn finishing_writes_choices_and_makes_a_merge_version() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let before = git(dir, &["rev-parse", "HEAD"]);
        let r = finish(
            &core,
            dir,
            all_choices(vec![
                HunkChoice::Ours,
                HunkChoice::Both,
                HunkChoice::Custom {
                    text: "직접 고침".into(),
                },
            ]),
        )
        .unwrap();
        assert_eq!(r, ActionResult::ConflictFinished);
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            // 11·12번째 줄(양쪽 모두 고침)은 한 곳으로 묶여 직접 고친 글로 바뀐다.
            "1\n내2\n3\n4\n5\n내6\n기능6\n7\n8\n9\n10\n직접 고침\n"
        );
        assert_eq!(std::fs::read(dir.join("b.bin")).unwrap(), b"\0theirs");
        assert!(!dir.join("d.txt").exists());
        assert_eq!(git(dir, &["status", "--porcelain"]), "");
        assert!(!dir.join(".git/MERGE_HEAD").exists());
        let parents = git(dir, &["rev-list", "--parents", "-n", "1", "HEAD"]);
        assert_eq!(parents.split_whitespace().count(), 3);
        assert_eq!(git(dir, &["log", "-1", "--format=%s"]), "기능 가져와 합침");
        assert!(
            backups(dir).contains("conflict-files-main"),
            "{}",
            backups(dir)
        );
        // Ctrl+Z로 합치기 전으로.
        core.undo(dir).unwrap();
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), before);
    }

    #[test]
    fn missing_or_mismatched_choices_change_nothing() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let mut partial = all_choices(vec![HunkChoice::Ours; 3]);
        partial.pop();
        let err = finish(&core, dir, partial).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
        let err = finish(&core, dir, all_choices(vec![HunkChoice::Ours; 2])).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::ConflictChanged);
        // 이진 파일을 곳마다 고를 수는 없다.
        let mut wrong = all_choices(vec![HunkChoice::Ours; 3]);
        wrong[1].choice = FileChoice::Hunks { choices: vec![] };
        let err = finish(&core, dir, wrong).unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::ConflictChanged);
        assert!(dir.join(".git/MERGE_HEAD").exists());
        assert_eq!(core.queries().conflict_state(dir).unwrap().files.len(), 3);
        assert_eq!(backups(dir), "", "쓰지 않았으면 백업도 없다");
    }

    #[test]
    fn a_hook_refusing_the_save_leaves_resolved_files_and_finish_again_saves() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let hook = dir.join(".git/hooks/commit-msg");
        std::fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        let _ = std::process::Command::new("chmod")
            .arg("+x")
            .arg(&hook)
            .status();
        let choices = all_choices(vec![HunkChoice::Theirs; 3]);
        assert!(finish(&core, dir, choices.clone()).is_err());
        assert!(dir.join(".git/MERGE_HEAD").exists());
        assert!(core.queries().conflict_state(dir).unwrap().files.is_empty());
        std::fs::remove_file(&hook).unwrap();
        // 이미 해결된 경로는 건너뛰고 저장만 한다.
        finish(&core, dir, choices).unwrap();
        assert!(!dir.join(".git/MERGE_HEAD").exists());
    }

    #[test]
    fn abort_backs_up_outside_edits_and_goes_back() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let before = git(dir, &["rev-parse", "HEAD"]);
        // 외부 편집기로 충돌 파일을 고쳐 두었다.
        std::fs::write(dir.join("a.txt"), "밖에서 고침\n").unwrap();
        let r = core
            .execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap();
        assert_eq!(r, ActionResult::ConflictAborted);
        assert!(!dir.join(".git/MERGE_HEAD").exists());
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), before);
        let backup = backups(dir);
        assert!(backup.contains("conflict-files-main"), "{backup}");
        assert_eq!(
            git(dir, &["show", &format!("{backup}:a.txt")]),
            "밖에서 고침"
        );
        // 합치는 중이 아니면 그만둘 것이 없다.
        let err = core
            .execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
    }
}

mod unpark {
    use super::*;
    use higit_core::view::{
        ActionInput, ActionResult, ConflictContext, ConflictFinishInput, FileChoice,
        FileResolution, HunkChoice, SwitchInput, TakeOutInput, TakeOutSource,
    };

    /// main에서 고친 a.txt를 기능으로 갈아타며 맡긴 뒤, 터미널에서 main의 같은 줄을 다르게 저장했다.
    /// 지금은 main에 있고 맡긴 파일을 꺼내면 겹친다.
    fn parked_overlap() -> (tempfile::TempDir, Core) {
        let repo = tempfile::tempdir().unwrap();
        let dir = repo.path();
        git(dir, &["init", "-q", "-b", "main"]);
        for (k, v) in [
            ("core.autocrlf", "false"),
            ("user.name", "테스트"),
            ("user.email", "t@example.com"),
            ("commit.gpgsign", "false"),
        ] {
            git(dir, &["config", k, v]);
        }
        std::fs::write(dir.join("a.txt"), "1\n2\n3\n").unwrap();
        commit_all(dir, "처음");
        git(dir, &["branch", "기능"]);
        let core = core(dir);
        std::fs::write(dir.join("a.txt"), "1\n내 고침\n3\n").unwrap();
        core.execute(
            dir,
            &ActionInput::Switch(SwitchInput {
                branch: "기능".into(),
                remote: None,
                dirty: None,
            }),
            None,
        )
        .unwrap();
        git(dir, &["switch", "-q", "main"]);
        std::fs::write(dir.join("a.txt"), "1\n다른 고침\n3\n").unwrap();
        commit_all(dir, "다른 고침");
        (repo, core)
    }

    fn take_out(core: &Core, dir: &Path) -> ActionResult {
        core.execute(
            dir,
            &ActionInput::TakeOut(TakeOutInput {
                source: TakeOutSource::Parked {
                    branch: "main".into(),
                },
            }),
            None,
        )
        .unwrap()
    }

    fn parked_ref(dir: &Path) -> String {
        git(
            dir,
            &["for-each-ref", "--format=%(refname)", "refs/higit/parked/"],
        )
    }

    #[test]
    fn resolving_parked_files_leaves_them_as_edits() {
        let (repo, core) = parked_overlap();
        let dir = repo.path();
        let head = git(dir, &["rev-parse", "HEAD"]);
        assert_eq!(
            take_out(&core, dir),
            ActionResult::MergeConflicts {
                paths: vec!["a.txt".into()]
            }
        );
        let state = core.queries().conflict_state(dir).unwrap();
        assert!(state.active && !state.other_operation);
        assert_eq!(state.context, ConflictContext::Unpark);
        assert_eq!(state.files.len(), 1);
        let r = core
            .execute(
                dir,
                &ActionInput::ConflictFinish(ConflictFinishInput {
                    resolutions: vec![FileResolution {
                        path: "a.txt".into(),
                        choice: FileChoice::Hunks {
                            choices: vec![HunkChoice::Theirs],
                        },
                    }],
                    locale: "ko".into(),
                }),
                None,
            )
            .unwrap();
        assert_eq!(r, ActionResult::ParkedResolved);
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "1\n내 고침\n3\n"
        );
        assert_eq!(git(dir, &["status", "--porcelain"]), "M a.txt");
        assert_eq!(
            git(dir, &["rev-parse", "HEAD"]),
            head,
            "버전은 만들지 않는다"
        );
        assert_eq!(parked_ref(dir), "");
        let backups = git(
            dir,
            &["for-each-ref", "--format=%(refname)", "refs/higit/backup/"],
        );
        assert!(backups.contains("parked-main"), "{backups}");
        assert!(!dir.join(".git/higit/unpark.json").exists());
        assert!(!core.queries().conflict_state(dir).unwrap().active);
    }

    #[test]
    fn stopping_goes_back_and_keeps_the_parked_files() {
        let (repo, core) = parked_overlap();
        let dir = repo.path();
        take_out(&core, dir);
        let r = core
            .execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap();
        assert_eq!(r, ActionResult::ConflictAborted);
        assert_eq!(git(dir, &["status", "--porcelain"]), "");
        assert_eq!(
            std::fs::read_to_string(dir.join("a.txt")).unwrap(),
            "1\n다른 고침\n3\n"
        );
        assert_eq!(parked_ref(dir), "refs/higit/parked/main");
        assert!(!dir.join(".git/higit/unpark.json").exists());
    }

    #[test]
    fn a_stash_pop_in_a_terminal_is_not_the_apps_unpark() {
        let (repo, core) = parked_overlap();
        let dir = repo.path();
        // 터미널에서 맡긴 stash를 직접 적용해 충돌을 만들었다(표식 없음).
        let oid = git(dir, &["rev-parse", "refs/higit/parked/main"]);
        let _ = std::process::Command::new("git")
            .args(["stash", "apply", &oid])
            .current_dir(dir)
            .output()
            .unwrap();
        let state = core.queries().conflict_state(dir).unwrap();
        assert!(!state.active);
        assert!(state.other_operation);
        let err = core
            .execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap_err();
        assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
    }
}

/// 충돌 해결 화면에서 고른 것을 저장소 안에 서명해 둔다(D-211).
mod draft {
    use super::*;
    use higit_core::view::{ActionInput, ConflictDraftView, DraftFile, DraftPick, HunkChoice};

    fn picks() -> ConflictDraftView {
        ConflictDraftView {
            files: vec![
                DraftFile {
                    path: "a.txt".into(),
                    pick: DraftPick::Hunks {
                        picks: vec![
                            Some(HunkChoice::Ours),
                            None,
                            Some(HunkChoice::Custom {
                                text: "직접 고친 줄\n".into(),
                            }),
                        ],
                    },
                },
                DraftFile {
                    path: "b.bin".into(),
                    pick: DraftPick::Theirs,
                },
            ],
        }
    }

    fn draft_file(dir: &Path) -> std::path::PathBuf {
        dir.join(".git/higit/conflict-draft.json")
    }

    #[test]
    fn saved_picks_come_back_with_the_state() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        assert_eq!(core.queries().conflict_state(dir).unwrap().draft, None);
        core.save_conflict_draft(dir, &picks()).unwrap();
        assert!(draft_file(dir).exists());
        assert_eq!(
            core.queries().conflict_state(dir).unwrap().draft,
            Some(picks())
        );
        // 다른 키(받은 zip, 다른 컴퓨터)로 서명한 파일은 쓰지 않는다.
        let other = self::core(dir);
        assert_eq!(other.queries().conflict_state(dir).unwrap().draft, None);
    }

    #[test]
    fn picks_that_do_not_fit_are_refused() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let refused = |draft: ConflictDraftView| {
            let err = core.save_conflict_draft(dir, &draft).unwrap_err();
            assert_eq!(ErrorView::from(&err).code, ErrorCode::InvalidInput);
        };
        // 충돌 경로에 없는 경로
        refused(ConflictDraftView {
            files: vec![DraftFile {
                path: "c.txt".into(),
                pick: DraftPick::Ours,
            }],
        });
        // 곳 번호 넘침
        refused(ConflictDraftView {
            files: vec![DraftFile {
                path: "a.txt".into(),
                pick: DraftPick::Hunks {
                    picks: vec![Some(HunkChoice::Ours); 4],
                },
            }],
        });
        // 이진 파일을 곳마다
        refused(ConflictDraftView {
            files: vec![DraftFile {
                path: "b.bin".into(),
                pick: DraftPick::Hunks { picks: vec![None] },
            }],
        });
        // 글 하나 1MB 넘김, 같은 경로 둘
        refused(ConflictDraftView {
            files: vec![DraftFile {
                path: "a.txt".into(),
                pick: DraftPick::Hunks {
                    picks: vec![Some(HunkChoice::Custom {
                        text: "x".repeat(1024 * 1024 + 1),
                    })],
                },
            }],
        });
        refused(ConflictDraftView {
            files: vec![
                DraftFile {
                    path: "b.bin".into(),
                    pick: DraftPick::Ours,
                },
                DraftFile {
                    path: "b.bin".into(),
                    pick: DraftPick::Theirs,
                },
            ],
        });
        assert!(!draft_file(dir).exists());
    }

    #[test]
    fn finishing_or_aborting_or_another_merge_removes_the_picks() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        core.save_conflict_draft(dir, &picks()).unwrap();
        core.execute(dir, &ActionInput::ConflictAbort, None)
            .unwrap();
        assert!(!draft_file(dir).exists());
        assert!(
            core.save_conflict_draft(dir, &picks()).is_err(),
            "합치는 중이 아니면 거절"
        );

        // 같은 브랜치가 다른 버전이 되어 다시 합치면 앞의 선택은 맞지 않아 지운다.
        git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
        core.save_conflict_draft(dir, &picks()).unwrap();
        git(dir, &["merge", "--abort"]);
        git(dir, &["switch", "-q", "기능"]);
        std::fs::write(dir.join("e.txt"), "더\n").unwrap();
        commit_all(dir, "기능 더");
        git(dir, &["switch", "-q", "main"]);
        git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
        assert_eq!(core.queries().conflict_state(dir).unwrap().draft, None);
        assert!(!draft_file(dir).exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_linked_higit_folder_is_not_written() {
        let repo = merging();
        let dir = repo.path();
        let core = core(dir);
        let outside = tempfile::tempdir().unwrap();
        let _ = std::fs::remove_dir_all(dir.join(".git/higit"));
        std::os::unix::fs::symlink(outside.path(), dir.join(".git/higit")).unwrap();
        assert!(core.save_conflict_draft(dir, &picks()).is_err());
        assert!(!outside.path().join("conflict-draft.json").exists());
    }
}
