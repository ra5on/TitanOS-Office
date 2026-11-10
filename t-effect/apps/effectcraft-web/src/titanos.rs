//! Saving inside a TitanOS window.
//!
//! TitanOS frames the app and catches the files it downloads (its `host.js`): each one opens the
//! save dialog of the NAS, and the window answers every such file with a
//! `{type: "webapp-saved", ok}` message once it is written, or once the dialog is cancelled or the
//! write failed.
//!
//! A project the session saved is in browser storage at that moment, not on the NAS. Until the
//! window says it was written, the session stays modified: a cancelled dialog must not leave a
//! project that looks saved while the NAS holds nothing.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use effectcraft_engine::Session;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// A file handed to the window that it has not answered yet.
enum Handed {
    /// A project file written this frame: [`settle`] decides whether it is the session's own.
    Written(String),
    /// The session's project, as of `revision`: an answer `ok` makes that state the saved one.
    Project { path: String, revision: u64 },
    /// An export, a copy, anything else: its answer changes nothing in the session.
    Other,
}

thread_local! {
    static HOSTED: Cell<Option<bool>> = const { Cell::new(None) };
    static HANDED: RefCell<VecDeque<Handed>> = const { RefCell::new(VecDeque::new()) };
}

/// Whether the page runs in a TitanOS window: framed, and opened with the `bridge` parameter the
/// window adds to the app's address.
pub fn hosted() -> bool {
    if let Some(known) = HOSTED.with(Cell::get) {
        return known;
    }
    let hosted = web_sys::window().is_some_and(|w| {
        let framed = w.parent().ok().flatten().is_some_and(|p| !js_sys::Object::is(p.as_ref(), w.as_ref()));
        let asked = w.location().search().ok().and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok()).is_some_and(|q| q.has("bridge"));
        framed && asked
    });
    HOSTED.with(|h| h.set(Some(hosted)));
    hosted
}

fn is_project(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".ecproj") || lower.ends_with(".ecprojx")
}

/// A file goes out as a download, which the window catches: `path` for a file the engine wrote
/// (maybe the project), `None` for a render output.
pub fn handed(path: Option<&str>) {
    if !hosted() {
        return;
    }
    let entry = match path {
        Some(p) if is_project(p) => Handed::Written(p.to_string()),
        _ => Handed::Other,
    };
    HANDED.with(|h| h.borrow_mut().push_back(entry));
}

/// After the frame's commands: a project file written this frame that the session took as its own
/// saved state (Save, Save As) is not on the NAS yet, so the session is modified again until the
/// window's answer. Any other project file (Save a Copy) is a copy.
pub fn settle(session: &mut Session) {
    HANDED.with(|h| {
        for entry in h.borrow_mut().iter_mut() {
            let path = match entry {
                Handed::Written(path) => std::mem::take(path),
                _ => continue,
            };
            *entry = if session.path.as_deref() == Some(path.as_str()) && !session.is_dirty() {
                session.mark_unsaved();
                Handed::Project { path, revision: session.revision }
            } else {
                Handed::Other
            };
        }
    });
}

/// The window's answer for the oldest file it has: written (`ok`), or not (cancelled, failed).
fn answered(session: &mut Session, ok: bool) {
    // A file of this frame is sorted first.
    settle(session);
    let Some(entry) = HANDED.with(|h| h.borrow_mut().pop_front()) else { return };
    if let Handed::Project { path, revision } = entry
        && ok
        && session.revision == revision
        && session.path.as_deref() == Some(path.as_str())
    {
        // Nothing changed since the save: what is on the NAS is the project as it is now.
        session.mark_saved();
    }
}

/// Hear the window's answers. Only messages of the frame's parent count.
pub fn install() {
    if !hosted() {
        return;
    }
    let Some(window) = web_sys::window() else { return };
    let parent: Option<JsValue> = window.parent().ok().flatten().map(Into::into);
    let on_message = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
        let get = |object: &JsValue, key: &str| js_sys::Reflect::get(object, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED);
        let source = get(&event, "source");
        if !parent.as_ref().is_some_and(|p| js_sys::Object::is(p, &source)) {
            return;
        }
        let data = get(&event, "data");
        if !data.is_object() || get(&data, "type").as_string().as_deref() != Some("webapp-saved") {
            return;
        }
        let ok = get(&data, "ok").as_bool() == Some(true);
        crate::post(move |app| answered(&mut app.session, ok));
    });
    if window.add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref()).is_ok() {
        // Lives as long as the page.
        on_message.forget();
    }
}
