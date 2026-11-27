//! Accept only the actual host-opened PDF and its explicit, ordered page selection.
use super::PdfCraftApp;

impl PdfCraftApp {
    pub(super) fn take_host_open(&mut self) {
        if let Some(file) = self.host_selection.as_ref().and_then(|s| s.take_open()) {
            // Password prompts, conversion or a refused open must never authorize the previous tab.
            self.host_document = None;
            let before = self.views.len();
            match self.open_bytes_with_focus(&file.name, None, file.bytes, !self.startup_superseded) {
                Ok(()) if self.views.len() == before + 1 && file.name.to_ascii_lowercase().ends_with(".pdf") => {
                    if let Some(doc) = self.views.last().and_then(|view| self.session.get(view.id)) {
                        self.host_document = Some((doc.id, doc.edit_generation(), file.document_id));
                    }
                }
                Ok(()) => {}
                Err(e) => self.notify_fmt("Couldn't open {name}: {e}", &[("name", &file.name), ("e", &e)]),
            }
        }
    }
    pub(super) fn answer_host_selection(&self) {
        if let Some(bridge) = &self.host_selection {
            use titanos_agent_selection::{Region, Selection};
            let bound = self.host_document.as_ref().zip(self.active_ids()).filter(|((id, _, _), (_, active))| id == active);
            bridge.answer(bound.map(|((_, _, ticket), _)| ticket.as_str()), || {
                let Some(((id, generation, _), (idx, _))) = bound else { return Selection::Unbound };
                let Some(doc) = self.session.get(*id) else { return Selection::Unbound };
                if self.has_unsaved_work(idx) {
                    return Selection::Dirty;
                }
                if doc.edit_generation() != *generation {
                    return Selection::Unbound;
                }
                if self.modal_open() || self.combine_showing() {
                    return Selection::None;
                }
                Selection::numbered(self.views[idx].selected.iter().copied().map(Some), Region::Page)
            });
        }
    }
}
