//! 최근 프로젝트 목록(D-032). 앱 데이터 폴더의 `recent.json`에 둔다.
//!
//! Git 상태가 아니므로 Action·Query가 아니다(D-010). 파일 형식에만 serde를 쓰고,
//! 화면에 보내는 타입은 `view`에 따로 둔다(D-019).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 목록에 남기는 최대 개수(고정하지 않은 것).
pub const MAX_RECENT: usize = 20;
/// 고정할 수 있는 프로젝트 수(D-203). 최근 한도 밖에서도 남는다.
pub const MAX_PINNED: usize = 10;
/// 프로젝트마다 고정할 수 있는 브랜치 수와 이름 길이(바이트).
pub const MAX_PINNED_BRANCHES: usize = 10;
pub const MAX_BRANCH_NAME_BYTES: usize = 200;
const FILE_NAME: &str = "recent.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentEntry {
    /// 정규화된 저장소 최상위 경로.
    pub path: String,
    pub name: String,
    /// 마지막으로 연 시각(unix 초).
    pub last_opened_at: i64,
    /// 고정(D-203). 형식 버전은 그대로이고, 거짓이면 쓰지 않아 옛 앱도 읽는다.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pinned: bool,
    /// 고정한 브랜치 이름. 표시에만 쓰고 git 명령으로 보내지 않는다.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pinned_branches: Vec<String>,
}

/// 고정하지 못한 까닭.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinError {
    /// 목록에 없는 프로젝트.
    NotListed,
    /// 고정 한도(프로젝트 10개, 브랜치 10개).
    Limit,
    /// 너무 긴 브랜치 이름.
    TooLong,
}

#[derive(Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    projects: Vec<RecentEntry>,
}

pub struct RecentStore {
    /// 파일 위치. 앱 데이터 폴더를 쓸 수 없으면 `None`(메모리에만 둔다).
    file: Option<PathBuf>,
    entries: Mutex<Vec<RecentEntry>>,
}

impl RecentStore {
    /// 메모리에만 두는 목록(테스트, 앱 데이터 폴더를 못 만들 때).
    pub fn in_memory() -> Self {
        Self {
            file: None,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// `dir/recent.json`을 읽는다. 깨졌거나 모르는 형식이면 `.bad`로 옮기고 빈 목록으로 시작한다.
    pub fn open(dir: &Path) -> Self {
        let file = dir.join(FILE_NAME);
        let entries = match std::fs::read_to_string(&file) {
            Ok(text) => match serde_json::from_str::<FileFormat>(&text) {
                Ok(f) if f.version == FORMAT_VERSION => {
                    // 손으로 고친 파일도 한도 안에서만 믿는다.
                    let mut projects = f.projects;
                    for p in &mut projects {
                        p.pinned_branches
                            .retain(|b| !b.is_empty() && b.len() <= MAX_BRANCH_NAME_BYTES);
                        p.pinned_branches.truncate(MAX_PINNED_BRANCHES);
                    }
                    keep_within_limits(&mut projects);
                    projects
                }
                _ => {
                    let _ = std::fs::rename(&file, dir.join(format!("{FILE_NAME}.bad")));
                    Vec::new()
                }
            },
            Err(_) => Vec::new(),
        };
        Self {
            file: Some(file),
            entries: Mutex::new(entries),
        }
    }

    /// 최근 것부터.
    pub fn list(&self) -> Vec<RecentEntry> {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// 저장소를 열었다고 기록한다(맨 앞으로, 시각 갱신). 경로가 UTF-8이 아니면 기록하지 않는다.
    pub fn touch(&self, repo: &Path) {
        let Some(path) = repo.to_str() else {
            return;
        };
        let name = repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        // 고정은 물려받는다.
        let old = entries
            .iter()
            .position(|e| platform::same_path(Path::new(&e.path), repo))
            .map(|i| entries.remove(i));
        entries.insert(
            0,
            RecentEntry {
                path: path.to_string(),
                name,
                last_opened_at: now(),
                pinned: old.as_ref().is_some_and(|e| e.pinned),
                pinned_branches: old.map(|e| e.pinned_branches).unwrap_or_default(),
            },
        );
        keep_within_limits(&mut entries);
        self.save(&entries);
    }

    /// 프로젝트를 고정하거나 푼다(D-203).
    pub fn pin(&self, path: &Path, pinned: bool) -> Result<(), PinError> {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let count = entries.iter().filter(|e| e.pinned).count();
        let entry = entries
            .iter_mut()
            .find(|e| platform::same_path(Path::new(&e.path), path))
            .ok_or(PinError::NotListed)?;
        if pinned && !entry.pinned && count >= MAX_PINNED {
            return Err(PinError::Limit);
        }
        entry.pinned = pinned;
        keep_within_limits(&mut entries);
        self.save(&entries);
        Ok(())
    }

    /// 프로젝트의 브랜치를 고정하거나 푼다. 브랜치가 있는지는 부르는 쪽이 본다.
    pub fn pin_branch(&self, path: &Path, branch: &str, pinned: bool) -> Result<(), PinError> {
        if branch.len() > MAX_BRANCH_NAME_BYTES {
            return Err(PinError::TooLong);
        }
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = entries
            .iter_mut()
            .find(|e| platform::same_path(Path::new(&e.path), path))
            .ok_or(PinError::NotListed)?;
        let has = entry.pinned_branches.iter().any(|b| b == branch);
        match (pinned, has) {
            (true, false) if entry.pinned_branches.len() >= MAX_PINNED_BRANCHES => {
                return Err(PinError::Limit);
            }
            (true, false) => entry.pinned_branches.push(branch.to_string()),
            (false, true) => entry.pinned_branches.retain(|b| b != branch),
            _ => return Ok(()),
        }
        self.save(&entries);
        Ok(())
    }

    pub fn remove(&self, path: &Path) {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.retain(|e| !platform::same_path(Path::new(&e.path), path));
        self.save(&entries);
    }

    /// 임시 파일에 쓰고 바꿔 끼운다. 실패해도 프로젝트 열기는 실패시키지 않는다(다음 쓰기에서 다시).
    fn save(&self, entries: &[RecentEntry]) {
        let Some(file) = &self.file else {
            return;
        };
        let body = FileFormat {
            version: FORMAT_VERSION,
            projects: entries.to_vec(),
        };
        let Ok(text) = serde_json::to_string_pretty(&body) else {
            return;
        };
        let tmp = file.with_extension("json.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, file);
        }
    }
}

/// 고정하지 않은 것은 최근 20개, 고정한 것은 10개까지(순서는 그대로).
fn keep_within_limits(entries: &mut Vec<RecentEntry>) {
    let (mut pinned, mut recent) = (0, 0);
    entries.retain(|e| {
        let n = if e.pinned { &mut pinned } else { &mut recent };
        *n += 1;
        *n <= if e.pinned { MAX_PINNED } else { MAX_RECENT }
    });
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
