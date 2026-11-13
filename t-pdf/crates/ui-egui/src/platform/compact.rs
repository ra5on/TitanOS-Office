//! The narrow layout, for phones (TitanOS PLAN §87). A window narrower than [`MAX_WIDTH`], or a
//! short one that is not wide (a phone held sideways), has no room for the page beside the Tools
//! pane, the side panel, the rail and the quick bar: at 390 pt they took all of it and the page
//! showed at 13 %. There the page takes the whole width, a pane covers it while one is chosen
//! instead of sitting beside it, and the rail's page and zoom controls move to a bar below.
//!
//! The layout follows the window's size every frame and is never saved: widened again, the
//! window shows the panes as they were.

/// Narrower than this, the window is narrow.
pub const MAX_WIDTH: f32 = 700.0;
/// Shorter than this and narrower than [`SHORT_MAX_WIDTH`]: a phone on its side (360 to 430 pt
/// high), not a short desktop window.
pub const SHORT_HEIGHT: f32 = 460.0;
pub const SHORT_MAX_WIDTH: f32 = 1000.0;
/// The smallest touch target: 44 pt, as Apple's guidelines and TitanOS ask.
pub const TOUCH: f32 = 44.0;
/// The gutter beside the pages in the narrow layout, where the quick bar no longer floats.
pub const GUTTER: f32 = 6.0;

/// Touch-sized controls in the narrow layout: buttons, menu entries and other stock controls are at
/// least [`TOUCH`] tall there, and egui's own height elsewhere. Only when the layout changes
/// (`applied` is the last one given), so the rest of the style is left as the theme set it.
pub fn sync_style(ctx: &egui::Context, compact: bool, applied: &mut Option<bool>) {
    if *applied == Some(compact) {
        return;
    }
    let height = if compact { TOUCH } else { egui::style::Spacing::default().interact_size.y };
    ctx.all_styles_mut(|s| s.spacing.interact_size.y = height);
    *applied = Some(compact);
}

/// The size of a pane header's buttons (Close, Back) beside the page.
const PANE_BUTTON: f32 = 26.0;

/// The size of a pane header's buttons: [`TOUCH`] in the narrow layout, where the pane covers the
/// page and Close is the way back to it.
pub fn pane_button(ctx: &egui::Context) -> f32 {
    if is_compact(ctx.content_rect().size()) { TOUCH } else { PANE_BUTTON }
}

/// Whether a window of `size` (points) gets the narrow layout. A size egui does not know yet (the
/// first frame) is not narrow.
pub fn is_compact(size: egui::Vec2) -> bool {
    let known = size.x.is_finite() && size.y.is_finite() && size.x > 0.0 && size.y > 0.0;
    known && (size.x < MAX_WIDTH || (size.y < SHORT_HEIGHT && size.x < SHORT_MAX_WIDTH))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    #[test]
    fn phones_are_narrow_either_way_up() {
        for size in [vec2(360.0, 740.0), vec2(390.0, 844.0), vec2(430.0, 932.0), vec2(844.0, 390.0), vec2(932.0, 430.0)] {
            assert!(is_compact(size), "{size:?}");
        }
    }

    #[test]
    fn desktop_windows_keep_their_layout() {
        // 800 pt is the narrowest window the earlier T-PDF fixes were checked at.
        for size in [vec2(1280.0, 800.0), vec2(800.0, 600.0), vec2(800.0, 480.0), vec2(1024.0, 400.0), vec2(700.0, 500.0)] {
            assert!(!is_compact(size), "{size:?}");
        }
    }

    #[test]
    fn a_size_not_known_yet_is_not_narrow() {
        for size in [vec2(0.0, 0.0), vec2(f32::NAN, 800.0), vec2(390.0, f32::INFINITY), vec2(-1.0, 300.0)] {
            assert!(!is_compact(size), "{size:?}");
        }
    }
}
