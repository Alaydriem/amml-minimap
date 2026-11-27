use alaydriem_minimap::MinimapMod;
use amm_client::link::LinkUses;
use amm_client::test_support::FakeTable;
use amml_api::amml_abi::{AmmlDirKind, ClientTick, WorldLeft};
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{AmmlMod, BlockPos, BlockTypeId};
use tempfile::TempDir;

use crate::support::{FakeTerrain, MinimapRig};

const MANIFEST: &str = include_str!("../amml.hcl");

// the minimap enabled on a fake game with a player in a loaded world
struct ModRig {
    game: FakeGame,
    _dir: TempDir,
}

impl ModRig {
    fn new(manager: bool) -> Self {
        let game = MinimapRig::game();
        let dir = tempfile::tempdir().unwrap();
        game.set_dir(AmmlDirKind::DATA, &dir.path().display().to_string());
        game.set_client(MinimapRig::player(0, FakeTerrain::world(&[(0, 0)])));
        if manager {
            FakeTable::provide(&game.ctx()).unwrap();
        }
        game.load::<MinimapMod>().unwrap();
        game.enable::<MinimapMod>().unwrap();
        Self { game, _dir: dir }
    }

    fn tick(&self) {
        MinimapRig::tick(&self.game);
    }

    fn logged(&self, line: &str) -> bool {
        self.game.logs().iter().any(|logged| logged == line)
    }
}

#[test]
fn link_uses_are_complete_and_every_row_is_required() {
    assert_eq!(LinkUses::check(&MinimapMod::USES), Ok(()));
    assert!(MinimapMod::USES.rows().iter().all(|row| row.required != 0));
}

#[test]
fn the_manifest_names_the_mod_and_lists_no_entries() {
    assert!(MANIFEST.contains("namespace   = \"alaydriem\""));
    assert!(MANIFEST.contains("name        = \"minimap\""));
    assert!(MANIFEST.contains("api_version = 3"));
    assert!(!MANIFEST.contains("entries"));
    assert!(!MANIFEST.contains("amml_version"));
}

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    let manifest = MANIFEST.replace("\r\n", "\n");

    assert_eq!(manifest.matches(&MinimapMod::USES.to_hcl()).count(), 1);
    assert_eq!(MinimapMod::USES.write_into(&manifest), manifest);
}

#[test]
fn with_the_manager_the_first_tick_links_and_enters_the_world() {
    let rig = ModRig::new(true);

    rig.tick();

    assert!(rig.logged("info minimap block changes on"));
    assert!(rig.logged("info minimap linked"));
    assert!(rig.logged("info minimap world entered dimension=0"));
    rig.game.disable::<MinimapMod>();
}

#[test]
fn without_the_manager_the_minimap_waits_and_enters_nothing() {
    let rig = ModRig::new(false);

    for _ in 0..5 {
        rig.tick();
    }

    assert!(!rig.logged("info minimap linked"));
    assert!(!rig.logged("info minimap world entered dimension=0"));
    rig.game.disable::<MinimapMod>();
}

#[test]
fn a_world_leave_event_leaves_the_world() {
    let rig = ModRig::new(true);
    rig.tick();

    rig.game.raise(&mut WorldLeft { header: FakeGame::header::<WorldLeft>(), epoch: 0 });

    assert!(rig.logged("info minimap world left"));
    rig.game.disable::<MinimapMod>();
}

#[test]
fn a_type_change_reads_its_column_and_a_same_type_change_is_skipped() {
    let rig = ModRig::new(true);
    for _ in 0..3 {
        rig.tick();
    }

    rig.game.raise_block_changed(0, BlockPos { x: 4, y: 64, z: 2 }, 0, BlockTypeId(1), BlockTypeId(1));
    rig.game.raise_block_changed(0, BlockPos { x: 3, y: 64, z: 2 }, 0, FakeTerrain::SAND, FakeTerrain::STONE);
    rig.tick();

    let columns: Vec<String> =
        rig.game.logs().into_iter().filter(|line| line.starts_with("info minimap block change column")).collect();
    assert_eq!(columns, vec!["info minimap block change column x=3 z=2"]);
    rig.game.disable::<MinimapMod>();
}

#[test]
fn disable_gives_the_cursor_back_then_leaves_the_manager() {
    let rig = ModRig::new(true);
    rig.tick();
    assert_eq!(rig.game.window_requests(), vec!["clip_cursor centre"]);

    rig.game.disable::<MinimapMod>();

    assert_eq!(rig.game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert!(rig.logged("info minimap cursor clip off reason=disabled"));
    assert!(FakeTable::calls().contains(&"unregister"));
}

#[test]
fn a_menu_drops_the_cursor_clip() {
    let rig = ModRig::new(true);
    rig.tick();

    rig.game
        .set_client(FakeClient { mouse_captured: Some(false), ..MinimapRig::player(0, FakeTerrain::world(&[(0, 0)])) });
    rig.tick();

    assert_eq!(rig.game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert!(rig.logged("info minimap cursor clip off reason=mouse_free"));
    rig.game.disable::<MinimapMod>();
}

#[test]
fn a_refused_required_listen_fails_enable() {
    let game = MinimapRig::game();
    let dir = tempfile::tempdir().unwrap();
    game.set_dir(AmmlDirKind::DATA, &dir.path().display().to_string());
    game.refuse_listen::<ClientTick>();
    game.load::<MinimapMod>().unwrap();

    assert!(game.enable::<MinimapMod>().is_err());
    assert!(game.logs().iter().any(|line| line.starts_with("error minimap off")), "{:?}", game.logs());
}
