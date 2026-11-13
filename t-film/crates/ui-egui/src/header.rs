//! The header bar (Premiere 26 layout, 38 pt): Home and Import / Edit / Export on the left, the
//! document title centred, the workspace name and quick actions on the right. On macOS the menus
//! live in the native menu bar (set up by the app); elsewhere a compact in-window menu bar follows
//! the mode tabs.

use egui::{Color32, Rect, Sense, Stroke, pos2, vec2};

use crate::FilmcraftApp;
use crate::icons::{self, Icon};
use crate::state::Mode;
use crate::theme::Tokens;

/// Height of the header bar. With `integrated_titlebar` it sits in the macOS title-bar strip.
pub const HEIGHT: f32 = 38.0;

/// Draws the header bar in `rect` and handles its clicks: Home, the mode tabs, the workspace name
/// and its menu (the workspaces, then Reset to Saved Layout, Save Changes to this Workspace, Save as
/// New Workspace… and Edit Workspaces…), Discord and the quick actions.
pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, t.header_bg);
    p.line_segment([rect.left_bottom(), rect.right_bottom()], Stroke::new(1.0, Color32::BLACK));
    let drag = ui.interact(rect, egui::Id::unique("header-drag"), Sense::click_and_drag());
    if drag.drag_started_by(egui::PointerButton::Primary) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        let max = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!max));
    }
    let mut x = rect.min.x + if app.integrated_titlebar { 104.0 } else { 14.0 };
    // Home
    let home = Rect::from_center_size(pos2(x + 10.0, rect.center().y), vec2(26.0, 26.0));
    let hresp = ui.interact(home, egui::Id::unique("hdr-home"), Sense::click()).on_hover_text(tl!("Home"));
    app.auto.add("header.home", home, "Home");
    if hresp.hovered() {
        p.rect_filled(home, 4.0, t.hover);
    }
    icons::paint(&p, home.shrink(5.0), Icon::Home, if hresp.hovered() { t.text } else { t.text_dim });
    if hresp.clicked() {
        app.ui.mode = Mode::Import;
    }
    x = home.max.x + 16.0;
    // Mode tabs (14 pt; active = primary text with a 2 pt underline under the label)
    for (m, label, shown) in [(Mode::Import, "Import", tl!("Import")), (Mode::Edit, "Edit", tl!("Edit")), (Mode::Export, "Export", tl!("Export"))] {
        let active = app.ui.mode == m;
        let galley = p.layout_no_wrap(shown.to_string(), Tokens::ui(14.0), if active { t.tab_text_active } else { t.tab_text });
        let r = Rect::from_min_size(pos2(x - 6.0, rect.min.y + 6.0), vec2(galley.size().x + 12.0, rect.height() - 12.0));
        let resp = ui.interact(r, egui::Id::unique(("mode", label)), Sense::click());
        app.auto.add(&format!("header.mode.{}", label.to_ascii_lowercase()), r, label);
        let col = if active || resp.hovered() { t.tab_text_active } else { t.tab_text };
        let gw = galley.size().x;
        p.galley_with_override_text_color(pos2(x, rect.center().y - galley.size().y / 2.0), galley, col);
        if active {
            p.line_segment([pos2(x, rect.min.y + 29.0), pos2(x + gw, rect.min.y + 29.0)], Stroke::new(2.0, t.tab_text_active));
        }
        if resp.clicked() {
            app.ui.mode = m;
        }
        x += gw + 24.0;
    }
    // The right cluster: the quick actions at a 38 pt pitch from the right edge. What the window
    // has no room for goes to a "⋯" button at the left of the cluster, and the menus become one
    // "Menü" button when even that leaves them no room: nothing is drawn over anything else.
    let right = rect.max.x - 14.0;
    let slot = |i: usize| Rect::from_center_size(pos2(right - 14.0 - SLOT_PITCH * i as f32, rect.center().y), vec2(28.0, 28.0));
    let menu_x = x + 6.0;
    let compact_menus = app.ui.show_menu_bar && slot(0).min.x < menu_x + crate::menus::menu_bar_width(app, ui) + CLUSTER_GAP;
    let menus_end = if !app.ui.show_menu_bar {
        x
    } else if compact_menus {
        menu_x + crate::menus::menu_button_width(ui)
    } else {
        menu_x + crate::menus::menu_bar_width(app, ui)
    };
    let shown = shown_actions(|i| slot(i).min.x, menus_end + CLUSTER_GAP);
    // In-window menus when there is no native menu bar.
    let mut left_end = x;
    if app.ui.show_menu_bar {
        let menu_rect = Rect::from_min_max(pos2(menu_x, rect.min.y + 7.0), pos2(menus_end.max(menu_x + 1.0), rect.max.y - 7.0));
        let mut mu = ui.new_child(egui::UiBuilder::new().max_rect(menu_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
        mu.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        mu.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        mu.style_mut().visuals.override_text_color = Some(t.text_dim);
        left_end = if compact_menus { crate::menus::menu_button(app, &mut mu) } else { crate::menus::menu_bar(app, &mut mu) }.max(x);
    }
    // Document title, centred
    let title = if app.session.is_dirty() { tlf!("{name} - Edited", name = app.session.project.name) } else { app.session.project.name.clone() };
    let mut clicked = None;
    for (i, action) in QUICK_ACTIONS[..shown].iter().enumerate() {
        let r = slot(i);
        let (icon, tip) = action.look(app);
        let resp = ui.interact(r, egui::Id::unique(("hdr", action.id())), Sense::click()).on_hover_text(&tip);
        app.auto.add(&format!("header.{}", action.id()), r, &tip);
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(ui.painter(), r.shrink(6.0), icon, if resp.hovered() { t.text } else { t.text_dim });
        if *action == Quick::Appearance {
            resp.context_menu(|ui| crate::panels::settings::appearance_context_menu(app, ui));
        }
        if resp.clicked() {
            clicked = Some((*action, r));
        }
    }
    // The left edge of the cluster, and the "⋯" button with the actions that did not fit.
    let mut cluster_left = if shown == 0 { right + 10.0 } else { slot(shown - 1).min.x };
    if shown < QUICK_ACTIONS.len() {
        let r = slot(shown);
        cluster_left = r.min.x;
        let more = ui.interact(r, egui::Id::unique(("hdr", "more")), Sense::click()).on_hover_text(tl!("More"));
        app.auto.add("header.more", r, "More");
        if more.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        let dot = if more.hovered() { t.text } else { t.text_dim };
        for dx in [-5.0, 0.0, 5.0] {
            ui.painter().circle_filled(r.center() + vec2(dx, 0.0), 1.6, dot);
        }
        egui::Popup::menu(&more).show(|ui| {
            let rows: Vec<(Quick, Icon, String)> = QUICK_ACTIONS[shown..].iter().map(|action| (*action, action.look(app).0, action.look(app).1)).collect();
            let width = rows.iter().map(|(_, _, tip)| ui.painter().layout_no_wrap(tip.clone(), Tokens::ui(13.0), t.text).size().x).fold(0.0, f32::max) + 44.0;
            for (action, icon, tip) in rows {
                let (row, item) = ui.allocate_exact_size(vec2(width.max(ui.available_width()), 26.0), Sense::click());
                item.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &tip));
                app.auto.add(&format!("header.more.{}", action.id()), row, &tip);
                if item.hovered() {
                    ui.painter().rect_filled(row, 4.0, t.hover);
                }
                icons::paint(ui.painter(), Rect::from_center_size(pos2(row.min.x + 14.0, row.center().y), vec2(16.0, 16.0)), icon, t.text_dim);
                ui.painter().text(pos2(row.min.x + 30.0, row.center().y), egui::Align2::LEFT_CENTER, &tip, Tokens::ui(13.0), t.text);
                if item.clicked() {
                    clicked = Some((action, r));
                    ui.close();
                }
            }
        });
    }
    let ws_button = clicked.filter(|(action, _)| *action == Quick::Workspaces).map(|(_, r)| r);
    if let Some((action, r)) = clicked {
        match action {
            Quick::Fullscreen => {
                let fs = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fs));
            }
            Quick::Volume => app.ui.status = tl!("Master volume: use the Audio Track Mixer").into(),
            Quick::Search => app.show_panel(crate::dock::PanelKind::Effects),
            Quick::Progress => app.ui.mode = Mode::Export,
            Quick::QuickExport => {
                app.ui.export.quick_open = !app.ui.export.quick_open;
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique("quick-export-toggled"), true));
                ui.ctx().data_mut(|d| d.insert_temp(egui::Id::unique("quick-export-anchor"), r.center().x));
            }
            Quick::Appearance => crate::panels::settings::cycle_appearance(app, ui.ctx()),
            Quick::Workspaces => {}
        }
    }
    // Quick Export: a popup with File Name & Location, a preset list and Export (Premiere 26), under
    // the button that opened it.
    let qx = ui.ctx().data(|d| d.get_temp::<f32>(egui::Id::unique("quick-export-anchor"))).unwrap_or_else(|| slot(4).center().x) + 14.0;
    crate::panels::export_mode::quick_export(app, ui.ctx(), pos2((qx - 340.0).max(rect.min.x + 8.0), rect.max.y + 4.0));
    // workspace name (caps)
    let ws = crate::i18n::to_uppercase(crate::i18n::t(&app.ui.workspace));
    let wg = p.layout_no_wrap(ws.clone(), Tokens::ui(11.0), t.text_dim);
    let wr = Rect::from_min_size(pos2(cluster_left - wg.size().x, rect.center().y - 10.0), vec2(wg.size().x + 8.0, 20.0));
    // The name and the GitHub button are extras (the Workspaces button opens the same list, Help
    // has the link): in a narrow window the in-window menus reach them, and each is left out
    // rather than drawn over the menus. The name stands beside its Workspaces button only.
    let ws_fits = shown == QUICK_ACTIONS.len() && wr.min.x >= left_end + 8.0;
    let mut ws_name_clicked = false;
    if ws_fits {
        let wresp = ui.interact(wr, egui::Id::unique("hdr-ws-name"), Sense::click());
        app.auto.add("header.workspaceName", wr, &ws);
        p.galley_with_override_text_color(pos2(wr.min.x + 4.0, rect.center().y - wg.size().y / 2.0), wg, if wresp.hovered() { t.text } else { t.text_dim });
        ws_name_clicked = wresp.clicked();
    }
    // Where the right cluster begins, for what is placed left of it.
    let cluster_left = if ws_fits { wr.min.x } else { cluster_left };
    // TitanOS-Office on GitHub: a labelled button, one click away while there is room for it.
    {
        let label = "GitHub";
        let g = p.layout_no_wrap(label.to_string(), Tokens::ui(12.0), Color32::WHITE);
        let w = g.size().x + 34.0;
        let r = Rect::from_min_size(pos2(cluster_left - w - 14.0, rect.center().y - 12.0), vec2(w, 24.0));
        let button_fits = r.min.x >= left_end + 8.0;
        let mut clicked = false;
        if button_fits {
            let resp = ui.interact(r, egui::Id::unique("hdr-discord"), Sense::click()).on_hover_text("TitanOS-Office on GitHub");
            app.auto.add("header.discord", r, "TitanOS-Office on GitHub");
            ui.painter().rect_filled(r, 12.0, if resp.hovered() { t.accent_hover } else { t.accent });
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.min.x + 14.0, r.center().y), vec2(14.0, 14.0)), Icon::Code, Color32::WHITE);
            ui.painter().galley(pos2(r.min.x + 25.0, r.center().y - g.size().y / 2.0), g, Color32::WHITE);
            clicked = resp.clicked();
        }
        // Localized menus and user titles can be wider than English. Use the actual gap; a title
        // with no room for its whole name is left out (the Project panel names the project).
        let left = left_end + 12.0;
        let right = (if button_fits { r.min.x } else { cluster_left }) - 12.0;
        let galley = p.layout_no_wrap(title, Tokens::ui(14.0), t.tab_text_active);
        if galley.size().x <= right - left {
            let half = galley.size().x / 2.0;
            let center = if rect.center().x - half >= left && rect.center().x + half <= right { rect.center().x } else { (left + right) / 2.0 };
            p.galley(pos2(center - half, rect.center().y - galley.size().y / 2.0), galley, t.tab_text_active);
        }

        if clicked {
            crate::links::open(app, ui.ctx(), crate::links::DISCORD);
        }
    }
    let popup_id = egui::Id::unique("workspaces-popup");
    if ws_button.is_some() || ws_name_clicked {
        ui.ctx().data_mut(|d| d.insert_temp(popup_id, true));
        let at = if ws_name_clicked { wr.min.x } else { ws_button.map_or(wr.min.x, |r| r.min.x) };
        ui.ctx().data_mut(|d| d.insert_temp(popup_id.with("x"), at));
    }
    let open = ui.ctx().data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
    if open {
        let at = ui.ctx().data(|d| d.get_temp::<f32>(popup_id.with("x"))).unwrap_or(wr.min.x);
        let anchor = pos2(at.min(rect.max.x - 240.0).max(rect.min.x + 8.0), rect.max.y + 4.0);
        let area = egui::Area::new(popup_id.with("area")).order(egui::Order::Foreground).fixed_pos(anchor).show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                    ui.set_min_width(220.0);
                    for w in crate::dock::names(&app.workspaces) {
                        let sel = app.ui.workspace == w;
                        if ui.selectable_label(sel, crate::i18n::t(&w)).clicked() {
                            app.set_workspace(&w);
                            ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                        }
                    }
                    ui.separator();
                    // the actions of Window ▸ Workspaces, as in Premiere's workspace menu
                    let actions = [
                        ("reset", tl!("Reset to Saved Layout")),
                        ("saveChanges", tl!("Save Changes to this Workspace")),
                        ("saveAs", tl!("Save as New Workspace…")),
                        ("edit", tl!("Edit Workspaces…")),
                    ];
                    for (action, label) in actions {
                        let resp = ui.button(label);
                        app.auto.add(&format!("header.workspaces.{action}"), resp.rect, label);
                        if resp.clicked() {
                            ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
                            let ctx = ui.ctx().clone();
                            if let Err(e) = crate::menus::invoke(app, &ctx, &format!("window.workspace.{action}"), serde_json::json!({})) {
                                app.ui.status = e;
                            }
                        }
                    }
                });
            });
        });
        if area.response.clicked_elsewhere() && !(ws_button.is_some() || ws_name_clicked) {
            ui.ctx().data_mut(|d| d.insert_temp(popup_id, false));
        }
    }
}

/// The distance between two quick-action buttons.
const SLOT_PITCH: f32 = 38.0;
/// The least room between the menus and the right cluster.
const CLUSTER_GAP: f32 = 8.0;

/// A quick action at the right of the header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quick {
    Fullscreen,
    Volume,
    Search,
    Progress,
    QuickExport,
    Appearance,
    Workspaces,
}

/// The quick actions from the right edge leftwards; the last ones go to the "⋯" button first.
const QUICK_ACTIONS: [Quick; 7] = [Quick::Fullscreen, Quick::Volume, Quick::Search, Quick::Progress, Quick::QuickExport, Quick::Appearance, Quick::Workspaces];

impl Quick {
    /// The automation id (`header.<id>`).
    fn id(self) -> &'static str {
        match self {
            Quick::Fullscreen => "fullscreen",
            Quick::Volume => "volume",
            Quick::Search => "search",
            Quick::Progress => "notifications",
            Quick::QuickExport => "quickExport",
            Quick::Appearance => "appearance",
            Quick::Workspaces => "workspaces",
        }
    }

    /// Its icon and name (the tooltip, and the row in the "⋯" menu). Appearance Mode shows the
    /// mode in use.
    fn look(self, app: &FilmcraftApp) -> (Icon, String) {
        match self {
            Quick::Fullscreen => (Icon::Fullscreen, tl!("Full screen").into()),
            Quick::Volume => (Icon::Speaker, tl!("Volume").into()),
            Quick::Search => (Icon::Search, tl!("Search").into()),
            Quick::Progress => (Icon::Bell, tl!("Progress").into()),
            Quick::QuickExport => (Icon::Export, tl!("Quick Export").into()),
            Quick::Appearance => {
                let (icon, mode) = match app.session.prefs.appearance.appearance_mode.as_str() {
                    "auto" => (Icon::Monitor, tl!("Sync with system")),
                    "light" => (Icon::Sun, tl!("Light")),
                    _ => (Icon::Moon, tl!("Dark")),
                };
                (icon, format!("{}: {mode}", tl!("Appearance Mode")))
            }
            Quick::Workspaces => (Icon::Workspaces, tl!("Workspaces").into()),
        }
    }
}

/// How many quick actions get a button: all when every slot starts at or after `min_x`, else as
/// many as leave the next slot, the "⋯" button, there too. `left(i)` is the left edge of slot `i`
/// (slot 0 at the right edge).
fn shown_actions(left: impl Fn(usize) -> f32, min_x: f32) -> usize {
    let n = QUICK_ACTIONS.len();
    if left(n - 1) >= min_x {
        return n;
    }
    (0..n).rev().find(|&k| left(k) >= min_x).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use egui::{Rect, pos2, vec2};

    use crate::FilmcraftApp;
    use crate::i18n::Language;

    /// The right edge of the menus and the left edge of the right cluster (the quick actions, the
    /// "⋯" button, the workspace name and GitHub) drawn in a header `width` wide.
    fn header_edges(language: Language, width: f32) -> (f32, f32, bool) {
        crate::i18n::set_current(language);
        let mut app = FilmcraftApp::new(filmcraft_engine::Session::default());
        app.ui.language = language;
        app.ui.show_menu_bar = true;
        let mut harness = egui_kittest::Harness::builder().with_size(vec2(width, 200.0)).build_ui_state(
            |ui, app: &mut FilmcraftApp| {
                app.auto.begin_frame();
                super::show(app, ui, Rect::from_min_size(pos2(0.0, 0.0), vec2(ui.ctx().content_rect().width(), super::HEIGHT)));
            },
            app,
        );
        crate::theme::install(&harness.ctx, &harness.state().tokens);
        harness.run_steps(4);
        let elements = harness.state().auto.all();
        let rect = |e: &crate::automation::Element| Rect::from_min_max(pos2(e.rect[0], e.rect[1]), pos2(e.rect[0] + e.rect[2], e.rect[1] + e.rect[3]));
        let menus_end = elements.iter().filter(|e| e.id.starts_with("menu.")).map(|e| rect(e).max.x).fold(0.0, f32::max);
        let left_part = ["header.home", "header.mode."];
        let cluster = elements.iter().filter(|e| e.id.starts_with("header.") && !left_part.iter().any(|p| e.id.starts_with(p)));
        let cluster_start = cluster.map(|e| rect(e).min.x).fold(f32::INFINITY, f32::min);
        let compact = elements.iter().any(|e| e.id == "menu.compact");
        crate::i18n::set_current(Language::En);
        (menus_end, cluster_start, compact)
    }

    /// PLAN 80-3: the menus never run under the quick actions. The menu bar keeps every title
    /// while it fits; the actions without room go to "⋯", and in a narrow window the menus are
    /// one "Menü" button.
    #[test]
    fn the_menus_and_the_quick_actions_never_overlap() {
        for language in [Language::En, Language::De] {
            for width in [1600.0, 1280.0, 1100.0, 960.0, 800.0, 700.0, 600.0] {
                let (menus_end, cluster_start, compact) = header_edges(language, width);
                assert!(menus_end > 0.0, "{language:?} at {width}: no menus drawn");
                assert!(menus_end <= cluster_start, "{language:?} at {width}: menus end at {menus_end}, the cluster starts at {cluster_start}");
                if width >= 1100.0 {
                    assert!(!compact, "{language:?} at {width}: the menu bar has room");
                }
                if language == Language::De && width <= 800.0 {
                    assert!(compact, "{language:?} at {width}: the menu bar has no room");
                }
            }
        }
    }

    #[test]
    fn workspace_name_uppercasing_is_turkish_aware() {
        crate::i18n::set_current(crate::i18n::Language::Tr);
        assert_eq!(crate::i18n::to_uppercase("Birleştirme"), "BİRLEŞTİRME");
        assert_eq!(crate::i18n::to_uppercase("Altyazılar ve Grafikler"), "ALTYAZILAR VE GRAFİKLER");
        crate::i18n::set_current(crate::i18n::Language::En);
        assert_eq!(crate::i18n::to_uppercase("Birleştirme"), "BIRLEŞTIRME");
    }
}
