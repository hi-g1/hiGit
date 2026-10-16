//! index 항목과 블롭 읽기(비밀 파일 검사, 충돌 해결이 쓴다).

use super::CliBackend;
use crate::{BlobHead, GitError, IndexEntry, Oid, RepoPath, Result};

/// 이보다 큰 블롭은 앞부분을 읽지 않는다(크기만 알린다).
const READ_LIMIT: u64 = 1024 * 1024;
/// 한 번에 읽는 블롭 전체 크기 상한.
const TOTAL_LIMIT: u64 = 32 * 1024 * 1024;

impl CliBackend {
    /// `ls-files -s -z`: `<모드> <해시> <stage>\t<경로>\0`.
    pub(super) fn ls_index(&self, repo: &RepoPath) -> Result<Vec<IndexEntry>> {
        let out = self
            .runner
            .run(Some(repo.as_path()), &["ls-files", "-s", "-z"])?;
        parse_index(&out.stdout_lossy())
    }

    pub(super) fn blob_heads(
        &self,
        repo: &RepoPath,
        oids: &[Oid],
        max_bytes: usize,
    ) -> Result<Vec<BlobHead>> {
        if oids.is_empty() {
            return Ok(Vec::new());
        }
        let dir = Some(repo.as_path());
        let list: String = oids.iter().map(|o| format!("{o}\n")).collect();
        let check =
            self.runner
                .run_with_stdin(dir, &["cat-file", "--batch-check"], list.as_bytes())?;
        let mut heads = Vec::new();
        let mut to_read = Vec::new();
        let mut total = 0u64;
        for line in check.stdout_lossy().lines() {
            // `<해시> blob <크기>` 또는 `<해시> missing`
            let mut parts = line.split(' ');
            let (Some(oid), Some(kind), Some(size)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let (Some(oid), Ok(size)) = (Oid::parse(oid), size.parse::<u64>()) else {
                continue;
            };
            if kind != "blob" {
                continue;
            }
            if size <= READ_LIMIT && total + size <= TOTAL_LIMIT {
                total += size;
                to_read.push(oid.clone());
            }
            heads.push(BlobHead {
                oid,
                size,
                head: Vec::new(),
            });
        }
        if to_read.is_empty() {
            return Ok(heads);
        }
        let list: String = to_read.iter().map(|o| format!("{o}\n")).collect();
        let out = self
            .runner
            .run_with_stdin(dir, &["cat-file", "--batch"], list.as_bytes())?;
        for (oid, bytes) in parse_batch(&out.stdout)? {
            if let Some(h) = heads.iter_mut().find(|h| h.oid == oid) {
                h.head = bytes[..bytes.len().min(max_bytes)].to_vec();
            }
        }
        Ok(heads)
    }
}

impl CliBackend {
    /// `ls-tree -z <rev> -- <path>`의 정확히 그 경로 줄. 일반 파일(100644·100755)의 블롭만.
    pub(super) fn blob_at_impl(
        &self,
        repo: &RepoPath,
        rev: &str,
        path: &str,
    ) -> Result<Option<Oid>> {
        if !super::park::safe_relative(path) {
            return Err(GitError::Parse(format!("저장소 밖 경로: {path}")));
        }
        let mut args = vec!["--literal-pathspecs", "ls-tree", "-z"];
        super::args::push_revs(
            &mut args,
            super::args::Cmd::LsTree,
            self.git_version(),
            &[rev],
        );
        args.extend(["--", path]);
        let out = match self.runner.run(Some(repo.as_path()), &args) {
            Ok(out) => out,
            // 버전이 없으면(첫 부모가 없음 등) 블롭도 없다.
            Err(_) => return Ok(None),
        };
        for record in out.stdout_lossy().split('\0').filter(|r| !r.is_empty()) {
            let Some((meta, name)) = record.split_once('\t') else {
                continue;
            };
            let mut parts = meta.split(' ');
            if let (Some(mode), Some("blob"), Some(oid)) =
                (parts.next(), parts.next(), parts.next())
                && name == path
                && matches!(mode, "100644" | "100755")
            {
                return Ok(Oid::parse(oid));
            }
        }
        Ok(None)
    }

    /// 크기를 먼저 보고(`--batch-check`) `max_bytes` 이하일 때만 읽는다(`--batch`).
    pub(super) fn read_blob_impl(
        &self,
        repo: &RepoPath,
        oid: &Oid,
        max_bytes: u64,
    ) -> Result<Option<(u64, Option<Vec<u8>>)>> {
        let dir = Some(repo.as_path());
        let line = format!("{oid}\n");
        let check =
            self.runner
                .run_with_stdin(dir, &["cat-file", "--batch-check"], line.as_bytes())?;
        let text = check.stdout_lossy();
        let mut parts = text.trim_end().split(' ');
        let (Some(_), Some("blob"), Some(size)) = (parts.next(), parts.next(), parts.next()) else {
            return Ok(None);
        };
        let size: u64 = size
            .parse()
            .map_err(|_| GitError::Parse(format!("cat-file: {text}")))?;
        if size > max_bytes {
            return Ok(Some((size, None)));
        }
        let out = self
            .runner
            .run_with_stdin(dir, &["cat-file", "--batch"], line.as_bytes())?;
        let bytes = parse_batch(&out.stdout)?
            .into_iter()
            .find(|(o, _)| o == oid)
            .map(|(_, b)| b);
        Ok(bytes.map(|b| (size, Some(b))))
    }
}

pub(super) fn parse_index(text: &str) -> Result<Vec<IndexEntry>> {
    let mut entries = Vec::new();
    for record in text.split('\0').filter(|r| !r.is_empty()) {
        let bad = || GitError::Parse(format!("ls-files: {record}"));
        let (meta, path) = record.split_once('\t').ok_or_else(bad)?;
        let mut parts = meta.split(' ');
        let (Some(mode), Some(oid), Some(stage)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(bad());
        };
        entries.push(IndexEntry {
            path: path.to_string(),
            mode: mode.to_string(),
            oid: Oid::parse(oid).ok_or_else(bad)?,
            stage: stage.parse().map_err(|_| bad())?,
        });
    }
    Ok(entries)
}

/// `cat-file --batch` 출력: `<해시> <종류> <크기>\n<내용>\n`이 이어진다.
fn parse_batch(mut out: &[u8]) -> Result<Vec<(Oid, Vec<u8>)>> {
    let mut blobs = Vec::new();
    while !out.is_empty() {
        let bad = || GitError::Parse("cat-file --batch".into());
        let nl = out.iter().position(|b| *b == b'\n').ok_or_else(bad)?;
        let header = String::from_utf8_lossy(&out[..nl]).into_owned();
        out = &out[nl + 1..];
        let mut parts = header.split(' ');
        let (Some(oid), Some(_kind), Some(size)) = (parts.next(), parts.next(), parts.next())
        else {
            // `<해시> missing` 같은 줄은 내용이 없다.
            continue;
        };
        let size: usize = size.parse().map_err(|_| bad())?;
        if out.len() < size + 1 {
            return Err(bad());
        }
        blobs.push((Oid::parse(oid).ok_or_else(bad)?, out[..size].to_vec()));
        out = &out[size + 1..];
    }
    Ok(blobs)
}
