use egui::Context;
use minimap_core::structs::ui::{MapAction, UiFrame};

use crate::runtime::UiDrafts;
use crate::views::{DialogView, HintView, ListView, MenuView, ToastView, ToolsView};

// every UI panel for one frame: on the world map the tools card, the list, the hint bar, the menu
// and the dialog; on either map the message line. The panels push what they ask for onto `actions`
pub struct MapScreen;

impl MapScreen {
    pub fn show(ctx: &Context, frame: &UiFrame<'_>, drafts: &mut UiDrafts, actions: &mut Vec<MapAction>) {
        if frame.panels {
            let tools_bottom = ToolsView::show(ctx, frame, actions);
            if frame.ui.list_open() {
                ListView::show(ctx, frame, tools_bottom, drafts, actions);
            }
            HintView::show(ctx, frame.keys);
            if let Some(menu) = frame.ui.menu() {
                MenuView::show(ctx, frame, menu, actions);
            }
            if let Some(dialog) = frame.ui.dialog() {
                DialogView::show(ctx, dialog, drafts, actions);
            }
        }
        if let Some(message) = frame.ui.message() {
            ToastView::show(ctx, message);
        }
    }
}
