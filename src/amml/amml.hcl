namespace   = "alaydriem"
name        = "minimap"
entry       = "alaydriem_minimap.dll"
api_version = 3

uses {
  feature "minimap" {
    event "client.tick" {
      version  = 1
      required = true
    }
    event "world.left" {
      version  = 1
      required = true
    }
    event "input.key" {
      version  = 1
      required = true
    }
    event "registry.provider_ready" {
      version  = 1
      required = true
    }
    event "registry.provider_gone" {
      version  = 1
      required = true
    }
    event "mods.loaded" {
      version  = 1
      required = true
    }
    event "input.lease_suspended" {
      version  = 1
      required = true
    }
    event "input.lease_resumed" {
      version  = 1
      required = true
    }
    event "input.lease_lost" {
      version  = 1
      required = true
    }
    event "input.injected_released" {
      version  = 2
      required = true
    }
    event "overlay.screenshot_taken" {
      version  = 1
      required = true
    }
    event "overlay.screenshot_written" {
      version  = 1
      required = true
    }
    area "input.lease" {
      version  = 1
      required = true
    }
    area "client.read" {
      version  = 1
      required = true
    }
    area "client.mouse" {
      version  = 1
      required = true
    }
    area "window.read" {
      version  = 1
      required = true
    }
    area "actor.read" {
      version  = 3
      required = true
    }
    area "world.chunk_read" {
      version  = 1
      required = true
    }
  }
  feature "world_identity" {
    area "client.world_info" {
      version  = 1
      required = true
    }
  }
  feature "biomes" {
    area "world.biomes" {
      version  = 1
      required = true
    }
  }
  feature "block_changes" {
    event "world.block_changed" {
      version  = 2
      required = true
    }
  }
  feature "vanilla_colors" {
    area "world.map_color" {
      version  = 1
      required = true
    }
  }
  feature "cursor_clip" {
    area "window.cursor_clip" {
      version  = 1
      required = true
    }
  }
}

listing {
  title       = "Mini Map"
  summary     = "Minimap and full-screen world map with waypoints, cave mode, and PNG export."
  description = <<-EOT
    A full map mod that grows as you explore the world. View a full map, place waypoints and markers, view chunk borders, find slime chunks, and place a minimap anywhere on your screen during gameplay to find your favorite spots, with support for PNG exports.
  EOT
  authors     = ["Charles R. Portwood II"]
  license     = "BSD-3-Clause"
  source      = "https://github.com/alaydriem/amml-minimap"
  homepage    = "https://github.com/alaydriem/amml-minimap#readme"
  tags        = ["minimap", "map", "waypoints", "cave-mode", "export", "client"]
  icon        = "icon.png"
}
