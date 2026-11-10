use amml_i18n::text::Text;
use egui::{Area, Id, pos2};
use minimap_ui::message::MinimapMessage;
use minimap_ui::testing::{PassOutput, UiHarness};
use minimap_ui::views::ToggleRow;

use crate::support::Frames;

// one pass of a chunk grid row with `note`; returns whether it reported a click and the switch's value after
fn pass(
    harness: &mut UiHarness,
    out: &mut PassOutput,
    cursor: (f32, f32),
    down: bool,
    with_note: bool,
) -> (bool, bool) {
    let mut on = false;
    let mut clicked = false;
    let input = Frames::input(cursor, down, &[]);
    harness.run(&input, out, |ctx| {
        Area::new(Id::unique("toggle-row-test")).fixed_pos(pos2(20.0, 20.0)).show(ctx, |ui| {
            ui.set_width(300.0);
            let note = with_note.then(|| Text::message(MinimapMessage::ZoomIn));
            clicked = ToggleRow::new(Text::message(MinimapMessage::ChunkGrid), &mut on).note(note).show(ui);
        });
    });
    (clicked, on)
}

#[test]
fn toggle_row_flips_on_a_label_click() {
    let (mut harness, mut out) = (UiHarness::new(), PassOutput::new());
    // a new area is laid out in its first frame and painted from the second
    pass(&mut harness, &mut out, (0.0, 0.0), false, false);
    pass(&mut harness, &mut out, (0.0, 0.0), false, false);
    let [left, top, right, bottom] = out.text_rect("Chunk grid").unwrap_or_else(|| panic!("no label"));
    let at = ((left + right) * 0.5, (top + bottom) * 0.5);

    pass(&mut harness, &mut out, at, false, false);
    pass(&mut harness, &mut out, at, true, false);
    let (clicked, on) = pass(&mut harness, &mut out, at, false, false);

    assert!(clicked);
    assert!(on);
}

#[test]
fn toggle_row_shows_its_note_only_when_given() {
    let (mut harness, mut out) = (UiHarness::new(), PassOutput::new());

    pass(&mut harness, &mut out, (0.0, 0.0), false, true);
    pass(&mut harness, &mut out, (0.0, 0.0), false, true);
    let with_note = out.draws_text("zoom in");
    pass(&mut harness, &mut out, (0.0, 0.0), false, false);

    assert!(with_note);
    assert!(!out.draws_text("zoom in"));
}
