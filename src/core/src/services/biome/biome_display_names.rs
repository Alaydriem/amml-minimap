// the game's internal biome names and the English names players see, from the Minecraft Wiki
// Bedrock Edition biome list
pub struct BiomeDisplayNames;

impl BiomeDisplayNames {
    // sorted by internal name, so a lookup is a binary search
    pub const ENTRIES: &'static [(&'static str, &'static str)] = &[
        ("bamboo_jungle", "Bamboo Jungle"),
        ("bamboo_jungle_hills", "Bamboo Jungle Hills"),
        ("basalt_deltas", "Basalt Deltas"),
        ("beach", "Beach"),
        ("birch_forest", "Birch Forest"),
        ("birch_forest_hills", "Birch Forest Hills"),
        ("birch_forest_hills_mutated", "Tall Birch Hills"),
        ("birch_forest_mutated", "Old Growth Birch Forest"),
        ("cherry_grove", "Cherry Grove"),
        ("cold_beach", "Snowy Beach"),
        ("cold_ocean", "Cold Ocean"),
        ("cold_taiga", "Snowy Taiga"),
        ("cold_taiga_hills", "Snowy Taiga Hills"),
        ("cold_taiga_mutated", "Snowy Taiga Mountains"),
        ("crimson_forest", "Crimson Forest"),
        ("deep_cold_ocean", "Deep Cold Ocean"),
        ("deep_dark", "Deep Dark"),
        ("deep_frozen_ocean", "Deep Frozen Ocean"),
        ("deep_lukewarm_ocean", "Deep Lukewarm Ocean"),
        ("deep_ocean", "Deep Ocean"),
        ("deep_warm_ocean", "Deep Warm Ocean"),
        ("desert", "Desert"),
        ("desert_hills", "Desert Hills"),
        ("desert_mutated", "Desert Lakes"),
        ("dripstone_caves", "Dripstone Caves"),
        ("extreme_hills", "Windswept Hills"),
        ("extreme_hills_edge", "Mountain Edge"),
        ("extreme_hills_mutated", "Windswept Gravelly Hills"),
        ("extreme_hills_plus_trees", "Windswept Forest"),
        ("extreme_hills_plus_trees_mutated", "Gravelly Mountains+"),
        ("flower_forest", "Flower Forest"),
        ("forest", "Forest"),
        ("forest_hills", "Wooded Hills"),
        ("frozen_ocean", "Frozen Ocean"),
        ("frozen_peaks", "Frozen Peaks"),
        ("frozen_river", "Frozen River"),
        ("grove", "Grove"),
        ("hell", "Nether Wastes"),
        ("ice_mountains", "Snowy Mountains"),
        ("ice_plains", "Snowy Plains"),
        ("ice_plains_spikes", "Ice Spikes"),
        ("jagged_peaks", "Jagged Peaks"),
        ("jungle", "Jungle"),
        ("jungle_edge", "Sparse Jungle"),
        ("jungle_edge_mutated", "Modified Jungle Edge"),
        ("jungle_hills", "Jungle Hills"),
        ("jungle_mutated", "Modified Jungle"),
        ("legacy_frozen_ocean", "Legacy Frozen Ocean"),
        ("lukewarm_ocean", "Lukewarm Ocean"),
        ("lush_caves", "Lush Caves"),
        ("mangrove_swamp", "Mangrove Swamp"),
        ("meadow", "Meadow"),
        ("mega_taiga", "Old Growth Pine Taiga"),
        ("mega_taiga_hills", "Giant Tree Taiga Hills"),
        ("mesa", "Badlands"),
        ("mesa_bryce", "Eroded Badlands"),
        ("mesa_plateau", "Badlands Plateau"),
        ("mesa_plateau_mutated", "Modified Badlands Plateau"),
        ("mesa_plateau_stone", "Wooded Badlands"),
        ("mesa_plateau_stone_mutated", "Modified Wooded Badlands Plateau"),
        ("mushroom_island", "Mushroom Fields"),
        ("mushroom_island_shore", "Mushroom Field Shore"),
        ("ocean", "Ocean"),
        ("pale_garden", "Pale Garden"),
        ("plains", "Plains"),
        ("redwood_taiga_hills_mutated", "Giant Spruce Taiga Hills"),
        ("redwood_taiga_mutated", "Old Growth Spruce Taiga"),
        ("river", "River"),
        ("roofed_forest", "Dark Forest"),
        ("roofed_forest_mutated", "Dark Forest Hills"),
        ("savanna", "Savanna"),
        ("savanna_mutated", "Windswept Savanna"),
        ("savanna_plateau", "Savanna Plateau"),
        ("savanna_plateau_mutated", "Shattered Savanna Plateau"),
        ("snowy_slopes", "Snowy Slopes"),
        ("soulsand_valley", "Soul Sand Valley"),
        ("stone_beach", "Stony Shore"),
        ("stony_peaks", "Stony Peaks"),
        ("sunflower_plains", "Sunflower Plains"),
        ("swampland", "Swamp"),
        ("swampland_mutated", "Swamp Hills"),
        ("taiga", "Taiga"),
        ("taiga_hills", "Taiga Hills"),
        ("taiga_mutated", "Taiga Mountains"),
        ("the_end", "The End"),
        ("warm_ocean", "Warm Ocean"),
        ("warped_forest", "Warped Forest"),
    ];

    pub const VANILLA: &'static str = "minecraft";

    // the player name of a vanilla biome; the game's name may carry the "minecraft:" namespace
    pub fn display(internal: &str) -> Option<&'static str> {
        let (namespace, name) = Self::split(internal);
        if namespace.is_some_and(|n| n != Self::VANILLA) {
            return None;
        }
        Self::ENTRIES.binary_search_by(|(entry, _)| (*entry).cmp(name)).ok().map(|i| Self::ENTRIES[i].1)
    }

    // the player name, else the name without its namespace in words ("my_pack:glow_marsh" is
    // "Glow Marsh"), for an add-on biome or one newer than this table. `out` keeps its capacity
    pub fn write_label(internal: &str, out: &mut String) {
        out.clear();
        if let Some(display) = Self::display(internal) {
            out.push_str(display);
            return;
        }
        let (_, name) = Self::split(internal);
        for (i, word) in name.split('_').filter(|w| !w.is_empty()).enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }

    fn split(internal: &str) -> (Option<&str>, &str) {
        match internal.split_once(':') {
            Some((namespace, name)) => (Some(namespace), name),
            None => (None, internal),
        }
    }
}
