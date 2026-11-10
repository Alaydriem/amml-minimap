use minimap_core::services::cave::CaveSurface;
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkSurface, ColumnSample};

const ROCK: [u8; 4] = [96, 96, 96, 255];

#[test]
fn see_through_columns_become_rock_and_every_probed_column_is_flat_at_the_floor() {
    let mut surface = ChunkSurface::empty();
    let ore = Rgba::new(200, 60, 20, 255).packed();
    surface.write_column(0, ColumnSample::new(0, 0, 70, 66, false, 0));
    surface.write_column(1, ColumnSample::new(ore, 0, 70, 66, false, 0));
    CaveSurface::new(ROCK).prepare_chunk(&mut surface, 32);
    assert_eq!(surface.surface[0], Rgba::new(96, 96, 96, 255).packed());
    assert_eq!(surface.surface[1], ore);
    assert_eq!((surface.height[0], surface.solid[0], surface.height[1]), (32, 32, 32));
    assert!(surface.is_skipped(2));
    assert_eq!(surface.height[2], ChunkSurface::NO_HEIGHT);
}

#[test]
fn a_single_column_is_prepared_the_same_way() {
    let rock = CaveSurface::new(ROCK);
    let column = rock.prepare_column(ColumnSample::new(0, 7, 90, 88, true, 0), 47);
    assert_eq!(
        (column.surface, column.water, column.height, column.solid),
        (Rgba::new(96, 96, 96, 255).packed(), 7, 47, 47)
    );
    assert!(column.retry);
}
