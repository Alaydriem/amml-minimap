use amml_i18n::message::Message;
use amml_i18n::text::{Args, Text};
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use amml_ui::tokens::{ContainerToken, container};
use egui::Rect;
use minimap_core::services::ui::DimensionName;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget};
use minimap_ui::id::MinimapUiId;
use minimap_ui::message::MinimapMessage;
use minimap_ui::runtime::MinimapLocale;
use minimap_ui::views::ToolsView;

use crate::support::Scene;

const GROUND: MenuTarget = MenuTarget::Ground { dim: 0, x: 316, y: 71, z: -58 };
const ITEMS: [MenuItem; 7] = [
    MenuItem::AddWaypoint,
    MenuItem::CopyCoordinates,
    MenuItem::ToggleWaypoints,
    MenuItem::EditWaypoint,
    MenuItem::RemoveWaypoint,
    MenuItem::ExportView,
    MenuItem::ExportArea,
];

// the world map in en_XA with the list open and the right-click menu open on the ground; with `dialog`, the menu's
// Add waypoint opened the waypoint dialog
// The game's font (Nordwand Mono, embedded) is much wider than egui's built-in one, and the map is zoomed out so
// the overlay switches show their "zoom in" note.
fn pseudo_scene(dialog: bool) -> Scene {
    let mut scene = Scene::new(true);
    Theme::install(scene.harness.context(), FontSource::Embedded);
    scene.harness.set_locale("en_XA");
    scene.view.zoom = 0.05;
    scene.ui.open_menu((900.0, 400.0), GROUND);
    if dialog {
        let mut store = WaypointStore::new(16, 1);
        scene.ui.apply(MapAction::Choose(MenuItem::AddWaypoint), "", 0, &mut scene.view, &mut store, 0);
    }
    scene.settle();
    scene
}

// the text minimap-core builds and the player's own names: shown as they come, never translated
fn model_words() -> Vec<String> {
    let mut words: Vec<String> = Scene::NAMES.iter().map(|(name, _, _)| (*name).to_owned()).collect();
    words.push(DimensionName::of(0).to_owned());
    words.push("M".to_owned());
    for shown in [true, false] {
        words.extend(ITEMS.iter().map(|item| item.label(shown).to_owned()));
    }
    words
}

#[test]
fn world_map_paints_no_english_word_in_pseudo_locale() {
    let english: Vec<&str> = MinimapMessage::ALL
        .iter()
        .flat_map(|message| message.msgid().split(|c: char| !c.is_ascii_alphabetic()))
        .filter(|word| word.len() >= 3 && !["count", "key"].contains(word))
        .collect();
    let allowed = model_words();
    for dialog in [false, true] {
        let scene = pseudo_scene(dialog);
        for shown in &scene.mesh.text_rects {
            let text = scene.mesh.text_of(shown);
            if allowed.iter().any(|word| word == text) {
                continue;
            }
            let word = english.iter().find(|word| text.contains(**word));
            assert!(word.is_none(), "dialog {dialog}: {text:?} has the English word {word:?}");
        }
    }
}

// a translation keeps its own case: the captions and the menu header paint the translated words as written, never
// put in capitals by code
#[test]
fn captions_paint_their_translation_as_written() {
    let localizer = MinimapLocale::localizer("en_XA");
    let words = |message: MinimapMessage| Text::message(message).resolve(&localizer).into_owned();
    let block = Text::format(MinimapMessage::BlockAt, Args::new().int("x", 316).int("z", -58));
    let block = block.resolve(&localizer).into_owned();
    for dialog in [false, true] {
        let scene = pseudo_scene(dialog);
        let captions = if dialog {
            vec![words(MinimapMessage::Name), words(MinimapMessage::Colour), words(MinimapMessage::Position)]
        } else {
            vec![words(MinimapMessage::Overlays), words(MinimapMessage::UnderCursor), block.clone()]
        };
        for caption in captions {
            assert!(scene.mesh.draws_text(&caption), "dialog {dialog}: {caption:?} is not painted as written");
        }
    }
}

#[test]
fn cards_paint_inside_their_cards_in_pseudo_locale() {
    for dialog in [false, true] {
        let scene = pseudo_scene(dialog);
        let ctx = scene.harness.context();
        // each card's width as designed; the area must not grow past it, and no text may end past it
        let cards = [
            (MinimapUiId::Tools, ToolsView::WIDTH),
            (MinimapUiId::List, ToolsView::WIDTH),
            (MinimapUiId::Menu, <container::Menu as ContainerToken>::VALUE),
            (MinimapUiId::Dialog, <container::Dialog as ContainerToken>::VALUE),
        ];
        let cards: Vec<(MinimapUiId, Rect)> = cards
            .iter()
            .filter_map(|(card, width)| {
                let area = ctx.memory(|memory| memory.area_rect(card.id()))?;
                Some((*card, Rect::from_min_size(area.min, egui::vec2(*width, area.height()))))
            })
            .collect();
        for (card, designed) in &cards {
            let area = ctx.memory(|memory| memory.area_rect(card.id())).unwrap_or(Rect::NOTHING);
            assert!(area.width() <= designed.width() + 0.5, "dialog {dialog}: {card:?} grew to {}", area.width());
        }
        for shown in &scene.mesh.text_rects {
            let [left, top, right, _] = shown.rect;
            for (card, designed) in cards.iter().filter(|(_, designed)| designed.contains(egui::pos2(left, top))) {
                let text = scene.mesh.text_of(shown);
                let edge = designed.right();
                assert!(right <= edge + 0.5, "dialog {dialog}: {card:?} {text:?} ends at {right}, past {edge}");
            }
        }
    }
}
