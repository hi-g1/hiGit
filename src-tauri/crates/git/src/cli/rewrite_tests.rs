//! 기록 정리(`rebase -i`, D-100). 실제 git으로.

use std::fs;
use std::path::Path;

use super::rewrite::{MAX_STEPS, todo};
use crate::testutil::{self, git};
use crate::{GitBackend, GitOp, Oid, OperationKind, RepoPath, RewriteKind, RewriteStep};

fn commit(p: &Path, file: &str, text: &str) -> Oid {
    fs::write(p.join(file), text).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", text]);
    oid(p, "HEAD")
}

fn oid(p: &Path, rev: &str) -> Oid {
    Oid::parse(git(p, &["rev-parse", rev]).trim()).unwrap()
}

fn run(p: &Path, op: &GitOp) -> crate::Result<crate::OpOutput> {
    testutil::backend().run(&RepoPath::new(p), op)
}

fn step(oid: &Oid, kind: RewriteKind) -> RewriteStep {
    RewriteStep {
        oid: oid.clone(),
        kind,
    }
}

fn rewrite(p: &Path, base: &Oid, steps: Vec<RewriteStep>) -> crate::Result<crate::OpOutput> {
    run(
        p,
        &GitOp::RewriteHistory {
            base: base.clone(),
            steps,
        },
    )
}

fn state(p: &Path) -> Option<crate::RebaseState> {
    testutil::backend().rebase_state(&RepoPath::new(p)).unwrap()
}

/// base 아래에서 위로 버전 이름들.
fn subjects(p: &Path, base: &Oid) -> Vec<String> {
    git(
        p,
        &["log", "--reverse", "--format=%s", &format!("{base}..HEAD")],
    )
    .lines()
    .map(str::to_string)
    .collect()
}

/// 처음 + 서로 다른 파일을 고치는 버전 셋.
fn three(p: &Path) -> (Oid, Oid, Oid, Oid) {
    let base = commit(p, "a.txt", "처음");
    let b = commit(p, "b.txt", "둘");
    let c = commit(p, "c.txt", "셋");
    let d = commit(p, "d.txt", "넷");
    (base, b, c, d)
}

#[test]
fn reorders() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    use RewriteKind::Pick;
    rewrite(
        p,
        &base,
        vec![step(&d, Pick), step(&b, Pick), step(&c, Pick)],
    )
    .unwrap();
    assert_eq!(subjects(p, &base), ["넷", "둘", "셋"]);
    assert!(state(p).is_none());
    assert_eq!(git(p, &["symbolic-ref", "HEAD"]).trim(), "refs/heads/main");
}

#[test]
fn fixup_joins_into_the_line_above() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    use RewriteKind::*;
    rewrite(
        p,
        &base,
        vec![step(&b, Pick), step(&c, Fixup), step(&d, Pick)],
    )
    .unwrap();
    assert_eq!(subjects(p, &base), ["둘", "넷"]);
    let files = git(p, &["show", "--name-only", "--format=", "HEAD~1"]);
    assert!(
        files.contains("b.txt") && files.contains("c.txt"),
        "{files}"
    );
}

#[test]
fn drop_removes_the_version() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    use RewriteKind::*;
    rewrite(
        p,
        &base,
        vec![step(&b, Pick), step(&c, Drop), step(&d, Pick)],
    )
    .unwrap();
    assert_eq!(subjects(p, &base), ["둘", "넷"]);
    assert!(!p.join("c.txt").exists());
}

#[test]
fn edit_stops_then_amend_and_continue_changes_the_message() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    use RewriteKind::*;
    rewrite(
        p,
        &base,
        vec![step(&b, Pick), step(&c, Edit), step(&d, Pick)],
    )
    .unwrap();
    let st = state(p).expect("멈춤");
    assert!(st.stopped_at_edit);
    assert_eq!(st.head_name, "refs/heads/main");
    assert_eq!(st.onto, Some(base.clone()));
    assert_eq!(st.orig_head, Some(d.clone()));
    assert_eq!(st.todo.len(), 1, "{:?}", st.todo);
    assert!(st.todo[0].starts_with("pick "), "{:?}", st.todo);
    assert_eq!(st.done.len(), 2, "{:?}", st.done);
    run(
        p,
        &GitOp::Amend {
            message: Some("고친 셋".into()),
        },
    )
    .unwrap();
    run(
        p,
        &GitOp::OperationContinue {
            kind: OperationKind::Rebase,
        },
    )
    .unwrap();
    assert!(state(p).is_none());
    assert_eq!(subjects(p, &base), ["둘", "고친 셋", "넷"]);
}

#[test]
fn overlapping_changes_stop_with_a_conflict() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let base = commit(p, "f.txt", "1\n");
    let b = commit(p, "f.txt", "2\n");
    let c = commit(p, "f.txt", "3\n");
    use RewriteKind::Pick;
    // c(2→3)를 처음(1) 위에 먼저 놓으면 겹친다.
    assert!(rewrite(p, &base, vec![step(&c, Pick), step(&b, Pick)]).is_err());
    let st = state(p).expect("멈춤");
    assert!(!st.stopped_at_edit);
    // stage 2는 정리된 기록(HEAD), stage 3은 옮기는 버전(REBASE_HEAD).
    assert_eq!(git(p, &["show", ":2:f.txt"]), "1\n");
    assert_eq!(git(p, &["show", ":3:f.txt"]), "3\n");
    assert_eq!(oid(p, "REBASE_HEAD"), c);
    run(
        p,
        &GitOp::OperationAbort {
            kind: OperationKind::Rebase,
        },
    )
    .unwrap();
    assert!(state(p).is_none());
    assert_eq!(oid(p, "HEAD"), c);
}

#[test]
fn refuses_with_changed_files_and_leaves_no_state() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    fs::write(p.join("b.txt"), "고침").unwrap();
    use RewriteKind::Pick;
    assert!(
        rewrite(
            p,
            &base,
            vec![step(&d, Pick), step(&b, Pick), step(&c, Pick)]
        )
        .is_err()
    );
    assert!(state(p).is_none());
    assert_eq!(oid(p, "HEAD"), d);
}

#[test]
fn repository_editor_settings_do_not_win() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    git(p, &["config", "sequence.editor", "false"]);
    git(p, &["config", "core.editor", "false"]);
    git(p, &["config", "rebase.abbreviateCommands", "true"]);
    git(p, &["config", "rebase.autoSquash", "true"]);
    use RewriteKind::Pick;
    rewrite(
        p,
        &base,
        vec![step(&c, Pick), step(&b, Pick), step(&d, Pick)],
    )
    .unwrap();
    assert_eq!(subjects(p, &base), ["셋", "둘", "넷"]);
}

#[test]
fn todo_list_is_checked() {
    let a = Oid::parse(&"a".repeat(40)).unwrap();
    let b = Oid::parse(&"b".repeat(40)).unwrap();
    use RewriteKind::*;
    assert_eq!(
        todo(&[step(&a, Edit), step(&b, Fixup)]).unwrap(),
        format!("edit {a}\nfixup {b}\n")
    );
    assert!(todo(&[]).is_err());
    assert!(todo(&[step(&a, Fixup)]).is_err());
    assert!(todo(&[step(&a, Drop), step(&b, Fixup)]).is_err());
    assert!(todo(&vec![step(&a, Pick); MAX_STEPS + 1]).is_err());
    assert!(todo(&vec![step(&a, Pick); MAX_STEPS]).is_ok());
}

#[test]
fn identity_is_known_only_when_set() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let path = RepoPath::new(p);
    assert!(testutil::backend().has_identity(&path).unwrap());
    git(p, &["config", "--unset", "user.name"]);
    git(p, &["config", "--unset", "user.email"]);
    git(p, &["config", "user.useConfigOnly", "true"]);
    assert!(!testutil::backend().has_identity(&path).unwrap());
}

#[test]
fn renamed_group_keeps_the_new_message_after_fixup() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let (base, b, c, d) = three(p);
    use RewriteKind::*;
    rewrite(
        p,
        &base,
        vec![step(&b, Edit), step(&c, Fixup), step(&d, Pick)],
    )
    .unwrap();
    assert!(state(p).unwrap().stopped_at_edit);
    run(
        p,
        &GitOp::Amend {
            message: Some("묶은 이름".into()),
        },
    )
    .unwrap();
    run(
        p,
        &GitOp::OperationContinue {
            kind: OperationKind::Rebase,
        },
    )
    .unwrap();
    assert_eq!(subjects(p, &base), ["묶은 이름", "넷"]);
}
