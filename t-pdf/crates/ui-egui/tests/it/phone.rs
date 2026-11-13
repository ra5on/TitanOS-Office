//! The narrow layout on a phone (TitanOS PLAN §87): the page takes the width when a document
//! opens, the top bars stay on the screen, and a desktop window keeps its panes, rail and tabs.

use egui::accesskit::Role;
use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use pdfcraft_ui_egui::PdfCraftApp;
use pdfcraft_ui_egui::canvas::{Fit, PT};

const PORTRAIT: Vec2 = vec2(390.0, 844.0);
const LANDSCAPE: Vec2 = vec2(844.0, 390.0);
const DESKTOP: Vec2 = vec2(1280.0, 800.0);
/// An A4 page's width in points.
const A4_WIDTH: f32 = 595.0;

/// Three A4 pages of Helvetica text.
fn fixture() -> Vec<u8> {
    let mut stream = String::new();
    for row in 0..40 {
        stream.push_str(&format!("BT /F1 11 Tf 60 {} Td (Line {row}: text a finger may drag across on a phone) Tj ET\n", 780 - row * 18));
    }
    let content = format!("<< /Length {} >>\nstream\n{stream}endstream", stream.len());
    let objs = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 7 0 R >> >> /Contents 6 0 R >>".to_owned(),
        content,
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
    ];
    let mut out = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objs.len() + 1).as_bytes());
    for o in offsets {
        out.extend_from_slice(format!("{o:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objs.len() + 1).as_bytes());
    out
}

/// The app at `size` with the fixture open, laid out.
fn window(size: Vec2) -> Harness<'static, PdfCraftApp> {
    let mut h = Harness::builder().with_size(size).build_eframe(|_cc| {
        let mut app = PdfCraftApp::new();
        app.set_option("language", "en").unwrap();
        app.open_bytes("doc.pdf", None, fixture()).unwrap();
        app
    });
    h.run_steps(6);
    h
}

/// The zoom that fits an A4 page into `width` points less a gutter of `side` each side.
fn fit(width: f32, side: f32) -> f32 {
    (width - 2.0 * side) / (A4_WIDTH * PT)
}

#[test]
fn a_phone_opens_the_page_at_the_width_of_the_screen() {
    for size in [PORTRAIT, LANDSCAPE] {
        let h = window(size);
        let view = &h.state().views[0];
        // Before the narrow layout, the Tools pane, the rail and the quick bar's gutters left the
        // page 13 % on a 390 pt phone.
        assert_eq!(view.fit, Fit::Width, "{size:?}");
        let want = fit(size.x, pdfcraft_ui_egui::compact::GUTTER);
        assert!((view.zoom - want).abs() < 0.01, "{size:?}: zoom {} for the width's {want}", view.zoom);
    }
}

#[test]
fn a_desktop_window_keeps_its_panes_rail_and_tabs() {
    let h = window(DESKTOP);
    // The rail, the mode tabs and the GitHub button, none of them in a "⋯" menu.
    for label in ["Comments", "All tools", "Convert", "GitHub"] {
        assert!(h.query_all_by_label(label).next().is_some(), "{label} is on the desktop");
    }
    assert!(h.query_by_label("More").is_none(), "no \"⋯\" menus on the desktop");
    // The page between the Tools pane and the rail, with the quick bar's gutter.
    let zoom = h.state().views[0].zoom;
    assert!(zoom < fit(DESKTOP.x - 272.0 - 49.0, 70.0) + 0.01, "zoom {zoom}");
}

#[test]
fn the_top_bars_of_a_phone_stay_on_the_screen_with_touch_sized_buttons() {
    for size in [PORTRAIT, LANDSCAPE] {
        let h = window(size);
        // The workspace menu is named for the workspace it shows.
        for label in ["Home", "More", "All tools", "Next page", "Previous page"] {
            let nodes: Vec<_> = h.query_all_by_label(label).collect();
            assert!(!nodes.is_empty(), "{size:?}: {label}");
            for node in nodes {
                let r = node.rect();
                assert!(r.left() >= 0.0 && r.right() <= size.x + 0.5, "{size:?}: {label} at {r:?}");
                assert!(r.height() >= pdfcraft_ui_egui::compact::TOUCH - 0.5, "{size:?}: {label} is {r:?}");
            }
        }
        for node in h.query_all_by_role(Role::Button) {
            let r = node.rect();
            assert!(r.right() <= size.x + 0.5, "{size:?}: a button at {r:?} runs off the screen");
        }
    }
}
