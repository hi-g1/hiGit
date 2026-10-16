//! 이미지 비교 화면 타입(D-206).

use serde::{Deserialize, Serialize};
use specta::Type;

/// 비교할 이미지 쌍. `commit`이 있으면 그 버전과 첫 부모, 없으면 지금 버전(HEAD)과 작업 폴더.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageRequest {
    pub path: String,
    /// 이름 바꿈의 원래 경로(이전 쪽).
    #[serde(default)]
    pub orig_path: Option<String>,
    #[serde(default)]
    pub commit: Option<String>,
}

/// 내용 머리로 정한 형식. 화면은 `data:` 주소의 형식으로 그대로 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
pub enum ImageMime {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
    #[serde(rename = "image/svg+xml")]
    Svg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImageBlob {
    pub mime: ImageMime,
    /// base64.
    pub data: String,
    pub bytes: u32,
}

/// 두 쪽. 한쪽이 이미지가 아니면 둘 다 없고(화면은 "이진 파일"), 한쪽이 3MB를 넘으면 `too_large`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ImagePairView {
    pub before: Option<ImageBlob>,
    pub after: Option<ImageBlob>,
    pub too_large: bool,
}
