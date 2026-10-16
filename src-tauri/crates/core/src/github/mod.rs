//! GitHub 연결(D-109). 상태·로그인 흐름·끊기·토큰 읽기를 한 곳에 둔다.
//!
//! 토큰은 OS 키체인에만 저장하고, 키체인을 못 쓰면 이번 실행 동안만 메모리(`SecretString`)에 둔다.
//! 비밀이 아닌 연결 정보는 `github.json`. 화면 타입·로그·오류에는 토큰이 들어가지 않는다.

mod errors;
mod helper;
pub(crate) mod pr;
mod store;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use hosting::{
    ExposeSecret, HostingError, HostingErrorKind, HostingProvider, LoginChallenge, LoginPoll,
    SecretString,
};

pub(crate) use errors::from_hosting;
pub use helper::git_credential_main;
use store::{StatusFile, StatusStore};

use crate::error::{CoreError, GithubFailure, Result};
use crate::view::{GitHubStatusView, LoginChallengeView, LoginPollView};

/// 키체인 항목(D-109).
pub const KEYCHAIN_SERVICE: &str = "dev.higit.desktop";
pub const KEYCHAIN_ACCOUNT: &str = "github.com";

/// 지금 시각을 주는 함수. 시험이 시간을 앞으로 돌린다.
pub type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

pub struct GitHubService {
    provider: Arc<dyn HostingProvider>,
    configured: bool,
    web_base: String,
    keychain: platform::Keychain,
    store: StatusStore,
    clock: Clock,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    login: Option<PendingLogin>,
    /// 로그인 시작마다 하나씩 는다. 늦게 끝난 폴링이 새 로그인을 건드리지 않게 한다.
    login_seq: u64,
    /// 이번 실행에 쓰는 토큰(키체인에서 읽었거나 쓴 것, 키체인을 못 써 메모리에만 둔 것).
    token: Option<SecretString>,
    /// 토큰이 바뀌거나 지워질 때마다 하나씩 는다. 늦게 온 401이 새 연결을 지우지 않게 한다.
    generation: u64,
    /// 이번 실행에 키체인에서 토큰을 읽었거나 키체인에 썼다(D-111 helper 조건).
    remembered: bool,
    /// 키체인을 못 써 이번 실행에만 둔 연결의 로그인 이름.
    session_login: Option<String>,
}

struct PendingLogin {
    seq: u64,
    challenge: LoginChallenge,
    expires_at: Instant,
    next_poll: Instant,
    interval: Duration,
    /// 서버에 묻는 중. 그동안 오는 폴링은 네트워크 없이 `Pending`.
    in_flight: bool,
}

/// 연결된 토큰과 그 세대.
pub(crate) struct TokenRef {
    pub token: SecretString,
    pub generation: u64,
}

impl GitHubService {
    pub fn new(
        provider: Arc<dyn HostingProvider>,
        config: &hosting::GitHubConfig,
        keychain: platform::Keychain,
        data_dir: Option<&std::path::Path>,
    ) -> Self {
        Self {
            provider,
            configured: config.is_configured(),
            web_base: config.web_base.clone(),
            keychain,
            store: data_dir.map_or_else(StatusStore::in_memory, StatusStore::open),
            clock: Arc::new(Instant::now),
            state: Mutex::new(State::default()),
        }
    }

    /// client id가 없고 네트워크도 키체인도 쓰지 않는 서비스(시험·기본값).
    pub fn unconfigured() -> Self {
        let config = hosting::GitHubConfig::new(None);
        let provider = hosting::GitHub::new(
            Arc::new(hosting::testing::FakeTransport::new()),
            config.clone(),
        );
        Self::new(
            Arc::new(provider),
            &config,
            platform::Keychain::in_memory(KEYCHAIN_SERVICE),
            None,
        )
    }

    /// 준비 안 된 서비스 하나(실행기 시험 등 GitHub를 쓰지 않는 곳의 `ExecCtx`용).
    pub fn none() -> &'static Self {
        static NONE: std::sync::OnceLock<GitHubService> = std::sync::OnceLock::new();
        NONE.get_or_init(Self::unconfigured)
    }

    /// **테스트 전용.** 로그인 흐름 없이 연결된 상태로 둔다. `remembered`면 키체인과 상태 파일에도
    /// 쓴 것처럼(이번 실행에 키체인을 쓴 것으로) 둔다.
    #[doc(hidden)]
    pub fn connect_for_tests(&self, login: &str, token: &str, remembered: bool) {
        let token = SecretString::from(token.to_owned());
        if remembered {
            let _ = self.keychain.set(KEYCHAIN_ACCOUNT, &token);
            let _ = self.store.write(&StatusFile::new(login, 1));
        }
        let mut state = self.state();
        state.token = Some(token);
        state.generation += 1;
        state.remembered = remembered;
        state.session_login = (!remembered).then(|| login.to_owned());
    }

    /// **테스트 전용.** 시각을 정한다.
    #[doc(hidden)]
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn now(&self) -> Instant {
        (self.clock)()
    }

    /// 설정 화면의 상태. 키체인을 건드리지 않는다.
    pub fn status(&self) -> GitHubStatusView {
        let file = self.store.read();
        let state = self.state();
        let (connected, login, remembered) = match (&state.session_login, file) {
            (Some(login), _) => (true, Some(login.clone()), false),
            (None, Some(file)) => (true, Some(file.login), true),
            (None, None) => (false, None, false),
        };
        GitHubStatusView {
            configured: self.configured,
            connected,
            login,
            remembered,
        }
    }

    /// 로그인을 시작한다. 진행 중이던 로그인은 버린다.
    pub fn connect_start(&self) -> Result<LoginChallengeView> {
        if !self.configured {
            return Err(failure(GithubFailure::NotConfigured));
        }
        let seq = {
            let mut state = self.state();
            state.login = None;
            state.login_seq += 1;
            state.login_seq
        };
        let challenge = self
            .provider
            .begin_login()
            .map_err(|e| from_hosting(&e, &[]))?;
        let now = self.now();
        let interval = Duration::from_secs(challenge.interval_secs);
        let view = LoginChallengeView {
            user_code: challenge.user_code.clone(),
            expires_in_secs: secs_u32(challenge.expires_in_secs),
            interval_secs: secs_u32(challenge.interval_secs),
        };
        let mut state = self.state();
        if state.login_seq != seq {
            return Err(failure(GithubFailure::Cancelled));
        }
        state.login = Some(PendingLogin {
            seq,
            expires_at: now + Duration::from_secs(challenge.expires_in_secs),
            next_poll: now + interval,
            interval,
            in_flight: false,
            challenge,
        });
        Ok(view)
    }

    /// 승인됐는지 묻는다. 간격 안이거나 이미 묻는 중이면 네트워크 없이 `Pending`.
    pub fn connect_poll(&self) -> Result<LoginPollView> {
        let (seq, challenge) = {
            let mut state = self.state();
            let now = self.now();
            let Some(pending) = state.login.as_mut() else {
                return Err(CoreError::InvalidInput("no login in progress"));
            };
            if now >= pending.expires_at {
                state.login = None;
                return Ok(LoginPollView::Expired);
            }
            if pending.in_flight || now < pending.next_poll {
                return Ok(pending_view(pending.interval));
            }
            pending.in_flight = true;
            (pending.seq, pending.challenge.clone())
        };
        let device_code = challenge.device_code.expose_secret().to_owned();
        let result = self.provider.poll_login(&challenge);
        let poll = match result {
            Ok(poll) => poll,
            Err(e) => {
                let configured = e.kind() != HostingErrorKind::NotConfigured;
                // 오류 뒤에도 간격을 지킨다(바로 다시 불러도 GitHub에 곧장 묻지 않게, D-121).
                let now = self.now();
                self.finish_poll(seq, |login| {
                    if !configured {
                        *login = None;
                    } else if let Some(pending) = login {
                        pending.in_flight = false;
                        pending.next_poll = now + pending.interval;
                    }
                });
                return Err(from_hosting(&e, &[&device_code]));
            }
        };
        match poll {
            LoginPoll::Pending => Ok(self.reschedule(seq, None)),
            LoginPoll::SlowDown { interval_secs } => {
                Ok(self.reschedule(seq, Some(Duration::from_secs(interval_secs))))
            }
            LoginPoll::Expired => {
                self.finish_poll(seq, |login| *login = None);
                Ok(LoginPollView::Expired)
            }
            LoginPoll::Denied => {
                self.finish_poll(seq, |login| *login = None);
                Ok(LoginPollView::Denied)
            }
            LoginPoll::Granted(token) => self.complete_login(seq, token, &device_code),
        }
    }

    /// 지금 로그인이 `seq`일 때만 `f`를 부른다.
    fn finish_poll(&self, seq: u64, f: impl FnOnce(&mut Option<PendingLogin>)) {
        let mut state = self.state();
        if state.login.as_ref().map(|p| p.seq) == Some(seq) {
            f(&mut state.login);
        }
    }

    fn reschedule(&self, seq: u64, interval: Option<Duration>) -> LoginPollView {
        let now = self.now();
        let mut state = self.state();
        match state.login.as_mut() {
            Some(pending) if pending.seq == seq => {
                if let Some(interval) = interval {
                    pending.interval = interval;
                }
                pending.in_flight = false;
                pending.next_poll = now + pending.interval;
                pending_view(pending.interval)
            }
            Some(pending) => pending_view(pending.interval),
            None => pending_view(interval.unwrap_or(Duration::from_secs(5))),
        }
    }

    fn complete_login(
        &self,
        seq: u64,
        token: SecretString,
        device_code: &str,
    ) -> Result<LoginPollView> {
        let user = match self.provider.current_user(&token) {
            Ok(user) => user,
            Err(e) => {
                self.finish_poll(seq, |login| {
                    if let Some(pending) = login {
                        pending.in_flight = false;
                    }
                });
                return Err(from_hosting(&e, &[device_code, token.expose_secret()]));
            }
        };
        {
            let state = self.state();
            if state.login.as_ref().map(|p| p.seq) != Some(seq) {
                return Err(failure(GithubFailure::Cancelled));
            }
        }
        // 키체인 → 상태 파일 순서. 둘 다 돼야 껐다 켜도 이어지는 연결이다.
        let remembered = match self.keychain.set(KEYCHAIN_ACCOUNT, &token) {
            Ok(()) => match self.store.write(&StatusFile::new(&user.login, user.id)) {
                Ok(()) => true,
                Err(_) => {
                    let _ = self.keychain.delete(KEYCHAIN_ACCOUNT);
                    false
                }
            },
            Err(_) => {
                // 이전 연결의 파일이 남아 있으면 지운다(다음 실행에 엉뚱한 상태를 보이지 않게).
                let _ = self.store.remove();
                false
            }
        };
        let mut state = self.state();
        state.login = None;
        state.token = Some(token);
        state.generation += 1;
        state.remembered = remembered;
        state.session_login = (!remembered).then(|| user.login.clone());
        Ok(LoginPollView::Connected { login: user.login })
    }

    /// 진행 중인 로그인을 버린다.
    pub fn connect_cancel(&self) {
        let mut state = self.state();
        state.login = None;
        state.login_seq += 1;
    }

    /// 연결을 끊는다. 키체인·메모리·상태 파일을 모두 지우려 하고 마지막 오류만 알린다.
    /// GitHub 쪽 권한 취소는 앱 권한 페이지에서 사람이 한다(client secret이 없다).
    pub fn disconnect(&self) -> Result<()> {
        self.forget();
        let mut last = Ok(());
        match self.keychain.delete(KEYCHAIN_ACCOUNT) {
            // 키체인이 없으면 저장한 것도 없다.
            Ok(()) | Err(platform::KeychainError::Unavailable) => {}
            Err(e) => last = Err(keychain_failure(&e)),
        }
        if let Err(e) = self.store.remove() {
            last = Err(CoreError::Github {
                kind: GithubFailure::Keychain,
                detail: Some(e.to_string()),
            });
        }
        last
    }

    /// 메모리의 연결을 지운다(토큰은 `SecretString`이라 놓을 때 지워진다).
    fn forget(&self) {
        let mut state = self.state();
        state.login = None;
        state.token = None;
        state.session_login = None;
        state.remembered = false;
        state.generation += 1;
    }

    /// 연결된 토큰. 메모리 → 키체인 순으로 찾고 처음 읽은 값을 둔다. 상태 파일은 있는데 키체인에
    /// 없으면 파일도 지우고 `NotConnected`.
    pub(crate) fn token(&self) -> Result<TokenRef> {
        {
            let state = self.state();
            if let Some(token) = &state.token {
                return Ok(TokenRef {
                    token: token.clone(),
                    generation: state.generation,
                });
            }
        }
        if self.store.read().is_none() {
            return Err(failure(GithubFailure::NotConnected));
        }
        match self.keychain.get(KEYCHAIN_ACCOUNT) {
            Ok(Some(token)) => {
                let mut state = self.state();
                if state.token.is_none() {
                    state.token = Some(token);
                    state.remembered = true;
                }
                let token = state
                    .token
                    .clone()
                    .ok_or_else(|| failure(GithubFailure::NotConnected))?;
                Ok(TokenRef {
                    token,
                    generation: state.generation,
                })
            }
            Ok(None) => {
                let _ = self.store.remove();
                Err(failure(GithubFailure::NotConnected))
            }
            Err(e) => Err(keychain_failure(&e)),
        }
    }

    /// 이번 실행에 키체인에서 읽었거나 쓴 토큰이 있다(helper를 붙여도 되는 조건, D-111).
    pub(crate) fn remembered(&self) -> bool {
        let state = self.state();
        state.remembered && state.token.is_some()
    }

    /// 401을 받았다. 그 토큰 세대가 지금과 같을 때만 연결을 지운다.
    pub(crate) fn on_unauthorized(&self, generation: u64) {
        if self.state().generation != generation {
            return;
        }
        let _ = self.disconnect();
    }

    /// 토큰을 실어 호출하고, 401이면 연결을 지운 뒤 `Expired`로 알린다.
    pub(crate) fn call<T>(
        &self,
        f: impl FnOnce(&dyn HostingProvider, &SecretString) -> std::result::Result<T, HostingError>,
    ) -> Result<T> {
        let TokenRef { token, generation } = self.token()?;
        match f(self.provider.as_ref(), &token) {
            Ok(value) => Ok(value),
            Err(e) => {
                if e.kind() == HostingErrorKind::Unauthorized {
                    self.on_unauthorized(generation);
                }
                Err(from_hosting(&e, &[token.expose_secret()]))
            }
        }
    }

    /// 원격 하나의 인증 방식(D-111). git이 실제로 쓸 받기·올리기 주소가 **모두**
    /// `https://github.com/…`이고, 저장소 로컬·워크트리 설정에 `http.*`·`remote.*.proxy`가 없고,
    /// 이번 실행에 키체인에서 토큰을 읽었거나 썼을 때만 `Helper`. 사람이 시킨 작업은 연결된
    /// 토큰을 키체인에서 읽어 보고(잠금 해제 창이 뜰 수 있다), 저절로 하는 확인은 읽지 않는다.
    pub(crate) fn auth_for(
        &self,
        git: &dyn git::GitBackend,
        repo: &git::RepoPath,
        remote: &str,
        background: bool,
    ) -> git::AuthMode {
        use git::AuthMode;
        let Ok(urls) = git.remote_urls(repo, remote) else {
            return AuthMode::UserDefault;
        };
        if !github_https_only(&urls.fetch) || !github_https_only(&urls.push) {
            return AuthMode::UserDefault;
        }
        match git.local_config(repo) {
            Ok(config) if !config.iter().any(|(key, _)| redirects_transport(key)) => {}
            _ => return AuthMode::UserDefault,
        }
        let ready = if background {
            self.remembered()
        } else {
            self.token().is_ok() && self.remembered()
        };
        if ready {
            AuthMode::Helper
        } else {
            AuthMode::UserDefault
        }
    }

    /// 코드를 넣는 페이지 주소(설정의 상수).
    pub fn device_login_url(&self) -> String {
        format!("{}/login/device", self.web_base)
    }

    /// 앱 권한 페이지(GitHub 쪽 연결 취소).
    pub fn app_settings_url(&self) -> String {
        format!("{}/settings/applications", self.web_base)
    }

    /// 검토 요청 하나의 페이지. 소유자·이름은 `parse_remote`가 검사한 값이고 번호는 1 이상.
    pub fn pull_request_url(&self, repo: &hosting::RemoteRepo, number: u32) -> Result<String> {
        if number == 0 || !hosting::valid_owner(&repo.owner) || !hosting::valid_name(&repo.name) {
            return Err(CoreError::InvalidInput("pull request page"));
        }
        Ok(format!(
            "{}/{}/{}/pull/{number}",
            self.web_base, repo.owner, repo.name
        ))
    }
}

/// 비어 있지 않고 모두 사용자 정보 없는 `https://github.com/…`인지.
pub(crate) fn github_https_only(urls: &[String]) -> bool {
    !urls.is_empty()
        && urls
            .iter()
            .all(|url| url.starts_with("https://") && hosting::GitHub::detect(url).is_some())
}

/// 받은 저장소가 연결을 돌리거나 TLS 검증을 끌 수 있는 설정 키(소문자, D-111).
fn redirects_transport(key: &str) -> bool {
    key.starts_with("http.") || (key.starts_with("remote.") && key.ends_with(".proxy"))
}

fn pending_view(interval: Duration) -> LoginPollView {
    LoginPollView::Pending {
        interval_secs: secs_u32(interval.as_secs()),
    }
}

fn secs_u32(secs: u64) -> u32 {
    u32::try_from(secs).unwrap_or(u32::MAX)
}

fn failure(kind: GithubFailure) -> CoreError {
    CoreError::Github { kind, detail: None }
}

fn keychain_failure(e: &platform::KeychainError) -> CoreError {
    CoreError::Github {
        kind: GithubFailure::Keychain,
        detail: Some(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use hosting::testing::FakeTransport;

    use super::*;

    const CLIENT_ID: &str = "Ov23liTESTCLIENT01";
    const TOKEN: &str = "gho_TESTSECRET0123456789";
    const DEVICE: &str = "3584d83530557fdd1f46af8289938c8ef79f9dc5";

    struct Fixture {
        fake: Arc<FakeTransport>,
        offset: Arc<Mutex<Duration>>,
        service: GitHubService,
    }

    impl Fixture {
        fn advance(&self, secs: u64) {
            *self.offset.lock().unwrap() += Duration::from_secs(secs);
        }
        fn requests(&self) -> usize {
            self.fake.requests().len()
        }
    }

    fn fixture(keychain: platform::Keychain, dir: Option<&std::path::Path>) -> Fixture {
        let fake = Arc::new(FakeTransport::new());
        let config = hosting::GitHubConfig::new(Some(CLIENT_ID));
        let provider = hosting::GitHub::new(fake.clone(), config.clone());
        let base = Instant::now();
        let offset = Arc::new(Mutex::new(Duration::ZERO));
        let clock_offset = offset.clone();
        let service = GitHubService::new(Arc::new(provider), &config, keychain, dir)
            .with_clock(Arc::new(move || base + *clock_offset.lock().unwrap()));
        Fixture {
            fake,
            offset,
            service,
        }
    }

    fn device_json(interval: u64) -> String {
        format!(
            r#"{{"device_code":"{DEVICE}","user_code":"WDJB-MJHT",
                "verification_uri":"https://github.com/login/device","expires_in":900,"interval":{interval}}}"#
        )
    }

    fn granted() -> String {
        format!(r#"{{"access_token":"{TOKEN}","token_type":"bearer","scope":"repo"}}"#)
    }

    const USER: &str = r#"{"login":"octocat","id":583231,"name":"Mona"}"#;

    /// 시작 → 토큰 → 사용자까지 한 번에.
    fn connect(f: &Fixture) {
        f.fake.push_json(200, &device_json(5));
        f.fake.push_json(200, &granted());
        f.fake.push_json(200, USER);
        f.service.connect_start().unwrap();
        f.advance(5);
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Connected {
                login: "octocat".into()
            }
        );
    }

    #[test]
    fn unconfigured_cannot_start() {
        let service = GitHubService::unconfigured();
        assert!(!service.status().configured);
        let err = service.connect_start().unwrap_err();
        assert!(matches!(
            err,
            CoreError::Github {
                kind: GithubFailure::NotConfigured,
                ..
            }
        ));
    }

    #[test]
    fn polls_respect_interval_and_slow_down() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = platform::Keychain::in_memory(KEYCHAIN_SERVICE);
        let f = fixture(keychain.clone(), Some(dir.path()));
        f.fake.push_json(200, &device_json(5));
        f.fake
            .push_json(200, r#"{"error":"authorization_pending"}"#);
        f.fake
            .push_json(200, r#"{"error":"slow_down","interval":10}"#);
        f.fake.push_json(200, &granted());
        f.fake.push_json(200, USER);

        let challenge = f.service.connect_start().unwrap();
        assert_eq!(challenge.user_code, "WDJB-MJHT");
        assert_eq!(challenge.interval_secs, 5);
        // 간격 안의 폴링은 네트워크를 쓰지 않는다.
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Pending { interval_secs: 5 }
        );
        assert_eq!(f.requests(), 1);
        f.advance(5);
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Pending { interval_secs: 5 }
        );
        assert_eq!(f.requests(), 2);
        f.advance(5);
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Pending { interval_secs: 10 }
        );
        assert_eq!(f.requests(), 3);
        f.advance(5);
        f.service.connect_poll().unwrap();
        assert_eq!(f.requests(), 3, "늘어난 간격 안");
        f.advance(5);
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Connected {
                login: "octocat".into()
            }
        );
        assert_eq!(f.requests(), 5);

        let status = f.service.status();
        assert!(status.connected && status.remembered);
        assert_eq!(status.login.as_deref(), Some("octocat"));
        assert_eq!(
            keychain
                .get(KEYCHAIN_ACCOUNT)
                .unwrap()
                .unwrap()
                .expose_secret(),
            TOKEN
        );
        assert!(dir.path().join("github.json").is_file());
        assert!(f.service.remembered());
        // 로그인이 끝났으니 더 묻지 않는다.
        assert!(f.service.connect_poll().is_err());

        // 껐다 켠 것: 상태 파일로 연결됨, 토큰은 처음 쓸 때 키체인에서.
        let again = fixture(keychain, Some(dir.path()));
        assert!(again.service.status().connected);
        assert!(!again.service.remembered());
        assert_eq!(again.service.token().unwrap().token.expose_secret(), TOKEN);
        assert!(again.service.remembered());
    }

    #[test]
    fn expiry_is_checked_without_network() {
        let f = fixture(platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
        f.fake.push_json(200, &device_json(5));
        f.service.connect_start().unwrap();
        f.advance(901);
        assert_eq!(f.service.connect_poll().unwrap(), LoginPollView::Expired);
        assert_eq!(f.requests(), 1);
    }

    #[test]
    fn failed_poll_still_waits_for_the_interval() {
        let f = fixture(platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
        f.fake.push_json(200, &device_json(5));
        f.fake.push_json(503, "unavailable");
        f.service.connect_start().unwrap();
        f.advance(5);
        assert!(f.service.connect_poll().is_err());
        assert_eq!(f.requests(), 2);
        // 바로 다시 불러도 묻지 않는다.
        assert_eq!(
            f.service.connect_poll().unwrap(),
            LoginPollView::Pending { interval_secs: 5 }
        );
        assert_eq!(f.requests(), 2);
    }

    #[test]
    fn denied_and_expired_answers_end_the_login() {
        let f = fixture(platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
        f.fake.push_json(200, &device_json(5));
        f.fake.push_json(200, r#"{"error":"access_denied"}"#);
        f.service.connect_start().unwrap();
        f.advance(5);
        assert_eq!(f.service.connect_poll().unwrap(), LoginPollView::Denied);
        assert!(f.service.connect_poll().is_err());
        assert!(!f.service.status().connected);
    }

    #[test]
    fn second_start_replaces_the_first() {
        let f = fixture(platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
        f.fake.push_json(200, &device_json(5));
        f.fake.push_json(200, &device_json(7));
        f.service.connect_start().unwrap();
        let second = f.service.connect_start().unwrap();
        assert_eq!(second.interval_secs, 7);
        f.advance(5);
        // 첫 번째의 간격(5초)이 아니라 두 번째(7초)를 따른다.
        f.service.connect_poll().unwrap();
        assert_eq!(f.requests(), 2);
        f.service.connect_cancel();
        assert!(f.service.connect_poll().is_err());
    }

    #[test]
    fn keychain_unavailable_keeps_session_only() {
        let dir = tempfile::tempdir().unwrap();
        let f = fixture(
            platform::Keychain::unavailable(KEYCHAIN_SERVICE),
            Some(dir.path()),
        );
        connect(&f);
        let status = f.service.status();
        assert!(status.connected);
        assert!(!status.remembered);
        assert!(!f.service.remembered());
        assert_eq!(f.service.token().unwrap().token.expose_secret(), TOKEN);
        assert!(!dir.path().join("github.json").exists());
        // 껐다 켜면 연결되어 있지 않다.
        let again = fixture(
            platform::Keychain::unavailable(KEYCHAIN_SERVICE),
            Some(dir.path()),
        );
        assert!(!again.service.status().connected);
    }

    #[test]
    fn unauthorized_disconnects_only_the_same_generation() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = platform::Keychain::in_memory(KEYCHAIN_SERVICE);
        let f = fixture(keychain.clone(), Some(dir.path()));
        connect(&f);
        let old = f.service.token().unwrap().generation;
        f.fake.push_json(401, r#"{"message":"Bad credentials"}"#);
        let err = f.service.call(|p, t| p.current_user(t)).unwrap_err();
        assert!(matches!(
            err,
            CoreError::Github {
                kind: GithubFailure::Expired,
                ..
            }
        ));
        assert!(!f.service.status().connected);
        assert!(keychain.get(KEYCHAIN_ACCOUNT).unwrap().is_none());
        assert!(!dir.path().join("github.json").exists());

        // 새로 연결한 뒤 늦게 온 옛 401은 새 연결을 지우지 않는다.
        connect(&f);
        f.service.on_unauthorized(old);
        assert!(f.service.status().connected);
    }

    #[test]
    fn missing_keychain_entry_clears_state_file() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = platform::Keychain::in_memory(KEYCHAIN_SERVICE);
        let f = fixture(keychain.clone(), Some(dir.path()));
        connect(&f);
        keychain.delete(KEYCHAIN_ACCOUNT).unwrap();
        let again = fixture(keychain, Some(dir.path()));
        assert!(again.service.status().connected);
        assert!(again.service.token().is_err());
        assert!(!again.service.status().connected);
    }

    #[test]
    fn disconnect_clears_everything_and_reports_last_error() {
        let dir = tempfile::tempdir().unwrap();
        let keychain = platform::Keychain::in_memory(KEYCHAIN_SERVICE);
        let f = fixture(keychain.clone(), Some(dir.path()));
        connect(&f);
        // 키체인 지우기가 실패해도 나머지는 지운다.
        keychain.deny_next_for_tests(KEYCHAIN_ACCOUNT);
        let err = f.service.disconnect().unwrap_err();
        assert!(matches!(
            err,
            CoreError::Github {
                kind: GithubFailure::Keychain,
                ..
            }
        ));
        assert!(!f.service.status().connected);
        assert!(!dir.path().join("github.json").exists());
        assert!(f.service.token().is_err());
    }

    #[test]
    fn token_never_shows_up() {
        let f = fixture(platform::Keychain::in_memory(KEYCHAIN_SERVICE), None);
        connect(&f);
        f.fake.push_json(500, &format!("server echoed {TOKEN}"));
        let err = f.service.call(|p, t| p.current_user(t)).unwrap_err();
        let view = crate::view::ErrorView::from(&err);
        let shown = format!(
            "{:?} {:?} {err} {err:?} {view:?}",
            f.service.status(),
            f.service.connect_poll()
        );
        assert!(!shown.contains("TESTSECRET"), "{shown}");
        assert!(!shown.contains(DEVICE), "{shown}");
    }
}
