id           = "alaydriem-minimap"
display_name = "Minimap"
version      = "0.1.0"
author       = "Alaydriem"
config_path  = "config.hcl"

# region tiles are 256 KiB each; 512 MiB holds about 2000 of them
texture_budget_mb    = 512
texture_budget_count = 2200

claim "Input" {
  mode   = "Shared"
  reason = "Reads keys and the pointer while the world map is open"
}
claim "Hud" {
  mode   = "Shared"
  reason = "Shows the corner map"
}
claim "Overlay" {
  mode   = "Shared"
  reason = "Shows the world map and its panels"
}

keybind "toggle_world_map" {
  label       = "Open or close the world map"
  default     = "M"
  capability  = "Overlay"
  scope       = "Global"
  config_path = "keys.toggle_world_map"
}
keybind "toggle_minimap" {
  label       = "Show or hide the corner map"
  default     = "Ctrl+M"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.toggle_minimap"
}
keybind "zoom_in" {
  label       = "Corner map zoom in"
  default     = "Equals"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.zoom_in"
}
keybind "zoom_out" {
  label       = "Corner map zoom out"
  default     = "Minus"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.zoom_out"
}
keybind "add_waypoint" {
  label       = "Add a waypoint here"
  default     = "N"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.add_waypoint"
}
keybind "toggle_cave" {
  label       = "Cave view on or off"
  default     = "K"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.toggle_cave"
}
keybind "toggle_coordinates" {
  label       = "Show or hide coordinates"
  default     = "Ctrl+K"
  capability  = "Hud"
  scope       = "Global"
  config_path = "keys.toggle_coordinates"
}

section "general" {
  title = "General"

  field "font_size" {
    label   = "Map text size"
    help    = "Size of the coordinates and waypoint names, in pixels"
    kind    = { float = { min = 6.0, max = 96.0, step = 0.5 } }
    default = 18.0
    apply   = "Live"
  }
}

section "mini_map" {
  title = "Corner map"

  field "mini_map.size_ratio" {
    label   = "Size"
    help    = "Corner map width as a part of the screen width"
    kind    = { float = { min = 0.02, max = 1.0, step = 0.01 } }
    default = 0.15
    apply   = "Live"
  }
  field "mini_map.radius_chunks" {
    label   = "Zoom (chunks)"
    help    = "Chunks shown from the player to the edge; the zoom keys change it"
    kind    = { int = { min = 1, max = 64, step = 1 } }
    default = 5
    apply   = "Live"
  }
  field "mini_map.min_radius_chunks" {
    label   = "Closest zoom (chunks)"
    kind    = { int = { min = 1, max = 64, step = 1 } }
    default = 2
    apply   = "Live"
  }
  field "mini_map.max_radius_chunks" {
    label   = "Farthest zoom (chunks)"
    kind    = { int = { min = 1, max = 64, step = 1 } }
    default = 16
    apply   = "Live"
  }
  field "mini_map.margin_right" {
    label   = "Right margin"
    kind    = { float = { min = 0.0, max = 2000.0, step = 1.0, unit = "px" } }
    default = 20.0
    apply   = "Live"
  }
  field "mini_map.margin_top" {
    label   = "Top margin"
    kind    = { float = { min = 0.0, max = 2000.0, step = 1.0, unit = "px" } }
    default = 20.0
    apply   = "Live"
  }
  field "mini_map.widget_scale" {
    label   = "Marker scale"
    help    = "Size of the player arrow and the waypoint markers"
    kind    = { float = { min = 0.5, max = 3.0, step = 0.05 } }
    default = 1.0
    apply   = "Live"
  }
  field "mini_map.line_thickness" {
    label   = "Chunk line width"
    kind    = { float = { min = 0.0, max = 16.0, step = 0.5, unit = "px" } }
    default = 1.0
    apply   = "Live"
  }
  field "mini_map.border_thickness" {
    label   = "Border width"
    kind    = { float = { min = 0.0, max = 16.0, step = 0.5, unit = "px" } }
    default = 2.0
    apply   = "Live"
  }
  field "mini_map.coord_text_margin" {
    label   = "Coordinates gap"
    kind    = { float = { min = 0.0, max = 200.0, step = 1.0, unit = "px" } }
    default = 8.0
    apply   = "Live"
  }
  field "mini_map.show_coordinates" {
    label   = "Show coordinates"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "mini_map.show_nether_coordinates" {
    label   = "Show Nether coordinates"
    help    = "Also shows the matching Nether or Overworld position"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "mini_map.rotate_with_player" {
    label   = "Turn with the player"
    help    = "Keeps the player's facing at the top"
    kind    = "bool"
    default = false
    apply   = "Live"
  }
  field "mini_map.background_color" {
    label   = "Background"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 255
    apply   = "Live"
  }
  field "mini_map.chunk_line_color" {
    label   = "Chunk lines"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 2021161215
    apply   = "Live"
  }
  field "mini_map.current_chunk_color" {
    label   = "Current chunk"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 983629823
    apply   = "Live"
  }
  field "mini_map.border_color" {
    label   = "Border"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 1347440895
    apply   = "Live"
  }
  field "mini_map.player_arrow_outline_color" {
    label   = "Player arrow outline"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 255
    apply   = "Live"
  }
  field "mini_map.player_arrow_fill_color" {
    label   = "Player arrow fill"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 3694277375
    apply   = "Live"
  }
  field "mini_map.coord_text_color" {
    label   = "Coordinates text"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 4294967295
    apply   = "Live"
  }
  field "mini_map.coord_shadow_color" {
    label   = "Coordinates shadow"
    kind    = { color = { alpha = true, encoding = "PackedRgbaU32" } }
    default = 200
    apply   = "Live"
  }
}

section "world_map" {
  title = "World map"

  field "world_map.show_coordinates" {
    label   = "Show coordinates"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "world_map.show_nether_coordinates" {
    label   = "Show Nether coordinates"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "world_map.rotate_with_player" {
    label   = "Turn with the player"
    kind    = "bool"
    default = false
    apply   = "Live"
  }
  field "world_map.show_chunk_grid" {
    label   = "Chunk grid"
    kind    = "bool"
    default = false
    apply   = "Live"
  }
  field "world_map.show_slime_chunks" {
    label   = "Slime chunks"
    kind    = "bool"
    default = false
    apply   = "Live"
  }
  field "world_map.min_zoom" {
    label   = "Farthest zoom"
    help    = "Pixels per block"
    kind    = { float = { min = 0.01, max = 10.0, step = 0.01 } }
    default = 0.2
    apply   = "Live"
  }
  field "world_map.max_zoom" {
    label   = "Closest zoom"
    help    = "Pixels per block"
    kind    = { float = { min = 0.01, max = 400.0, step = 0.5 } }
    default = 40.0
    apply   = "Live"
  }
  field "world_map.zoom_step" {
    label   = "Zoom step"
    help    = "Zoom factor for one wheel notch"
    kind    = { float = { min = 1.01, max = 4.0, step = 0.01 } }
    default = 1.15
    apply   = "Live"
  }
  field "world_map.waypoint_label_zoom" {
    label   = "Waypoint names from zoom"
    help    = "Waypoint names show from this zoom up, in pixels per block"
    kind    = { float = { min = 0.01, max = 400.0, step = 0.01 } }
    default = 0.5
    apply   = "Live"
  }
  field "world_map.max_new_textures_per_frame" {
    label   = "New tiles per frame"
    kind    = { int = { min = 1, max = 64, step = 1 } }
    default = 4
    apply   = "Restart"
  }
  field "world_map.texture_idle_frames" {
    label   = "Unused tile life (frames)"
    kind    = { int = { min = 1, max = 1000000, step = 60 } }
    default = 600
    apply   = "Restart"
  }
  field "world_map.hover_biome_regions" {
    label   = "Hover biome cache (regions)"
    kind    = { int = { min = 1, max = 1024, step = 1 } }
    default = 16
    apply   = "Restart"
  }
  field "world_map.slime_cache_regions" {
    label   = "Slime cache (regions)"
    kind    = { int = { min = 64, max = 65536, step = 64 } }
    default = 1024
    apply   = "Restart"
  }
}

section "smooth_camera" {
  title = "Smooth camera"

  field "smooth_camera.spring_strength" {
    label   = "Spring strength"
    kind    = { float = { min = 0.0, max = 1000.0, step = 1.0 } }
    default = 36.0
    apply   = "Live"
  }
  field "smooth_camera.damping" {
    label   = "Damping"
    kind    = { float = { min = 0.0, max = 1000.0, step = 0.5 } }
    default = 7.0
    apply   = "Live"
  }
  field "smooth_camera.max_delta_time" {
    label   = "Longest step"
    kind    = { float = { min = 0.001, max = 1.0, step = 0.001, unit = "s" } }
    default = 0.1
    apply   = "Live"
  }
}

section "waypoints" {
  title = "Waypoints"

  field "waypoints.enabled" {
    label   = "Waypoints"
    kind    = "bool"
    default = true
    apply   = "WorldReload"
  }
  field "waypoints.show_on_minimap" {
    label   = "Show on the corner map"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "waypoints.max_edge_arrows" {
    label   = "Edge arrows"
    help    = "Waypoints outside the corner map shown as arrows on its edge"
    kind    = { int = { min = 0, max = 64, step = 1 } }
    default = 16
    apply   = "Live"
  }
  field "waypoints.label_scale" {
    label   = "Name scale"
    kind    = { float = { min = 0.5, max = 4.0, step = 0.05 } }
    default = 1.0
    apply   = "Live"
  }
  field "waypoints.max_per_world" {
    label   = "Waypoints per world"
    kind    = { int = { min = 1, max = 4096, step = 1 } }
    default = 512
    apply   = "Restart"
  }
}

section "terrain" {
  title = "Terrain"

  field "terrain.enable" {
    label   = "Scan terrain"
    kind    = "bool"
    default = true
    apply   = "Live"
  }
  field "terrain.scan_radius" {
    label   = "Scan radius (blocks)"
    kind    = { int = { min = 16, max = 1024, step = 16 } }
    default = 128
    apply   = "WorldReload"
  }
  field "terrain.scan_max_chunks_per_frame" {
    label   = "Chunks scanned per tick"
    kind    = { int = { min = 1, max = 1024, step = 1 } }
    default = 32
    apply   = "WorldReload"
  }
  field "terrain.rescan_interval_frames" {
    label   = "Rescan interval (ticks)"
    kind    = { int = { min = 1, max = 1000000, step = 20 } }
    default = 1200
    apply   = "WorldReload"
  }
  field "terrain.region_bake_threshold" {
    label   = "Chunks before a region bake"
    kind    = { int = { min = 1, max = 256, step = 1 } }
    default = 64
    apply   = "WorldReload"
  }
  field "terrain.enable_disk_cache" {
    label   = "Disk cache"
    kind    = "bool"
    default = true
    apply   = "WorldReload"
  }
  field "terrain.enable_transparent_water" {
    label   = "Transparent water"
    kind    = "bool"
    default = true
    apply   = "Restart"
  }
  field "terrain.water_opacity_per_depth" {
    label   = "Water opacity per block"
    kind    = { float = { min = 0.0, max = 1.0, step = 0.01 } }
    default = 0.1
    apply   = "Restart"
  }
  field "terrain.water_max_opacity" {
    label   = "Most water opacity"
    kind    = { float = { min = 0.0, max = 1.0, step = 0.01 } }
    default = 0.6
    apply   = "Restart"
  }
}

section "shadow" {
  title = "Shading"

  field "terrain.shadow.render_style" {
    label   = "Style"
    kind    = { enum = { options = [{ value = 0, label = "Flat" }, { value = 1, label = "Gradient" }, { value = 2, label = "Shadow map" }] } }
    default = 2
    apply   = "Restart"
  }
  field "terrain.shadow.shadow_level" {
    label   = "Shadow strength"
    kind    = { int = { min = 1, max = 1000, step = 5 } }
    default = 130
    apply   = "Restart"
  }
  field "terrain.shadow.render_scale" {
    label   = "Shadow scale"
    kind    = { int = { min = 1, max = 16, step = 1 } }
    default = 2
    apply   = "Restart"
  }
  field "terrain.shadow.pcf_radius" {
    label   = "Shadow blur"
    kind    = { int = { min = 0, max = 8, step = 1 } }
    default = 1
    apply   = "Restart"
  }
  field "terrain.shadow.light_azimuth" {
    label   = "Light direction"
    kind    = { float = { min = 0.0, max = 360.0, step = 1.0, unit = "deg" } }
    default = 315.0
    apply   = "Restart"
  }
  field "terrain.shadow.light_zenith" {
    label   = "Light height"
    kind    = { float = { min = 1.0, max = 89.0, step = 1.0, unit = "deg" } }
    default = 60.0
    apply   = "Restart"
  }
}

section "cave_mode" {
  title = "Cave view"

  field "cave_mode.solid_color" {
    label   = "Solid rock"
    kind    = { color = { alpha = true, encoding = "ByteArray4" } }
    default = [96, 96, 96, 255]
    apply   = "WorldReload"
  }
  field "cave_mode.band_size" {
    label   = "Band height"
    kind    = { enum = { options = [{ value = 8, label = "8 blocks" }, { value = 16, label = "16 blocks" }, { value = 32, label = "32 blocks" }] } }
    default = 16
    apply   = "WorldReload"
  }
  field "cave_mode.band_margin" {
    label   = "Band margin"
    kind    = { int = { min = 0, max = 15, step = 1 } }
    default = 2
    apply   = "WorldReload"
  }
  field "cave_mode.scan_radius" {
    label   = "Scan radius (blocks)"
    kind    = { int = { min = 16, max = 1024, step = 16 } }
    default = 64
    apply   = "WorldReload"
  }
}

section "export" {
  title = "Export"

  field "export.max_pixels" {
    label   = "Largest image side"
    kind    = { int = { min = 256, max = 65536, step = 256 } }
    default = 16384
    apply   = "WorldReload"
  }
}

section "memory" {
  title = "Memory"

  field "memory.chunk_surface_budget_mb" {
    label   = "Chunk data (MiB)"
    kind    = { int = { min = 8, max = 4096, step = 8 } }
    default = 64
    apply   = "Restart"
  }
  field "memory.tile_budget_mb" {
    label   = "Map tiles in memory (MiB)"
    kind    = { int = { min = 8, max = 4096, step = 8 } }
    default = 256
    apply   = "Restart"
  }
  field "memory.tile_overrun_percent" {
    label   = "Tile overrun (%)"
    kind    = { int = { min = 0, max = 100, step = 1 } }
    default = 25
    apply   = "Restart"
  }
  field "memory.gpu_texture_budget_mb" {
    label   = "Map tiles on the GPU (MiB)"
    help    = "Capped by the texture budget the mod manager grants"
    kind    = { int = { min = 16, max = 2048, step = 16 } }
    default = 496
    apply   = "Restart"
  }
}

section "bake" {
  title = "Bake"

  field "bake.worker_threads" {
    label   = "Worker threads"
    help    = "0 picks a count from the processor"
    kind    = { int = { min = 0, max = 64, step = 1 } }
    default = 0
    apply   = "Restart"
  }
  field "bake.queue_capacity" {
    label   = "Queue size"
    kind    = { int = { min = 64, max = 1000000, step = 64 } }
    default = 4096
    apply   = "Restart"
  }
}

section "store" {
  title = "Disk store"

  field "store.journal_max_mb" {
    label   = "Journal size (MiB)"
    kind    = { int = { min = 4, max = 4096, step = 4 } }
    default = 64
    apply   = "Restart"
  }
  field "store.sync_interval_ms" {
    label   = "Sync interval (ms)"
    kind    = { int = { min = 100, max = 60000, step = 100 } }
    default = 1000
    apply   = "Restart"
  }
  field "store.write_queue_capacity" {
    label   = "Write queue size"
    kind    = { int = { min = 16, max = 1000000, step = 16 } }
    default = 1024
    apply   = "Restart"
  }
}

section "debug" {
  title = "Debug"

  field "debug.telemetry" {
    label   = "Telemetry"
    help    = "Writes telemetry.csv in the data folder"
    kind    = "bool"
    default = false
    apply   = "Restart"
  }
  field "debug.telemetry_interval_ms" {
    label   = "Telemetry interval (ms)"
    kind    = { int = { min = 100, max = 60000, step = 100 } }
    default = 1000
    apply   = "Restart"
  }
  field "debug.capture_probes" {
    label   = "Record probe captures"
    kind    = "bool"
    default = false
    apply   = "Restart"
  }
}

section "keys" {
  title = "Keys"

  field "keys.toggle_world_map" {
    label   = "Open or close the world map"
    kind    = "chord"
    default = "M"
    apply   = "Live"
  }
  field "keys.toggle_minimap" {
    label   = "Show or hide the corner map"
    kind    = "chord"
    default = "Ctrl+M"
    apply   = "Live"
  }
  field "keys.zoom_in" {
    label   = "Corner map zoom in"
    kind    = "chord"
    default = "Equals"
    apply   = "Live"
  }
  field "keys.zoom_out" {
    label   = "Corner map zoom out"
    kind    = "chord"
    default = "Minus"
    apply   = "Live"
  }
  field "keys.add_waypoint" {
    label   = "Add a waypoint here"
    kind    = "chord"
    default = "N"
    apply   = "Live"
  }
  field "keys.toggle_cave" {
    label   = "Cave view on or off"
    kind    = "chord"
    default = "K"
    apply   = "Live"
  }
  field "keys.toggle_coordinates" {
    label   = "Show or hide coordinates"
    kind    = "chord"
    default = "Ctrl+K"
    apply   = "Live"
  }
}
