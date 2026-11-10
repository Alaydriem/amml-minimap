use minimap_core::services::export::AreaRow;

const SIDE: usize = 256;

// a tile whose texel (x, z) is (x, z, 7, 255), except see-through where `clear` says
fn tile(clear: impl Fn(usize, usize) -> bool) -> Vec<u8> {
    let mut pixels = vec![0u8; SIDE * SIDE * 4];
    for z in 0..SIDE {
        for x in 0..SIDE {
            if !clear(x, z) {
                pixels[(z * SIDE + x) * 4..(z * SIDE + x) * 4 + 4].copy_from_slice(&[x as u8, z as u8, 7, 255]);
            }
        }
    }
    pixels
}

fn texel(row: &[u8], width: usize, x: usize, z: usize) -> [u8; 4] {
    let i = (z * width + x) * 4;
    [row[i], row[i + 1], row[i + 2], row[i + 3]]
}

#[test]
fn at_one_block_per_pixel_the_tile_is_copied_into_its_column() {
    let width = SIDE * 3;
    let mut row = vec![9u8; width * SIDE * 4];
    AreaRow::place(&mut row, width, 1, &tile(|_, _| false), 1);
    assert_eq!(texel(&row, width, SIDE + 5, 6), [5, 6, 7, 255]);
    assert_eq!(texel(&row, width, 2 * SIDE - 1, 255), [255, 255, 7, 255]);
    assert_eq!(texel(&row, width, 0, 0), [9, 9, 9, 9]);
}

#[test]
fn a_coarser_scale_averages_each_block_of_texels() {
    let width = SIDE / 2;
    let mut row = vec![0u8; width * (SIDE / 2) * 4];
    AreaRow::place(&mut row, width, 0, &tile(|_, _| false), 2);
    assert_eq!(texel(&row, width, 3, 5), [6, 10, 7, 255]);
}

#[test]
fn see_through_texels_lower_the_alpha_but_not_the_colour_and_an_empty_block_stays_clear() {
    let width = SIDE / 2;
    let mut row = vec![1u8; width * (SIDE / 2) * 4];
    AreaRow::place(&mut row, width, 0, &tile(|x, z| x % 2 == 1 || z >= 200), 2);
    assert_eq!(texel(&row, width, 3, 5), [6, 10, 7, 127]);
    assert_eq!(texel(&row, width, 3, 100), [0, 0, 0, 0]);
}
