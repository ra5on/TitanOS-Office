//! Pure host-mode rules, shared by the bridge ([`crate::host`]) and testable without a browser
//! (like [`crate::prefs`]): the name the bridge fills in when a host's open request has none.
//! Which pages host the app is `craft-web-host`'s.

/// The file name used when an open request carries no usable one.
pub const DEFAULT_OPEN_NAME: &str = "Book.xlsx";

/// The name `craft-web-host` gives an open request that carries no usable one (it cleans a host's
/// name and falls back to this).
const NAMELESS: &str = "document";

/// A usable file name for an open request: the cleaned name the crate hands over, or
/// [`DEFAULT_OPEN_NAME`] when the host sent none.
pub fn open_name(name: &str) -> String {
    if name == NAMELESS { DEFAULT_OPEN_NAME.to_string() } else { name.to_string() }
}

/// Preserve disjoint rectangles. A bounding rectangle would disclose unselected cells.
pub fn selected_ranges(selection: &gridcraft_engine::selection::Selection) -> Option<Vec<String>> {
    let areas = &selection.ranges;
    if areas.is_empty() || areas.len() > 16 || areas.iter().any(|r| r.count() > 200 || r.is_full_rows() || r.is_full_cols()) {
        return None;
    }
    Some(areas.iter().map(|r| r.a1()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_receives_only_selected_rectangles() {
        use gridcraft_engine::{cell, core::RangeRef, selection::Selection};
        let mut selection = Selection::range(RangeRef::new(cell(1, 1), cell(6, 3)));
        selection.ranges.push(RangeRef::cell(cell(9, 5)));
        assert_eq!(selected_ranges(&selection), Some(vec!["B2:D7".into(), "F10".into()]));
        selection.ranges.push(RangeRef::new(cell(0, 0), cell(200, 0)));
        assert_eq!(selected_ranges(&selection), None);
    }

    #[test]
    fn identical_names_bind_only_the_actual_successful_open() {
        use gridcraft_engine::{Session, io};
        use gridcraft_ui_egui::{Services, SheetApp};
        let mut session = Session::new();
        session.run("file.new", serde_json::json!({})).unwrap();
        let data = io::save_file_bytes(&session.active().unwrap().wb, "shared.xlsx").unwrap();
        let mut app = SheetApp::new(session, Services::default());
        let first = app.open_received_bytes("shared.xlsx", &data).unwrap();
        let second = app.open_received_bytes("shared.xlsx", &data).unwrap();
        assert_ne!(first, second);
        assert_eq!(Some(second), app.session.active().map(|d| d.uid));
        assert_eq!(app.open_received_bytes("shared.xlsx", b"PK\x03\x04truncated"), None);
    }

    #[test]
    fn open_names_fall_back_to_the_default_when_the_host_sent_none() {
        assert_eq!(open_name("Budget.xlsx"), "Budget.xlsx");
        assert_eq!(open_name("Sheet2!Book.xlsx"), "Sheet2!Book.xlsx");
        assert_eq!(open_name("document"), DEFAULT_OPEN_NAME, "what the crate says for a missing or blank name");
    }
}
