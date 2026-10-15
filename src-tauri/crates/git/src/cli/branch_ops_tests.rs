//! 태그·브랜치 쓰기와 태그 이름 검사(D-103, D-104). 실제 git으로.

use std::fs;
use std::path::Path;

use crate::testutil::{self, git};
use crate::{GitBackend, GitOp, Oid, RepoPath};

fn commit(p: &Path, file: &str, text: &str) -> Oid {
    fs::write(p.join(file), text).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", text]);
    Oid::parse(git(p, &["rev-parse", "HEAD"]).trim()).unwrap()
}

fn run(p: &Path, op: GitOp) -> crate::Result<crate::OpOutput> {
    testutil::backend().run(&RepoPath::new(p), &op)
}

#[test]
fn tag_names_follow_git_rules() {
    let b = testutil::backend();
    for good in ["v1.0", "릴리스/첫째", "fix-1", "a.b"] {
        assert!(b.check_tag_name(good).unwrap(), "{good}");
    }
    for bad in [
        "", "-x", "a b", "a..b", "a~b", "a:b", "a.lock", "HEAD", "a/", ".",
    ] {
        assert!(!b.check_tag_name(bad).unwrap(), "{bad}");
    }
}

#[test]
fn lightweight_tags_even_with_signing_on() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    git(p, &["config", "tag.gpgSign", "true"]);
    run(
        p,
        GitOp::TagCreate {
            name: "한글-태그".into(),
            oid: first.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        git(p, &["cat-file", "-t", "refs/tags/한글-태그"]).trim(),
        "commit"
    );
    assert_eq!(git(p, &["rev-parse", "한글-태그"]).trim(), first.as_str());
    // 같은 이름은 실패한다.
    assert!(
        run(
            p,
            GitOp::TagCreate {
                name: "한글-태그".into(),
                oid: first
            }
        )
        .is_err()
    );
    run(
        p,
        GitOp::TagDelete {
            name: "한글-태그".into(),
        },
    )
    .unwrap();
    assert!(git(p, &["tag", "--list"]).trim().is_empty());
}

#[test]
fn branch_delete_and_rename_carry_settings() {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "처음");
    git(p, &["branch", "기능"]);
    git(p, &["config", "branch.기능.description", "설명"]);
    run(
        p,
        GitOp::BranchRename {
            from: "기능".into(),
            to: "새-기능".into(),
            force: false,
        },
    )
    .unwrap();
    assert_eq!(
        git(p, &["config", "branch.새-기능.description"]).trim(),
        "설명"
    );
    // 대소문자만 바꾸기(대소문자를 가리지 않는 파일 시스템에서도).
    run(
        p,
        GitOp::BranchRename {
            from: "새-기능".into(),
            to: "새-기능X".into(),
            force: false,
        },
    )
    .unwrap();
    run(
        p,
        GitOp::BranchRename {
            from: "새-기능X".into(),
            to: "새-기능x".into(),
            force: true,
        },
    )
    .unwrap();
    assert!(git(p, &["branch", "--list", "새-기능x"]).contains("새-기능x"));
    run(
        p,
        GitOp::BranchDelete {
            name: "새-기능x".into(),
        },
    )
    .unwrap();
    assert!(git(p, &["branch", "--list"]).trim().ends_with("main"));
    // 설정 구역도 지워진다.
    let config = fs::read_to_string(p.join(".git/config")).unwrap();
    assert!(!config.contains("새-기능"), "{config}");
}

#[test]
fn upstream_can_be_unset_and_set() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    // 연결이 없을 때 끊기는 성공이다.
    run(
        p,
        GitOp::UnsetUpstream {
            branch: "main".into(),
        },
    )
    .unwrap();
    git(p, &["remote", "add", "origin", "https://example.com/x.git"]);
    git(
        p,
        &["update-ref", "refs/remotes/origin/main", first.as_str()],
    );
    run(
        p,
        GitOp::SetUpstream {
            branch: "main".into(),
            upstream: "origin/main".into(),
        },
    )
    .unwrap();
    assert_eq!(
        git(p, &["rev-parse", "--abbrev-ref", "main@{upstream}"]).trim(),
        "origin/main"
    );
    run(
        p,
        GitOp::UnsetUpstream {
            branch: "main".into(),
        },
    )
    .unwrap();
    let cmd = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "main@{upstream}"])
        .current_dir(p)
        .output()
        .unwrap();
    assert!(!cmd.status.success());
}

#[test]
fn create_and_switch_at_an_older_version() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "하나\n");
    commit(p, "a.txt", "둘\n");
    run(
        p,
        GitOp::CreateAndSwitchAt {
            name: "옛-버전에서".into(),
            oid: first.clone(),
        },
    )
    .unwrap();
    assert_eq!(git(p, &["branch", "--show-current"]).trim(), "옛-버전에서");
    assert_eq!(git(p, &["rev-parse", "HEAD"]).trim(), first.as_str());
    assert_eq!(fs::read_to_string(p.join("a.txt")).unwrap(), "하나\n");
}
