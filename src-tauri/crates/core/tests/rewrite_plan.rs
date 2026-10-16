//! 기록 정리 계획 조회(`rewrite_plan`)를 실제 git으로 시험한다(계획 008 E14a, D-100, D-106).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::sync::Arc;

use higit_core::view::{OpenHow, RewriteBlocked};
use higit_core::{Core, GitBackend};

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args(["-c", "core.autocrlf=false", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

fn commit(dir: &Path, file: &str, message: &str) -> String {
    std::fs::write(dir.join(file), message).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", message]);
    git(dir, &["rev-parse", "HEAD"])
}

/// 처음, 둘, 셋, 넷, 다섯(각자 다른 파일).
fn five() -> (tempfile::TempDir, Vec<String>) {
    let repo = tempfile::tempdir().unwrap();
    let dir = repo.path();
    git(dir, &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("core.autocrlf", "false"),
        ("user.name", "테스트"),
        ("user.email", "t@example.com"),
    ] {
        git(dir, &["config", k, v]);
    }
    let oids = ["처음", "둘", "셋", "넷", "다섯"]
        .iter()
        .enumerate()
        .map(|(i, m)| commit(dir, &format!("{i}.txt"), m))
        .collect();
    (repo, oids)
}

fn core(dir: &Path) -> Core {
    let backend = git::CliBackend::new("git", Arc::new(|_| {}));
    let core = Core::with_backend(Arc::new(backend) as Arc<dyn GitBackend>).unwrap();
    core.open_project(dir, OpenHow::Picked).unwrap();
    core
}

fn plan(core: &Core, dir: &Path, oids: &[&String]) -> higit_core::view::RewritePlanView {
    core.queries()
        .rewrite_plan(dir, oids.iter().map(|o| o.to_string()).collect())
        .unwrap()
}

#[test]
fn a_contiguous_range_gives_cards_newest_first() {
    let (repo, o) = five();
    let dir = repo.path();
    git(dir, &["commit", "-q", "--amend", "-m", "다섯\n\n본문"]);
    let top = git(dir, &["rev-parse", "HEAD"]);
    let core = core(dir);
    let p = plan(&core, dir, &[&o[2], &o[1], &o[3]]);
    assert_eq!(p.blocked, None);
    let names: Vec<_> = p.cards.iter().map(|c| c.summary.as_str()).collect();
    assert_eq!(names, ["넷", "셋", "둘"]);
    assert_eq!(p.cards[0].author, "테스트");
    assert_eq!(p.stale_labels, 0);
    // 전체 메시지를 담는다.
    let p = plan(&core, dir, &[&top]);
    assert_eq!(p.cards[0].message.trim_end(), "다섯\n\n본문");
}

#[test]
fn blocked_reasons() {
    let (repo, o) = five();
    let dir = repo.path();
    let core = core(dir);
    let blocked = |oids: &[&String]| plan(&core, dir, oids).blocked;
    assert_eq!(
        blocked(&[&o[1], &o[3]]),
        Some(RewriteBlocked::NotOnBranchLine)
    );
    assert_eq!(blocked(&[]), Some(RewriteBlocked::NotOnBranchLine));
    let bad = "xyz".to_string();
    assert_eq!(blocked(&[&bad]), Some(RewriteBlocked::NotOnBranchLine));
    // 첫 버전이 들어 있으면 카드는 보여주되 막는다.
    let p = plan(&core, dir, &[&o[0], &o[1]]);
    assert_eq!(p.blocked, Some(RewriteBlocked::IncludesFirstVersion));
    assert_eq!(p.cards.len(), 2);
    // 올린 버전.
    git(dir, &["update-ref", "refs/remotes/origin/main", &o[2]]);
    let p = plan(&core, dir, &[&o[2], &o[3]]);
    assert_eq!(p.blocked, Some(RewriteBlocked::Published));
    assert_eq!(p.cards.len(), 2);
    assert_eq!(blocked(&[&o[3]]), None, "올린 버전 위만 고치면 된다");
    git(dir, &["update-ref", "-d", "refs/remotes/origin/main"]);
    // 고친 파일.
    std::fs::write(dir.join("1.txt"), "고침").unwrap();
    assert_eq!(blocked(&[&o[3]]), Some(RewriteBlocked::HasChanges));
    git(dir, &["checkout", "--", "1.txt"]);
    // 50개 넘게.
    let many: Vec<String> = (0..51).map(|i| format!("{i:040x}")).collect();
    let refs: Vec<&String> = many.iter().collect();
    assert_eq!(blocked(&refs), Some(RewriteBlocked::TooMany));
    // 특정 버전을 보는 중.
    git(dir, &["switch", "-q", "--detach", &o[4]]);
    assert_eq!(blocked(&[&o[3]]), Some(RewriteBlocked::Detached));
    git(dir, &["switch", "-q", "main"]);
    // 다른 브랜치의 버전.
    git(dir, &["switch", "-q", "-c", "옆", &o[2]]);
    let side = commit(dir, "옆.txt", "옆");
    git(dir, &["switch", "-q", "main"]);
    assert_eq!(blocked(&[&side]), Some(RewriteBlocked::NotOnBranchLine));
}

#[test]
fn merges_operations_and_stale_labels() {
    let (repo, o) = five();
    let dir = repo.path();
    let core = core(dir);
    // 다른 이름표가 다시 놓일 버전을 가리킨다.
    git(dir, &["tag", "v1", &o[3]]);
    git(dir, &["branch", "남은", &o[4]]);
    assert_eq!(plan(&core, dir, &[&o[2]]).stale_labels, 2);
    assert_eq!(plan(&core, dir, &[&o[4]]).stale_labels, 1);
    // 구간 위에 합친 버전이 있다.
    git(dir, &["switch", "-q", "-c", "기능", &o[1]]);
    commit(dir, "기능.txt", "기능");
    git(dir, &["switch", "-q", "main"]);
    git(dir, &["merge", "-q", "--no-ff", "-m", "합침", "기능"]);
    assert_eq!(
        plan(&core, dir, &[&o[3]]).blocked,
        Some(RewriteBlocked::HasMerge)
    );
    // 합치는 중.
    git(dir, &["reset", "-q", "--hard", &o[4]]);
    git(dir, &["switch", "-q", "-c", "겹침", &o[3]]);
    std::fs::write(dir.join("4.txt"), "겹침").unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "겹침"]);
    git(dir, &["switch", "-q", "main"]);
    git(dir, &["merge", "겹침"]);
    assert_eq!(
        plan(&core, dir, &[&o[3]]).blocked,
        Some(RewriteBlocked::OtherOperation)
    );
}
