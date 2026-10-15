//! 버전 기록 노선도의 끝점 고르기(D-088).

use git::{Branch, BranchKind, Oid};

/// 끝점 상한. 첫 쪽이 정하고 다음 쪽은 같은 목록을 되돌려 받는다.
pub(crate) const MAX_TIPS: usize = 200;
/// 다음 쪽이 되돌려 보낼 수 있는 끝점 수(지금 위치 하나를 더해도 넘지 않게).
pub(crate) const MAX_TIPS_IN: usize = MAX_TIPS + 1;

/// 지금 위치, 기본 브랜치 끝, 그 밖의 로컬 브랜치 끝(최근에 고친 것 먼저) 순서로 겹치지 않게
/// 200개까지. 돌려주는 둘째 값은 넣지 못한 끝점 수.
pub(crate) fn pick_tips(
    head: Option<&Oid>,
    default_branch: Option<&str>,
    branches: &[Branch],
) -> (Vec<Oid>, u32) {
    let mut local: Vec<&Branch> = branches
        .iter()
        .filter(|b| b.kind == BranchKind::Local)
        .collect();
    local.sort_by(|a, b| b.last_time.cmp(&a.last_time).then(a.name.cmp(&b.name)));
    let default_tip = default_branch
        .and_then(|name| local.iter().find(|b| b.name == name))
        .and_then(|b| Oid::parse(&b.oid));
    let mut all: Vec<Oid> = Vec::new();
    for oid in head
        .cloned()
        .into_iter()
        .chain(default_tip)
        .chain(local.iter().filter_map(|b| Oid::parse(&b.oid)))
    {
        if !all.contains(&oid) {
            all.push(oid);
        }
    }
    let cut = all.len().saturating_sub(MAX_TIPS);
    all.truncate(MAX_TIPS);
    (all, u32::try_from(cut).unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branch(name: &str, n: u32, time: i64) -> Branch {
        Branch {
            name: name.into(),
            refname: format!("refs/heads/{name}"),
            kind: BranchKind::Local,
            oid: format!("{n:040x}"),
            is_head: false,
            upstream: None,
            ahead: 0,
            behind: 0,
            gone: false,
            last_author: String::new(),
            last_time: time,
        }
    }

    fn oid(n: u32) -> Oid {
        Oid::parse(&format!("{n:040x}")).unwrap()
    }

    #[test]
    fn head_then_default_then_recent_without_repeats() {
        let mut remote = branch("origin/main", 9, 99);
        remote.kind = BranchKind::Remote {
            remote: "origin".into(),
            branch: "main".into(),
        };
        let branches = [
            branch("main", 1, 10),
            branch("옛것", 2, 5),
            branch("새것", 3, 50),
            branch("같은 곳", 3, 40),
            remote,
        ];
        let (tips, cut) = pick_tips(Some(&oid(3)), Some("main"), &branches);
        assert_eq!(tips, vec![oid(3), oid(1), oid(2)]);
        assert_eq!(cut, 0);
    }

    #[test]
    fn stops_at_two_hundred() {
        let branches: Vec<Branch> = (1..=205)
            .map(|n| branch(&format!("b{n}"), n, i64::from(n)))
            .collect();
        let (tips, cut) = pick_tips(Some(&oid(1)), Some("b2"), &branches);
        assert_eq!(tips.len(), MAX_TIPS);
        assert_eq!(cut, 5);
        assert_eq!(tips[0], oid(1));
        assert_eq!(tips[1], oid(2));
        assert_eq!(tips[2], oid(205));
    }

    #[test]
    fn empty_repo_has_no_tips() {
        assert_eq!(pick_tips(None, None, &[]), (Vec::new(), 0));
    }
}
