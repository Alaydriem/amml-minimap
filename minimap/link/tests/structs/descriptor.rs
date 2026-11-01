use amm_core::services::descriptor::DescriptorChecker;
use amm_core::services::settings::SchemaChecker;
use amm_core::structs::descriptor::{
    ApplyMode, Capability, ClaimMode, Descriptor as Parsed, FieldKind, KeyScope, SettingField,
};
use amm_core::structs::settings::HclDocument;
use minimap_core::config::{Config, ConfigMerge};
use minimap_core::structs::input::KeybindAction;
use minimap_link::Descriptor;

fn parsed() -> Parsed {
    match DescriptorChecker::new(128, 256).parse(Descriptor::TEXT) {
        Ok(descriptor) => descriptor,
        Err(error) => panic!("the manager refused the descriptor: {error:?}"),
    }
}

#[test]
fn the_manager_checker_accepts_the_descriptor() {
    let descriptor = parsed();
    assert_eq!(descriptor.namespace, "alaydriem");
    assert_eq!(descriptor.name, "minimap");
    assert_eq!(descriptor.id(), "@alaydriem/minimap");
    assert_eq!(descriptor.config_path, "config.hcl");
}

#[test]
fn claims_are_shared_input_hud_overlay() {
    let descriptor = parsed();
    let claims: Vec<_> = descriptor.claims.iter().map(|claim| (claim.capability, claim.mode)).collect();
    assert_eq!(
        claims,
        [
            (Capability::Input, ClaimMode::Shared),
            (Capability::Hud, ClaimMode::Shared),
            (Capability::Overlay, ClaimMode::Shared)
        ]
    );
    assert!(descriptor.claims.iter().all(|claim| !claim.hide_vanilla_hud && !claim.block_player_turn));
}

#[test]
fn keybind_order_matches_keybind_action() {
    let descriptor = parsed();
    let actions: Vec<_> = descriptor.keybinds.iter().map(|keybind| keybind.action.as_str()).collect();
    let expected: Vec<_> = KeybindAction::ALL.iter().map(|action| action.id()).collect();
    assert_eq!(actions, expected);
    assert!(descriptor.keybinds.iter().all(|keybind| keybind.scope == KeyScope::Global));
}

#[test]
fn schema_defaults_are_the_config_defaults() {
    let descriptor = parsed();
    let checked = SchemaChecker::check(&descriptor.settings, HclDocument::empty());
    let text = checked.document.text();
    let config: Config = match hcl::from_str(&text) {
        Ok(config) => config,
        Err(error) => panic!("the default document does not parse as Config: {error}\n{text}"),
    };
    assert_eq!(config, Config::default(), "{text}");
}

// a valid value for the field that is not its default
fn other_value(field: &SettingField) -> hcl::Value {
    match &field.kind {
        FieldKind::Bool => hcl::Value::from(field.default != hcl::Value::from(true)),
        FieldKind::Int { min, max, .. } => {
            let value = if field.default == hcl::Value::from(*min) { *max } else { *min };
            hcl::Value::from(value)
        }
        FieldKind::Float { min, max, .. } => {
            let value = if field.default.as_f64() == Some(*min) { *max } else { *min };
            hcl::Value::from(value)
        }
        FieldKind::Enum { options } => match options.iter().find(|option| option.value != field.default) {
            Some(option) => option.value.clone(),
            None => panic!("{} has one option", field.path),
        },
        FieldKind::Color { .. } => match &field.default {
            hcl::Value::Array(_) => hcl::Value::from(vec![1u32, 2, 3, 4]),
            _ => hcl::Value::from(0x0102_0304u32),
        },
        FieldKind::Chord => hcl::Value::from("Ctrl+Shift+F9"),
    }
}

#[test]
fn every_schema_field_reaches_a_config_field() {
    let descriptor = parsed();
    let defaults = SchemaChecker::check(&descriptor.settings, HclDocument::empty()).document;
    for field in descriptor.settings.iter().flat_map(|section| &section.fields) {
        let text = defaults.with(&field.path, &other_value(field)).text();
        let config: Config = match hcl::from_str(&text) {
            Ok(config) => config,
            Err(error) => panic!("{} does not parse: {error}", field.path),
        };
        assert_ne!(config, Config::default(), "{} is not read into Config", field.path);
    }
}

// every Live field reaches a running runtime at once, every WorldReload field at the next world, and nothing
// else changes before a restart; the descriptor's apply modes and ConfigMerge must agree
#[test]
fn apply_modes_match_the_config_merge() {
    let descriptor = parsed();
    let defaults = SchemaChecker::check(&descriptor.settings, HclDocument::empty()).document;
    let current = Config::default();
    for field in descriptor.settings.iter().flat_map(|section| &section.fields) {
        let new: Config = match hcl::from_str(&defaults.with(&field.path, &other_value(field)).text()) {
            Ok(config) => config,
            Err(error) => panic!("{} does not parse: {error}", field.path),
        };
        let live = ConfigMerge::live(&current, &new) != current;
        let world = ConfigMerge::world(&current, &new) != current;
        let expected = match field.apply {
            ApplyMode::Live => (true, true),
            ApplyMode::WorldReload => (false, true),
            ApplyMode::Restart => (false, false),
        };
        assert_eq!((live, world), expected, "{} is {:?}", field.path, field.apply);
    }
}

#[test]
fn the_descriptor_asks_for_512_mib_in_2200_textures() {
    let descriptor = parsed();

    assert_eq!((descriptor.texture_budget_mb, descriptor.texture_budget_count), (Some(512), Some(2200)));
}
