use minimap_core::config::TerrainConfig;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::input::{KeyPhase, KeybindAction, KeybindInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::errors::CaptureError;
use minimap_runtime::services::CaptureCodec;
use minimap_runtime::structs::{CaptureEvent, CaptureHeader, SubmitStatus, TickInput};

fn every_kind() -> Vec<CaptureEvent> {
    let mut surface = ChunkSurface::empty();
    surface.write_column(5, ColumnSample::new(0xFF10_2030, 0x0500_00C8, 70, 64, true, 0));
    let player = PlayerSnapshot { present: true, x: -12.5, y: 70.0, z: 300.25, yaw: 91.0, dim: 1 };
    vec![
        CaptureEvent::Enter { dim: 1 },
        CaptureEvent::Tick(TickInput {
            player,
            foreground: true,
            cursor_hidden: false,
            dead: false,
            mouse_grabbed: true,
            ..TickInput::default()
        }),
        CaptureEvent::Submit {
            pos: ChunkPos::new(-3, 7, 1),
            status: SubmitStatus::Loaded,
            probe: Box::new(ChunkProbe::new(surface, [21; 256])),
        },
        CaptureEvent::Submit {
            pos: ChunkPos::new(4, 4, 1),
            status: SubmitStatus::NotProbed,
            probe: Box::new(ChunkProbe::new(ChunkSurface::empty(), [0; 256])),
        },
        CaptureEvent::BiomeName { id: 4, name: "forest".to_string() },
        CaptureEvent::Column {
            pos: ChunkPos::new(0, -1, 0),
            index: 255,
            sample: ColumnSample::new(1, 2, -60, -61, false, 21),
        },
        CaptureEvent::Block { x: -100, z: 2_000_000, dim: 2 },
        CaptureEvent::Keybind(KeybindInput {
            action: KeybindAction::ToggleCoordinates,
            phase: KeyPhase::Repeat,
            mouse_grabbed: false,
        }),
        CaptureEvent::Retired(CaptureCodec::KEY),
        CaptureEvent::Retired(CaptureCodec::MOUSE),
        CaptureEvent::Retired(CaptureCodec::FRAME),
        CaptureEvent::Leave,
    ]
}

#[test]
fn every_event_kind_survives_a_round_trip() {
    for event in every_kind() {
        let mut bytes = Vec::new();
        CaptureCodec::encode(&event, &mut bytes);
        let len = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
        assert_eq!(len, bytes.len() - CaptureCodec::RECORD_HEADER_BYTES);
        assert_eq!(Some(len), CaptureCodec::payload_bytes(bytes[0]));
        let decoded = CaptureCodec::decode(bytes[0], &bytes[CaptureCodec::RECORD_HEADER_BYTES..]).unwrap();
        assert_eq!(decoded, event);
    }
}

#[test]
fn the_header_round_trips_with_its_settings_and_compression() {
    let mut terrain = TerrainConfig::default();
    terrain.shadow.render_scale = 4;
    let header = CaptureHeader::new(&terrain);
    let mut bytes = Vec::new();
    CaptureCodec::write_header(&mut bytes, &header, CaptureCodec::RAW).unwrap();
    assert_eq!(&bytes[0..4], b"LMCP");
    let (read, compression) = CaptureCodec::read_header(&mut bytes.as_slice()).unwrap();
    assert_eq!(read, header);
    assert_eq!(compression, CaptureCodec::RAW);
}

#[test]
fn a_foreign_file_is_rejected() {
    let bytes = b"PNG\x00 not a capture at all, but long enough".to_vec();
    assert!(matches!(CaptureCodec::read_header(&mut bytes.as_slice()), Err(CaptureError::BadMagic)));
}

#[test]
fn bad_records_are_rejected_not_misread() {
    assert!(matches!(CaptureCodec::decode(200, &[]), Err(CaptureError::UnknownTag(200))));
    assert!(matches!(CaptureCodec::decode(CaptureCodec::KEY, &[0; 3]), Err(CaptureError::WrongSize { .. })));
    let mut submit = vec![0u8; CaptureCodec::payload_bytes(CaptureCodec::SUBMIT).unwrap()];
    submit[12] = 9;
    assert!(matches!(CaptureCodec::decode(CaptureCodec::SUBMIT, &submit), Err(CaptureError::BadValue(_))));
    let mut column = vec![0u8; CaptureCodec::payload_bytes(CaptureCodec::COLUMN).unwrap()];
    column[12..16].copy_from_slice(&256u32.to_le_bytes());
    assert!(matches!(CaptureCodec::decode(CaptureCodec::COLUMN, &column), Err(CaptureError::BadValue(_))));
}

#[test]
fn a_version_one_capture_is_refused() {
    let mut bytes = Vec::new();
    CaptureCodec::write_header(&mut bytes, &CaptureHeader::new(&TerrainConfig::default()), CaptureCodec::RAW).unwrap();
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    assert!(matches!(CaptureCodec::read_header(&mut bytes.as_slice()), Err(CaptureError::UnsupportedVersion(1))));
}

#[test]
fn an_overlong_biome_name_is_cut_to_the_record_size() {
    let mut bytes = Vec::new();
    CaptureCodec::encode(&CaptureEvent::BiomeName { id: 1, name: "x".repeat(80) }, &mut bytes);
    assert_eq!(
        bytes.len() - CaptureCodec::RECORD_HEADER_BYTES,
        CaptureCodec::payload_bytes(CaptureCodec::BIOME_NAME).unwrap()
    );
    let decoded = CaptureCodec::decode(bytes[0], &bytes[CaptureCodec::RECORD_HEADER_BYTES..]).unwrap();
    assert_eq!(decoded, CaptureEvent::BiomeName { id: 1, name: "x".repeat(CaptureCodec::NAME_BYTES) });
}

// captures recorded before the HCL config wrote the terrain block with camelCase keys
#[test]
fn a_header_with_camel_case_terrain_keys_still_reads() {
    let json = br#"{"scanRadius":48,"enableTransparentWater":false,"shadow":{"renderStyle":0,"pcfRadius":3}}"#;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(CaptureCodec::MAGIC);
    bytes.extend_from_slice(&CaptureCodec::VERSION.to_le_bytes());
    bytes.extend_from_slice(&[CaptureCodec::RAW, 0]);
    bytes.extend_from_slice(&7u64.to_le_bytes());
    bytes.extend_from_slice(&(json.len() as u32).to_le_bytes());
    bytes.extend_from_slice(json);

    let (header, _) = CaptureCodec::read_header(&mut bytes.as_slice()).unwrap();

    assert_eq!(header.terrain.scan_radius, 48);
    assert!(!header.terrain.enable_transparent_water);
    assert_eq!((header.terrain.shadow.render_style, header.terrain.shadow.pcf_radius), (0, 3));
}
