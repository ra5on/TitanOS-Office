//! The program's wordmark. TitanOS-Office: the ArtCraft wordmark (docs/brand/) is removed, as
//! the brand licence requires for modified versions; the name is drawn as plain text instead.

/// Draw the wordmark `height` points tall with its left edge at `left_center` (vertically centred).
/// Returns the drawn rect.
pub fn paint_wordmark(ui: &egui::Ui, left_center: egui::Pos2, height: f32, dark: bool) -> Option<egui::Rect> {
    let color = if dark { egui::Color32::WHITE } else { egui::Color32::from_rgb(0x1a, 0x1a, 0x1a) };
    let r = ui.painter().text(left_center, egui::Align2::LEFT_CENTER, "T-Film", egui::FontId::proportional(height), color);
    Some(r)
}
