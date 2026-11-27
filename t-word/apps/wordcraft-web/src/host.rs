//! Host modes: a page that runs WordCraft (as an `<iframe>`, or in its own document) opens
//! documents in it and stores what it saves instead of the browser downloading. The messages and
//! the two transports are `craft-web-host`'s (storytold/craft-libs), the same contract as the
//! other Craft apps' bridges, with the `wordcraft:` prefix. This module is WordCraft's part of it.
//!
//! - `?host=parent` (an `<iframe>`, such as a Nextcloud app) talks to the same-origin parent over
//!   `postMessage`; `data-host="page"` on the canvas (the page's own document, no iframe) uses
//!   DOM `CustomEvent`s on the canvas instead. `?host=parent` wins when both are present. With no
//!   parent, or a parent on another origin, there is no bridge: the normal app with downloads.
//! - `wordcraft:ready` once the app is running and listening; send a file now. WordCraft reads
//!   bytes, so it says `file: false` and a host sends `bytes`.
//! - `wordcraft:open` (from the host): `{ name, bytes }`. It opens like a document picked with
//!   File › Open or dropped on the window; a file over [`MAX_OPEN_BYTES`] is refused before it is
//!   copied.
//! - `wordcraft:save` (to the host): `{ name, saveAs, bytes }` for every file WordCraft would
//!   otherwise download. The host stores it and reports failures itself. `saveAs` says whether
//!   the file becomes the document's file:
//!   - Save: `false`. It writes the opened file under its own name and format, as the desktop
//!     app does. A name the parent never sent (an untitled document, an imported .doc saved as
//!     .docx) is a new file: `true`, as for Save As.
//!   - Save As, and File › Export to another document format (which runs Save As): `true` for a
//!     format that keeps everything (.docx and the other Word packages, .odt, .rtf). The document
//!     is now that file; the next Save sends its name with `false`.
//!   - Save As or Export to a format that doesn't (PDF, Markdown, plain text), Export as PDF and
//!     Export Page as PNG: `false`. They are copies; the document's file stays the same.
//! - `wordcraft:dirty` whenever unsaved changes appear or go away, `wordcraft:failed` when the
//!   app couldn't start.
//!
//! In both host modes the host sends the document to edit, so no sample opens, saves are handed
//! to the host rather than downloaded, the host guards leaving the page, and `?author=` names the
//! user for comments and tracked changes.

use crate::agent::{self, BoundDocument};
use std::cell::RefCell;
use std::rc::Rc;
use titanos_agent_selection::CanvasBridge;
use wordcraft_docx::SavedParagraphs;
use wordcraft_ui_egui::async_saves::{AsyncSaves, SaveOutcome};

use craft_web_host::web::{FileData, Host, Message};
use wordcraft_ui_egui::{HostDocument, Inbox, Services, WordApp};

/// The app id in the host messages (`wordcraft:ready`).
const APP: &str = "wordcraft";

/// The name a document gets when the host sent none (the crate says `document` for a blank name).
const DEFAULT_NAME: &str = "Document.docx";

/// The largest document taken from the host page, refused before it is copied.
const MAX_OPEN_BYTES: u64 = 512 << 20;

/// The longest user name taken from `?author=`.
const MAX_AUTHOR_CHARS: usize = 200;

#[derive(Clone)]
pub struct Bridge {
    host: Rc<Host>,
    canvas: web_sys::HtmlCanvasElement,
    agent: Rc<RefCell<AgentState>>,
}

#[derive(Default)]
struct AgentState {
    bridge: Option<CanvasBridge>,
    bound: Option<BoundDocument>,
    pending: Option<SaveTicket>,
    scoped_host: bool,
    saves: AsyncSaves,
}

struct SaveTicket {
    ui_ticket: u64,
    id: String,
    native: HostDocument,
    source: Option<SavedParagraphs>,
}

impl Bridge {
    /// The bridge for this page, or `None` when WordCraft isn't hosted: no `?host=parent` and no
    /// `data-host="page"` on the canvas, an unframed `?host=parent` page (it stays the normal app
    /// with downloads), or a parent on another origin.
    pub fn detect(canvas: &web_sys::HtmlCanvasElement) -> Option<Self> {
        Some(Self { host: Rc::new(Host::connect(APP, canvas)?), canvas: canvas.clone(), agent: Default::default() })
    }

    /// Hand saved and exported files to the host instead of downloading them, report unsaved
    /// changes to it, and hear the files it sends.
    pub fn connect(&self, services: &mut Services, inbox: Inbox, ctx: egui::Context) {
        let wake = ctx.clone();
        self.agent.borrow_mut().bridge = CanvasBridge::install(&self.canvas, APP, MAX_OPEN_BYTES as usize, move || wake.request_repaint());
        self.listen(inbox, ctx);
        let bridge = self.clone();
        services.host_save = Some(Box::new(move |name: &str, bytes: &[u8], save_as: bool, native| bridge.post_save(name, bytes, save_as, native)));
        // Anything else that would download goes to the host too.
        let bridge = self.clone();
        services.download = Some(Box::new(move |name: &str, bytes: &[u8]| bridge.post_save(name, bytes, false, None)));
        // NAS saves remain pending until this exact ticket receives a receipt.
        services.async_saves = Some(self.agent.borrow().saves.clone());
        // The crate does not repeat a state it already reported.
        let host = self.host.clone();
        services.on_dirty = Some(Box::new(move |dirty: bool| host.post_dirty(dirty)));
    }

    /// `?author=`: the host's name for the user signs comments and tracked changes.
    pub fn set_author(app: &mut WordApp, author: &str) {
        let author: String = author.trim().chars().take(MAX_AUTHOR_CHARS).collect();
        if !author.is_empty() {
            app.session.author = author;
        }
    }

    pub fn post_ready(&self) {
        let _ = self.host.post_ready(env!("CARGO_PKG_VERSION"), false);
    }

    /// The app couldn't start (no WebGPU or WebGL2, for example): let the host say so.
    pub fn post_failed(&self, error: &str) {
        let _ = self.host.post_failed(error, None);
    }

    fn post_save(&self, path: &str, bytes: &[u8], save_as: bool, native: Option<HostDocument>) -> Result<(), String> {
        let name = std::path::Path::new(path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string());
        let (ticket, save_as) = {
            let mut state = self.agent.borrow_mut();
            let Some(native) = native.filter(|_| state.bridge.is_some()) else {
                return self.host.post_save(&name, bytes, None, save_as);
            };
            // Keep ordinary page hosts that never sent a NAS ticket on their
            // historical save path. Agent data is bound only after a real HostOpen.
            if !state.scoped_host {
                return self.host.post_save(&name, bytes, None, save_as);
            }
            if state.pending.is_some() {
                return Err("The previous document save is still awaiting its host acknowledgement.".to_owned());
            }
            let ui_ticket = state.saves.start();
            let id = format!("word-{}-{}-{}", native.document, native.view, ui_ticket);
            let replace = state.bound.as_ref().is_some_and(|bound| bound.same_view(native));
            state.pending = Some(SaveTicket { ui_ticket, id: id.clone(), native, source: agent::source(&name, bytes) });
            (id, save_as || !replace)
        };
        let result = self.host.post_save(&name, bytes, Some(&ticket), save_as);
        if let Err(err) = &result {
            let mut state = self.agent.borrow_mut();
            if let Some(save) = state.pending.take() {
                state.saves.finish(save.ui_ticket, SaveOutcome::Failed(err.clone()));
            }
        }
        result
    }

    pub fn begin_frame(&self, app: &mut WordApp) {
        let Some(canvas) = self.agent.borrow().bridge.clone() else {
            return;
        };
        if let Some(open) = canvas.take_open() {
            {
                let mut state = self.agent.borrow_mut();
                state.scoped_host = true;
                state.bound = None;
                if let Some(save) = state.pending.take() {
                    state.saves.finish(save.ui_ticket, SaveOutcome::Cancelled);
                }
            }
            self.agent.borrow_mut().bound = BoundDocument::open(app, open.document_id, &open.name, &open.bytes);
        }
        if let Some(id) = canvas.take_failed() {
            let mut state = self.agent.borrow_mut();
            if state.pending.as_ref().is_some_and(|save| save.id == id)
                && let Some(save) = state.pending.take()
            {
                state.saves.finish(save.ui_ticket, SaveOutcome::Failed("The host could not save the document.".to_owned()));
            }
        }
        if let Some(saved) = canvas.take_saved() {
            let mut state = self.agent.borrow_mut();
            if state.pending.as_ref().is_some_and(|save| save.id == saved.id)
                && let Some(save) = state.pending.take()
            {
                state.saves.finish(save.ui_ticket, SaveOutcome::Saved);
                if save.native == app.host_document() && app.has_document() && !app.session.dirty && canvas.accept_saved(&saved) {
                    state.bound = Some(BoundDocument::new(saved.document_id, save.native, save.source));
                }
            }
        }
    }

    pub fn after_input(&self, app: &WordApp) {
        let state = self.agent.borrow();
        if let Some(canvas) = &state.bridge {
            let bound = state.bound.as_ref().filter(|bound| bound.same_view(app.host_document()));
            canvas.answer(bound.map(|b| b.ticket.as_str()), || bound.map_or(titanos_agent_selection::Selection::Unbound, |b| b.selection(app)));
        }
    }

    /// Hear `wordcraft:open` and queue the file like a dropped one.
    fn listen(&self, inbox: Inbox, ctx: egui::Context) {
        self.host.listen(move |message| {
            if let Message::Open { name, data } = message {
                deliver_open(&inbox, &ctx, &name, &data);
            }
        });
    }
}

/// The file an open message carries, pushed onto the inbox for the next frame, like a dropped
/// file. A host's file is untrusted input: an oversized one is refused before it is copied into
/// the app's memory (the data's own length, not the host's claim). The crate has cleaned the name
/// (and cut it to 255 characters); a blank one comes as `document` and becomes `Document.docx`.
fn deliver_open(inbox: &Inbox, ctx: &egui::Context, name: &str, data: &FileData) {
    let name = if name == "document" { DEFAULT_NAME } else { name }.to_string();
    let FileData::Bytes(array) = data else {
        log::warn!("host file {name} came as a File; T-Word reads bytes only; not opened");
        return;
    };
    if data.size() > MAX_OPEN_BYTES as f64 {
        log::warn!("host file is {} bytes, over the {} MiB host open limit; not opened", data.size(), MAX_OPEN_BYTES >> 20);
        return;
    }
    inbox.lock().unwrap_or_else(|e| e.into_inner()).push((name, array.to_vec()));
    ctx.request_repaint();
}
