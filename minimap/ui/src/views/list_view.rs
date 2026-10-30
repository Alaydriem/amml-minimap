use amml_i18n::text::{Args, Text};
use amml_ui::LeviUiMessage;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{IconButton, IconButtonSize, Surface, SurfaceLevel, TextField, TextFieldKind};
use egui::{Align2, Area, Color32, Context, Order, ScrollArea, Sense, Ui, pos2, vec2};
use minimap_core::services::ui::{WaypointDistance, WaypointFilter};
use minimap_core::structs::ui::{MapAction, UiFrame};
use minimap_core::structs::waypoint::Waypoint;

use crate::id::MinimapUiId;
use crate::message::MinimapMessage;
use crate::runtime::UiDrafts;
use crate::views::{SplitRow, ToolsView, Words};

// the waypoint list under the tools card on the left: a header with the count and a close button,
// the search field, the rows of the player's dimension and a footer hint
pub struct ListView;

impl ListView {
    pub const ROW_HEIGHT: f32 = 44.0;
    pub const MIN_ROWS: f32 = 2.0;
    // the header, the search field, the footer and the card's padding, around the rows
    const CHROME: f32 = 150.0;
    const BODY: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const SMALL: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Regular);
    // the coordinates at the right of a row; Paper has no 13 px type size
    const COORDS_SIZE: f32 = 13.0;

    // `top` is the tools card's bottom edge; the rows fill the height left on the screen
    pub fn show(ctx: &Context, frame: &UiFrame<'_>, top: f32, drafts: &mut UiDrafts, actions: &mut Vec<MapAction>) {
        let (gap, small) = (<spacing::L as SpacingToken>::VALUE, <spacing::S as SpacingToken>::VALUE);
        let top = top + small;
        let rows_height =
            (ctx.content_rect().bottom() - gap - top - Self::CHROME).max((Self::ROW_HEIGHT + 4.0) * Self::MIN_ROWS);
        let area = Area::new(MinimapUiId::List.id()).fixed_pos(pos2(gap, top)).order(Order::Foreground);
        area.show(ctx, |ui| {
            let card = Surface::frame(SurfaceLevel::Card, Elevation::Floating);
            let inner = ToolsView::WIDTH - card.total_margin().sum().x;
            card.show(ui, |ui| {
                ui.set_width(inner);
                Self::header(ui, frame.in_dimension, actions);
                ui.add_space(small);
                Self::search(ui, drafts);
                ui.add_space(small);
                Self::rows(ui, frame, rows_height, actions);
                ui.add_space(small);
                ui.separator();
                let footer = Text::message(MinimapMessage::ListFooter);
                ui.label(Words::rich(ui, &footer, Self::SMALL, <color::TextFaint as ColorToken>::VALUE));
            });
        });
    }

    fn header(ui: &mut Ui, count: usize, actions: &mut Vec<MapAction>) {
        let close =
            IconButton::new(Icon::Close).size(IconButtonSize::Small).tooltip(Text::message(LeviUiMessage::Close));
        let (closed, ()) = SplitRow::show(
            ui,
            |ui| close.show(ui).clicked(),
            |ui| {
                let title = Text::message(MinimapMessage::Waypoints);
                ui.label(Words::rich(ui, &title, ToolsView::TITLE, <color::Text as ColorToken>::VALUE));
                let label = Text::plural(MinimapMessage::WaypointsInDimension, count as u64, Args::new());
                ui.label(Words::rich(ui, &label, Self::NOTE, <color::TextDim as ColorToken>::VALUE));
            },
        );
        if closed {
            actions.push(MapAction::ToggleList);
        }
    }

    fn search(ui: &mut Ui, drafts: &mut UiDrafts) {
        let placeholder = Text::message(MinimapMessage::SearchWaypoints);
        let field = TextField::new(MinimapUiId::Search.id(), &mut drafts.search, TextFieldKind::Search, placeholder);
        field.show(ui, ui.available_width());
        Self::limit(&mut drafts.search, WaypointFilter::QUERY_CHARS);
    }

    /// Cuts `text` to `chars` characters: the amml-ui field has no limit of its own.
    pub fn limit(text: &mut String, chars: usize) {
        if let Some((end, _)) = text.char_indices().nth(chars) {
            text.truncate(end);
        }
    }

    fn rows(ui: &mut Ui, frame: &UiFrame<'_>, height: f32, actions: &mut Vec<MapAction>) {
        if frame.rows.is_empty() {
            let empty = Text::message(MinimapMessage::NoWaypointsHere);
            ui.label(Words::rich(ui, &empty, Self::BODY, <color::TextFaint as ColorToken>::VALUE));
            return;
        }
        ScrollArea::vertical().id_salt(MinimapUiId::Rows).max_height(height).auto_shrink([false, true]).show_rows(
            ui,
            Self::ROW_HEIGHT,
            frame.rows.len(),
            |ui, range| {
                for &index in &frame.rows[range] {
                    let Some(waypoint) = frame.waypoints.get(index) else { continue };
                    if Self::row(ui, waypoint, frame.player_block) {
                        let (x, z) = (f64::from(waypoint.x) + 0.5, f64::from(waypoint.z) + 0.5);
                        actions.push(MapAction::CentreOn { x, z });
                    }
                }
            },
        );
    }

    // one waypoint: its colour, name, distance and coordinates; true when clicked
    fn row(ui: &mut Ui, waypoint: &Waypoint, player: (i32, i32, i32)) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW_HEIGHT), Sense::click());
        let painter = ui.painter();
        if response.hovered() {
            Draw::fill(painter, rect, Corner::Control, <color::BgHover as ColorToken>::VALUE);
        }
        let x0 = rect.left() + 10.0;
        let [r, g, b, a] = waypoint.colour;
        painter.circle_filled(pos2(x0, rect.center().y), 6.0, Color32::from_rgba_unmultiplied(r, g, b, a));
        let coords = format!("{}, {}", waypoint.x, waypoint.z);
        let right = pos2(rect.right() - 8.0, rect.center().y);
        let dim = <color::TextDim as ColorToken>::VALUE;
        let coords = painter.text(right, Align2::RIGHT_CENTER, coords, FontFace::Regular.id(Self::COORDS_SIZE), dim);
        // the name is the player's own text: shown as typed, cut before the coordinates
        let room = coords.left() - 8.0 - (x0 + 16.0);
        let text = <color::Text as ColorToken>::VALUE;
        let name = Draw::galley(painter, &waypoint.name, Self::BODY, text, TextFit::Truncate(room));
        painter.galley(pos2(x0 + 16.0, rect.top() + 6.0), name, text);
        let blocks = WaypointDistance::blocks(player, (waypoint.x, waypoint.y, waypoint.z));
        let distance = if WaypointDistance::is_here(blocks) {
            Text::message(MinimapMessage::Here)
        } else {
            Text::plural(MinimapMessage::BlocksAway, u64::from(blocks), Args::new())
        };
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let sub = pos2(x0 + 16.0, rect.top() + 25.0);
        Draw::text(painter, sub, Align2::LEFT_TOP, &distance, Self::SMALL, faint, TextFit::Truncate(room));
        response.clicked()
    }
}
