//! 노선도용 기록(`log_graph`)과 태그 목록. 실제 git으로 확인한다.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::testutil::{self, git};
use crate::{Commit, GitBackend, GraphLogQuery, Oid, RepoPath};

fn commit(p: &Path, file: &str, message: &str) -> Oid {
    fs::write(p.join(file), message).unwrap();
    git(p, &["add", "-A"]);
    git(p, &["commit", "-q", "-m", message]);
    oid(p, "HEAD")
}

fn oid(p: &Path, rev: &str) -> Oid {
    Oid::parse(git(p, &["rev-parse", rev]).trim()).unwrap()
}

fn graph(p: &Path, tips: &[&Oid], exclude: &[&Oid], skip: u32, limit: u32) -> Vec<Commit> {
    testutil::backend()
        .log_graph(
            &RepoPath::new(p),
            &GraphLogQuery {
                tips: tips.iter().map(|o| (*o).clone()).collect(),
                exclude: exclude.iter().map(|o| (*o).clone()).collect(),
                skip,
                limit,
            },
        )
        .unwrap()
}

fn summaries(commits: &[Commit]) -> Vec<&str> {
    commits.iter().map(|c| c.summary.as_str()).collect()
}

/// main: 처음 → 메인 → 합침(곁가지), 곁가지: 처음 → 곁 하나 → 곁 둘. 기능: 처음 → 기능(안 합침).
fn branched() -> tempfile::TempDir {
    let repo = testutil::init_repo();
    let p = repo.path();
    commit(p, "a.txt", "처음");
    git(p, &["switch", "-q", "-c", "곁가지"]);
    commit(p, "b.txt", "곁 하나");
    commit(p, "b2.txt", "곁 둘");
    git(p, &["switch", "-q", "main"]);
    commit(p, "c.txt", "메인");
    git(p, &["merge", "-q", "--no-edit", "-m", "합침", "곁가지"]);
    git(p, &["switch", "-q", "-c", "기능", "HEAD~1"]);
    commit(p, "d.txt", "기능");
    git(p, &["switch", "-q", "main"]);
    repo
}

#[test]
fn children_come_before_parents() {
    let repo = branched();
    let p = repo.path();
    let all = graph(p, &[&oid(p, "main"), &oid(p, "기능")], &[], 0, 100);
    assert_eq!(all.len(), 6);
    let at: HashMap<&str, usize> = all
        .iter()
        .enumerate()
        .map(|(i, c)| (c.oid.as_str(), i))
        .collect();
    for c in &all {
        for parent in &c.parents {
            assert!(
                at[c.oid.as_str()] < at[parent.as_str()],
                "{} 앞에 부모",
                c.summary
            );
        }
    }
    // 위상 순서는 한 줄기를 끝까지 이어 보여 준다(곁가지 둘이 붙어 있다).
    let s = summaries(&all);
    let one = s.iter().position(|x| *x == "곁 하나").unwrap();
    let two = s.iter().position(|x| *x == "곁 둘").unwrap();
    assert_eq!(two + 1, one);
}

#[test]
fn pages_follow_on_without_gaps_or_repeats() {
    let repo = branched();
    let p = repo.path();
    let tips = [&oid(p, "main"), &oid(p, "기능")];
    let whole = graph(p, &tips, &[], 0, 100);
    let mut paged = Vec::new();
    for skip in (0..8).step_by(2) {
        paged.extend(graph(p, &tips, &[], skip, 2));
    }
    assert_eq!(summaries(&paged), summaries(&whole));
}

#[test]
fn tips_and_exclusions() {
    let repo = branched();
    let p = repo.path();
    let only_main = graph(p, &[&oid(p, "main")], &[], 0, 100);
    assert!(!summaries(&only_main).contains(&"기능"));
    // 처음 버전과 그 조상을 빼면 그 뒤의 버전만 남는다(`^해시`가 `--end-of-options` 뒤에서 제외로 읽힘).
    let first = oid(p, "main~1~1");
    let after = graph(p, &[&oid(p, "기능")], &[&first], 0, 100);
    assert_eq!(summaries(&after), ["기능", "메인"]);
    assert!(graph(p, &[], &[], 0, 100).is_empty());
}

#[test]
fn tags_point_at_versions() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    let second = commit(p, "b.txt", "둘째");
    git(p, &["tag", "가벼운", first.as_str()]);
    git(p, &["tag", "-a", "v1.0", "-m", "릴리스", second.as_str()]);
    // 버전이 아닌 것(트리)을 가리키는 태그와 태그를 가리키는 태그는 뺀다.
    let tree = git(p, &["rev-parse", "HEAD^{tree}"]);
    git(p, &["tag", "트리", tree.trim()]);
    git(p, &["tag", "-a", "겹태그", "-m", "겹", "v1.0"]);
    let tags = testutil::backend().tags(&RepoPath::new(p)).unwrap();
    let pairs: Vec<(&str, &str)> = tags
        .iter()
        .map(|t| (t.name.as_str(), t.commit.as_str()))
        .collect();
    assert_eq!(
        pairs,
        [("v1.0", second.as_str()), ("가벼운", first.as_str())]
    );
}

#[test]
fn tag_list_stops_at_a_thousand() {
    let repo = testutil::init_repo();
    let p = repo.path();
    let first = commit(p, "a.txt", "처음");
    // update-ref 1001번 대신 packed-refs를 한 번에 쓴다(이름순).
    let mut packed = String::from("# pack-refs with: peeled fully-peeled sorted \n");
    for i in 0..1001 {
        packed.push_str(&format!("{first} refs/tags/t{i:04}\n"));
    }
    fs::write(p.join(".git/packed-refs"), packed).unwrap();
    let tags = testutil::backend().tags(&RepoPath::new(p)).unwrap();
    assert_eq!(tags.len(), 1000);
    assert_eq!(tags[0].name, "t0000");
}
