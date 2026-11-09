//! TitanOS-Office: Ctrl+S (and Ctrl+Shift+S) while a cell is being edited puts the draft in,
//! keeps the selection on the cell and saves, as the text-box editor already does.
use egui::{Event, Key, Modifiers};
use gridcraft_engine::Session;
use gridcraft_engine::core::{CellRef, Value};
use gridcraft_ui_egui::SheetApp;

type Harness = egui_kittest::Harness<'static, SheetApp>;

fn key(h: &mut Harness, k: Key, m: Modifiers) {
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: true, repeat: false, modifiers: m });
    h.input_mut().events.push(Event::Key { key: k, physical_key: Some(k), pressed: false, repeat: false, modifiers: m });
    h.run_steps(3);
}

#[test]
fn ctrl_s_while_editing_a_cell_commits_and_saves() {
    for (n, modifiers) in [Modifiers::COMMAND, Modifiers::COMMAND | Modifiers::SHIFT].into_iter().enumerate() {
        let mut s = Session::new();
        s.new_workbook();
        let mut h = crate::support::harness(s, 4);
        let downloaded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = downloaded.clone();
        h.state_mut().services.download = Some(Box::new(move |_, bytes| *sink.lock().unwrap() = bytes.to_vec()));
        let text = format!("Entwurf {n}");
        h.state_mut().begin_edit(Some(text.clone()), false);
        h.run_steps(3);
        assert!(h.state().editor.is_some(), "the cell is being edited");
        key(&mut h, Key::S, modifiers);
        assert!(h.state().editor.is_none(), "the draft went in");
        let d = h.state().session.active().unwrap();
        assert_eq!(d.selection.active, CellRef::parse("A1").unwrap(), "the selection stays on the cell");
        let bytes = downloaded.lock().unwrap().clone();
        assert!(!bytes.is_empty(), "the workbook was saved");
        let (saved, _) = gridcraft_engine::io::open_bytes("saved.xlsx", &bytes).unwrap();
        assert_eq!(saved.sheet(0).unwrap().value(CellRef::parse("A1").unwrap()), Value::text(text.as_str()));
    }
}
