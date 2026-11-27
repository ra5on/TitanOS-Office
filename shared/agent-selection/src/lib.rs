//! Document-bound coordinates for TitanOS page-hosted editors.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]

#[cfg(target_arch = "wasm32")]
mod canvas;
#[cfg(target_arch = "wasm32")]
pub use canvas::{CanvasBridge, HostOpen};

/// Success acknowledgment for a specific save initiated by the native editor.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HostSaved {
    pub id: String,
    pub document_id: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Region {
    Paragraph(u32),
    Cells { sheet: String, range: String },
    Page(u32),
    Slide(u32),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Selection {
    Selected(Vec<Region>),
    None,
    Dirty,
    Unsupported,
    Unbound,
}

impl Selection {
    /// Resolve the selected native indices into document order. A missing item or excess
    /// selection makes the whole request unsupported instead of leaking a partial selection.
    pub fn numbered(
        indices: impl IntoIterator<Item = Option<usize>>,
        region: fn(u32) -> Region,
    ) -> Self {
        let mut values = Vec::new();
        for index in indices {
            let Some(index) = index.filter(|n| *n < 100_000) else {
                return Self::Unsupported;
            };
            if values.len() == 16 {
                return Self::Unsupported;
            }
            values.push(index as u32 + 1);
        }
        if values.is_empty() {
            return Self::None;
        }
        values.sort_unstable();
        if values.windows(2).any(|p| p[0] == p[1]) {
            return Self::Unsupported;
        }
        Self::Selected(values.into_iter().map(region).collect())
    }
}

#[cfg(test)]
mod region_tests {
    use super::*;
    #[test]
    fn explicit_page_and_slide_indices_only() {
        assert_eq!(Selection::numbered([], Region::Page), Selection::None);
        assert_eq!(
            Selection::numbered([Some(2), Some(1)], Region::Page),
            Selection::Selected(vec![Region::Page(2), Region::Page(3)])
        );
        assert_eq!(
            Selection::numbered([Some(1)], Region::Slide),
            Selection::Selected(vec![Region::Slide(2)])
        );
        for indices in [
            vec![None],
            vec![Some(0), None],
            vec![Some(1), Some(1)],
            vec![Some(100_000)],
            (0..17).map(Some).collect(),
        ] {
            assert_eq!(
                Selection::numbered(indices, Region::Slide),
                Selection::Unsupported
            );
        }
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
mod binding {
    use super::{HostSaved, Selection};

    #[derive(Default)]
    pub(super) struct Binding {
        current: Option<String>,
        request: Option<(String, String)>,
        saving: Option<String>,
        failed: Option<String>,
        saved: Option<HostSaved>,
        offered: Option<HostSaved>,
    }
    pub(super) fn token(s: &str) -> bool {
        s.len() == 32
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }
    impl Binding {
        pub fn open(&mut self, ticket: Option<String>) {
            self.current = ticket;
            self.request = None;
            self.saving = None;
            self.failed = None;
            self.saved = None;
            self.offered = None;
        }
        pub fn begin_save(&mut self, ticket: Option<String>) {
            let Some(id) = ticket.filter(|s| !s.is_empty() && s.len() <= 128) else {
                return;
            };
            self.open(None);
            self.saving = Some(id);
        }
        pub fn failed(&mut self, id: String) {
            if self.saving.as_deref() == Some(&id) {
                self.saving = None;
                self.failed = Some(id);
            }
        }
        pub fn take_failed(&mut self) -> Option<String> {
            self.failed.take()
        }
        pub fn saved(&mut self, id: String, document_id: String) {
            if token(&document_id) && self.saving.as_deref() == Some(&id) {
                self.saving = None;
                self.saved = Some(HostSaved { id, document_id });
            }
        }
        pub fn take_saved(&mut self) -> Option<HostSaved> {
            let saved = self.saved.take()?;
            self.offered = Some(saved.clone());
            Some(saved)
        }
        pub fn accept_saved(&mut self, saved: &HostSaved) -> bool {
            if self.offered.as_ref() != Some(saved) {
                return false;
            }
            self.offered = None;
            // The host may request the new selection before the editor's next frame accepts the save.
            self.current = Some(saved.document_id.clone());
            true
        }
        pub fn request(&mut self, id: String, doc: String) {
            if token(&id) && token(&doc) {
                self.request = Some((id, doc));
            }
        }
        pub fn take(
            &mut self,
            bound: Option<&str>,
            selected: impl FnOnce() -> Selection,
        ) -> Option<(String, String, Selection)> {
            let (id, doc) = self.request.take()?;
            let value = if self.current.as_deref() == Some(&doc) && bound == Some(&doc) {
                selected()
            } else {
                Selection::Unbound
            };
            let value = match value {
                Selection::Selected(items) if items.is_empty() => Selection::None,
                Selection::Selected(items) if items.len() > 16 => Selection::Unsupported,
                value => value,
            };
            Some((id, doc, value))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::Region;
        const DOC: &str = "ffeeddccbbaa99887766554433221100";
        const REQ: &str = "11223344556677889900aabbccddeeff";

        fn answer(b: &mut Binding, native: Option<&str>, value: Selection) -> Option<Selection> {
            b.take(native, || value).map(|(_, _, value)| value)
        }

        #[test]
        fn only_answer_once_for_the_current_native_document_and_request() {
            let mut b = Binding::default();
            b.open(Some(DOC.into()));
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                answer(&mut b, None, Selection::Dirty),
                Some(Selection::Unbound)
            );
            assert_eq!(answer(&mut b, Some(DOC), Selection::Dirty), None);
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                answer(
                    &mut b,
                    Some(DOC),
                    Selection::Selected(vec![Region::Page(2)])
                ),
                Some(Selection::Selected(vec![Region::Page(2)]))
            );
            b.request(REQ.into(), DOC.into());
            b.open(None); // legacy/local file or unverified save
            assert_eq!(answer(&mut b, Some(DOC), Selection::Dirty), None);
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                answer(&mut b, Some(DOC), Selection::Dirty),
                Some(Selection::Unbound)
            );
        }

        #[test]
        fn save_needs_matching_write_ack_and_explicit_native_confirmation()
        -> Result<(), &'static str> {
            let mut b = Binding::default();
            b.open(Some(DOC.into()));
            b.begin_save(None); // Exporting a separate copy does not replace the source.
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                answer(&mut b, Some(DOC), Selection::None),
                Some(Selection::None)
            );
            b.begin_save(Some("failed".into()));
            b.failed("other".into());
            assert!(b.take_failed().is_none());
            b.failed("failed".into());
            assert_eq!(b.take_failed().as_deref(), Some("failed"));
            b.begin_save(Some("save1".into()));
            b.saved("other".into(), DOC.into());
            assert!(b.take_saved().is_none());
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                answer(&mut b, Some(DOC), Selection::None),
                Some(Selection::Unbound)
            );
            b.saved("save1".into(), DOC.into());
            let first = b.take_saved().ok_or("missing first save")?;
            b.open(Some(REQ.into()));
            assert!(!b.accept_saved(&first));
            b.begin_save(Some("save2".into()));
            b.saved("save2".into(), DOC.into());
            let saved = b.take_saved().ok_or("missing second save")?;
            b.request(REQ.into(), DOC.into());
            assert!(b.accept_saved(&saved));
            assert!(!b.accept_saved(&saved));
            assert_eq!(
                answer(&mut b, Some(DOC), Selection::None),
                Some(Selection::None)
            );
            Ok(())
        }

        #[test]
        fn bound_requests_only_and_no_truncated_selections() {
            let mut b = Binding::default();
            b.open(Some(DOC.into()));
            b.request("invalid".into(), DOC.into());
            assert_eq!(answer(&mut b, Some(DOC), Selection::Dirty), None);
            for (count, expected) in [(0, Selection::None), (17, Selection::Unsupported)] {
                b.request(REQ.into(), DOC.into());
                assert_eq!(
                    answer(
                        &mut b,
                        Some(DOC),
                        Selection::Selected((1..=count).map(Region::Slide).collect())
                    ),
                    Some(expected)
                );
            }
        }
    }
}
