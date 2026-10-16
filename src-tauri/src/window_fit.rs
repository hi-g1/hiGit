//! 되살린 창이 화면 안에 들어오게 한 번 고친다(D-217). 모니터를 떼었거나 배율이 바뀌어 창이 화면 밖에 남는 일을 막는다.

/// 물리 픽셀 사각형.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

/// 모니터 하나의 작업 영역(작업 표시줄 뺌)과 배율.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen {
    pub work: Rect,
    pub scale: f64,
}

/// 앱의 최소 창 크기(논리 픽셀, `tauri.conf.json`과 같다).
pub const MIN_LOGICAL: (u32, u32) = (800, 560);

fn overlap(a: Rect, b: Rect) -> u64 {
    let left = i64::from(a.x).max(i64::from(b.x));
    let top = i64::from(a.y).max(i64::from(b.y));
    let right = (i64::from(a.x) + i64::from(a.w)).min(i64::from(b.x) + i64::from(b.w));
    let bottom = (i64::from(a.y) + i64::from(a.h)).min(i64::from(b.y) + i64::from(b.h));
    let w = u64::try_from(right - left).unwrap_or(0);
    let h = u64::try_from(bottom - top).unwrap_or(0);
    w * h
}

fn scaled(logical: u32, scale: f64) -> u32 {
    let px = (f64::from(logical) * scale).round();
    if px.is_finite() && px >= 0.0 && px <= f64::from(u32::MAX) {
        // 위에서 범위를 확인했다.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let px = px as u32;
        px
    } else {
        logical
    }
}

/// 창이 어느 모니터 작업 영역과도 절반 넘게 겹치지 않으면 첫 모니터(주 모니터) 가운데로 옮긴다. 크기는 그 모니터보다
/// 크면 줄이고 최소보다 작으면 늘린다. 모니터를 모르면 그대로 둔다.
pub fn fit(win: Rect, screens: &[Screen], min_logical: (u32, u32)) -> Rect {
    let Some(primary) = screens.first() else {
        return win;
    };
    let area = u64::from(win.w) * u64::from(win.h);
    let best = screens
        .iter()
        .max_by_key(|s| overlap(win, s.work))
        .copied()
        .unwrap_or(*primary);
    let visible = area > 0 && overlap(win, best.work) * 2 > area;
    let screen = if visible { best } else { *primary };
    let min_w = scaled(min_logical.0, screen.scale).min(screen.work.w);
    let min_h = scaled(min_logical.1, screen.scale).min(screen.work.h);
    let w = win.w.clamp(min_w, screen.work.w.max(min_w));
    let h = win.h.clamp(min_h, screen.work.h.max(min_h));
    let centered = |start: i32, span: u32, size: u32| {
        let free = i64::from(span.saturating_sub(size)) / 2;
        i32::try_from(i64::from(start) + free).unwrap_or(start)
    };
    let (mut x, mut y) = if visible {
        (win.x, win.y)
    } else {
        (
            centered(screen.work.x, screen.work.w, w),
            centered(screen.work.y, screen.work.h, h),
        )
    };
    // 줄인 뒤에도 오른쪽·아래가 작업 영역을 넘으면 안으로 민다.
    let clamp_axis = |pos: i32, size: u32, start: i32, span: u32| -> i32 {
        let max = i64::from(start) + i64::from(span) - i64::from(size);
        let pos = i64::from(pos).min(max).max(i64::from(start));
        i32::try_from(pos).unwrap_or(start)
    };
    x = clamp_axis(x, w, screen.work.x, screen.work.w);
    y = clamp_axis(y, h, screen.work.y, screen.work.h);
    Rect { x, y, w, h }
}

/// 메인 창에 적용한다. 최대화했으면 건드리지 않는다.
pub fn apply(window: &tauri::WebviewWindow) {
    if window.is_maximized().unwrap_or(false) {
        return;
    }
    let (Ok(pos), Ok(size), Ok(monitors)) = (
        window.outer_position(),
        window.inner_size(),
        window.available_monitors(),
    ) else {
        return;
    };
    let primary = window.primary_monitor().ok().flatten();
    let to_screen = |m: &tauri::Monitor| Screen {
        work: Rect {
            x: m.work_area().position.x,
            y: m.work_area().position.y,
            w: m.work_area().size.width,
            h: m.work_area().size.height,
        },
        scale: m.scale_factor(),
    };
    // 주 모니터를 맨 앞에 둔다(화면 밖이면 그 가운데로).
    let first = primary.as_ref().map(to_screen);
    let screens: Vec<Screen> = first
        .into_iter()
        .chain(monitors.iter().map(to_screen).filter(|s| Some(*s) != first))
        .collect();
    let now = Rect {
        x: pos.x,
        y: pos.y,
        w: size.width,
        h: size.height,
    };
    let next = fit(now, &screens, MIN_LOGICAL);
    if next.w != now.w || next.h != now.h {
        let _ = window.set_size(tauri::PhysicalSize::new(next.w, next.h));
    }
    if next.x != now.x || next.y != now.y {
        let _ = window.set_position(tauri::PhysicalPosition::new(next.x, next.y));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FHD: Screen = Screen {
        work: Rect {
            x: 0,
            y: 0,
            w: 1920,
            h: 1040,
        },
        scale: 1.0,
    };
    /// 오른쪽에 붙은 배율 1.5 모니터.
    const RIGHT: Screen = Screen {
        work: Rect {
            x: 1920,
            y: 0,
            w: 2560,
            h: 1400,
        },
        scale: 1.5,
    };

    fn r(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn inside_stays() {
        assert_eq!(
            fit(r(100, 80, 1200, 780), &[FHD], MIN_LOGICAL),
            r(100, 80, 1200, 780)
        );
        assert_eq!(
            fit(r(2000, 100, 1500, 900), &[FHD, RIGHT], MIN_LOGICAL),
            r(2000, 100, 1500, 900)
        );
    }

    #[test]
    fn off_screen_or_mostly_off_moves_to_primary_center() {
        // 떼어 낸 모니터 자리
        assert_eq!(
            fit(r(5000, 200, 1200, 780), &[FHD], MIN_LOGICAL),
            r(360, 130, 1200, 780)
        );
        // 절반 넘게 밖(왼쪽으로 1000px 나감)
        assert_eq!(
            fit(r(-1000, 100, 1200, 780), &[FHD], MIN_LOGICAL),
            r(360, 130, 1200, 780)
        );
    }

    #[test]
    fn straddling_two_screens_stays_on_the_bigger_part() {
        // 오른쪽 모니터와 더 많이 겹친다 → 그 모니터 안으로만 민다.
        // 배율 1.5라 최소 높이는 840px이다.
        let got = fit(r(1700, 100, 1200, 780), &[FHD, RIGHT], MIN_LOGICAL);
        assert_eq!(got, r(1920, 100, 1200, 840));
    }

    #[test]
    fn sizes_are_clamped() {
        // 모니터보다 크면 작업 영역에 맞춘다.
        assert_eq!(
            fit(r(0, 0, 3000, 2000), &[FHD], MIN_LOGICAL),
            r(0, 0, 1920, 1040)
        );
        // 최소보다 작으면 늘린다(배율 반영).
        assert_eq!(
            fit(r(10, 10, 300, 200), &[FHD], MIN_LOGICAL),
            r(10, 10, 800, 560)
        );
        assert_eq!(
            fit(r(2000, 10, 300, 200), &[FHD, RIGHT], MIN_LOGICAL),
            r(2000, 10, 1200, 840)
        );
        // 줄이거나 늘린 뒤 넘치면 안으로 민다.
        assert_eq!(
            fit(r(1300, 600, 800, 560), &[FHD], MIN_LOGICAL),
            r(1120, 480, 800, 560)
        );
    }

    #[test]
    fn no_monitors_keeps_the_window() {
        assert_eq!(
            fit(r(-9000, -9000, 10, 10), &[], MIN_LOGICAL),
            r(-9000, -9000, 10, 10)
        );
    }
}
