//! Native document identity, saved bytes and the current main-story selection.
//! File names and browser DOM selections do not identify a saved NAS document.
use std::collections::BTreeSet;
use titanos_agent_selection::{Region, Selection};
use wordcraft_doc::{Pos, StoryRef};
use wordcraft_docx::SavedParagraphs;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{HostDocument, WordApp};

pub struct BoundDocument {
    pub ticket: String,
    native: HostDocument,
    source: Option<SavedParagraphs>,
}

impl BoundDocument {
    pub fn new(ticket: String, native: HostDocument, source: Option<SavedParagraphs>) -> Self {
        Self { ticket, native, source }
    }

    /// Keep the ticket attached to this open call, through the native parser.
    /// A prompt or an already-open namesake doesn't identify these exact bytes.
    pub fn open(app: &mut WordApp, ticket: String, name: &str, bytes: &[u8]) -> Option<Self> {
        let info = app
            .run(
                "file.open",
                serde_json::json!({
                    "path": name, "data": wordcraft_engine::cmd::insert::base64_encode(bytes)
                }),
            )
            .ok()?;
        let stamp = app.host_document();
        if info.get("id").and_then(serde_json::Value::as_u64) != Some(stamp.document)
            || info.get("alreadyOpen").and_then(serde_json::Value::as_bool) != Some(false)
            || !app.has_document()
        {
            return None;
        }
        app.ui.backstage = false;
        Some(Self::new(ticket, stamp, source(name, bytes)))
    }

    pub fn same_view(&self, now: HostDocument) -> bool {
        self.native.document == now.document && self.native.view == now.view
    }

    pub fn selection(&self, app: &WordApp) -> Selection {
        let now = app.host_document();
        if !self.same_view(now) || !app.has_document() || app.ui.backstage {
            return Selection::Unbound;
        }
        if self.native.revision != now.revision || app.host_selection_dirty() {
            return Selection::Dirty;
        }
        if app.dialog.is_some() || app.colors.is_some() || app.session.math.is_some() || app.smart_art_row_focused {
            return Selection::None;
        }
        let Some(source) = &self.source else {
            return Selection::Unsupported;
        };
        paragraphs(&app.session, source)
    }
}

pub fn source(name: &str, bytes: &[u8]) -> Option<SavedParagraphs> {
    name.rsplit_once('.').filter(|(_, ext)| ext.eq_ignore_ascii_case("docx"))?;
    SavedParagraphs::from_docx(bytes)
}

fn paragraphs(session: &Session, source: &SavedParagraphs) -> Selection {
    if session.member.is_some() || !session.also_selected.is_empty() {
        return Selection::Unsupported;
    }
    if session.sel.is_collapsed() && session.column_segments().is_none() {
        return Selection::None;
    }
    if !source.matches(&session.doc) {
        return Selection::Unsupported;
    }
    let mut items = BTreeSet::new();
    let add = |items: &mut BTreeSet<u32>, left: &Pos, right: &Pos| -> Option<()> {
        for pos in [left, right] {
            if pos.story != StoryRef::Body || pos.path.0.len() != 1 {
                return None;
            }
            let paragraph = session.doc.para_at(pos)?;
            if !paragraph.text.is_char_boundary(pos.off) {
                return None;
            }
        }
        let (start, end) = if left <= right { (left, right) } else { (right, left) };
        if start != end {
            let first = start.path.0[0];
            let limit = end.path.0[0].checked_add(u32::from(end.off != 0))?;
            if limit.saturating_sub(first) > 16 {
                return None;
            }
            for index in first..limit {
                items.insert(index.checked_add(1)?);
            }
        }
        (items.len() <= 16).then_some(())
    };
    if let Some(segments) = session.column_segments() {
        for (left, right) in segments {
            if add(&mut items, left, right).is_none() {
                return Selection::Unsupported;
            }
        }
    } else if add(&mut items, &session.sel.anchor, &session.sel.focus).is_none() {
        return Selection::Unsupported;
    }
    if items.is_empty() { Selection::None } else { Selection::Selected(items.into_iter().map(Region::Paragraph).collect()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wordcraft_doc::Document;

    #[test]
    fn selected_paragraphs_follow_the_saved_model_and_selection_direction() {
        let doc = Document::from_text("Erster Absatz\n\nGleich\nGleich");
        let bytes = wordcraft_docx::write(&doc).unwrap();
        let shape = source("Ordner.docx", &bytes).unwrap();
        let mut session = Session::new(wordcraft_docx::read(&bytes).unwrap());
        assert_eq!(paragraphs(&session, &shape), Selection::None);
        session.sel.anchor = Pos::body(3, 0);
        session.sel.focus = Pos::body(1, 0);
        assert_eq!(paragraphs(&session, &shape), Selection::Selected(vec![Region::Paragraph(2), Region::Paragraph(3)]));
        session.sel.focus.story = StoryRef::Part(7);
        assert_eq!(paragraphs(&session, &shape), Selection::Unsupported);
        assert!(source("export.pdf", &bytes).is_none());
    }

    #[test]
    fn native_open_binds_exact_bytes_and_invalidates_after_other_file_intent() {
        let mut app = WordApp::new(Session::new(Document::new()), Default::default());
        let first = wordcraft_docx::write(&Document::from_text("Eins\nZwei")).unwrap();
        let other = wordcraft_docx::write(&Document::from_text("Neue Datei")).unwrap();
        let a = BoundDocument::open(&mut app, "first".into(), "Vertrag.docx", &first).unwrap();
        app.session.sel.focus = Pos::body(1, 2);
        assert_eq!(a.selection(&app), Selection::Selected(vec![Region::Paragraph(1), Region::Paragraph(2)]));
        let b = BoundDocument::open(&mut app, "second".into(), "Vertrag.docx", &other).unwrap();
        assert!(!a.same_view(app.host_document()));
        assert_eq!(a.selection(&app), Selection::Unbound);
        assert_eq!(b.selection(&app), Selection::None);
        app.session.dirty = true;
        assert_eq!(b.selection(&app), Selection::Dirty);
        app.session.dirty = false;
        assert!(BoundDocument::open(&mut app, "broken".into(), "Vertrag.docx", b"unreadable").is_none());
        assert_eq!(b.selection(&app), Selection::Unbound);
    }

    #[test]
    fn editable_save_captures_the_serialized_revision_and_an_export_keeps_its_original() {
        use std::{cell::RefCell, rc::Rc};
        let mut app = WordApp::new(Session::new(Document::new()), Default::default());
        let first = wordcraft_docx::write(&Document::from_text("Eins\nZwei")).unwrap();
        let original = BoundDocument::open(&mut app, "first".into(), "Bericht.docx", &first).unwrap();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let captured = calls.clone();
        app.services.download = Some(Box::new(|_, _| Ok(())));
        app.services.host_save = Some(Box::new(move |name, bytes, save_as, doc| {
            captured.borrow_mut().push((name.to_owned(), source(name, bytes), save_as, doc));
            Ok(())
        }));
        app.run("file.saveAs", serde_json::json!({"path":"Auszug.txt"})).unwrap();
        let (_, _, _, stamp) = calls.borrow_mut().pop().unwrap();
        assert!(stamp.is_none());
        assert!(original.same_view(app.host_document()));
        app.run("file.save", serde_json::json!({})).unwrap();
        let (name, map, save_as, stamp) = calls.borrow_mut().pop().unwrap();
        assert_eq!(name, "Bericht.docx");
        assert!(!save_as);
        assert_eq!(stamp, Some(app.host_document()));
        assert!(map.unwrap().matches(&app.session.doc));
    }

    #[test]
    fn more_than_sixteen_paragraphs_and_wrong_source_are_refused() {
        let doc = Document::from_text(&["Same"; 17].join("\n"));
        let bytes = wordcraft_docx::write(&doc).unwrap();
        let shape = source("A.docx", &bytes).unwrap();
        let mut session = Session::new(doc);
        session.sel.focus = Pos::body(16, 1);
        assert_eq!(paragraphs(&session, &shape), Selection::Unsupported);
        let other = wordcraft_docx::write(&Document::from_text("Same")).unwrap();
        assert_eq!(paragraphs(&session, &source("B.docx", &other).unwrap()), Selection::Unsupported);
    }
}
