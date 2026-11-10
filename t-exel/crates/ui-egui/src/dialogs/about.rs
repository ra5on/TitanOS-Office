use super::Dialog;

/// Help ▸ About GridCraft: the app, its contributors and the AI models that helped. The credits
/// are compiled in (`crate::credits`, docs/contributors.md).
pub(super) fn about(ui: &mut egui::Ui, d: &mut Dialog) {
    ui.horizontal(|ui| {
        for tab in ["About", "Contributors", "Models"] {
            if ui.selectable_label(d.tab == tab, tab).clicked() {
                d.tab = tab.to_string();
            }
        }
    });
    ui.separator();
    match d.tab.as_str() {
        "Contributors" => crate::credits::contributors_ui(ui),
        "Models" => crate::credits::models_ui(ui),
        _ => {
            ui.heading("T-Exel");
            ui.label(format!("Version {}", option_env!("TITANOS_OFFICE_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))));
            ui.add_space(4.0);
            ui.label("The spreadsheet of TitanOS, a modified version of GridCraft (MIT OR Apache-2.0).");
            ui.hyperlink_to("Source code: github.com/ra5on/TitanOS-Office", "https://github.com/ra5on/TitanOS-Office");
            ui.hyperlink_to("Original: github.com/storytold/gridcraft", "https://github.com/storytold/gridcraft");
        }
    }
}
