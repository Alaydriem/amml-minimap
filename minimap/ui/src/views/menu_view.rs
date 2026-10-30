use amml_i18n::localize::Localizer;
use amml_i18n::text::{Args, Text, UserText};
use amml_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use amml_ui::font::FontFace;
use amml_ui::icon::Icon;
use amml_ui::text::UiLocale;
use amml_ui::tokens::{ColorToken, ContainerToken, color, container};
use amml_ui::widget::{Menu, MenuItem as MenuRow, MenuLeading};
use egui::{Area, Context, Order, Sense, Ui, pos2, vec2};
use minimap_core::structs::ui::{MapAction, MapMenu, MenuItem, MenuTarget, UiFrame};

use crate::id::MinimapUiId;
use crate::message::MinimapMessage;

// the right-click menu at the point that opened it: a header naming the block or the waypoint,
// then one amml-ui menu item per entry; a left press anywhere else closes it
pub struct MenuView;

impl MenuView {
    const OFFSET: f32 = 6.0;
    const HEADER: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const HEADER_HEIGHT: f32 = 22.0;

    pub fn show(ctx: &Context, frame: &UiFrame<'_>, menu: &MapMenu, actions: &mut Vec<MapAction>) {
        let ppp = ctx.pixels_per_point().max(f32::EPSILON);
        let at = pos2(menu.at.0 / ppp + Self::OFFSET, menu.at.1 / ppp + Self::OFFSET);
        let shown = Area::new(MinimapUiId::Menu.id()).fixed_pos(at).order(Order::Foreground).show(ctx, |ui| {
            Menu::frame().show(ui, |ui| {
                ui.set_width(<container::Menu as ContainerToken>::VALUE - Menu::frame().total_margin().sum().x);
                Self::header(ui, frame, menu.target);
                for item in &menu.items {
                    if *item == MenuItem::ExportView {
                        Menu::separator(ui);
                    }
                    if Self::row(ui, *item, frame.ui.waypoints_shown()) {
                        actions.push(MapAction::Choose(*item));
                    }
                }
            });
        });
        let pressed_outside = ctx.input(|input| {
            input.pointer.primary_pressed()
                && input.pointer.interact_pos().is_some_and(|pos| !shown.response.rect.contains(pos))
        });
        if pressed_outside {
            actions.push(MapAction::CloseMenu);
        }
    }

    // "BLOCK x, z" for the ground, the waypoint's name for a marker, in the language of `localizer`. The messages are
    // written in capitals; the player's waypoint name, which no one translates, is put in capitals here
    pub fn title(frame: &UiFrame<'_>, target: MenuTarget, localizer: &Localizer) -> String {
        match target {
            MenuTarget::Ground { x, z, .. } => {
                let args = Args::new().int("x", i64::from(x)).int("z", i64::from(z));
                Text::format(MinimapMessage::BlockAt, args).resolve(localizer).into_owned()
            }
            MenuTarget::Marker(id) => frame.waypoints.iter().find(|w| w.id == id).map_or_else(
                || Text::message(MinimapMessage::Waypoint).resolve(localizer).into_owned(),
                |w| w.name.to_uppercase(),
            ),
        }
    }

    fn header(ui: &mut Ui, frame: &UiFrame<'_>, target: MenuTarget) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEADER_HEIGHT), Sense::hover());
        let title = Self::title(frame, target, &UiLocale::get(ui.ctx()));
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let room = rect.width() - 8.0;
        let galley = Draw::galley(ui.painter(), &title, Self::HEADER, faint, TextFit::Truncate(room));
        let at = pos2(rect.left() + 8.0, rect.center().y - galley.size().y * 0.5);
        ui.painter().galley(at, galley, faint);
    }

    pub fn icon(item: MenuItem, waypoints_shown: bool) -> Icon {
        match item {
            MenuItem::AddWaypoint => Icon::Add,
            MenuItem::CopyCoordinates => Icon::Copy,
            MenuItem::ToggleWaypoints if waypoints_shown => Icon::Hide,
            MenuItem::ToggleWaypoints => Icon::Show,
            MenuItem::EditWaypoint => Icon::Edit,
            MenuItem::RemoveWaypoint => Icon::Delete,
            MenuItem::ExportView => Icon::Camera,
            MenuItem::ExportArea => Icon::Share,
        }
    }

    // one item; true when clicked. The item's words come from minimap-core and stay English for now
    fn row(ui: &mut Ui, item: MenuItem, waypoints_shown: bool) -> bool {
        let label = Text::user(UserText::from_model(item.label(waypoints_shown).to_owned()));
        MenuRow::new(label).leading(MenuLeading::Icon(Self::icon(item, waypoints_shown))).show(ui).clicked()
    }
}
