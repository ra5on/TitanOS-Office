//! Saving inside a TitanOS window.
//!
//! TitanOS frames the app and tells it in the address how they talk (`?bridge=`):
//!
//! - `adapter` (the catalog's entry today): the window catches the files the app downloads (its
//!   `host.js`); each one opens the save dialog of the NAS.
//! - `host`: the Craft host events on the canvas, with the `effectcraft:` prefix, as the bridged
//!   programs (T-Film, T-Word) speak them. `effectcraft:ready` once the app runs,
//!   `effectcraft:open {name, bytes}` from the window, `effectcraft:save {name, bytes, saveAs}`
//!   for every file the app would download, `effectcraft:dirty {dirty}` when unsaved changes
//!   appear or go away. A Save of the project the window opened goes back into that file
//!   (`saveAs: false`); Save As and the first Save of a new project ask in the window's dialog
//!   (`saveAs: true`); exports and copies are other files (`saveAs: false` under their own name).
//!
//! Either way the window answers every file with a `{type: "webapp-saved", ok}` message once it
//! is written, or once the dialog is cancelled or the write failed.
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

/// The prefix of the host events: the original's id, which TitanOS keeps for its own builds.
const EVENTS: &str = "effectcraft";

thread_local! {
    static HOSTED: Cell<Option<bool>> = const { Cell::new(None) };
    static BRIDGED: Cell<Option<bool>> = const { Cell::new(None) };
    static HANDED: RefCell<VecDeque<Handed>> = const { RefCell::new(VecDeque::new()) };
    /// The canvas the host events are dispatched on and heard from (host mode).
    static CANVAS: RefCell<Option<web_sys::HtmlCanvasElement>> = const { RefCell::new(None) };
    /// Files the engine wrote this frame that go to the window at the next [`settle`] (host mode).
    static WRITTEN: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// The project file the window has as its document: the one it opened here, or the last one
    /// it was sent with `saveAs: true`.
    static HOST_DOCUMENT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// A name was asked for since the last save (Save As, the first Save of a new project).
    static NAME_ASKED: Cell<bool> = const { Cell::new(false) };
    /// The unsaved-changes state last told to the window.
    static TOLD_DIRTY: Cell<Option<bool>> = const { Cell::new(None) };
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

fn query_value(name: &str) -> Option<String> {
    web_sys::window().and_then(|w| w.location().search().ok()).and_then(|s| web_sys::UrlSearchParams::new_with_str(&s).ok()).and_then(|q| q.get(name))
}

/// Whether the window speaks the host events with the app (`?bridge=host`) rather than catching
/// its downloads.
pub fn bridged() -> bool {
    if let Some(known) = BRIDGED.with(Cell::get) {
        return known;
    }
    let bridged = hosted() && query_value("bridge").as_deref() == Some("host");
    BRIDGED.with(|b| b.set(Some(bridged)));
    bridged
}

/// Dispatch the host event `kind` with `detail` on the canvas. False when it could not be sent.
fn dispatch(kind: &str, detail: &js_sys::Object) -> bool {
    let Some(canvas) = CANVAS.with(|c| c.borrow().clone()) else { return false };
    let sent = (|| -> Result<bool, JsValue> {
        let constructor: js_sys::Function = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("CustomEvent"))?.dyn_into()?;
        let init = js_sys::Object::new();
        js_sys::Reflect::set(&init, &JsValue::from_str("detail"), detail)?;
        let event = js_sys::Reflect::construct(&constructor, &js_sys::Array::of2(&JsValue::from_str(&format!("{EVENTS}:{kind}")), &init))?;
        canvas.dispatch_event(event.unchecked_ref::<web_sys::Event>())
    })();
    if let Err(e) = &sent {
        log::warn!("host event {kind}: {e:?}");
    }
    sent.is_ok()
}

fn detail(fields: &[(&str, JsValue)]) -> js_sys::Object {
    let object = js_sys::Object::new();
    for (key, value) in fields {
        let _ = js_sys::Reflect::set(&object, &JsValue::from_str(key), value);
    }
    object
}

/// Hand a file to the window (host mode): `effectcraft:save`. False when it could not be sent.
pub fn send_file(name: &str, data: &[u8], save_as: bool) -> bool {
    let bytes = js_sys::Uint8Array::from(data);
    dispatch("save", &detail(&[("name", JsValue::from_str(name)), ("bytes", bytes.into()), ("saveAs", JsValue::from_bool(save_as))]))
}

/// A file the engine wrote (host mode): it goes to the window at the next [`settle`], when the
/// frame's commands have settled and the session says whether it is the project's own file.
pub fn wrote(path: &str) {
    WRITTEN.with(|w| w.borrow_mut().push(path.to_string()));
    crate::repaint();
}

/// A save dialog asked for a name (Save As, the first Save of a new project): the next project
/// save is one the window asks about, even under the open project's name.
pub fn name_asked() {
    NAME_ASKED.with(|n| n.set(true));
}

/// Host mode: hand this frame's written files to the window, then tell it when unsaved changes
/// appeared or went away.
fn sync_host(session: &Session) {
    for path in WRITTEN.with(|w| std::mem::take(&mut *w.borrow_mut())) {
        let Some(bytes) = crate::files::get(&path) else { continue };
        let name = path.rsplit('/').next().filter(|n| !n.is_empty()).unwrap_or("effectcraft-output");
        // The session's own project file becomes (or stays) the window's document; anything else
        // (Save a Copy, a still, a script's file) is another file beside it.
        let own = is_project(&path) && session.path.as_deref() == Some(path.as_str());
        let known = HOST_DOCUMENT.with(|d| d.borrow().as_deref() == Some(path.as_str()));
        let save_as = own && (NAME_ASKED.with(|n| n.replace(false)) || !known);
        if send_file(name, &bytes, save_as) {
            if save_as {
                HOST_DOCUMENT.with(|d| *d.borrow_mut() = Some(path.clone()));
            }
            handed(Some(&path));
        }
    }
}

/// Host mode: the window hears the unsaved-changes state when it changes.
fn tell_dirty(dirty: bool) {
    if TOLD_DIRTY.with(|t| t.replace(Some(dirty))) != Some(dirty) {
        dispatch("dirty", &detail(&[("dirty", JsValue::from_bool(dirty))]));
    }
}

/// `effectcraft:open {name, bytes}` from the window: the file opens like one picked with
/// File ▸ Open (a project) or imports like a dropped one (media).
fn opened(event: &JsValue) {
    let get = |object: &JsValue, key: &str| js_sys::Reflect::get(object, &JsValue::from_str(key)).unwrap_or(JsValue::UNDEFINED);
    let detail = get(event, "detail");
    let (Some(name), bytes) = (get(&detail, "name").as_string(), get(&detail, "bytes")) else { return };
    if !bytes.is_object() {
        return;
    }
    let path = crate::files::path_for(&name);
    crate::files::put(&path, js_sys::Uint8Array::new(&bytes).to_vec().into());
    if is_project(&path) {
        HOST_DOCUMENT.with(|d| *d.borrow_mut() = Some(path.clone()));
    }
    crate::post(move |app| crate::open_or_import(app, vec![path]));
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
    if bridged() {
        sync_host(session);
    }
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
    if bridged() {
        tell_dirty(session.is_dirty());
    }
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

/// Hear the window's answers (only messages of the frame's parent count) and, in host mode, the
/// files it opens here.
pub fn install(canvas: &web_sys::HtmlCanvasElement) {
    if !hosted() {
        return;
    }
    if bridged() {
        CANVAS.with(|c| *c.borrow_mut() = Some(canvas.clone()));
        let on_open = Closure::<dyn FnMut(JsValue)>::new(|event: JsValue| opened(&event));
        if canvas.add_event_listener_with_callback(&format!("{EVENTS}:open"), on_open.as_ref().unchecked_ref()).is_ok() {
            // Lives as long as the page.
            on_open.forget();
        }
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

/// Host mode: the app runs and listens, the window may send its file now.
pub fn ready() {
    if bridged() {
        dispatch("ready", &detail(&[("version", JsValue::from_str(env!("CARGO_PKG_VERSION")))]));
    }
}
