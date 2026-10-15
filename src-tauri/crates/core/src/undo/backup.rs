//! 백업 ref(D-007, D-043). 되돌리기와 caution 이상 액션 전에 지금 버전을 살려 둔다.
//! 30일 정리는 반복 2.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use git::{GitBackend, Oid, RefExpect, RepoPath};

use crate::Result;

pub(crate) const BACKUP_PREFIX: &str = "refs/higit/backup/";

/// 이름 조각의 상한(바이트). 파일 시스템의 파일 이름 상한(255)보다 넉넉히 작게 둔다.
/// 넘으면 백업 ref를 쓰지 못하고, 그러면 되돌리기가 거부된다.
const MAX_LABEL_BYTES: usize = 200;

/// `refs/higit/backup/<UTC 시각>-<순번>/<이름>`. 이름은 맡긴 ref처럼 `/`를 바꿔 적는다.
pub(crate) fn ref_name(label: &str) -> String {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut label = label.replace('%', "%25").replace('/', "%2F");
    if label.len() > MAX_LABEL_BYTES {
        let mut end = MAX_LABEL_BYTES;
        while !label.is_char_boundary(end) {
            end -= 1;
        }
        label.truncate(end);
    }
    format!("{BACKUP_PREFIX}{}-{n}/{label}", utc_stamp(secs))
}

/// `oid`를 백업 ref로 남긴다. 쓰기에 실패하면(사용자 hook이 거부 등) 오류를 돌려주고,
/// 부르는 쪽은 본 동작을 하지 않는다.
pub(crate) fn backup_oid(
    git: &dyn GitBackend,
    repo: &RepoPath,
    oid: &Oid,
    label: &str,
) -> Result<String> {
    let name = ref_name(label);
    git.update_ref(repo, &name, oid, &RefExpect::Absent)?;
    Ok(name)
}

/// 백업 ref를 지우기 전까지 두는 날 수(D-079). 설정 화면이 생기면 설정으로.
pub(crate) const KEEP_DAYS: u64 = 30;
/// 한 번에 지우는 백업 ref 수. 남으면 다음에 열 때 이어 지운다.
pub(crate) const PRUNE_AT_ONCE: usize = 20;

/// `refs/higit/backup/<시각>-<순번>/<이름>`에서 만든 시각(unix 초). 형식이 다르면 `None`.
pub(crate) fn created_at(refname: &str) -> Option<u64> {
    let rest = refname.strip_prefix(BACKUP_PREFIX)?;
    let (stamp, _) = rest.split_once('-')?;
    parse_stamp(stamp)
}

/// 이름의 시각이 `keep_days`보다 오래된 백업 ref를 `at_most`개까지 지운다. 지운 수.
/// 시각을 읽을 수 없는 ref는 지우지 않는다. 맡긴 ref(`refs/higit/parked/`)는 건드리지 않는다.
pub(crate) fn prune(
    git: &dyn GitBackend,
    repo: &RepoPath,
    now: u64,
    keep_days: u64,
    at_most: usize,
) -> Result<usize> {
    let limit = keep_days * 86_400;
    let mut removed = 0;
    for r in git.list_refs(repo, BACKUP_PREFIX)? {
        if removed >= at_most {
            break;
        }
        let old = created_at(&r.name).is_some_and(|at| now.saturating_sub(at) > limit);
        if let (true, Some(oid)) = (old, Oid::parse(&r.oid)) {
            git.delete_ref(repo, &r.name, &RefExpect::Is(oid))?;
            removed += 1;
        }
    }
    Ok(removed)
}

/// `20261015T093000Z` → unix 초. 날짜 계산은 Howard Hinnant의 days_from_civil.
fn parse_stamp(stamp: &str) -> Option<u64> {
    let b = stamp.as_bytes();
    if b.len() != 16 || b[8] != b'T' || b[15] != b'Z' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| stamp.get(r)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0..4)?, num(4..6)?, num(6..8)?);
    let (h, mi, s) = (num(9..11)?, num(11..13)?, num(13..15)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || s > 59 {
        return None;
    }
    let y = if mo <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = if mo > 2 { mo - 3 } else { mo + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    u64::try_from(days * 86_400 + h * 3600 + mi * 60 + s).ok()
}

/// unix 초 → `20261015T093000Z`. 날짜 계산은 Howard Hinnant의 civil_from_days.
pub(crate) fn utc_stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    format!("{y:04}{mo:02}{d:02}T{h:02}{m:02}{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps() {
        assert_eq!(utc_stamp(0), "19700101T000000Z");
        assert_eq!(utc_stamp(1_792_054_200), "20261015T085000Z");
        assert_eq!(utc_stamp(951_782_400), "20000229T000000Z");
    }

    #[test]
    fn stamps_round_trip() {
        for secs in [0, 951_782_400, 1_792_054_200, 4_102_444_799] {
            assert_eq!(parse_stamp(&utc_stamp(secs)), Some(secs), "{secs}");
        }
        assert_eq!(parse_stamp("2026-10-15"), None);
        assert_eq!(parse_stamp("20261315T000000Z"), None);
        let name = format!("{BACKUP_PREFIX}20261015T085000Z-3/undo-main");
        assert_eq!(created_at(&name), Some(1_792_054_200));
        assert_eq!(created_at("refs/higit/parked/main"), None);
    }

    #[test]
    fn names_are_unique_and_flat() {
        let a = ref_name("feature/x");
        let b = ref_name("feature/x");
        assert_ne!(a, b);
        assert!(a.starts_with(BACKUP_PREFIX));
        assert!(a.ends_with("/feature%2Fx"), "{a}");
    }

    #[test]
    fn long_labels_are_cut_on_a_character_boundary() {
        let long = format!("undo-{}", "가/".repeat(100));
        let name = ref_name(&long);
        let label = name.rsplit('/').next().unwrap();
        assert!(label.len() <= MAX_LABEL_BYTES, "{}", label.len());
        assert!(label.starts_with("undo-가%2F"));
    }
}
