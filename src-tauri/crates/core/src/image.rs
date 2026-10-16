//! 이미지 비교의 형식 판정(D-206). 확장자가 아니라 내용 머리로 정하고, 화면은 `<img>`로만 그린다.
//! 디코드하지 않는다(웹뷰가 한다). svg 안의 스크립트는 `<img>`에서 돌지 않는다.

use crate::view::ImageMime;

/// svg를 찾을 때 보는 앞부분 크기.
const SVG_SCAN: usize = 1024;

/// 내용 머리로 이미지 형식을 정한다. png·jpeg·gif·webp는 시그니처, svg는 BOM·공백·`<?xml …?>`·주석·
/// `<!DOCTYPE …>`을 건너뛰고 `<svg`로 시작하는지.
pub fn sniff(bytes: &[u8]) -> Option<ImageMime> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(ImageMime::Png);
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some(ImageMime::Jpeg);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some(ImageMime::Gif);
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(ImageMime::Webp);
    }
    is_svg(&bytes[..bytes.len().min(SVG_SCAN)]).then_some(ImageMime::Svg)
}

fn is_svg(head: &[u8]) -> bool {
    let mut rest = head.strip_prefix(b"\xef\xbb\xbf").unwrap_or(head);
    loop {
        rest = trim_start(rest);
        if let Some(after) = rest.strip_prefix(b"<?xml") {
            match find(after, b"?>") {
                Some(end) => rest = &after[end + 2..],
                None => return false,
            }
        } else if let Some(after) = rest.strip_prefix(b"<!--") {
            match find(after, b"-->") {
                Some(end) => rest = &after[end + 3..],
                None => return false,
            }
        } else if rest.len() >= 9 && rest[..9].eq_ignore_ascii_case(b"<!doctype") {
            match find(rest, b">") {
                Some(end) => rest = &rest[end + 1..],
                None => return false,
            }
        } else {
            break;
        }
    }
    rest.starts_with(b"<svg")
        && rest
            .get(4)
            .is_some_and(|c| c.is_ascii_whitespace() || *c == b'>' || *c == b'/')
}

fn trim_start(bytes: &[u8]) -> &[u8] {
    let n = bytes.iter().take_while(|b| b.is_ascii_whitespace()).count();
    &bytes[n..]
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures() {
        assert_eq!(
            sniff(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"),
            Some(ImageMime::Png)
        );
        assert_eq!(
            sniff(&[0xff, 0xd8, 0xff, 0xe0, 0, 0x10]),
            Some(ImageMime::Jpeg)
        );
        assert_eq!(sniff(b"GIF89a\x01\0\x01\0"), Some(ImageMime::Gif));
        assert_eq!(sniff(b"RIFF\x24\0\0\0WEBPVP8 "), Some(ImageMime::Webp));
        // 잘린 머리·빈 파일·이름만 그럴듯한 html
        assert_eq!(sniff(b"\x89PNG"), None);
        assert_eq!(sniff(b"RIFF\0\0\0\0WAVE"), None);
        assert_eq!(sniff(b""), None);
        assert_eq!(sniff(b"<!DOCTYPE html><html>"), None);
    }

    #[test]
    fn svg_after_prolog_and_comments() {
        assert_eq!(
            sniff(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
            Some(ImageMime::Svg)
        );
        let long = b"\xef\xbb\xbf\n  <?xml version=\"1.0\"?>\n<!-- made by hand -->\n<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"x\">\n<svg width=\"1\">";
        assert_eq!(sniff(long), Some(ImageMime::Svg));
        // 스크립트가 든 svg도 svg다. 화면은 `<img>`로만 그려 스크립트가 돌지 않는다.
        assert_eq!(
            sniff(b"<svg><script>alert(1)</script></svg>"),
            Some(ImageMime::Svg)
        );
        assert_eq!(sniff(b"<svgx>"), None);
        assert_eq!(sniff(b"<?xml version=\"1.0\""), None);
        assert_eq!(sniff("<!-- 끝나지 않은 주석 <svg>".as_bytes()), None);
        assert_eq!(sniff(b"<html><svg></svg></html>"), None);
    }
}
