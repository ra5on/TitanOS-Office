//! The narrow layout's chrome ([`crate::compact`], phones): what the top bars keep, what moves into
//! their "⋯" menus, and the bar below the page that takes the rail's page and zoom controls. Every
//! control is at least [`TOUCH`] square, and nothing is cut short or drawn over another.

use egui::{Align, Layout, Rect, Sense, Stroke, vec2};

use crate::compact::TOUCH;
use crate::theme::{self, ThemePreference, Tokens};
use crate::{Dialog, Mode, PdfCraftApp, PropsTab, RightPanel, icons};

/// The workspaces the mode bar offers, as its tabs do on a wider window.
const MODES: [(Mode, &str); 4] = [(Mode::AllTools, "All tools"), (Mode::Edit, "Edit"), (Mode::Convert, "Convert"), (Mode::Sign, "E-Sign")];

/// The side panels the mode bar's "⋯" opens, as the rail does on a wider window.
const PANELS: [(RightPanel, &str, &str); 8] = [
    (RightPanel::Comments, "message-square-text", "Comments"),
    (RightPanel::Bookmarks, "bookmark", "Bookmarks"),
    (RightPanel::Pages, "files", "Page thumbnails"),
    (RightPanel::Fields, "text-cursor-input", "Form fields"),
    (RightPanel::Layers, "layers", "Layers"),
    (RightPanel::Attachments, "paperclip", "Attachments"),
    (RightPanel::Signatures, "signature", "Signatures"),
    (RightPanel::Tags, "tag", "Tags"),
];

/// A button that opens a menu: its label and a chevron, [`TOUCH`] tall.
fn menu_button(ui: &mut egui::Ui, label: &str, font: egui::FontId, tip: &str) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let galley = crate::bidi::layout_ui(ui, label.to_owned(), font, t.text);
    let (rect, resp) = ui.allocate_exact_size(vec2(galley.size().x + 36.0, TOUCH), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), label));
    let press = theme::Press::track(ui, &resp);
    press.wash(ui, rect.shrink2(vec2(0.0, 4.0)), 6, false);
    let body = rect.translate(press.offset());
    ui.painter().galley(body.left_center() + vec2(10.0, -galley.size().y / 2.0), galley, t.text);
    icons::paint(ui, Rect::from_center_size(body.right_center() - vec2(15.0, 0.0), vec2(14.0, 14.0)), "chevron-down", 13.0, t.text_muted);
    theme::hand(resp).on_hover_text(tip)
}

/// One entry of a "⋯" menu: its icon and label, as tall as a touch target.
fn entry(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let width = ui.available_width().max(200.0);
    let galley = crate::bidi::layout_ui(ui, label.to_owned(), theme::regular(14.0), t.text);
    let (rect, resp) = ui.allocate_exact_size(vec2(width.max(galley.size().x + 52.0), TOUCH), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, ui.is_enabled(), selected, label));
    let press = theme::Press::track(ui, &resp);
    press.wash(ui, rect, 6, selected);
    let tint = if selected { t.accent_text } else { t.icon };
    icons::paint(ui, Rect::from_center_size(rect.left_center() + vec2(20.0, 0.0), vec2(18.0, 18.0)), icon, 17.0, tint);
    ui.painter().galley(rect.left_center() + vec2(42.0, -galley.size().y / 2.0), galley, t.text);
    theme::hand(resp)
}

/// The tab strip's "⋯": what sits at its right end on a wider window.
pub(super) fn strip_menu(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let resp = icons::button(ui, "ellipsis", TOUCH, false, tl!("More"));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(240.0);
        if entry(ui, "settings", tl!("Preferences"), false).clicked() {
            app.execute("app.preferences");
            ui.close();
        }
        for (pref, icon, label, command) in [
            (ThemePreference::System, "monitor", "Use system setting", "view.theme.system"),
            (ThemePreference::Light, "sun", "Light gray", "view.theme.light"),
            (ThemePreference::Dark, "moon", "Dark gray", "view.theme.dark"),
        ] {
            let shown = format!("{}: {}", tl!("Display theme"), tl!(label));
            if entry(ui, icon, &shown, app.theme_preference == pref).clicked() {
                app.execute(command);
                ui.close();
            }
        }
        if entry(ui, "circle-help", tl!("Keyboard shortcuts"), false).clicked() {
            app.dialog = Some(Dialog::Shortcuts);
            ui.close();
        }
        if entry(ui, "messages-square", "GitHub", false).clicked() {
            app.execute("help.discord");
            ui.close();
        }
    });
}

/// The mode bar after its menus: the workspace as a menu instead of four tabs, the search button,
/// and a "⋯" with the document's buttons and the side panels the rail opens on a wider window.
pub(super) fn mode_bar_rest(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let current = MODES.iter().find(|(m, _)| *m == app.mode).map_or_else(|| tl!("Read mode"), |(_, label)| tl!(label));
    let resp = menu_button(ui, current, theme::semibold(13.5), tl!("Workspace"));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(220.0);
        for (mode, label) in MODES {
            let icon = match mode {
                Mode::Edit => "file-pen-line",
                Mode::Convert => "file-output",
                Mode::Sign => "signature",
                _ => "layout-grid",
            };
            // Chosen again, the workspace shows its tools once more.
            if entry(ui, icon, tl!(label), app.mode == mode).clicked() {
                app.switch_mode(mode, true);
                ui.close();
            }
        }
    });
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        document_menu(app, ui);
        let tip = format!("{} ({})", tl!("Find tools and commands"), crate::commands::command_shortcut_label(ui.ctx(), "view.palette"));
        if icons::button(ui, "search", TOUCH, false, &tip).clicked() {
            app.palette_open = true;
        }
        // A phone on its side: the page and zoom controls here, and the height to the page.
        if nav_in_mode_bar(ui.ctx())
            && let Some((index, count)) = nav_view(app)
        {
            let width = app.views.get(index).map_or(f32::INFINITY, |view| nav_widths(ui, view, count).2);
            if ui.available_width() >= width + NAV_GAP {
                ui.add_space(NAV_GAP);
                ui.allocate_ui_with_layout(vec2(width, TOUCH), Layout::left_to_right(Align::Center), |ui| nav_controls(app, ui, index, count, false));
            }
        }
    });
}

/// The mode bar's "⋯": save, print and properties, and the side panels.
fn document_menu(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let has_doc = app.active.is_some();
    let resp = icons::button(ui, "ellipsis", TOUCH, false, tl!("More"));
    egui::Popup::menu(&resp).show(|ui| {
        ui.set_min_width(240.0);
        ui.add_enabled_ui(has_doc, |ui| {
            if entry(ui, "save", tl!("Save"), false).clicked() {
                app.run_command("file.save");
                ui.close();
            }
            let may_print = pdfcraft_engine::commands::command("print.dialog").is_some_and(|c| app.command_enabled(c));
            let print = ui.add_enabled_ui(may_print, |ui| entry(ui, "printer", tl!("Print…"), false)).inner;
            if !may_print {
                print.on_disabled_hover_text(crate::print_refused());
            } else if print.clicked() {
                app.run_command("print.dialog");
                ui.close();
            }
            if entry(ui, "info", tl!("Document Properties"), false).clicked() {
                app.dialog = Some(Dialog::Properties(PropsTab::Description));
                ui.close();
            }
            ui.separator();
            for (panel, icon, label) in PANELS {
                if entry(ui, icon, tl!(label), app.right == Some(panel)).clicked() {
                    app.choose_right_panel(Some(panel));
                    ui.close();
                }
            }
        });
    });
}

/// Wide enough (a phone on its side) that the page and zoom controls fit in the mode bar, which
/// leaves the short window's height to the page.
const NAV_IN_MODE_BAR: f32 = 600.0;

/// Where the page and zoom controls go in the narrow layout: the mode bar when the window is wide
/// enough, else a bar of their own below the page.
fn nav_in_mode_bar(ctx: &egui::Context) -> bool {
    ctx.content_rect().width() >= NAV_IN_MODE_BAR
}

/// The view the page and zoom controls act on, when there is a page to move through: not over a
/// pane, Organize Pages or the expanded All tools view.
fn nav_view(app: &PdfCraftApp) -> Option<(usize, usize)> {
    if !app.compact || app.compact_pane.is_some() || app.mode == Mode::AllTools && app.all_tools_full {
        return None;
    }
    let (index, id) = app.active_ids()?;
    let count = app.session.get(id)?.info.pages.len();
    (count > 0 && !app.views.get(index)?.organize).then_some((index, count))
}

/// The rail's page and zoom controls, left to right: the page and the steps to the one before and
/// after, and the zoom, whose menu has the rail's view choices (fit width, fit page, page display,
/// read mode). Runs the command a menu entry asks for.
/// Between the page steps and the zoom when the controls sit together in the mode bar.
const NAV_GAP: f32 = 8.0;

/// What the page and zoom controls show: "3 / 12" and "48%".
fn nav_texts(view: &crate::canvas::DocView, count: usize) -> (usize, String, String) {
    // Out of range (an empty document): never past the end, as the rail.
    let shown = if view.current < count { view.current + 1 } else { count };
    (shown, format!("{shown} / {count}"), format!("{:.0}%", view.zoom * 100.0))
}

/// The page label's width, the zoom menu button's, and the width of all of [`nav_controls`]
/// side by side.
fn nav_widths(ui: &egui::Ui, view: &crate::canvas::DocView, count: usize) -> (f32, f32, f32) {
    let (_, page, zoom) = nav_texts(view, count);
    let width = |text: String| ui.painter().layout_no_wrap(text, theme::medium(13.0), egui::Color32::WHITE).size().x;
    let page = width(page).max(40.0) + 12.0;
    let zoom = width(zoom) + 36.0;
    (page, zoom, 4.0 * TOUCH + page + zoom + NAV_GAP)
}

fn nav_controls(app: &mut PdfCraftApp, ui: &mut egui::Ui, index: usize, count: usize, spread: bool) {
    let t = Tokens::get(ui.ctx());
    let Some(view) = app.views.get_mut(index) else { return };
    let (page_w, zoom_w, _) = nav_widths(ui, view, count);
    let (shown, page, zoom) = nav_texts(view, count);
    ui.spacing_mut().item_spacing.x = 0.0;
    if icons::button(ui, "chevron-up", TOUCH, false, tl!("Previous page")).clicked() {
        view.step_page(false);
    }
    let of = crate::i18n::fmt(tl!("Page {p} of {n}"), &[("p", &shown.to_string()), ("n", &count.to_string())]);
    let galley = ui.painter().layout_no_wrap(page, theme::medium(13.0), t.text);
    let (rect, resp) = ui.allocate_exact_size(vec2(page_w, TOUCH), Sense::hover());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &of));
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, t.text);
    if icons::button(ui, "chevron-down", TOUCH, false, tl!("Next page")).clicked() {
        view.step_page(true);
    }
    // In a bar of their own, the zoom goes to the far end.
    let gap = if spread { ui.available_width() - 2.0 * TOUCH - zoom_w } else { NAV_GAP };
    ui.add_space(gap.max(NAV_GAP));
    let tip = crate::commands::key_tip(ui.ctx(), tl!("Zoom out ({key})"), crate::commands::ZOOM_OUT);
    if icons::button(ui, "zoom-out", TOUCH, false, &tip).clicked() {
        view.zoom_step(false);
    }
    let tip = crate::i18n::fmt(tl!("Page display: {layout}"), &[("layout", tl!(view.layout.label()))]);
    let resp = menu_button(ui, &zoom, theme::medium(13.0), &tip);
    let picked = egui::Popup::menu(&resp)
        .show(|ui| {
            ui.set_min_width(240.0);
            super::rail_view_menu(ui, view)
        })
        .and_then(|r| r.inner);
    let tip = crate::commands::key_tip(ui.ctx(), tl!("Zoom in ({key})"), crate::commands::ZOOM_IN);
    if icons::button(ui, "zoom-in", TOUCH, false, &tip).clicked() {
        view.zoom_step(true);
    }
    if let Some(command) = picked {
        app.execute(command);
    }
}

/// The bar below the page in the narrow layout, in place of the rail, when the mode bar has no
/// room for its controls ([`nav_controls`]): the page steps at its start, the zoom at its end.
pub fn compact_nav(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    if nav_in_mode_bar(ui.ctx()) {
        return;
    }
    let Some((index, count)) = nav_view(app) else { return };
    let t = Tokens::get(ui.ctx());
    egui::Panel::bottom("compact_nav")
        .exact_size(TOUCH + 4.0)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(egui::Margin::symmetric(6, 2)).stroke(Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| nav_controls(app, ui, index, count, true));
        });
}
