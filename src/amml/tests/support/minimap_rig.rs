use amm_client::test_support::FakeTable;
use amml_api::amml_abi::{BlockChanged, ClientTick, KeyInput, WorldLeft};
use amml_api::testing::{FakeClient, FakeGame, FakeWindow, FakeWorld};
use amml_api::{Area, Ctx, Rotation, Vec3};

// a fake loader that serves every event and area of the minimap, with a focused window and a player
pub struct MinimapRig;

impl MinimapRig {
    pub const AREAS: [Area; 10] = [
        Area::INPUT_LEASE,
        Area::CLIENT_READ,
        Area::CLIENT_MOUSE,
        Area::WINDOW_READ,
        Area::ACTOR_READ.at(3),
        Area::WORLD_CHUNK_READ,
        Area::CLIENT_WORLD_INFO,
        Area::WORLD_BIOMES,
        Area::WORLD_MAP_COLOR,
        Area::WINDOW_CURSOR_CLIP,
    ];

    pub fn game() -> FakeGame {
        let game = FakeTable::game();
        game.offer::<ClientTick>();
        game.offer::<WorldLeft>();
        game.offer::<KeyInput>();
        game.offer::<BlockChanged>();
        for area in Self::AREAS {
            game.offer_area(area);
        }
        game.set_window(FakeWindow::default());
        game.open_gate();
        game
    }

    pub fn ctx(game: &FakeGame) -> Ctx {
        FakeTable::ctx_of(game, minimap_link::structs::Descriptor::TEXT)
    }

    // a live player at eye height 80 in `dimension`, holding the mouse, in `world`
    pub fn player(dimension: i32, world: FakeWorld) -> FakeClient {
        FakeClient {
            local_player: true,
            world_active: true,
            mouse_captured: Some(true),
            player_alive: Some(true),
            eye_position: Some(Vec3 { x: 8.5, y: 80.0, z: 8.5 }),
            position: Some(Vec3 { x: 8.5, y: 78.38, z: 8.5 }),
            rotation: Some(Rotation { yaw: 90.0, pitch: 0.0 }),
            dimension: Some(dimension),
            world: Some(world),
            ..FakeClient::default()
        }
    }

    pub fn tick(game: &FakeGame) {
        game.raise_with_client(&mut ClientTick { header: FakeGame::header::<ClientTick>() });
    }
}
