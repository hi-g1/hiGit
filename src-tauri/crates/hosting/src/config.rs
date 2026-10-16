//! GitHub 연결 설정. 비밀이 아닌 값만 둔다(client id는 공개 값이다).

/// device flow로 받을 권한. 비공개 저장소 올리기·검토 요청에 필요한 최소다(D-109).
pub const GITHUB_SCOPE: &str = "repo";
pub const GITHUB_API_BASE: &str = "https://api.github.com";
pub const GITHUB_WEB_BASE: &str = "https://github.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubConfig {
    /// OAuth App의 client id. 없으면 연결을 시작할 수 없다("준비 안 됨").
    pub client_id: Option<String>,
    pub api_base: String,
    pub web_base: String,
    pub scope: String,
}

impl GitHubConfig {
    /// 실제 github.com 주소로 만든다. 형식이 틀린 client id는 없는 것으로 본다.
    pub fn new(client_id: Option<&str>) -> Self {
        Self {
            client_id: client_id
                .filter(|id| valid_client_id(id))
                .map(str::to_owned),
            api_base: GITHUB_API_BASE.to_owned(),
            web_base: GITHUB_WEB_BASE.to_owned(),
            scope: GITHUB_SCOPE.to_owned(),
        }
    }

    /// 시험이 가짜 서버 주소를 넣는다.
    pub fn with_bases(mut self, api_base: &str, web_base: &str) -> Self {
        self.api_base = api_base.trim_end_matches('/').to_owned();
        self.web_base = web_base.trim_end_matches('/').to_owned();
        self
    }

    pub fn is_configured(&self) -> bool {
        self.client_id.is_some()
    }
}

/// client id는 영숫자 8~64자만 받는다.
pub fn valid_client_id(id: &str) -> bool {
    (8..=64).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_id_rules() {
        let ok = ["Ov23liABCDEF12345678", "abcdefgh", &"a".repeat(64)];
        for id in ok {
            assert!(valid_client_id(id), "{id}");
            assert!(GitHubConfig::new(Some(id)).is_configured());
        }
        let bad = [
            "",
            "short1",
            "Iv1.abcdef123",
            "abc defgh",
            "abcdefgh\n",
            "가나다라마바사아",
            "abcd&scope=admin",
            &"a".repeat(65),
        ];
        for id in bad {
            assert!(!valid_client_id(id), "{id:?}");
            assert!(!GitHubConfig::new(Some(id)).is_configured());
        }
        assert!(!GitHubConfig::new(None).is_configured());
    }

    #[test]
    fn defaults_point_to_github() {
        let config = GitHubConfig::new(None);
        assert_eq!(config.api_base, "https://api.github.com");
        assert_eq!(config.web_base, "https://github.com");
        assert_eq!(config.scope, "repo");
        let local = config.with_bases("http://127.0.0.1:9/", "http://127.0.0.1:9");
        assert_eq!(local.api_base, "http://127.0.0.1:9");
        assert_eq!(local.web_base, "http://127.0.0.1:9");
    }
}
