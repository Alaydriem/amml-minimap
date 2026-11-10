use amml_i18n::text::Text;
use amml_ui::draw::{Corner, Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::{Icon, IconSize};
use amml_ui::tokens::rule::Elevation;
use amml_ui::tokens::{ColorToken, SpacingToken, color, spacing};
use amml_ui::widget::{SectionHeader, Surface, SurfaceLevel};
use egui::{Align2, Area, Context, Order, Rect, RichText, Sense, Ui, pos2, vec2};
use minimap_core::services::draw::OverlayGate;
use minimap_core::services::ui::DimensionName;
use minimap_core::structs::ui::{MapAction, UiFrame};

use crate::id::MinimapUiId;
use crate::message::MinimapMessage;
use crate::views::{HoverView, SplitRow, ToggleRow, Words};

// the top-left card: the dimension, the overlay switches, the waypoint list switch and what is
// under the cursor
pub struct ToolsView;

impl ToolsView {
    /// The width of the tools card and of the waypoint list under it. Paper draws no minimap card.
    pub const WIDTH: f32 = 300.0;
    // the card titles: Paper has no 16 px size, so the label size (15.5)
    pub const TITLE: TextStyle = TextStyle::new(TypeSize::Label, FontFace::Semibold);
    const ROW: TextStyle = TextStyle::new(TypeSize::Body, FontFace::Regular);
    const NOTE: TextStyle = TextStyle::new(TypeSize::Note, FontFace::Regular);
    const WAYPOINTS_ROW: f32 = 32.0;
    // the count badge on the waypoint list switch
    const COUNT_BADGE: egui::Vec2 = vec2(28.0, 18.0);

    // returns the card's bottom edge, where the waypoint list starts
    pub fn show(ctx: &Context, frame: &UiFrame<'_>, actions: &mut Vec<MapAction>) -> f32 {
        let gap = <spacing::L as SpacingToken>::VALUE;
        let small = <spacing::S as SpacingToken>::VALUE;
        let area = Area::new(MinimapUiId::Tools.id()).fixed_pos(pos2(gap, gap)).order(Order::Foreground);
        let shown = area.show(ctx, |ui| {
            let card = Surface::frame(SurfaceLevel::Card, Elevation::Floating);
            let inner = Self::WIDTH - card.total_margin().sum().x;
            card.show(ui, |ui| {
                ui.set_width(inner);
                Self::header(ui, frame.dim);
                ui.add_space(small);
                SectionHeader::new(Text::message(MinimapMessage::Overlays)).show(ui);
                let gated = !OverlayGate::open(frame.view.zoom);
                let note = gated.then(|| Text::message(MinimapMessage::ZoomIn));
                let mut grid = frame.view.show_chunk_grid;
                if ToggleRow::new(Text::message(MinimapMessage::ChunkGrid), &mut grid).note(note.clone()).show(ui) {
                    actions.push(MapAction::ToggleGrid);
                }
                let mut slime = frame.view.show_slime_chunks;
                if frame.dim == DimensionName::OVERWORLD
                    && ToggleRow::new(Text::message(MinimapMessage::SlimeChunks), &mut slime).note(note).show(ui)
                {
                    actions.push(MapAction::ToggleSlime);
                }
                if Self::waypoints_row(ui, frame.in_dimension, frame.ui.list_open()) {
                    actions.push(MapAction::ToggleList);
                }
                ui.add_space(small);
                ui.separator();
                ui.add_space(small);
                SectionHeader::new(Text::message(MinimapMessage::UnderCursor)).show(ui);
                HoverView::show(ui, frame.hover);
            });
        });
        shown.response.rect.bottom()
    }

    fn header(ui: &mut Ui, dim: i32) {
        SplitRow::show(
            ui,
            |ui| {
                // the dimension name comes from minimap-core and stays English for now
                let name = RichText::new(DimensionName::of(dim))
                    .font(Self::NOTE.font_id())
                    .color(<color::TextDim as ColorToken>::VALUE);
                ui.label(name);
            },
            |ui| {
                let title = Text::message(MinimapMessage::WorldMap);
                ui.label(Words::rich(ui, &title, Self::TITLE, <color::Text as ColorToken>::VALUE));
            },
        );
    }

    // the waypoint list switch: an icon, the label, the count and a chevron; true when clicked
    fn waypoints_row(ui: &mut Ui, count: usize, open: bool) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::WAYPOINTS_ROW), Sense::click());
        let painter = ui.painter();
        let row = rect.expand2(vec2(<spacing::S as SpacingToken>::VALUE * 0.75, 0.0));
        if open {
            Draw::fill(painter, row, Corner::Control, <color::AccentSoft as ColorToken>::VALUE);
        } else if response.hovered() {
            Draw::fill(painter, row, Corner::Control, <color::BgHover as ColorToken>::VALUE);
        }
        let icon = if open { <color::Accent as ColorToken>::VALUE } else { <color::TextDim as ColorToken>::VALUE };
        let y = rect.center().y;
        let side = IconSize::Regular.points();
        Icon::Pin.paint(
            painter,
            Rect::from_center_size(pos2(rect.left() + 8.0, y), vec2(side, side)),
            IconSize::Regular,
            icon,
        );
        let badge = Rect::from_center_size(pos2(rect.right() - 36.0, y), Self::COUNT_BADGE);
        let text = <color::Text as ColorToken>::VALUE;
        let room = badge.left() - 8.0 - (rect.left() + 26.0);
        let label = Text::message(MinimapMessage::Waypoints);
        Draw::text(
            painter,
            pos2(rect.left() + 26.0, y),
            Align2::LEFT_CENTER,
            &label,
            Self::ROW,
            text,
            TextFit::Truncate(room),
        );
        Draw::fill(painter, badge, Corner::Full, <color::BgField as ColorToken>::VALUE);
        let dim = <color::TextDim as ColorToken>::VALUE;
        painter.text(badge.center(), Align2::CENTER_CENTER, count.to_string(), FontFace::Regular.id(12.0), dim);
        let small = IconSize::Small.points();
        let chevron = Rect::from_center_size(pos2(rect.right() - 8.0, y), vec2(small, small));
        Icon::Chevron.paint(painter, chevron, IconSize::Small, dim);
        response.clicked()
    }
}
