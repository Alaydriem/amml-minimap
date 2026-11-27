use alaydriem_minimap::services::CursorClip;
use amml_api::testing::FakeGame;
use minimap_runtime::structs::CursorAction;

use crate::support::{FakeTerrain, MinimapRig};

fn clip() -> (FakeGame, CursorClip) {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
    let ctx = MinimapRig::ctx(&game);
    let clip = CursorClip::new(ctx.window(), ctx.log());
    (game, clip)
}

fn lines(game: &FakeGame) -> Vec<String> {
    game.logs().into_iter().filter(|line| line.contains("cursor clip")).collect()
}

#[test]
fn a_clip_asked_in_play_clips_once_and_says_so() {
    let (game, mut clip) = clip();

    clip.apply(CursorAction::ClipToCentre, true, Some(true));
    clip.apply(CursorAction::ClipToCentre, true, Some(true));
    clip.apply(CursorAction::None, true, Some(true));

    assert_eq!(game.window_requests(), vec!["clip_cursor centre"]);
    assert_eq!(lines(&game), vec!["info minimap cursor clip on"]);
}

#[test]
fn a_release_gives_the_cursor_back() {
    let (game, mut clip) = clip();
    clip.apply(CursorAction::ClipToCentre, true, Some(true));

    clip.apply(CursorAction::Release, true, Some(true));

    assert_eq!(game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert_eq!(lines(&game)[1], "info minimap cursor clip off reason=released");
}

#[test]
fn a_free_mouse_drops_the_clip_whatever_the_plan_asks() {
    let (game, mut clip) = clip();
    clip.apply(CursorAction::ClipToCentre, true, Some(true));

    clip.apply(CursorAction::ClipToCentre, false, Some(true));

    assert_eq!(game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert_eq!(lines(&game)[1], "info minimap cursor clip off reason=mouse_free");
}

#[test]
fn focus_loss_keeps_the_request_and_says_the_clip_is_off_until_focus_returns() {
    let (game, mut clip) = clip();
    clip.apply(CursorAction::ClipToCentre, true, Some(true));

    clip.apply(CursorAction::None, true, Some(false));
    clip.apply(CursorAction::None, true, Some(true));

    assert_eq!(game.window_requests(), vec!["clip_cursor centre"]);
    assert_eq!(
        lines(&game),
        vec!["info minimap cursor clip on", "info minimap cursor clip off reason=focus", "info minimap cursor clip on"]
    );
}

#[test]
fn a_world_leave_drops_a_clip_and_does_nothing_without_one() {
    let (game, mut clip) = clip();
    clip.drop_clip("left");
    clip.apply(CursorAction::ClipToCentre, true, Some(true));

    clip.drop_clip("left");

    assert_eq!(game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert_eq!(lines(&game), vec!["info minimap cursor clip on", "info minimap cursor clip off reason=left"]);
}

#[test]
fn a_focus_loss_that_also_frees_the_mouse_is_logged_as_focus() {
    let (game, mut clip) = clip();
    clip.apply(CursorAction::ClipToCentre, true, Some(true));

    clip.apply(CursorAction::None, false, Some(false));

    assert_eq!(game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert_eq!(lines(&game)[1], "info minimap cursor clip off reason=focus");
}

#[test]
fn no_clip_is_asked_while_the_loader_knows_no_window() {
    let (game, mut clip) = clip();

    clip.apply(CursorAction::ClipToCentre, true, None);

    assert!(game.window_requests().is_empty());
    assert!(lines(&game).is_empty());
}

#[test]
fn a_refused_clip_is_warned_once_per_error_kind() {
    let (game, mut clip) = clip();
    game.withdraw_area(amml_api::Area::WINDOW_CURSOR_CLIP);

    for _ in 0..3 {
        clip.apply(CursorAction::ClipToCentre, true, Some(true));
    }

    let warnings = game.logs().into_iter().filter(|line| line.starts_with("warn minimap cursor clip refused")).count();
    assert_eq!(warnings, 1);
}
