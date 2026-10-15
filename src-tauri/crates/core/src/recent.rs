//! 최근 프로젝트 목록(D-032). 앱 데이터 폴더의 `recent.json`에 둔다.
//!
//! Git 상태가 아니므로 Action·Query가 아니다(D-010). 파일 형식에만 serde를 쓰고,
//! 화면에 보내는 타입은 `view`에 따로 둔다(D-019).

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 목록에 남기는 최대 개수.
pub const MAX_RECENT: usize = 20;
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
                Ok(f) if f.version == FORMAT_VERSION => f.projects,
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
        entries.retain(|e| !platform::same_path(Path::new(&e.path), repo));
        entries.insert(
            0,
            RecentEntry {
                path: path.to_string(),
                name,
                last_opened_at: now(),
            },
        );
        entries.truncate(MAX_RECENT);
        self.save(&entries);
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

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
