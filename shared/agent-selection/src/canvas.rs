use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Array, ArrayBuffer, Object, Reflect, Uint8Array};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{CustomEvent, CustomEventInit, HtmlCanvasElement};

use super::binding::{Binding, token};
use super::{HostSaved, Region, Selection};

/// Bytes and ticket from the same accepted open event. Only bind after the file successfully opens.
pub struct HostOpen {
    pub document_id: String,
    pub name: String,
    pub bytes: Vec<u8>,
}
type Listener = (String, Closure<dyn FnMut(CustomEvent)>);

/// Installed exclusively for DOM events on the editor's own data-host=page canvas.
#[derive(Clone)]
pub struct CanvasBridge(Rc<Inner>);
struct Inner {
    canvas: HtmlCanvasElement,
    prefix: String,
    listeners: RefCell<Vec<Listener>>,
    binding: RefCell<Binding>,
    open: RefCell<Option<HostOpen>>,
}

impl Drop for Inner {
    fn drop(&mut self) {
        for (name, listener) in self.listeners.get_mut().drain(..) {
            let _ = self.canvas.remove_event_listener_with_callback_and_bool(
                &name,
                listener.as_ref().unchecked_ref(),
                true,
            );
        }
    }
}
fn prop(obj: &JsValue, key: &str) -> JsValue {
    Reflect::get(obj, &key.into()).unwrap_or(JsValue::UNDEFINED)
}
fn valid_token(obj: &JsValue, key: &str) -> Option<String> {
    prop(obj, key).as_string().filter(|s| token(s))
}
impl CanvasBridge {
    /// Register before the ordinary host listener. Opens without a valid revision keep their normal path.
    pub fn install(
        canvas: &HtmlCanvasElement,
        prefix: &str,
        max_bytes: usize,
        wake: impl Fn() + 'static,
    ) -> Option<Self> {
        if canvas.get_attribute("data-host").as_deref() != Some("page") {
            return None;
        }
        let bridge = Self(Rc::new(Inner {
            canvas: canvas.clone(),
            prefix: prefix.to_owned(),
            listeners: RefCell::new(Vec::new()),
            binding: RefCell::default(),
            open: RefCell::default(),
        }));
        let wake = Rc::new(wake);
        let repaint = wake.clone();
        bridge.listen("open", move |inner, event| {
            inner.binding.borrow_mut().open(None);
            inner.open.borrow_mut().take();
            let obj = event.detail();
            let Some(id) = valid_token(&obj, "documentId") else {
                return;
            };
            let Some(name) = prop(&obj, "name").as_string() else {
                return;
            };
            if name.is_empty()
                || name.chars().count() > 255
                || name.contains(['/', '\\'])
                || name.chars().any(char::is_control)
            {
                return;
            }
            let bytes = prop(&obj, "bytes");
            let view = if let Some(view) = bytes.dyn_ref::<Uint8Array>() {
                view.clone()
            } else if bytes.is_instance_of::<ArrayBuffer>() {
                Uint8Array::new(&bytes)
            } else {
                return;
            };
            if view.length() == 0 || view.length() as usize > max_bytes {
                return;
            }
            // Capture the ticket and *these bytes* together, so the regular handler must not open them a second time.
            event.stop_immediate_propagation();
            inner.binding.borrow_mut().open(Some(id.clone()));
            *inner.open.borrow_mut() = Some(HostOpen {
                document_id: id,
                name,
                bytes: view.to_vec(),
            });
            repaint();
        })?;
        bridge.listen("save", |inner, event| {
            inner.open.borrow_mut().take();
            inner
                .binding
                .borrow_mut()
                .begin_save(prop(&event.detail(), "id").as_string());
        })?;
        let repaint = wake.clone();
        bridge.listen("saved", move |inner, event| {
            let obj = event.detail();
            if let (Some(id), Some(doc)) = (
                prop(&obj, "id").as_string(),
                valid_token(&obj, "documentId"),
            ) {
                inner.binding.borrow_mut().saved(id, doc);
                repaint();
            }
        })?;
        bridge.listen("agent-selection-request", move |inner, event| {
            let obj = event.detail();
            if prop(&obj, "version").as_f64() != Some(1.0) {
                return;
            }
            if let (Some(id), Some(doc)) = (
                valid_token(&obj, "requestId"),
                valid_token(&obj, "documentId"),
            ) {
                inner.binding.borrow_mut().request(id, doc);
                wake();
            }
        })?;
        Some(bridge)
    }
    fn listen(&self, suffix: &str, mut f: impl FnMut(&Inner, CustomEvent) + 'static) -> Option<()> {
        let weak = Rc::downgrade(&self.0);
        let listener = Closure::new(move |event: CustomEvent| {
            if let Some(inner) = weak.upgrade()
                && event
                    .target()
                    .as_ref()
                    .is_some_and(|t| t == inner.canvas.unchecked_ref::<web_sys::EventTarget>())
            {
                f(&inner, event);
            }
        });
        let name = format!("{}:{suffix}", self.0.prefix);
        self.0
            .canvas
            .add_event_listener_with_callback_and_bool(
                &name,
                listener.as_ref().unchecked_ref(),
                true,
            )
            .ok()?;
        self.0.listeners.borrow_mut().push((name, listener));
        Some(())
    }
    pub fn take_saved(&self) -> Option<HostSaved> {
        self.0.binding.borrow_mut().take_saved()
    }
    /// Call only after matching the native file and serialized revision to this exact save.
    pub fn accept_saved(&self, saved: &HostSaved) -> bool {
        self.0.binding.borrow_mut().accept_saved(saved)
    }
    pub fn take_open(&self) -> Option<HostOpen> {
        self.0.open.borrow_mut().take()
    }

    /// Call from the editor frame with the ticket of the active, successfully opened native document.
    pub fn answer(&self, bound: Option<&str>, selected: impl FnOnce() -> Selection) {
        let Some((id, doc, value)) = self.0.binding.borrow_mut().take(bound, selected) else {
            return;
        };
        let attrs = Object::new();
        let set = |key: &str, value: &JsValue| Reflect::set(&attrs, &key.into(), value).ok();
        let items = Array::new();
        let status = match value {
            Selection::None => "none",
            Selection::Dirty => "dirty",
            Selection::Unsupported => "unsupported",
            Selection::Unbound => "unbound",
            Selection::Selected(regions) => {
                for item in regions {
                    let obj = Object::new();
                    let field = |k: &str, v: JsValue| Reflect::set(&obj, &k.into(), &v).ok();
                    match item {
                        Region::Page(n) => field("page", n.into()),
                        Region::Slide(n) => field("slide", n.into()),
                        Region::Paragraph(n) => field("paragraph", n.into()),
                        Region::Cells { sheet, range } => {
                            field("sheet", sheet.into());
                            field("range", range.into())
                        }
                    };
                    items.push(&obj);
                }
                "selected"
            }
        };
        set("version", &1.into());
        set("requestId", &id.into());
        set("documentId", &doc.into());
        set("status", &status.into());
        set("items", &items);
        let event = CustomEventInit::new();
        event.set_detail(&attrs);
        if let Ok(event) = CustomEvent::new_with_event_init_dict(
            &format!("{}:agent-selection", self.0.prefix),
            &event,
        ) {
            let _ = self.0.canvas.dispatch_event(&event);
        }
    }
}
