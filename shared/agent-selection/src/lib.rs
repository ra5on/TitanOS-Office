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

#[cfg(any(test, target_arch = "wasm32"))]
mod binding {
    use super::{HostSaved, Selection};

    #[derive(Default)]
    pub(super) struct Binding {
        current: Option<String>,
        request: Option<(String, String)>,
        saving: Option<String>,
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
            self.saved = None;
            self.offered = None;
        }
        pub fn begin_save(&mut self, ticket: Option<String>) {
            self.open(None);
            self.saving = ticket.filter(|s| !s.is_empty() && s.len() <= 128);
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
            self.open(Some(saved.document_id.clone()));
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

        #[test]
        fn only_answer_once_for_the_current_native_document_and_request() {
            let mut binding = Binding::default();
            binding.open(Some(DOC.into()));
            binding.request(REQ.into(), DOC.into());
            assert_eq!(
                binding.take(None, || panic!("other tab")).unwrap().2,
                Selection::Unbound
            );
            assert!(binding.take(Some(DOC), || panic!("replay")).is_none());
            binding.request(REQ.into(), DOC.into());
            assert_eq!(
                binding
                    .take(Some(DOC), || Selection::Selected(vec![Region::Page(2)]))
                    .unwrap()
                    .2,
                Selection::Selected(vec![Region::Page(2)])
            );
            binding.request(REQ.into(), DOC.into());
            binding.open(None); // legacy/local file or unverified save
            assert!(
                binding
                    .take(Some(DOC), || panic!("invalidated request"))
                    .is_none()
            );
            binding.request(REQ.into(), DOC.into());
            assert_eq!(
                binding
                    .take(Some(DOC), || panic!("stale editor ticket"))
                    .unwrap()
                    .2,
                Selection::Unbound
            );
        }

        #[test]
        fn save_needs_matching_write_ack_and_explicit_native_confirmation() {
            let mut b = Binding::default();
            b.open(Some(DOC.into()));
            b.begin_save(Some("save1".into()));
            b.saved("other".into(), DOC.into());
            assert!(b.take_saved().is_none());
            b.request(REQ.into(), DOC.into());
            assert_eq!(
                b.take(Some(DOC), || panic!("unconfirmed save")).unwrap().2,
                Selection::Unbound
            );
            b.saved("save1".into(), DOC.into());
            let saved = b.take_saved().unwrap();
            b.open(Some(REQ.into()));
            assert!(!b.accept_saved(&saved));
            b.begin_save(Some("save2".into()));
            b.saved("save2".into(), DOC.into());
            let saved = b.take_saved().unwrap();
            assert!(b.accept_saved(&saved));
            assert!(!b.accept_saved(&saved));
        }
        #[test]
        fn bound_requests_only_and_no_truncated_selections() {
            let mut b = Binding::default();
            b.open(Some(DOC.into()));
            b.request("invalid".into(), DOC.into());
            assert!(b.take(Some(DOC), || panic!("invalid token")).is_none());
            for (count, expected) in [(0, Selection::None), (17, Selection::Unsupported)] {
                b.request(REQ.into(), DOC.into());
                assert_eq!(
                    b.take(Some(DOC), || Selection::Selected(
                        (1..=count).map(Region::Slide).collect()
                    ))
                    .unwrap()
                    .2,
                    expected
                );
            }
        }
    }
}
