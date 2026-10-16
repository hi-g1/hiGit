//! 기계용 출력 파서가 이상한 바이트에 패닉하지 않는지(D-038, D-222). 고정 씨앗 xorshift로 무작위 바이트, 실제
//! 출력 표본을 자르고 뒤섞고 바이트를 바꾼 입력을 파서마다 2만 번 넣는다. 결과는 `Ok`든 `Err`든 상관없다.

use crate::Remote;

const ROUNDS: usize = 20_000;

/// 의존성 없는 고정 씨앗 난수(xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        usize::try_from(self.next() % (n as u64)).unwrap_or(0)
    }
}

/// 파서가 좋아할 만한 바이트를 자주 고른다(NUL, 줄바꿈, 탭, 공백, 숫자, 헥스, 구분 기호, 높은 바이트).
fn byte(rng: &mut Rng) -> u8 {
    const SPECIAL: &[u8] = b"\0\n\t \r\x1e#@+-,.:/\\0123456789abcdefABCDEF<>=|*()[]'\"";
    match rng.below(4) {
        0 => u8::try_from(rng.below(256)).unwrap_or(0),
        _ => SPECIAL[rng.below(SPECIAL.len())],
    }
}

/// 표본 하나를 바꾼 입력: 그대로, 잘라내기, 두 표본 이어 붙이기, 바이트 바꾸기·끼우기·빼기, 순수 무작위.
fn input(rng: &mut Rng, samples: &[&[u8]]) -> Vec<u8> {
    let pick = |rng: &mut Rng| samples[rng.below(samples.len())].to_vec();
    let mut out = match rng.below(6) {
        0 => (0..rng.below(300)).map(|_| byte(rng)).collect(),
        1 => {
            let s = pick(rng);
            let cut = rng.below(s.len() + 1);
            s[..cut].to_vec()
        }
        2 => {
            let mut s = pick(rng);
            s.extend(pick(rng));
            s
        }
        _ => pick(rng),
    };
    for _ in 0..rng.below(8) {
        match rng.below(3) {
            0 if !out.is_empty() => {
                let i = rng.below(out.len());
                out[i] = byte(rng);
            }
            1 => {
                let i = rng.below(out.len() + 1);
                out.insert(i, byte(rng));
            }
            _ if !out.is_empty() => {
                let i = rng.below(out.len());
                out.remove(i);
            }
            _ => {}
        }
    }
    out
}

/// 파서 하나를 `ROUNDS`번. 글자를 받는 파서는 UTF-8이 아닌 바이트를 대체 문자로 바꿔 넣는다(실제 호출과 같다).
fn shake(seed: u64, samples: &[&[u8]], mut parse: impl FnMut(&[u8], &str)) {
    let mut rng = Rng(seed);
    for _ in 0..ROUNDS {
        let bytes = input(&mut rng, samples);
        let text = String::from_utf8_lossy(&bytes);
        parse(&bytes, &text);
    }
}

const H1: &str = "1111111111111111111111111111111111111111";
const H2: &str = "2222222222222222222222222222222222222222";

#[test]
fn status_porcelain_v2() {
    let sample = format!(
        "# branch.oid {H1}\0# branch.head main\0# branch.upstream origin/main\0# branch.ab +1 -2\0\
         1 .M N... 100644 100644 100644 {H1} {H1} a.txt\0\
         2 R. N... 100644 100644 100644 {H1} {H2} R100 새 이름.txt\0옛 이름.txt\0\
         u UU N... 100644 100644 100644 100644 {H1} {H2} {H1} 충돌.txt\0? 새 파일\0! 무시\0"
    );
    shake(
        1,
        &[
            sample.as_bytes(),
            b"# branch.oid (initial)\0# branch.head (detached)\0",
        ],
        |_, t| {
            let _ = super::status::parse(t);
        },
    );
}

#[test]
fn log_records() {
    let rec = |f: [&str; 7]| format!("\x1e{}\n", f.join("\0"));
    let sample = rec([H1, "1111111", H2, "이름", "a@b.c", "1700000000", "메시지"])
        + &rec([H2, "2222222", "", "이름", "a@b.c", "1690000000", ""]);
    shake(2, &[sample.as_bytes()], |_, t| {
        let _ = super::log::parse(t);
    });
}

#[test]
fn unified_diff() {
    let sample = "diff --git a/a.txt b/a.txt\nindex 1..2 100644\n--- a/a.txt\n+++ b/a.txt\n\
                  @@ -1,3 +1,3 @@ 함수\n 하나\n-둘\n+두울\n 셋\n@@ -10 +10,2 @@\n-끝\n+끝\n+더함\n\\ No newline at end of file\n";
    let binary = "diff --git a/b.png b/b.png\nBinary files a/b.png and b/b.png differ\n";
    shake(3, &[sample.as_bytes(), binary.as_bytes()], |_, t| {
        let _ = super::diff::parse(t);
    });
}

#[test]
fn refs_branches_tags_and_remotes() {
    let rec = |f: &[&str]| f.join("\0") + "\n";
    let branches = rec(&[
        "refs/heads/main",
        H1,
        "*",
        "origin/main",
        "ahead 1, behind 2",
        "테스트",
        "1700000000",
        "메시지",
    ]) + &rec(&[
        "refs/remotes/origin/feature",
        H2,
        " ",
        "",
        "",
        "남",
        "1690000000",
        "기능",
    ]);
    let tags = rec(&["v1", H1, "commit", "", ""]) + &rec(&["v2", H2, "tag", H1, "commit"]);
    let refs = format!("{H1} refs/higit/parked/main\n{H2} refs/higit/backup/x\n");
    let remotes = "origin\0https://github.com/a/b.git\0up.stream\0git@github.com:c/d.git\0";
    let known = [Remote {
        name: "origin".into(),
        url: "https://github.com/a/b.git".into(),
    }];
    let samples: [&[u8]; 4] = [
        branches.as_bytes(),
        tags.as_bytes(),
        refs.as_bytes(),
        remotes.as_bytes(),
    ];
    shake(4, &samples, |_, t| {
        let _ = super::branches::parse_branches(t, &known);
        let _ = super::branches::parse_tags(t);
        let _ = super::branches::parse_refs(t);
        let _ = super::branches::parse_remotes(t);
    });
}

#[test]
fn stats_names_and_messages() {
    let stats = format!("\x1e{H1}\n1\t2\ta.txt\0-\t-\tb.png\0\x1e{H2}\n\n");
    let names = "M\0a.txt\0R100\0옛.txt\0새.txt\0D\0지움.txt\0A\0새로.txt\0";
    let messages = format!("\x1e{H1}\0제목\n\n본문 줄\n\x1e{H2}\0두 번째\n");
    let samples: [&[u8]; 3] = [stats.as_bytes(), names.as_bytes(), messages.as_bytes()];
    shake(5, &samples, |_, t| {
        let _ = super::stats::parse(t, false);
        let _ = super::stats::parse(t, true);
        let _ = super::history::parse_name_status(t);
        let _ = super::history::parse_messages(t, false);
        let _ = super::history::parse_messages(t, true);
    });
}

#[test]
fn index_unmerged_and_cat_file_batch() {
    let index = format!(
        "100644 {H1} 0\ta.txt\0100644 {H1} 1\tc.txt\0100644 {H2} 2\tc.txt\0100755 {H1} 3\tc.txt\0"
    );
    let batch = format!("{H1} blob 5\nhello\n{H2} blob 0\n\n{H1} missing\n");
    let samples: [&[u8]; 2] = [index.as_bytes(), batch.as_bytes()];
    shake(6, &samples, |b, t| {
        let _ = super::blobs::parse_index(t);
        let _ = super::merge_check::unmerged(t);
        let _ = super::blobs::parse_batch(b);
    });
}

#[test]
fn merge_file_segments() {
    let sample =
        b"same\n<<<<<<< ours\nmine\n||||||| base\norig\n=======\ntheirs\n>>>>>>> theirs\ntail";
    shake(7, &[sample], |b, _| {
        let _ = super::segments::parse_segments(b);
    });
}

#[test]
fn counts_ls_remote_and_symref() {
    let ls = format!("{H1}\trefs/heads/main\n{H2}\trefs/tags/v1\n");
    let symref = format!("ref: refs/heads/main\tHEAD\n{H1}\tHEAD\n");
    let samples: [&[u8]; 3] = [b"3\t4\n", ls.as_bytes(), symref.as_bytes()];
    shake(8, &samples, |_, t| {
        let _ = super::refs::parse_counts(t);
        let _ = super::refpush::parse_ls_remote(t, "refs/heads/main");
        let _ = super::refpush::parse_symref_head(t);
    });
}

#[test]
fn config_reflog_and_stash() {
    let config = "local\0core.fsmonitor\ntouch x\0global\0user.name\n이름\0worktree\0core.bare\0";
    let reflog = format!(
        "{H1}\0001700000000\0commit: 저장\0메시지\0\n{H2}\0001690000000\0checkout: moving\0두 번째\0"
    );
    let stash = format!("{H1}\0001700000000\0On main: 맡김\0\n{H2}\0001690000000\0WIP\0");
    let samples: [&[u8]; 3] = [config.as_bytes(), reflog.as_bytes(), stash.as_bytes()];
    shake(9, &samples, |_, t| {
        let _ = super::parse_config(t);
        let _ = super::parse_reflog(t);
        let _ = super::park::parse_stashes(t);
    });
}

#[test]
fn progress_versions_and_oids() {
    let samples: [&[u8]; 4] = [
        b"Receiving objects:  45% (450/1000), 1.2 MiB | 3 MiB/s",
        b"git version 2.45.1.windows.1\n",
        H1.as_bytes(),
        b"Resolving deltas: 100% (10/10), done.",
    ];
    shake(10, &samples, |_, t| {
        let _ = crate::progress::parse_line(t);
        let _ = crate::GitVersion::parse(t);
        let _ = crate::Oid::parse(t);
    });
}
