use crate::noita::types::{AABB, Color, Entity, StdString, StdVec};
#[repr(C)]
#[derive(Debug)]
pub struct ExplosionConfigVTable {}

#[repr(C)]
#[derive(Debug)]
pub struct ExplosionConfig {
    pub vftable: &'static ExplosionConfigVTable,
    pub never_cache: bool,
    padding1: [u8; 3],
    pub explosion_radius: f32,
    pub explosion_sprite: StdString,
    pub explosion_sprite_emissive: bool,
    pub explosion_sprite_additive: bool,
    pub explosion_sprite_random_rotation: bool,
    padding2: u8,
    pub explosion_sprite_lifetime: f32,
    pub damage: f32,
    pub damage_critical: DamageCriticalConfig,
    pub camera_shake: f32,
    pub particle_effect: bool,
    padding3: [u8; 3],
    pub load_this_entity: StdString,
    pub light_enabled: bool,
    padding4: [u8; 3],
    pub light_fade_time: f32,
    pub light_r: usize,
    pub light_g: usize,
    pub light_b: usize,
    pub light_radius_coeff: f32,
    pub hole_enabled: bool,
    pub destroy_non_platform_solid_enabled: bool,
    padding5: [u8; 2],
    pub electricity_count: isize,
    pub min_radius_for_cracks: isize,
    pub crack_count: isize,
    pub knockback_force: f32,
    pub hole_destroy_liquid: bool,
    pub hole_destroy_physics_dynamic: bool,
    padding6: [u8; 2],
    pub create_cell_material: StdString,
    pub create_cell_probability: isize,
    pub background_lightning_count: isize,
    pub spark_material: StdString,
    pub material_sparks_min_hp: isize,
    pub material_sparks_probability: isize,
    pub material_sparks_count: ValueRangeInt,
    pub material_sparks_enabled: bool,
    pub material_sparks_real: bool,
    pub material_sparks_scale_with_hp: bool,
    pub sparks_enabled: bool,
    pub sparks_count: ValueRangeInt,
    pub sparks_inner_radius_coeff: f32,
    pub stains_enabled: bool,
    padding7: [u8; 3],
    pub stains_radius: f32,
    pub ray_energy: isize,
    pub max_durability_to_destroy: isize,
    pub gore_particle_count: isize,
    pub shake_vegetation: bool,
    pub damage_mortals: bool,
    pub physics_throw_enabled: bool,
    padding8: u8,
    pub physics_explosion_power: ValueRange,
    pub physics_multiplier_ragdoll_force: f32,
    pub cell_explosion_power: f32,
    pub cell_explosion_radius_min: f32,
    pub cell_explosion_radius_max: f32,
    pub cell_explosion_velocity_min: f32,
    pub cell_explosion_damage_required: f32,
    pub cell_explosion_probability: f32,
    pub cell_power_ragdoll_coeff: f32,
    pub pixel_sprites_enabled: bool,
    pub is_digger: bool,
    pub audio_enabled: bool,
    padding9: [u8; 1],
    pub audio_event_name: StdString,
    pub audio_liquid_amount_normalized: f32,
    pub delay: ValueRangeInt,
    pub explosion_delay_id: isize,
    pub not_scaled_by_gamefx: bool,
    padding10: [u8; 3],
    pub who_is_responsible: usize,
    pub null_damage: bool,
    padding11: [u8; 3],
    pub dont_damage_this: usize,
    pub impl_send_message_to_this: usize,
    pub impl_position: Vec2,
    pub impl_delay_frame: isize,
}

#[repr(C)]
#[derive(Debug)]
pub struct DamageCriticalConfigVTable {}

#[repr(C)]
#[derive(Debug)]
pub struct DamageCriticalConfig {
    pub vftable: &'static DamageCriticalConfigVTable,
    pub chance: isize,
    pub damage_multiplier: f32,
    pub m_succeeded: bool,
    padding1: [u8; 3],
}

#[repr(C)]
#[derive(Debug)]
pub struct GridCosmeticParticleConfigVTable {}

#[repr(C)]
#[derive(Debug)]
pub struct GridCosmeticParticleConfig {
    pub vftable: &'static GridCosmeticParticleConfigVTable,
    pub m_material_id: isize,
    pub vel: Vec2,
    pub vel_random: AABB,
    pub color: Color,
    pub lifetime: ValueRange,
    pub gravity: Vec2,
    pub cosmetic_force_create: bool,
    pub render_back: bool,
    pub render_on_grid: bool,
    pub draw_as_long: bool,
    pub airflow_force: f32,
    pub airflow_scale: f32,
    pub friction: f32,
    pub probability: f32,
    pub count: ValueRangeInt,
    pub particle_single_width: bool,
    pub fade_based_on_lifetime: bool,
    padding1: [u8; 2],
}
#[derive(Debug)]
#[repr(C)]
pub struct DamagesByTypeConfigVTable {}

#[derive(Debug)]
#[repr(C)]
pub struct DamagesByTypeConfig {
    pub vftable: &'static DamagesByTypeConfigVTable,
    pub melee: f32,
    pub projectile: f32,
    pub explosion: f32,
    pub electricity: f32,
    pub fire: f32,
    pub drill: f32,
    pub slice: f32,
    pub ice: f32,
    pub healing: f32,
    pub physics_hit: f32,
    pub radioactive: f32,
    pub poison: f32,
    pub overeating: f32,
    pub curse: f32,
    pub holy: f32,
}
#[derive(Debug)]
#[repr(C)]
pub struct PendingPortalConfigVTable {}
#[derive(Debug)]
#[repr(C)]
pub struct PendingPortalConfig {
    pub vftable: &'static PendingPortalConfigVTable,
    pub position: Vec2,
    pub target_position: Vec2,
    pub id: usize,
    pub target_id: usize,
    pub is_at_home: bool,
    padding1: [u8; 3],
    pub target_biome_name: StdString,
    pub entity: *mut Entity,
}
#[derive(Debug)]
#[repr(C)]
pub struct NpcPartyConfigVTable {}

#[derive(Debug)]
#[repr(C)]
pub struct NpcPartyConfig {
    pub vftable: &'static NpcPartyConfigVTable,
    pub position: Vec2,
    pub entities_exist: bool,
    padding1: [u8; 3],
    pub direction: isize,
    pub speed: f32,
    pub member_entities: StdVec<usize>,
    pub member_files: StdVec<StdString>,
}

#[derive(Debug)]
#[repr(C)]
pub struct CutThroughWorldConfigVTable {}

#[derive(Debug)]
#[repr(C)]
pub struct CutThroughWorldConfig {
    pub vftable: &'static CutThroughWorldConfigVTable,
    pub x: isize,
    pub y_min: isize,
    pub y_max: isize,
    pub radius: isize,
    pub edge_darkening_width: isize,
    pub global_id: usize,
}
#[derive(Debug)]
#[repr(C)]
pub struct GunConfigVTable {}

#[derive(Debug)]
#[repr(C)]
pub struct GunConfig {
    pub vftable: &'static GunConfigVTable,
    pub actions_per_round: isize,
    pub shuffle_deck_when_empty: bool,
    padding1: [u8; 3],
    pub reload_time: isize,
    pub deck_capacity: isize,
}

#[derive(Debug)]
#[repr(C)]
pub struct GunActionInfoConfigVTable {}

#[derive(Debug)]
#[repr(C)]
pub struct GunActionInfoConfig {
    pub vftable: &'static GunActionInfoConfigVTable,
    pub action_id: StdString,
    pub action_name: StdString,
    pub action_description: StdString,
    pub action_sprite_filename: StdString,
    pub action_unidentified_sprite_filename: StdString,
    pub action_type: isize,
    pub action_spawn_level: StdString,
    pub action_spawn_probability: StdString,
    pub action_spawn_requires_flag: StdString,
    pub action_spawn_manual_unlock: bool,
    padding1: [u8; 3],
    pub action_max_uses: isize,
    pub custom_xml_file: StdString,
    pub action_mana_drain: f32,
    pub action_is_dangerous_blast: bool,
    padding2: [u8; 3],
    pub action_draw_many_count: isize,
    pub action_ai_never_uses: bool,
    pub action_never_unlimited: bool,
    pub state_shuffled: bool,
    padding3: u8,
    pub state_cards_drawn: isize,
    pub state_discarded_action: bool,
    pub state_destroyed_action: bool,
    padding4: [u8; 2],
    pub fire_rate_wait: isize,
    pub speed_multiplier: f32,
    pub child_speed_multiplier: f32,
    pub dampening: f32,
    pub explosion_radius: f32,
    pub spread_degrees: f32,
    pub pattern_degrees: f32,
    pub screenshake: f32,
    pub recoil: f32,
    pub damage_melee_add: f32,
    pub damage_projectile_add: f32,
    pub damage_electricity_add: f32,
    pub damage_fire_add: f32,
    pub damage_explosion_add: f32,
    pub damage_ice_add: f32,
    pub damage_slice_add: f32,
    pub damage_healing_add: f32,
    pub damage_curse_add: f32,
    pub damage_drill_add: f32,
    pub damage_null_all: f32,
    pub damage_critical_chance: isize,
    pub damage_critical_multiplier: f32,
    pub explosion_damage_to_materials: f32,
    pub knockback_force: f32,
    pub reload_time: isize,
    pub lightning_count: isize,
    pub material: StdString,
    pub material_amount: isize,
    pub trail_material: StdString,
    pub trail_material_amount: isize,
    pub bounces: isize,
    pub gravity: f32,
    pub light: f32,
    pub blood_count_multiplier: f32,
    pub gore_particles: isize,
    pub ragdoll_fx: isize,
    pub friendly_fire: bool,
    padding5: [u8; 3],
    pub physics_impulse_coeff: f32,
    pub lifetime_add: isize,
    pub sprite: StdString,
    pub extra_entities: StdString,
    pub game_effect_entities: StdString,
    pub sound_loop_tag: StdString,
    pub projectile_file: StdString,
}
#[repr(C)]
#[derive(Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[repr(C)]
#[derive(Debug)]
pub struct Vec2i {
    pub x: isize,
    pub y: isize,
}
#[repr(C)]
#[derive(Debug)]
pub struct ValueRange {
    pub min: f32,
    pub max: f32,
}
#[repr(C)]
#[derive(Debug)]
pub struct ValueRangeInt {
    pub min: isize,
    pub max: isize,
}
