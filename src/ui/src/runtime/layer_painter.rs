use egui::{Area, Context, Event, Key, LayerId, MouseWheelUnit, Order, Sense, pos2};
use minimap_core::structs::draw::{DrawList, MapLayer};
use minimap_core::structs::ui::{MapAction, UiFrame};

use crate::id::MinimapUiId;
use crate::map::{DrawListShapes, LayerTextures};
use crate::runtime::UiDrafts;
use crate::views::MapScreen;

// paints one manager layer into the manager's egui context: the map layers on the background, the panels
// above them; returns what the panels and the world map's keys asked for
pub struct LayerPainter {
    shapes: DrawListShapes,
    drafts: UiDrafts,
    // what this pass asked for; keeps its capacity
    actions: Vec<MapAction>,
    // a text field had the keyboard at the end of the last pass; egui drops the focus on Escape before the
    // next pass runs, so the last pass's answer is the one that counts
    typing: bool,
}

impl LayerPainter {
    // points one wheel notch moves when the wheel reports points, not lines
    const POINTS_PER_LINE: f32 = 50.0;
    // the area under the panels that makes the open map count as egui's: the manager routes a press, a drag and
    // the wheel to a layer only while its egui wants the pointer, and egui does not count the Background layer

    pub fn new() -> Self {
        Self { shapes: DrawListShapes::new(), drafts: UiDrafts::new(), actions: Vec::new(), typing: false }
    }

    pub fn paint(
        &mut self,
        ctx: &Context,
        layer: MapLayer,
        list: &DrawList,
        textures: &LayerTextures,
        frame: Option<&UiFrame<'_>>,
    ) -> &[MapAction] {
        self.actions.clear();
        self.shapes.paint(list, &ctx.layer_painter(LayerId::background()), ctx.pixels_per_point(), textures);
        let Some(frame) = frame else { return &self.actions };
        if layer == MapLayer::Overlay && frame.panels {
            Self::map_area(ctx);
        }
        MapScreen::show(ctx, frame, &mut self.drafts, &mut self.actions);
        // a text field that had the keyboard keeps Escape and Ctrl+Z for itself
        if layer == MapLayer::Overlay && frame.panels && !self.typing {
            self.keys(ctx, frame);
        }
        self.typing = ctx.egui_wants_keyboard_input();
        &self.actions
    }

    // Escape closes the menu, else the world map; the dialog handles its own Escape. Ctrl+Z undoes the last
    // removal while it can
    fn keys(&mut self, ctx: &Context, frame: &UiFrame<'_>) {
        // the modifiers of the Z press itself: the manager sends them on each key event
        let (escape, undo) = ctx.input(|input| {
            let undo = input.events.iter().any(|event| {
                matches!(event, Event::Key { key: Key::Z, pressed: true, modifiers, .. } if modifiers.ctrl || modifiers.command)
            });
            (input.key_pressed(Key::Escape), undo)
        });
        if escape && frame.ui.menu().is_some() {
            self.actions.push(MapAction::CloseMenu);
        } else if escape && frame.ui.dialog().is_none() {
            self.actions.push(MapAction::CloseWorldMap);
        }
        if undo && frame.ui.can_undo() {
            self.actions.push(MapAction::UndoRemove);
        }
    }

    // the whole screen, under the panels (they are Foreground); it never moves and draws nothing
    fn map_area(ctx: &Context) {
        let screen = ctx.content_rect();
        Area::new(MinimapUiId::Map.id()).order(Order::Middle).fixed_pos(screen.min).movable(false).show(ctx, |ui| {
            ui.allocate_rect(screen, Sense::click_and_drag());
        });
    }

    // wheel notches in this pass, away from the user positive
    pub fn wheel_lines(ctx: &Context) -> f32 {
        ctx.input(|input| {
            input
                .events
                .iter()
                .map(|event| match event {
                    Event::MouseWheel { unit: MouseWheelUnit::Line, delta, .. } => delta.y,
                    Event::MouseWheel { unit: MouseWheelUnit::Point, delta, .. } => delta.y / Self::POINTS_PER_LINE,
                    _ => 0.0,
                })
                .sum()
        })
    }

    // a panel from the last pass is under this screen point, in physical pixels. The background layer and the map
    // area are the map, not a panel
    pub fn covers(ctx: &Context, at: (f32, f32)) -> bool {
        let ppp = ctx.pixels_per_point().max(f32::EPSILON);
        let map = LayerId::new(Order::Middle, MinimapUiId::Map.id());
        ctx.layer_id_at(pos2(at.0 / ppp, at.1 / ppp))
            .is_some_and(|layer| layer.order != Order::Background && layer != map)
    }

    // the waypoint list's search as typed
    pub fn search(&self) -> &str {
        &self.drafts.search
    }

    // a text field had the keyboard at the end of the last pass
    pub fn typing(&self) -> bool {
        self.typing
    }

    pub fn drafts(&self) -> &UiDrafts {
        &self.drafts
    }

    // the world was left or the map closed: typed text is dropped
    pub fn reset_drafts(&mut self) {
        self.drafts.clear();
        self.typing = false;
    }
}

impl Default for LayerPainter {
    fn default() -> Self {
        Self::new()
    }
}
