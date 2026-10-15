//! 실패한 git 명령의 종류를 가린다(D-021, D-023).
//!
//! 성공 출력은 기계용 형식만 파싱하지만, git은 실패 이유를 기계용으로 주지 않는다.
//! 그래서 `LC_ALL=C`로 받은 영어 원문의 고정 문구를 **이 파일에서만** 읽는다.
//! 모르는 문구는 `Other`가 되고, 화면은 일반 안내와 "자세히 보기"를 보여준다.

/// 화면이 쉬운 안내와 해결 버튼을 고르는 데 쓰는 실패 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureKind {
    /// 저장할 변경이 없다.
    NothingToCommit,
    /// 이름·이메일이 설정되어 있지 않다.
    IdentityMissing,
    /// 갈래 이름 규칙에 맞지 않는다.
    InvalidRefName,
    /// 같은 이름의 갈래가 이미 있다.
    BranchExists,
    /// 바꾸면 저장하지 않은 변경이 덮어써진다.
    WouldOverwrite,
    /// 내 저장과 원격 저장이 갈라져 fast-forward로 받을 수 없다.
    NotFastForward,
    /// 원격이 앞서 있어 올리기가 거부됐다.
    PushRejected,
    /// 연결된 원격 갈래가 없다.
    NoUpstream,
    /// 처음 접속하는 SSH 서버라 호스트 키 확인이 필요하다.
    SshHostUnverified,
    /// SSH 키가 없거나, 등록되지 않았거나, 비밀번호가 걸린 키가 ssh-agent에 없다.
    SshAuthFailed,
    /// HTTPS 인증이 필요하거나 실패했다.
    HttpAuthFailed,
    /// 원격에 닿을 수 없다(주소, 네트워크, 저장소 없음).
    RemoteUnreachable,
    /// 폴더가 저장소가 아니다(`discover`가 씀).
    NotARepository,
    /// 합치다 같은 곳이 겹쳤다.
    MergeConflict,
    /// 폴더 주인이 다른 사용자라 git이 저장소로 다루지 않는다(safe.directory, CVE-2022-24765).
    DubiousOwnership,
    /// 시간 제한이 지나 끝냈다.
    TimedOut,
    /// 맡겨 둔 파일이 지금 버전과 겹쳐 꺼내지 않았다(D-054).
    UnparkConflict,
    /// 허용하지 않은 전송 방식(`ext::`, `file://`, 로컬 경로 원격, D-035).
    ProtocolNotAllowed,
    Other,
}

/// 순서가 중요하다: SSH 인증 실패 뒤에는 "Could not read from remote repository"가 함께 오므로
/// 인증을 먼저 본다.
const RULES: &[(FailureKind, &[&str])] = &[
    (
        FailureKind::DubiousOwnership,
        &["detected dubious ownership"],
    ),
    (FailureKind::NotARepository, &["not a git repository"]),
    (
        FailureKind::ProtocolNotAllowed,
        &[
            "transport 'ext' not allowed",
            "transport 'file' not allowed",
            "transport 'http' not allowed",
            "transport 'fd' not allowed",
        ],
    ),
    (
        FailureKind::SshHostUnverified,
        &["Host key verification failed"],
    ),
    (
        FailureKind::SshAuthFailed,
        &[
            "Permission denied (publickey",
            "Permission denied, please try again",
        ],
    ),
    (
        FailureKind::HttpAuthFailed,
        &[
            "Authentication failed for",
            "could not read Username",
            "could not read Password",
            "terminal prompts disabled",
        ],
    ),
    (
        FailureKind::RemoteUnreachable,
        &[
            "Could not resolve host",
            "does not appear to be a git repository",
            "Could not read from remote repository",
            "Connection refused",
            "Connection timed out",
            "unable to access",
        ],
    ),
    (
        FailureKind::PushRejected,
        &["Updates were rejected", "[rejected]"],
    ),
    (
        FailureKind::NotFastForward,
        &[
            "Not possible to fast-forward",
            "diverging branches can't be fast-forwarded",
        ],
    ),
    (
        FailureKind::NoUpstream,
        &["has no upstream branch", "There is no tracking information"],
    ),
    (FailureKind::WouldOverwrite, &["would be overwritten by"]),
    (
        FailureKind::MergeConflict,
        &["Automatic merge failed", "CONFLICT ("],
    ),
    (
        FailureKind::IdentityMissing,
        &[
            "Please tell me who you are",
            "unable to auto-detect email address",
            "empty ident name",
        ],
    ),
    (FailureKind::BranchExists, &["already exists"]),
    (
        FailureKind::InvalidRefName,
        &["is not a valid branch name", "is not a valid ref name"],
    ),
    (
        FailureKind::NothingToCommit,
        &[
            "nothing to commit",
            "nothing added to commit",
            "no changes added to commit",
        ],
    ),
];

pub(crate) fn classify(stdout: &str, stderr: &str) -> FailureKind {
    for (kind, needles) in RULES {
        if needles
            .iter()
            .any(|n| stderr.contains(n) || stdout.contains(n))
        {
            return *kind;
        }
    }
    FailureKind::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    // 실제 git(LC_ALL=C)이 내는 원문 표본. 각 작업 단계에서 실제 재현 테스트를 더한다.
    #[test]
    fn classifies_samples() {
        let cases = [
            (
                "Host key verification failed.\nfatal: Could not read from remote repository.\n",
                FailureKind::SshHostUnverified,
            ),
            (
                "git@github.com: Permission denied (publickey).\nfatal: Could not read from remote repository.\n",
                FailureKind::SshAuthFailed,
            ),
            (
                "fatal: could not read Username for 'https://github.com': terminal prompts disabled\n",
                FailureKind::HttpAuthFailed,
            ),
            (
                "fatal: unable to access 'https://없는주소.invalid/x.git/': Could not resolve host: 없는주소.invalid\n",
                FailureKind::RemoteUnreachable,
            ),
            (
                " ! [rejected]        main -> main (fetch first)\nerror: failed to push some refs\nhint: Updates were rejected because the remote contains work\n",
                FailureKind::PushRejected,
            ),
            (
                "fatal: Not possible to fast-forward, aborting.\n",
                FailureKind::NotFastForward,
            ),
            (
                "fatal: The current branch 새갈래 has no upstream branch.\n",
                FailureKind::NoUpstream,
            ),
            (
                "error: Your local changes to the following files would be overwritten by checkout:\n\ta.txt\n",
                FailureKind::WouldOverwrite,
            ),
            (
                "Author identity unknown\n\n*** Please tell me who you are.\n",
                FailureKind::IdentityMissing,
            ),
            (
                "fatal: a branch named 'main' already exists\n",
                FailureKind::BranchExists,
            ),
            (
                "fatal: 'a..b' is not a valid branch name\n",
                FailureKind::InvalidRefName,
            ),
            (
                "fatal: transport 'ext' not allowed\n",
                FailureKind::ProtocolNotAllowed,
            ),
            ("fatal: something new\n", FailureKind::Other),
        ];
        for (stderr, want) in cases {
            assert_eq!(classify("", stderr), want, "{stderr}");
        }
    }

    #[test]
    fn reads_stdout_too() {
        // git commit은 "nothing to commit"을 stdout에 쓴다.
        assert_eq!(
            classify(
                "On branch main\nnothing to commit, working tree clean\n",
                ""
            ),
            FailureKind::NothingToCommit
        );
    }
}
