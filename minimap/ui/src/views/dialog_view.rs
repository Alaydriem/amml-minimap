use amml_i18n::localize::Localizer;
use amml_i18n::text::{Text, UserText};
use amml_ui::AmmlUiMessage;
use amml_ui::draw::{TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::text::UiLocale;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::selector::AxisColor;
use amml_ui::tokens::{ColorToken, ContainerToken, SpacingToken, color, container, spacing};
use amml_ui::widget::{
    ButtonKind, IconButton, IconButtonSize, SectionHeader, Surface, SurfaceLevel, TextButton, TextField, TextFieldKind,
};
use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Align2, Area, Color32, Context, Key, LayerId, Layout, Order, Sense, Stroke, Ui, pos2, vec2};
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{DialogTarget, MapAction, WaypointDialog};

use crate::id::MinimapUiId;
use crate::message::MinimapMessage;
use crate::runtime::UiDrafts;
use crate::views::{ListView, SplitRow, ToggleRow, Words};

// the waypoint dialog over a dimmed map: the name, the colour, the two minimap switches, the
// position, Cancel and Save. Enter saves and Escape cancels, in order with the typed text
pub struct DialogView;

impl DialogView {
    const SWATCH: f32 = 15.0;
    const SWATCH_GAP: f32 = 46.0;
    const SWATCH_ROW: f32 = 40.0;
    pub const SWATCHES_PER_ROW: usize = 6;
    const PADDING: i8 = 24;
    const TITLE: TextStyle = TextStyle::new(TypeSize::Title, FontFace::Semibold);
    const POSITION_SIZE: f32 = 14.0;

    pub fn show(ctx: &Context, dialog: &WaypointDialog, drafts: &mut UiDrafts, actions: &mut Vec<MapAction>) {
        let opened = drafts.dialog_serial != dialog.serial;
        if opened {
            drafts.name.clone_from(&dialog.initial_name);
            drafts.dialog_serial = dialog.serial;
        }
        Self::dim(ctx);
        let id = MinimapUiId::Dialog.id();
        let medium = <spacing::M as SpacingToken>::VALUE;
        Area::new(id).anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO).order(Order::Foreground).show(ctx, |ui| {
            let frame = Surface::frame(SurfaceLevel::Card, Elevation::Floating).inner_margin(Self::PADDING);
            let inner_margin = frame.total_margin().sum().x;
            frame.show(ui, |ui| {
                ui.set_width(<container::Dialog as ContainerToken>::VALUE - inner_margin);
                Self::title(ui, dialog, actions);
                ui.add_space(medium);
                SectionHeader::new(Text::message(MinimapMessage::Name)).show(ui);
                Self::name(ui, drafts, opened);
                ui.add_space(medium);
                SectionHeader::new(Text::message(MinimapMessage::Colour)).show(ui);
                Self::swatches(ui, dialog.colour, actions);
                ui.add_space(medium);
                SectionHeader::new(Text::message(MinimapMessage::OnTheMinimap)).show(ui);
                let mut edge = dialog.show_on_edge;
                if ToggleRow::new(Text::message(MinimapMessage::ArrowAtEdge), &mut edge).show(ui) {
                    actions.push(MapAction::DialogEdge);
                }
                let mut distance = dialog.show_distance;
                if ToggleRow::new(Text::message(MinimapMessage::ShowDistance), &mut distance).show(ui) {
                    actions.push(MapAction::DialogDistance);
                }
                ui.add_space(medium);
                SectionHeader::new(Text::message(MinimapMessage::Position)).show(ui);
                ui.label(Self::position(dialog.position, &UiLocale::get(ui.ctx())));
                ui.add_space(<spacing::L as SpacingToken>::VALUE);
                Self::buttons(ui, actions);
            });
        });
        ctx.move_to_top(LayerId::new(Order::Foreground, id));
        let (enter, escape) = ctx.input(|input| (input.key_pressed(Key::Enter), input.key_pressed(Key::Escape)));
        if enter {
            actions.push(MapAction::SaveDialog);
        } else if escape {
            actions.push(MapAction::CancelDialog);
        }
        // a dialog that closes gives the keyboard back to the game in the same frame; a refused
        // save takes the focus again in the next frame
        if actions.iter().any(|a| matches!(a, MapAction::SaveDialog | MapAction::CancelDialog)) {
            ctx.memory_mut(|memory| memory.stop_text_input());
        }
    }

    // a full-screen layer under the dialog: it dims the map and takes every press off it
    fn dim(ctx: &Context) {
        Area::new(MinimapUiId::Dim.id()).fixed_pos(pos2(0.0, 0.0)).order(Order::Foreground).show(ctx, |ui| {
            let screen = ctx.content_rect();
            let (rect, _) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(rect, 0, <color::Dim as ColorToken>::VALUE);
        });
    }

    pub fn heading(dialog: &WaypointDialog) -> MinimapMessage {
        match dialog.target {
            DialogTarget::New { .. } => MinimapMessage::NewWaypoint,
            DialogTarget::Edit(_) => MinimapMessage::EditWaypoint,
        }
    }

    fn title(ui: &mut Ui, dialog: &WaypointDialog, actions: &mut Vec<MapAction>) {
        let close =
            IconButton::new(Icon::Close).size(IconButtonSize::Small).tooltip(Text::message(AmmlUiMessage::Close));
        let (closed, ()) = SplitRow::show(
            ui,
            |ui| close.show(ui).clicked(),
            |ui| {
                let heading = Text::message(Self::heading(dialog));
                ui.label(Words::rich(ui, &heading, Self::TITLE, <color::Text as ColorToken>::VALUE));
            },
        );
        if closed {
            actions.push(MapAction::CancelDialog);
        }
    }

    fn name(ui: &mut Ui, drafts: &mut UiDrafts, focus: bool) {
        let id = MinimapUiId::DialogName.id();
        // the field has no placeholder: the NAME caption is right above it, as before
        let placeholder = Text::user(UserText::from_model(String::new()));
        TextField::new(id, &mut drafts.name, TextFieldKind::Plain, placeholder).show(ui, ui.available_width());
        ListView::limit(&mut drafts.name, WaypointStore::MAX_NAME_CHARS);
        let free = ui.ctx().memory(|memory| memory.focused().is_none());
        if focus || free {
            ui.memory_mut(|memory| memory.request_focus(id));
        }
    }

    // the palette in rows of SWATCHES_PER_ROW
    fn swatches(ui: &mut Ui, selected: [u8; 4], actions: &mut Vec<MapAction>) {
        let rows = WaypointStore::PALETTE.len().div_ceil(Self::SWATCHES_PER_ROW);
        let size = vec2(Self::SWATCH_GAP * Self::SWATCHES_PER_ROW as f32, Self::SWATCH_ROW * rows as f32);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        for (i, colour) in WaypointStore::PALETTE.iter().enumerate() {
            let (column, row) = (i % Self::SWATCHES_PER_ROW, i / Self::SWATCHES_PER_ROW);
            let x = rect.left() + 18.0 + column as f32 * Self::SWATCH_GAP;
            let centre = pos2(x, rect.top() + Self::SWATCH_ROW * (row as f32 + 0.5));
            let hit = egui::Rect::from_center_size(centre, vec2(Self::SWATCH * 2.4, Self::SWATCH * 2.4));
            let response = ui.interact(hit, ui.scope_id().with((MinimapUiId::Swatch, i)), Sense::click());
            let painter = ui.painter();
            if *colour == selected {
                painter.circle_stroke(
                    centre,
                    Self::SWATCH + 4.0,
                    Stroke::new(2.0, <color::OnAccent as ColorToken>::VALUE),
                );
            }
            let [r, g, b, a] = *colour;
            painter.circle_filled(centre, Self::SWATCH, Color32::from_rgba_unmultiplied(r, g, b, a));
            if response.clicked() {
                actions.push(MapAction::DialogColour(*colour));
            }
        }
    }

    // "X 316   Y 71   Z -58" with the axis letters in their colours, in the language of `localizer`
    pub fn position(position: (i32, i32, i32), localizer: &Localizer) -> LayoutJob {
        let font = FontFace::Regular.id(Self::POSITION_SIZE);
        let text = <color::Text as ColorToken>::VALUE;
        let mut job = LayoutJob::default();
        for (axis, letter, value, gap) in [
            (AxisColor::X, MinimapMessage::AxisX, position.0, "   "),
            (AxisColor::Y, MinimapMessage::AxisY, position.1, "   "),
            (AxisColor::Z, MinimapMessage::AxisZ, position.2, ""),
        ] {
            let letter = format!("{} ", Text::message(letter).resolve(localizer));
            job.append(&letter, 0.0, TextFormat::simple(font.clone(), axis.color()));
            job.append(&format!("{value}{gap}"), 0.0, TextFormat::simple(font.clone(), text));
        }
        job
    }

    fn buttons(ui: &mut Ui, actions: &mut Vec<MapAction>) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if TextButton::new(ButtonKind::Primary, Text::message(MinimapMessage::Save)).show(ui).clicked() {
                actions.push(MapAction::SaveDialog);
            }
            if TextButton::new(ButtonKind::Outline, Text::message(AmmlUiMessage::Cancel)).show(ui).clicked() {
                actions.push(MapAction::CancelDialog);
            }
        });
    }
}
