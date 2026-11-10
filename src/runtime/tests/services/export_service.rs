use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use minimap_core::config::TerrainConfig;
use minimap_core::services::export::AreaLayout;
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;
use minimap_core::structs::view::ViewState;
use minimap_runtime::services::ExportService;
use minimap_runtime::structs::Shared;
use minimap_store::services::RegionFiles;
use minimap_store::structs::RegionSnapshot;

fn shared() -> Arc<Shared> {
    Arc::new(Shared::new(
        ChunkStore::new(16),
        TileStore::new(16, 25),
        KnownEmptySet::new(16),
        BakeSettings::new(&TerrainConfig::default()),
        ViewState::new(4),
        WaypointStore::new(8, 1),
        MapUi::new(1.0),
    ))
}

fn solid(rgba: [u8; 4]) -> Vec<u8> {
    rgba.repeat(Tile::BYTES / 4)
}

// waits up to five seconds for the export's message
fn result(service: &ExportService) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Some(message) = service.take_result() {
            return message;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("no export result within five seconds");
}

fn only_file(dir: &Path) -> PathBuf {
    let files: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    assert_eq!(files.len(), 1, "{files:?}");
    files[0].clone()
}

// (width, height, RGBA pixels)
fn decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let mut reader =
        png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap())).read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    (info.width, info.height, pixels)
}

fn pixel(pixels: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * width + x) * 4) as usize;
    [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
}

#[test]
fn a_view_export_draws_its_regions_to_a_view_named_png_in_the_exports_folder() {
    let dir = tempfile::tempdir().unwrap();
    let shared = shared();
    let shown = RegionPos::new(0, 0, 0);
    Shared::lock(&shared.tiles).insert_loaded(shown, &solid([20, 21, 22, 255]), 1, &mut Vec::new()).unwrap();
    let regions = vec![shown, RegionPos::new(1, 0, 0)];
    let layout = AreaLayout::new(&regions, 16384).unwrap();
    let service = ExportService::start(&dir.path().join(ExportService::FOLDER), Arc::clone(&shared)).unwrap();
    service.enter_world("Test World");

    service.export_view(layout, regions, None, 0).unwrap();

    let message = result(&service);
    let file = only_file(&dir.path().join(ExportService::FOLDER));
    let name = file.file_name().unwrap().to_string_lossy().into_owned();
    assert!(name.starts_with("Test_World-overworld-") && name.ends_with("-view.png"), "{name}");
    assert_eq!(message, format!("Exported {name}"));
    let (width, height, pixels) = decode(&file);
    assert_eq!((width, height), (512, 256));
    assert_eq!(pixel(&pixels, width, 1, 1), [20, 21, 22, 255]);
    assert_eq!(pixel(&pixels, width, 300, 1), [0, 0, 0, 0]);
    service.shutdown();
}

#[test]
fn an_area_export_draws_tiles_from_memory_and_disk_and_leaves_gaps_clear() {
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache");
    let shared = shared();
    let memory = RegionPos::new(-1, 0, 0);
    let disk = RegionPos::new(1, 0, 0);
    Shared::lock(&shared.tiles).insert_loaded(memory, &solid([200, 10, 10, 255]), 1, &mut Vec::new()).unwrap();
    let mut snapshot = RegionSnapshot::new(disk, 1);
    snapshot.tile = solid([10, 10, 200, 255]).into_boxed_slice();
    RegionFiles::new(cache.clone()).write(&snapshot).unwrap();
    let regions = vec![memory, disk];
    let layout = AreaLayout::new(&regions, 16384).unwrap();
    let service = ExportService::start(&dir.path().join(ExportService::FOLDER), Arc::clone(&shared)).unwrap();
    service.enter_world("w");
    service.export_area(layout, regions, Some(cache), 0).unwrap();
    assert!(result(&service).starts_with("Exported w-overworld-"));
    let (width, height, pixels) = decode(&only_file(&dir.path().join(ExportService::FOLDER)));
    assert_eq!((width, height), (768, 256));
    assert_eq!(pixel(&pixels, width, 10, 10), [200, 10, 10, 255]);
    assert_eq!(pixel(&pixels, width, 300, 10), [0, 0, 0, 0]);
    assert_eq!(pixel(&pixels, width, 700, 200), [10, 10, 200, 255]);
    service.shutdown();
}

#[test]
fn a_second_export_is_refused_while_one_runs_and_leaving_the_world_stops_it() {
    let dir = tempfile::tempdir().unwrap();
    let shared = shared();
    let regions: Vec<RegionPos> = (0..64).flat_map(|z| (0..64).map(move |x| RegionPos::new(x, z, 0))).collect();
    for region in regions.iter().take(8) {
        Shared::lock(&shared.tiles).insert_loaded(*region, &solid([1, 2, 3, 255]), 1, &mut Vec::new()).unwrap();
    }
    let layout = AreaLayout::new(&regions, 16384).unwrap();
    let service = ExportService::start(&dir.path().join(ExportService::FOLDER), shared).unwrap();
    service.enter_world("w");
    service.export_area(layout, regions, None, 0).unwrap();
    assert!(service.is_busy());
    let one = vec![RegionPos::new(0, 0, 0)];
    let small = AreaLayout::new(&one, 256).unwrap();
    assert_eq!(service.export_view(small, one, None, 0), Err("An export is already running".to_string()));
    service.leave_world();
    assert_eq!(result(&service), "Export stopped: the world was left");
    assert!(!service.is_busy());
    assert_eq!(std::fs::read_dir(dir.path().join(ExportService::FOLDER)).unwrap().count(), 0);
    service.shutdown();
}

#[test]
fn without_a_world_an_export_is_refused_with_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let service = ExportService::start(&dir.path().join(ExportService::FOLDER), shared()).unwrap();
    let one = vec![RegionPos::new(0, 0, 0)];
    let layout = AreaLayout::new(&one, 256).unwrap();
    assert_eq!(service.export_view(layout, one, None, 0), Err("No world is open".to_string()));
    assert!(!service.is_busy());
    service.shutdown();
}

#[test]
fn files_go_to_the_configured_folder_else_pictures_screenshots_else_the_mod_data() {
    let data = Path::new("mods/data");
    let home = Path::new("C:/Users/someone");
    assert_eq!(ExportService::folder("D:/maps", Some(home), data), Path::new("D:/maps"));
    assert_eq!(ExportService::folder("  ", Some(home), data), home.join("Pictures").join("Screenshots"));
    assert_eq!(ExportService::folder("", None, data), data.join(ExportService::FOLDER));
}
