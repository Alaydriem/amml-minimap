namespace   = "alaydriem"
name        = "minimap"
entry       = "alaydriem_minimap.dll"
api_version = 2
amml_version = "26.60.6"
entries = [
  # required: the minimap refuses to enable without them
  "ClientInstance.update",
  "ClientInstance.getLocalPlayer",
  "ClientInstance.isWorldActive",
  "ClientInstance.getMouseGrabbed",
  "ClientInstance.getRegion",
  "Actor.stateVectorComponent",
  "Actor.actorRotationComponent",
  "StateVectorComponent.mPos",
  "ActorRotationComponent.mRot",
  "Actor.getDimension",
  "Dimension.getDimensionId",
  "Actor.getDimensionBlockSource",
  "Actor.isAlive",
  "Actor.mLevel",
  "BlockSource.getChunk",
  "LevelChunk.mSubChunks",
  "LevelChunk.getBlock",
  "SubChunk.size",
  "SubChunk.isPlaceHolderSubChunk",
  # required: the block material layout, by pointer (26.51) or by value (26.60)
  "BlockType.mMaterialInline",
  # a recipe helper: other entries are derived from it
  "BlockSource.vtable",
  # optional, minimap.server_identity
  "ClientInstance.getGameConnectionInfo",
  "GameConnectionInfo.size",
  "GameConnectionInfo.dtor",
  "Level.getSeed",
  # optional, minimap.world_folder
  "DBStorage.vtable",
  "DBStorage.ctor",
  "DBStorage.checkShutdownDone",
  # optional, minimap.biomes
  "Level.getBiomeRegistry",
  "BiomeRegistry.lookupById",
  "LevelChunk.getBiome",
  # optional, minimap.block_changes
  "BlockSourceListener.vtable",
  "BlockSourceListener.onSourceDestroyed",
  "BlockSourceListener.onBlockChanged",
  "BlockSource.addListener",
  "BlockSource.removeListener",
  "BlockSource.getDimensionId",
  # optional, minimap.vanilla_colors
  "Dimension.getBlockSourceFromMainChunkSource",
  "Block.tryGetComponent.BlockMapColorComponent",
  "BiomeColorSampling.getMapPolicy",
  "MapPolicy.get",
  "BlockType.getMapColor",
  "BiomeColorSampling.getWaterColor",
  "BiomeColorSampling.getMapGrassColor",
]

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
