use crate::noita::types::{B2Object, ComponentData, Entity, GameEffect};
pub trait Component {
    const NAME: &'static str;
}
impl Component for SetLightAlphaFromVelocityComponent {
    const NAME: &'static str = "SetLightAlphaFromVelocityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SetLightAlphaFromVelocityComponent {
    pub inherited_fields: ComponentData,
    pub max_velocity: f32,
    pub m_prev_position: [u8; 8],
    field3_0x54: u8,
    field4_0x55: u8,
    field5_0x56: u8,
    field6_0x57: u8,
}
impl Component for ItemAIKnowledgeComponent {
    const NAME: &'static str = "ItemAIKnowledgeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemAIKnowledgeComponent {
    pub inherited_fields: ComponentData,
    pub is_ranged_weapon: bool,
    pub is_throwable_weapon: bool,
    pub is_melee_weapon: bool,
    pub is_self_healing: bool,
    pub is_other_healing: bool,
    pub is_self_buffing: bool,
    pub is_other_buffing: bool,
    pub is_weapon: bool,
    pub is_known: bool,
    pub is_safe: bool,
    pub is_consumed: bool,
    pub never_use: bool,
    pub ranged_min_distance: f32,
}
impl Component for DroneLauncherComponent {
    const NAME: &'static str = "DroneLauncherComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DroneLauncherComponent {
    pub inherited_fields: ComponentData,
    pub drone_entity_file: [u8; 24],
}
impl Component for CharacterPlatformingComponent {
    const NAME: &'static str = "CharacterPlatformingComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CharacterPlatformingComponent {
    pub inherited_fields: ComponentData,
    pub velocity_min_x: [u8; 12],
    pub velocity_max_x: [u8; 12],
    pub velocity_min_y: [u8; 12],
    pub velocity_max_y: [u8; 12],
    pub run_velocity: [u8; 12],
    pub fly_velocity_x: [u8; 12],
    pub jump_velocity_x: f32,
    pub jump_velocity_y: f32,
    pub jump_keydown_buffer: isize,
    pub fly_speed_max_up: [u8; 12],
    pub fly_speed_max_down: [u8; 12],
    pub fly_speed_mult: f32,
    pub fly_speed_change_spd: f32,
    pub fly_model_player: bool,
    pub fly_smooth_y: bool,
    field16_0xbe: u8,
    field17_0xbf: u8,
    pub accel_x: f32,
    pub accel_x_air: f32,
    pub pixel_gravity: f32,
    pub swim_idle_buoyancy_coeff: f32,
    pub swim_down_buoyancy_coeff: f32,
    pub swim_up_buoyancy_coeff: f32,
    pub swim_drag: f32,
    pub swim_extra_horizontal_drag: f32,
    pub mouse_look: bool,
    field27_0xe1: u8,
    field28_0xe2: u8,
    field29_0xe3: u8,
    pub mouse_look_buffer: f32,
    pub keyboard_look: bool,
    field32_0xe9: u8,
    field33_0xea: u8,
    field34_0xeb: u8,
    pub turning_buffer: f32,
    pub animation_to_play: [u8; 24],
    pub animation_to_play_next: [u8; 24],
    pub run_animation_velocity_switching_threshold: f32,
    pub run_animation_velocity_switching_enabled: bool,
    field40_0x125: u8,
    field41_0x126: u8,
    field42_0x127: u8,
    pub turn_animation_frames_between: isize,
    pub precision_jumping_max_duration_frames: isize,
    pub audio_liquid_splash_isizeensity: f32,
    pub m_ex_animation_pos: [u8; 8],
    pub m_frames_in_air_counter: isize,
    pub m_is_precision_jumping: bool,
    field49_0x141: u8,
    field50_0x142: u8,
    field51_0x143: u8,
    pub m_precision_jumping_time: isize,
    pub m_precision_jumping_speed_x: f32,
    pub m_precision_jumping_time_left: isize,
    pub m_fly_throttle: f32,
    pub m_smoothed_flying_target_y: f32,
    pub m_jetpack_emitting: isize,
    pub m_next_turn_animation_frame: isize,
    pub m_frames_not_swimming: isize,
    pub m_frames_swimming: isize,
    pub m_should_crouch: bool,
    pub m_should_crouch_prev: bool,
    field63_0x16a: u8,
    field64_0x16b: u8,
    pub m_last_posture_switch_frame: isize,
    pub m_look_override_last_frame: isize,
    pub m_look_override_direction: isize,
}
impl Component for ItemPickUpperComponent {
    const NAME: &'static str = "ItemPickUpperComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemPickUpperComponent {
    pub inherited_fields: ComponentData,
    pub is_in_npc: bool,
    pub pick_up_any_item_buggy: bool,
    pub is_immune_to_kicks: bool,
    field4_0x4b: u8,
    pub only_pick_this_entity: isize,
    pub drop_items_on_death: bool,
    field7_0x51: u8,
    field8_0x52: u8,
    field9_0x53: u8,
    pub m_latest_item_overlap_info_box_position: [u8; 8],
    field11_0x5c: u8,
    field12_0x5d: u8,
    field13_0x5e: u8,
    field14_0x5f: u8,
}
impl Component for DebugLogMessagesComponent {
    const NAME: &'static str = "DebugLogMessagesComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DebugLogMessagesComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_y: f32,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
}
impl Component for ItemRechargeNearGroundComponent {
    const NAME: &'static str = "ItemRechargeNearGroundComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemRechargeNearGroundComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_y: f32,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
}
impl Component for GodInfoComponent {
    const NAME: &'static str = "GodInfoComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GodInfoComponent {
    pub inherited_fields: ComponentData,
    pub mana_current: f32,
    pub mana_max: f32,
    field3_0x50: u8,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub god_entity: *mut Entity,
}
impl Component for Inventory2Component {
    const NAME: &'static str = "Inventory2Component";
}
#[derive(Debug)]
#[repr(C)]
pub struct Inventory2Component {
    pub inherited_fields: ComponentData,
    pub quick_inventory_slots: isize,
    pub full_inventory_slots_x: isize,
    pub full_inventory_slots_y: isize,
    pub m_saved_active_item_index: usize,
    pub m_active_item: isize,
    pub m_actual_active_item: isize,
    pub m_active_stash: isize,
    pub m_throw_item: isize,
    pub m_item_holstered: bool,
    pub m_initialized: bool,
    pub m_force_refresh: bool,
    pub m_dont_log_next_item_equip: bool,
    pub m_smoothed_item_x_offset: f32,
    pub m_last_item_switch_frame: isize,
    pub m_intro_equip_item_lerp: f32,
    pub m_smoothed_item_angle_vec: [u8; 8],
}
impl Component for HomingComponent {
    const NAME: &'static str = "HomingComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct HomingComponent {
    pub inherited_fields: ComponentData,
    pub target_tag: [u8; 24],
    pub target_who_shot: bool,
    field3_0x61: u8,
    field4_0x62: u8,
    field5_0x63: u8,
    pub detect_distance: f32,
    pub homing_velocity_multiplier: f32,
    pub homing_targeting_coeff: f32,
    pub just_rotate_towards_target: bool,
    field10_0x71: u8,
    field11_0x72: u8,
    field12_0x73: u8,
    pub max_turn_rate: f32,
    pub predefined_target: isize,
    pub look_for_root_entities_only: bool,
    field16_0x7d: u8,
    field17_0x7e: u8,
    field18_0x7f: u8,
}
impl Component for AudioLoopComponent {
    const NAME: &'static str = "AudioLoopComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AudioLoopComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
    pub event_name: [u8; 24],
    pub auto_play: bool,
    pub auto_play_if_enabled: bool,
    pub play_on_component_enable: bool,
    pub calculate_material_lowpass: bool,
    pub set_speed_parameter: bool,
    pub set_speed_parameter_only_based_on_x_movement: bool,
    pub set_speed_parameter_only_based_on_y_movement: bool,
    field33_0x7f: u8,
    pub volume_autofade_speed: f32,
    pub m_volume: f32,
    pub m_isizeensity: f32,
    pub m_isizeensity2: f32,
    pub m_source: [u8; 4],
    pub m_frame_created: isize,
}
impl Component for CutThroughWorldDoneHereComponent {
    const NAME: &'static str = "CutThroughWorldDoneHereComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CutThroughWorldDoneHereComponent {
    pub inherited_fields: ComponentData,
    pub id_of_done_cut: usize,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for GenomeDataComponent {
    const NAME: &'static str = "GenomeDataComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GenomeDataComponent {
    pub inherited_fields: ComponentData,
    pub herd_id: [u8; 12],
    pub is_predator: bool,
    field3_0x55: u8,
    field4_0x56: u8,
    field5_0x57: u8,
    pub food_chain_rank: f32,
    pub friend_thundermage: [u8; 8],
    pub friend_firemage: [u8; 8],
    pub berserk_dont_attack_friends: bool,
    field10_0x6d: u8,
    field11_0x6e: u8,
    field12_0x6f: u8,
}
impl Component for HotspotComponent {
    const NAME: &'static str = "HotspotComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct HotspotComponent {
    pub inherited_fields: ComponentData,
    pub offset: [u8; 8],
    pub transform_with_scale: bool,
    field3_0x51: u8,
    field4_0x52: u8,
    field5_0x53: u8,
    pub sprite_hotspot_name: [u8; 24],
    field7_0x6c: u8,
    field8_0x6d: u8,
    field9_0x6e: u8,
    field10_0x6f: u8,
}
impl Component for DieIfSpeedBelowComponent {
    const NAME: &'static str = "DieIfSpeedBelowComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DieIfSpeedBelowComponent {
    pub inherited_fields: ComponentData,
    pub min_speed: f32,
    pub m_min_speed_squared: f32,
}
impl Component for EnergyShieldComponent {
    const NAME: &'static str = "EnergyShieldComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct EnergyShieldComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub damage_multiplier: f32,
    pub max_energy: f32,
    pub energy_required_to_shield: f32,
    pub recharge_speed: f32,
    pub sector_degrees: f32,
    pub energy: f32,
    pub m_prev_position: [u8; 8],
    field9_0x6c: u8,
    field10_0x6d: u8,
    field11_0x6e: u8,
    field12_0x6f: u8,
}
impl Component for VelocityComponent {
    const NAME: &'static str = "VelocityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct VelocityComponent {
    pub inherited_fields: ComponentData,
    pub gravity_x: f32,
    pub gravity_y: f32,
    field3_0x50: u8,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub air_friction: f32,
    pub terminal_velocity: f32,
    pub apply_terminal_velocity: bool,
    pub updates_velocity: bool,
    pub displace_liquid: bool,
    pub affect_physics_bodies: bool,
    pub limit_to_max_velocity: bool,
    field14_0x61: u8,
    field15_0x62: u8,
    field16_0x63: u8,
    pub liquid_death_threshold: isize,
    pub liquid_drag: f32,
    pub m_velocity: [u8; 8],
    pub m_prev_velocity: [u8; 8],
    pub m_latest_liquid_hit_count: isize,
    pub m_average_liquid_hit_count: isize,
    pub m_prev_position: [u8; 8],
    field24_0x8c: u8,
    field25_0x8d: u8,
    field26_0x8e: u8,
    field27_0x8f: u8,
}
impl Component for CharacterStatsComponent {
    const NAME: &'static str = "CharacterStatsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CharacterStatsComponent {
    pub inherited_fields: ComponentData,
    pub stats: [u8; 112],
}
impl Component for CharacterDataComponent {
    const NAME: &'static str = "CharacterDataComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CharacterDataComponent {
    pub inherited_fields: ComponentData,
    pub platforming_type: isize,
    pub collision_aabb_min_x: [u8; 12],
    pub collision_aabb_max_x: [u8; 12],
    pub collision_aabb_min_y: [u8; 12],
    pub collision_aabb_max_y: [u8; 12],
    field6_0x7c: u8,
    field7_0x7d: u8,
    field8_0x7e: u8,
    field9_0x7f: u8,
    pub buoyancy_check_offset_y: isize,
    pub liquid_velocity_coeff: f32,
    pub gravity: f32,
    pub fly_time_max: [u8; 12],
    pub fly_recharge_spd: f32,
    pub fly_recharge_spd_ground: f32,
    pub flying_needs_recharge: bool,
    field17_0xa1: u8,
    field18_0xa2: u8,
    field19_0xa3: u8,
    pub flying_in_air_wait_frames: isize,
    pub flying_recharge_removal_frames: isize,
    pub climb_over_y: isize,
    pub check_collision_max_size_x: isize,
    pub check_collision_max_size_y: isize,
    pub is_on_ground: bool,
    pub is_on_slippery_ground: bool,
    field27_0xba: u8,
    field28_0xbb: u8,
    pub ground_stickyness: f32,
    pub effect_hit_ground: bool,
    field31_0xc1: u8,
    field32_0xc2: u8,
    field33_0xc3: u8,
    pub eff_hg_damage_min: isize,
    pub eff_hg_damage_max: isize,
    pub eff_hg_position_x: f32,
    pub eff_hg_position_y: f32,
    pub eff_hg_size_x: f32,
    pub eff_hg_size_y: f32,
    pub eff_hg_velocity_min_x: f32,
    pub eff_hg_velocity_max_x: f32,
    pub eff_hg_velocity_min_y: f32,
    pub eff_hg_velocity_max_y: f32,
    pub eff_hg_offset_y: f32,
    pub eff_hg_update_box2d: bool,
    field46_0xf1: u8,
    field47_0xf2: u8,
    field48_0xf3: u8,
    pub eff_hg_b2force_multiplier: f32,
    pub destroy_ground: f32,
    pub send_transform_update_message: bool,
    pub dont_update_velocity_and_xform: bool,
    field53_0xfe: u8,
    field54_0xff: u8,
    pub m_frames_on_ground: isize,
    pub m_last_frame_on_ground: isize,
    pub m_velocity: [u8; 8],
    pub m_flying_time_left: f32,
    pub m_collided_horizontally: bool,
    field60_0x115: u8,
    field61_0x116: u8,
    field62_0x117: u8,
}
impl Component for CollisionTriggerComponent {
    const NAME: &'static str = "CollisionTriggerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CollisionTriggerComponent {
    pub inherited_fields: ComponentData,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub required_tag: [u8; 24],
    pub remove_component_when_triggered: bool,
    pub destroy_this_entity_when_triggered: bool,
    field7_0x6e: u8,
    field8_0x6f: u8,
    pub timer_for_destruction: isize,
    pub self_trigger: bool,
    field11_0x75: u8,
    field12_0x76: u8,
    field13_0x77: u8,
    pub skip_self_frames: isize,
    pub m_timer: isize,
}
impl Component for PhysicsBodyComponent {
    const NAME: &'static str = "PhysicsBodyComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsBodyComponent {
    pub inherited_fields: ComponentData,
    pub m_body: [*mut u8; 4],
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub m_body_id: *mut B2Object,
    field7_0x54: u8,
    field8_0x55: u8,
    field9_0x56: u8,
    field10_0x57: u8,
    pub is_external: bool,
    pub hax_fix_going_through_ground: bool,
    pub hax_fix_going_through_sand: bool,
    pub hax_wait_till_pixel_scenes_loaded: bool,
    field15_0x5c: u8,
    field16_0x5d: u8,
    field17_0x5e: u8,
    field18_0x5f: u8,
    pub is_enabled: bool,
    field20_0x61: u8,
    field21_0x62: u8,
    field22_0x63: u8,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub allow_sleep: bool,
    pub fixed_rotation: bool,
    field27_0x6e: u8,
    field28_0x6f: u8,
    pub buoyancy: f32,
    pub gravity_scale_if_has_no_image_shapes: f32,
    pub is_bullet: bool,
    pub is_static: bool,
    pub is_kinematic: bool,
    pub is_character: bool,
    pub go_through_sand: bool,
    pub gridworld_box2d: bool,
    pub auto_clean: bool,
    pub on_death_leave_physics_body: bool,
    pub on_death_really_leave_body: bool,
    pub update_entity_transform: bool,
    pub force_add_update_areas: bool,
    pub kills_entity: bool,
    pub projectiles_rotate_toward_velocity: bool,
    field44_0x85: u8,
    field45_0x86: u8,
    field46_0x87: u8,
    pub initial_velocity: [u8; 8],
    pub randomize_init_velocity: bool,
    pub m_active_state: bool,
    field50_0x92: u8,
    field51_0x93: u8,
    pub m_pixel_count: isize,
    pub m_local_position: [u8; 16],
    pub m_refreshed: bool,
    field55_0xa9: u8,
    field56_0xaa: u8,
    field57_0xab: u8,
    field58_0xac: u8,
    field59_0xad: u8,
    field60_0xae: u8,
    field61_0xaf: u8,
}
impl Component for GhostComponent {
    const NAME: &'static str = "GhostComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GhostComponent {
    pub inherited_fields: ComponentData,
    pub speed: f32,
    pub velocity: [u8; 8],
    pub new_hunt_target_check_every: isize,
    pub hunt_box_radius: f32,
    pub aggressiveness: f32,
    pub m_entity_home: isize,
    pub max_distance_from_home: f32,
    pub die_if_no_home: bool,
    field9_0x69: u8,
    field10_0x6a: u8,
    field11_0x6b: u8,
    pub m_frames_without_home: isize,
    pub target_tag: [u8; 24],
    pub m_target_position: [u8; 8],
    pub m_target_entity_id: isize,
    pub m_random_target: [u8; 8],
    pub m_next_target_check_frame: isize,
}
impl Component for DrugEffectComponent {
    const NAME: &'static str = "DrugEffectComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DrugEffectComponent {
    pub inherited_fields: ComponentData,
    pub drug_fx_target: [u8; 28],
    pub m_drug_fx_current: [u8; 28],
}
impl Component for VerletPhysicsComponent {
    const NAME: &'static str = "VerletPhysicsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct VerletPhysicsComponent {
    pub inherited_fields: ComponentData,
    pub num_poisizes: isize,
    pub num_links: isize,
    pub width: isize,
    field4_0x54: u8,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub resting_distance: f32,
    pub mass_min: f32,
    pub mass_max: f32,
    pub stiffness: f32,
    pub velocity_dampening: f32,
    pub liquid_damping: f32,
    pub gets_entity_velocity_coeff: f32,
    pub collide_with_cells: bool,
    pub simulate_gravity: bool,
    pub simulate_wind: bool,
    field18_0x77: u8,
    pub wind_change_speed: f32,
    constrain_stretching: bool,
    pub pixelate_sprite_transforms: bool,
    pub scale_sprite_x: bool,
    pub follow_entity_transform: bool,
    pub animation_target_offset: [u8; 8],
    pub animation_amount: f32,
    pub animation_speed: f32,
    pub animation_energy: f32,
    pub cloth_sprite_z_index: f32,
    pub stain_cells_probability: isize,
    pub cloth_color_edge: usize,
    pub cloth_color: usize,
    pub m_position_previous: [u8; 8],
    pub m_is_culled_previous: bool,
    field34_0xad: u8,
    field35_0xae: u8,
    field36_0xaf: u8,
    pub masses: [u8; 640],
    pub positions: [u8; 1280],
    pub positions_prev: [u8; 1280],
    pub velocities: [u8; 1280],
    pub dampenings: [u8; 640],
    pub freedoms: [u8; 640],
    pub links: [u8; 3840],
    pub colors: [u8; 640],
    pub materials: [u8; 640],
    pub sprite: [*mut u8; 4],
    field47_0x2b34: u8,
    field48_0x2b35: u8,
    field49_0x2b36: u8,
    field50_0x2b37: u8,
}
impl Component for MaterialSeaSpawnerComponent {
    const NAME: &'static str = "MaterialSeaSpawnerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MaterialSeaSpawnerComponent {
    pub inherited_fields: ComponentData,
    pub material: isize,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    field6_0x50: u8,
    field7_0x51: u8,
    field8_0x52: u8,
    field9_0x53: u8,
    pub offset: [u8; 8],
    pub speed: isize,
    pub sine_wavelength: f32,
    pub sine_amplitude: f32,
    pub noise_scale: f64,
    pub noise_threshold: f64,
    pub m_position: isize,
    pub frames_run: isize,
}
impl Component for LocationMarkerComponent {
    const NAME: &'static str = "LocationMarkerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LocationMarkerComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
}
impl Component for InventoryGuiComponent {
    const NAME: &'static str = "InventoryGuiComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct InventoryGuiComponent {
    pub inherited_fields: ComponentData,
    pub has_opened_inventory_edit: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub wallet_money_target: isize,
    pub imgui: [*mut u8; 4],
    pub m_last_frame_interacted: isize,
    pub m_last_frame_actions_visible: isize,
    pub m_last_purchased_action: *mut Entity,
    pub m_active: bool,
    field11_0x61: u8,
    field12_0x62: u8,
    field13_0x63: u8,
    pub m_alpha: f32,
    pub m_background_overlay_alpha: f32,
    pub m_frame_shake_reload_bar: isize,
    pub m_frame_shake_mana_bar: isize,
    pub m_frame_shake_fly_bar: isize,
    pub m_frame_shake_fire_rate_wait_bar: isize,
    pub m_display_fire_rate_wait_bar: bool,
    field21_0x7d: u8,
    field22_0x7e: u8,
    field23_0x7f: u8,
}
impl Component for SpriteParticleEmitterComponent {
    const NAME: &'static str = "SpriteParticleEmitterComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SpriteParticleEmitterComponent {
    pub inherited_fields: ComponentData,
    pub sprite_file: [u8; 24],
    pub sprite_centered: bool,
    pub sprite_random_rotation: bool,
    pub render_back: bool,
    field5_0x63: u8,
    pub delay: f32,
    pub lifetime: f32,
    pub color: [u8; 20],
    pub color_change: [u8; 20],
    pub additive: bool,
    pub emissive: bool,
    field12_0x96: u8,
    field13_0x97: u8,
    pub velocity: [u8; 8],
    pub gravity: [u8; 8],
    pub velocity_slowdown: f32,
    pub rotation: f32,
    pub angular_velocity: f32,
    pub use_velocity_as_rotation: bool,
    pub use_rotation_from_velocity_component: bool,
    pub use_rotation_from_entity: bool,
    field22_0xb7: u8,
    pub entity_velocity_multiplier: f32,
    pub scale: [u8; 8],
    pub scale_velocity: [u8; 8],
    pub z_index: f32,
    pub randomize_lifetime: [u8; 8],
    pub randomize_position: [u8; 16],
    pub randomize_position_inside_hitbox: bool,
    field30_0xe9: u8,
    field31_0xea: u8,
    field32_0xeb: u8,
    pub randomize_velocity: [u8; 16],
    pub randomize_scale: [u8; 16],
    pub randomize_rotation: [u8; 8],
    pub randomize_angular_velocity: [u8; 8],
    pub randomize_alpha: [u8; 8],
    pub randomize_animation_speed_coeff: [u8; 8],
    pub velocity_always_away_from_center: bool,
    field40_0x12d: u8,
    field41_0x12e: u8,
    field42_0x12f: u8,
    pub expand_randomize_position: [u8; 8],
    pub camera_bound: bool,
    field45_0x139: u8,
    field46_0x13a: u8,
    field47_0x13b: u8,
    pub camera_distance: f32,
    pub is_emitting: bool,
    field50_0x141: u8,
    field51_0x142: u8,
    field52_0x143: u8,
    pub count_min: isize,
    pub count_max: isize,
    pub emission_isizeerval_min_frames: isize,
    pub emission_isizeerval_max_frames: isize,
    pub entity_file: [u8; 24],
    pub m_next_emit_frame: isize,
}
impl Component for LightningComponent {
    const NAME: &'static str = "LightningComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LightningComponent {
    pub inherited_fields: ComponentData,
    pub config_explosion: [u8; 372],
    pub sprite_lightning_file: [u8; 24],
    pub is_projectile: bool,
    field4_0x1d5: u8,
    field5_0x1d6: u8,
    field6_0x1d7: u8,
    pub explosion_type: isize,
    pub m_ex_position: [u8; 8],
    pub m_arc_target: isize,
    pub arc_lifetime: isize,
    field11_0x1ec: u8,
    field12_0x1ed: u8,
    field13_0x1ee: u8,
    field14_0x1ef: u8,
}
impl Component for MagicXRayComponent {
    const NAME: &'static str = "MagicXRayComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MagicXRayComponent {
    pub inherited_fields: ComponentData,
    pub radius: isize,
    pub steps_per_frame: isize,
    pub m_step: isize,
    pub m_radius: isize,
}
impl Component for WalletValuableComponent {
    const NAME: &'static str = "WalletValuableComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WalletValuableComponent {
    pub inherited_fields: ComponentData,
    pub money_value: isize,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for PhysicsAIComponent {
    const NAME: &'static str = "PhysicsAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsAIComponent {
    pub inherited_fields: ComponentData,
    pub target_vec_max_len: f32,
    pub force_coeff: f32,
    pub force_balancing_coeff: f32,
    pub force_max: f32,
    pub torque_coeff: f32,
    pub torque_balancing_coeff: f32,
    pub torque_max: f32,
    pub torque_damaged_max: f32,
    pub torque_jump_random: f32,
    pub damage_deactivation_probability: isize,
    pub damage_deactivation_time_min: isize,
    pub damage_deactivation_time_max: isize,
    pub die_on_remaining_mass_percentage: f32,
    pub levitate: bool,
    pub v0_jump_logic: bool,
    pub v0_swim_logic: bool,
    pub v0_body_id_logic: bool,
    pub swim_check_y_min: isize,
    pub swim_check_y_max: isize,
    pub swim_check_side_x: isize,
    pub swim_check_side_y: isize,
    pub keep_inside_world: bool,
    pub free_if_static: bool,
    field24_0x92: u8,
    field25_0x93: u8,
    pub rotation_speed: f32,
    pub m_starting_mass: f32,
    pub m_main_body_found: bool,
    field29_0x9d: u8,
    field30_0x9e: u8,
    field31_0x9f: u8,
    pub m_next_frame_active: isize,
    pub m_rotation_target: f32,
    pub m_last_position_when_had_path: [u8; 8],
    pub m_has_last_position: bool,
    field36_0xb1: u8,
    field37_0xb2: u8,
    field38_0xb3: u8,
    field39_0xb4: u8,
    field40_0xb5: u8,
    field41_0xb6: u8,
    field42_0xb7: u8,
}
impl Component for EndingMcGuffinComponent {
    const NAME: &'static str = "EndingMcGuffinComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct EndingMcGuffinComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_y: f32,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
}
impl Component for PhysicsShapeComponent {
    const NAME: &'static str = "PhysicsShapeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsShapeComponent {
    pub inherited_fields: ComponentData,
    pub recreate: bool,
    pub is_circle: bool,
    pub is_box: bool,
    pub is_capsule: bool,
    pub is_based_on_sprite: bool,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    pub friction: f32,
    pub restitution: f32,
    pub density: f32,
    pub local_position_x: f32,
    pub local_position_y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub capsule_x_percent: f32,
    pub capsule_y_percent: f32,
    pub material: isize,
}
impl Component for PhysicsKeepInWorldComponent {
    const NAME: &'static str = "PhysicsKeepInWorldComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsKeepInWorldComponent {
    pub inherited_fields: ComponentData,
    pub check_whole_aabb: bool,
    pub predict_aabb: bool,
    pub keep_at_last_valid_pos: bool,
    field4_0x4b: u8,
    pub m_ex_position: [u8; 8],
    pub m_ex_rotation: f32,
}
impl Component for AltarComponent {
    const NAME: &'static str = "AltarComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AltarComponent {
    pub inherited_fields: ComponentData,
    pub recognized_entity_tags: [u8; 24],
    pub good_fx_material: isize,
    pub neutral_fx_material: isize,
    pub evil_fx_material: isize,
    pub uses_remaining: isize,
    pub m_recognized_entity_tags: [u8; 64],
    pub m_recognized_entity_tags_count: usize,
    field8_0xb4: u8,
    field9_0xb5: u8,
    field10_0xb6: u8,
    field11_0xb7: u8,
    pub m_current_entity_tags: [u8; 64],
}
impl Component for BlackHoleComponent {
    const NAME: &'static str = "BlackHoleComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct BlackHoleComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub particle_attractor_force: f32,
    pub damage_probability: f32,
    pub damage_amount: f32,
    pub m_particle_attractor_id: i16,
    field6_0x5a: u8,
    field7_0x5b: u8,
    field8_0x5c: u8,
    field9_0x5d: u8,
    field10_0x5e: u8,
    field11_0x5f: u8,
}
impl Component for GameStatsComponent {
    const NAME: &'static str = "GameStatsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GameStatsComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
    pub stats_filename: [u8; 24],
    pub is_player: bool,
    field27_0x79: u8,
    field28_0x7a: u8,
    field29_0x7b: u8,
    pub extra_death_msg: [u8; 24],
    pub dont_do_logplayerkill: bool,
    field32_0x95: u8,
    field33_0x96: u8,
    field34_0x97: u8,
    pub player_polymorph_count: isize,
    field36_0x9c: u8,
    field37_0x9d: u8,
    field38_0x9e: u8,
    field39_0x9f: u8,
}
impl Component for TelekinesisComponent {
    const NAME: &'static str = "TelekinesisComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct TelekinesisComponent {
    pub inherited_fields: ComponentData,
    pub min_size: usize,
    pub max_size: usize,
    pub radius: f32,
    pub throw_speed: f32,
    pub target_distance: f32,
    pub kick_to_use: bool,
    field7_0x5d: u8,
    field8_0x5e: u8,
    field9_0x5f: u8,
    pub m_state: isize,
    field11_0x64: u8,
    field12_0x65: u8,
    field13_0x66: u8,
    field14_0x67: u8,
    pub m_body_i_d: u64,
    pub m_start_body_max_extent: f32,
    pub m_start_aim_angle: f32,
    pub m_start_body_angle: f32,
    pub m_start_body_distance: f32,
    pub m_start_time: f32,
    pub m_min_body_distance: f32,
    pub m_interact: bool,
    field23_0x89: u8,
    field24_0x8a: u8,
    field25_0x8b: u8,
    field26_0x8c: u8,
    field27_0x8d: u8,
    field28_0x8e: u8,
    field29_0x8f: u8,
}
impl Component for InheritTransformComponent {
    const NAME: &'static str = "InheritTransformComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct InheritTransformComponent {
    pub inherited_fields: ComponentData,
    pub use_root_parent: bool,
    pub only_position: bool,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub parent_hotspot_tag: [u8; 24],
    pub parent_sprite_id: isize,
    pub always_use_immediate_parent_rotation: bool,
    pub rotate_based_on_x_scale: bool,
    field9_0x6a: u8,
    field10_0x6b: u8,
    pub _transform: [u8; 32],
    pub m_update_frame: isize,
}
impl Component for PhysicsImageShapeComponent {
    const NAME: &'static str = "PhysicsImageShapeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsImageShapeComponent {
    pub inherited_fields: ComponentData,
    pub is_root: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub body_id: isize,
    pub use_sprite: bool,
    pub is_circle: bool,
    pub centered: bool,
    field9_0x53: u8,
    pub offset_x: f32,
    pub offset_y: f32,
    field12_0x5c: u8,
    field13_0x5d: u8,
    field14_0x5e: u8,
    field15_0x5f: u8,
    pub image_file: [u8; 24],
    pub material: isize,
    pub m_body: [*mut u8; 4],
}
impl Component for ParticleEmitterComponent {
    const NAME: &'static str = "ParticleEmitterComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ParticleEmitterComponent {
    pub inherited_fields: ComponentData,
    pub emitted_material_name: [u8; 24],
    pub create_real_particles: bool,
    pub emit_real_particles: bool,
    pub emit_cosmetic_particles: bool,
    pub cosmetic_force_create: bool,
    pub render_back: bool,
    pub render_ultrabright: bool,
    pub collide_with_grid: bool,
    pub collide_with_gas_and_fire: bool,
    pub particle_single_width: bool,
    pub emit_only_if_there_is_space: bool,
    field12_0x6a: u8,
    field13_0x6b: u8,
    pub emitter_lifetime_frames: isize,
    pub fire_cells_dont_ignite_damagemodel: bool,
    pub color_is_based_on_pos: bool,
    field17_0x72: u8,
    field18_0x73: u8,
    pub color: usize,
    pub custom_alpha: f32,
    pub offset: [u8; 8],
    pub x_pos_offset_min: f32,
    pub y_pos_offset_min: f32,
    pub x_pos_offset_max: f32,
    pub y_pos_offset_max: f32,
    pub area_circle_radius: [u8; 8],
    pub area_circle_sector_degrees: f32,
    pub x_vel_min: f32,
    pub x_vel_max: f32,
    pub y_vel_min: f32,
    pub y_vel_max: f32,
    pub direction_random_deg: f32,
    pub gravity: [u8; 8],
    pub velocity_always_away_from_center: f32,
    pub lifetime_min: f32,
    pub lifetime_max: f32,
    pub airflow_force: f32,
    pub airflow_time: f32,
    pub airflow_scale: f32,
    pub friction: f32,
    pub attractor_force: f32,
    pub count_min: [u8; 12],
    pub count_max: [u8; 12],
    pub emission_isizeerval_min_frames: isize,
    pub emission_isizeerval_max_frames: isize,
    pub emission_chance: isize,
    pub custom_style: [u8; 4],
    pub delay_frames: isize,
    pub is_emitting: bool,
    pub use_material_inventory: bool,
    pub is_trail: bool,
    field52_0x10b: u8,
    pub trail_gap: f32,
    pub render_on_grid: bool,
    pub fade_based_on_lifetime: bool,
    pub draw_as_long: bool,
    field57_0x113: u8,
    pub b2_force: f32,
    pub set_magic_creation: bool,
    field60_0x119: u8,
    field61_0x11a: u8,
    field62_0x11b: u8,
    pub image_animation_file: [u8; 24],
    pub image_animation_colors_file: [u8; 24],
    pub image_animation_speed: f32,
    pub image_animation_loop: bool,
    field67_0x151: u8,
    field68_0x152: u8,
    field69_0x153: u8,
    pub image_animation_phase: f32,
    pub image_animation_emission_probability: f32,
    pub image_animation_raytrace_from_center: bool,
    pub image_animation_use_entity_rotation: bool,
    pub ignore_transform_updated_msg: bool,
    field75_0x15f: u8,
    pub m_ex_position: [u8; 8],
    pub m_material_inventory_max: isize,
    pub m_material_id: [u8; 12],
    pub m_next_emit_frame: isize,
    pub m_has_emitted: bool,
    field81_0x17d: u8,
    field82_0x17e: u8,
    field83_0x17f: u8,
    pub m_last_emit_position: [u8; 8],
    pub m_cached_image_animation: [u8; 4],
    pub m_image_based_animation_time: f32,
    pub m_collision_angles: *mut f32,
    pub m_particle_attractor_id: i16,
    field89_0x196: u8,
    field90_0x197: u8,
}
impl Component for HealthBarComponent {
    const NAME: &'static str = "HealthBarComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct HealthBarComponent {
    pub inherited_fields: ComponentData,
}
impl Component for WormPlayerComponent {
    const NAME: &'static str = "WormPlayerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WormPlayerComponent {
    pub inherited_fields: ComponentData,
    pub m_prev_position: [u8; 8],
    pub m_direction: [u8; 8],
}
impl Component for PlayerCollisionComponent {
    const NAME: &'static str = "PlayerCollisionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PlayerCollisionComponent {
    pub inherited_fields: ComponentData,
    pub getting_crushed_threshold: isize,
    pub moving_up_before_getting_crushed_threshold: isize,
    pub getting_crushed_counter: isize,
    pub stuck_in_ground_counter: isize,
    pub _d_e_b_u_g_stuck_in_static_ground: isize,
    pub m_collided_horizontally: bool,
    field7_0x5d: u8,
    field8_0x5e: u8,
    field9_0x5f: u8,
    pub m_physics_collision_hax: [*mut u8; 4],
    field11_0x64: u8,
    field12_0x65: u8,
    field13_0x66: u8,
    field14_0x67: u8,
}
impl Component for TextLogComponent {
    const NAME: &'static str = "TextLogComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct TextLogComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
    pub image_filename: [u8; 24],
    pub m_cached_name: [u8; 24],
}
impl Component for IKLimbComponent {
    const NAME: &'static str = "IKLimbComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct IKLimbComponent {
    pub inherited_fields: ComponentData,
    pub length: f32,
    pub thigh_extra_lenght: f32,
    pub end_position: [u8; 8],
    pub m_joisize_side_interpolation: f32,
    pub m_joisize_world_pos: [u8; 8],
    pub m_part0_prev_pos: [u8; 8],
    pub m_part0_prev_rotation: f32,
    pub m_part1_prev_pos: [u8; 8],
    pub m_part1_prev_rotation: f32,
    pub m_end_prev_pos: [u8; 8],
    field11_0x84: u8,
    field12_0x85: u8,
    field13_0x86: u8,
    field14_0x87: u8,
}
impl Component for KickComponent {
    const NAME: &'static str = "KickComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct KickComponent {
    pub inherited_fields: ComponentData,
    pub can_kick: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub max_force: [u8; 12],
    pub player_kickforce: [u8; 12],
    pub kick_radius: f32,
    pub kick_damage: [u8; 12],
    pub kick_knockback: [u8; 12],
    pub telekinesis_throw_speed: f32,
    pub kick_entities: [u8; 24],
    field12_0x9c: u8,
    field13_0x9d: u8,
    field14_0x9e: u8,
    field15_0x9f: u8,
}
impl Component for NullDamageComponent {
    const NAME: &'static str = "NullDamageComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct NullDamageComponent {
    pub inherited_fields: ComponentData,
    pub null_chance: f32,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for MaterialSuckerComponent {
    const NAME: &'static str = "MaterialSuckerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MaterialSuckerComponent {
    pub inherited_fields: ComponentData,
    pub material_type: isize,
    pub barrel_size: isize,
    pub num_cells_sucked_per_frame: isize,
    pub set_projectile_to_liquid: bool,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub last_material_id: isize,
    pub suck_gold: bool,
    pub suck_health: bool,
    pub suck_static_materials: bool,
    field12_0x5f: u8,
    pub suck_tag: [u8; 24],
    pub randomized_position: [u8; 16],
    pub m_amount_used: isize,
    pub m_gold_accumulator: isize,
    pub m_last_frame_picked_gold: isize,
    field18_0x94: u8,
    field19_0x95: u8,
    field20_0x96: u8,
    field21_0x97: u8,
}
impl Component for UIInfoComponent {
    const NAME: &'static str = "UIInfoComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct UIInfoComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
}
impl Component for LuaComponent {
    const NAME: &'static str = "LuaComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LuaComponent {
    pub inherited_fields: ComponentData,
    pub script_source_file: [u8; 24],
    pub vm_type: [u8; 4],
    pub execute_on_added: bool,
    pub execute_on_removed: bool,
    field5_0x66: u8,
    field6_0x67: u8,
    pub execute_every_n_frame: isize,
    pub execute_times: isize,
    pub limit_how_many_times_per_frame: isize,
    pub limit_to_every_n_frame: isize,
    pub limit_all_callbacks: bool,
    pub remove_after_executed: bool,
    pub enable_coroutines: bool,
    pub call_init_function: bool,
    pub script_enabled_changed: [u8; 24],
    pub script_damage_received: [u8; 24],
    pub script_damage_about_to_be_received: [u8; 24],
    pub script_item_picked_up: [u8; 24],
    pub script_shot: [u8; 24],
    pub script_collision_trigger_hit: [u8; 24],
    pub script_collision_trigger_timer_finished: [u8; 24],
    pub script_physics_body_modified: [u8; 24],
    pub script_pressure_plate_change: [u8; 24],
    pub script_inhaled_material: [u8; 24],
    pub script_death: [u8; 24],
    pub script_throw_item: [u8; 24],
    pub script_material_area_checker_failed: [u8; 24],
    pub script_material_area_checker_success: [u8; 24],
    pub script_electricity_receiver_switched: [u8; 24],
    pub script_electricity_receiver_electrified: [u8; 24],
    pub script_kick: [u8; 24],
    pub script_isizeeracting: [u8; 24],
    pub script_audio_event_dead: [u8; 24],
    pub script_wand_fired: [u8; 24],
    pub script_teleported: [u8; 24],
    pub script_portal_teleport_used: [u8; 24],
    pub script_polymorphing_to: [u8; 24],
    pub script_biome_entered: [u8; 24],
    pub m_last_execution_frame: isize,
    pub m_times_executed_this_frame: isize,
    pub m_mod_appends_done: bool,
    field42_0x2c5: u8,
    field43_0x2c6: u8,
    field44_0x2c7: u8,
    pub m_next_execution_time: isize,
    pub m_times_executed: isize,
    pub m_lua_manager: [*mut u8; 4],
    pub m_persistent_values: [u8; 8],
    field49_0x2dc: u8,
    field50_0x2dd: u8,
    field51_0x2de: u8,
    field52_0x2df: u8,
}
impl Component for StatusEffectDataComponent {
    const NAME: &'static str = "StatusEffectDataComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct StatusEffectDataComponent {
    pub inherited_fields: ComponentData,
    pub stain_effects: [u8; 12],
    pub stain_effect_cooldowns: [u8; 12],
    pub effects_previous: [u8; 12],
    pub ingestion_effects: [u8; 12],
    pub ingestion_effect_causes: [u8; 12],
    pub ingestion_effect_causes_many: [u8; 12],
    pub m_last_attacking_player_frame: isize,
    pub m_stain_effects_smoothed_for_u_i: [u8; 12],
    pub m_has_child_icons_cached: bool,
    field10_0xa1: u8,
    field11_0xa2: u8,
    field12_0xa3: u8,
    field13_0xa4: u8,
    field14_0xa5: u8,
    field15_0xa6: u8,
    field16_0xa7: u8,
}
impl Component for SpriteOffsetAnimatorComponent {
    const NAME: &'static str = "SpriteOffsetAnimatorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SpriteOffsetAnimatorComponent {
    pub inherited_fields: ComponentData,
    pub x_amount: f32,
    pub x_speed: f32,
    pub y_amount: f32,
    pub y_speed: f32,
    pub sprite_id: isize,
    pub x_phase: f32,
    pub x_phase_offset: f32,
    field8_0x64: u8,
    field9_0x65: u8,
    field10_0x66: u8,
    field11_0x67: u8,
}
impl Component for ElectricityReceiverComponent {
    const NAME: &'static str = "ElectricityReceiverComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ElectricityReceiverComponent {
    pub inherited_fields: ComponentData,
    pub offset_x: isize,
    pub offset_y: isize,
    pub radius: isize,
    pub active_time_frames: isize,
    pub switch_on_msg_isizeerval_frames: isize,
    pub electrified_msg_isizeerval_frames: isize,
    pub m_last_frame_electrified: isize,
    pub m_next_electrified_msg_frame: isize,
    pub m_next_switch_on_msg_frame: isize,
    field10_0x6c: u8,
    field11_0x6d: u8,
    field12_0x6e: u8,
    field13_0x6f: u8,
}
impl Component for CrawlerAnimalComponent {
    const NAME: &'static str = "CrawlerAnimalComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CrawlerAnimalComponent {
    pub inherited_fields: ComponentData,
    pub ray_length: f32,
    pub ray_count: isize,
    pub gravity: f32,
    pub terminal_velocity: f32,
    pub speed: f32,
    pub give_up_area_radius: isize,
    pub give_up_time: isize,
    pub attack_from_ceiling_check_ray_length: f32,
    pub attack_from_ceiling_check_every_n_frames: isize,
    pub collision_damage: f32,
    pub collision_damage_radius: f32,
    pub collision_damage_frames_between: isize,
    pub animate: bool,
    field14_0x79: u8,
    field15_0x7a: u8,
    field16_0x7b: u8,
    pub m_frame_next_give_up: isize,
    pub m_frame_next_damage: isize,
    pub m_frame_next_attack_from_ceiling_check: isize,
    field20_0x88: u8,
    field21_0x89: u8,
    field22_0x8a: u8,
    field23_0x8b: u8,
    field24_0x8c: u8,
    field25_0x8d: u8,
    field26_0x8e: u8,
    field27_0x8f: u8,
    field28_0x90: u8,
    field29_0x91: u8,
    field30_0x92: u8,
    field31_0x93: u8,
    field32_0x94: u8,
    field33_0x95: u8,
    field34_0x96: u8,
    field35_0x97: u8,
    pub m_prev_non_snapped_position: [u8; 8],
    pub m_prev_cell_position: [u8; 8],
    pub m_prev_cell_position2: [u8; 8],
    pub m_prev_cell_position3: [u8; 8],
    pub m_prev_cell_position4: [u8; 8],
    pub m_prev_cell_position5: [u8; 8],
    pub m_prev_cell_position6: [u8; 8],
    pub m_prev_cell_position7: [u8; 8],
    pub m_prev_cell_position8: [u8; 8],
    pub m_latest_position: [u8; 8],
    pub m_prev_falling: bool,
    pub m_is_initialized: bool,
    field48_0xea: u8,
    field49_0xeb: u8,
    pub m_velocity_y: f32,
    pub m_angle: f32,
    pub m_movement_step_accumulator: f32,
}
impl Component for IngestionComponent {
    const NAME: &'static str = "IngestionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct IngestionComponent {
    pub inherited_fields: ComponentData,
    pub ingestion_size: i64,
    pub ingestion_capacity: i64,
    pub ingestion_cooldown_delay_frames: usize,
    pub ingestion_reduce_every_n_frame: usize,
    pub overingestion_damage: f32,
    pub blood_healing_speed: f32,
    pub ingestion_satiation_material_tag: [u8; 24],
    pub m_ingestion_cooldown_frames: isize,
    pub m_next_overeating_msg_frame: isize,
    pub m_ingestion_satiation_material_tag_cached: [u8; 24],
    pub m_ingestion_satiation_material_cache: [u8; 8],
    pub m_damage_effect_lifetime: isize,
    field13_0xac: u8,
    field14_0xad: u8,
    field15_0xae: u8,
    field16_0xaf: u8,
}
impl Component for AttachToEntityComponent {
    const NAME: &'static str = "AttachToEntityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AttachToEntityComponent {
    pub inherited_fields: ComponentData,
    pub target: isize,
    pub only_position: bool,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub target_hotspot_tag: [u8; 24],
    pub target_sprite_id: isize,
    pub rotate_based_on_x_scale: bool,
    pub destroy_component_when_target_is_gone: bool,
    field10_0x6e: u8,
    field11_0x6f: u8,
    pub _transform: [u8; 32],
    pub m_update_frame: isize,
    field14_0x94: u8,
    field15_0x95: u8,
    field16_0x96: u8,
    field17_0x97: u8,
}
impl Component for InteractableComponent {
    const NAME: &'static str = "InteractableComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct InteractableComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub ui_text: [u8; 24],
    field3_0x64: u8,
    field4_0x65: u8,
    field5_0x66: u8,
    field6_0x67: u8,
    field7_0x68: u8,
    field8_0x69: u8,
    field9_0x6a: u8,
    field10_0x6b: u8,
    field11_0x6c: u8,
    field12_0x6d: u8,
    field13_0x6e: u8,
    field14_0x6f: u8,
    field15_0x70: u8,
    field16_0x71: u8,
    field17_0x72: u8,
    field18_0x73: u8,
    field19_0x74: u8,
    field20_0x75: u8,
    field21_0x76: u8,
    field22_0x77: u8,
    field23_0x78: u8,
    field24_0x79: u8,
    field25_0x7a: u8,
    field26_0x7b: u8,
    pub exclusivity_group: isize,
}
impl Component for ItemActionComponent {
    const NAME: &'static str = "ItemActionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemActionComponent {
    pub inherited_fields: ComponentData,
    pub action_id: [u8; 24],
}
impl Component for PhysicsJoisize2Component {
    const NAME: &'static str = "PhysicsJoisize2Component";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsJoisize2Component {
    pub inherited_fields: ComponentData,
    pub joisize_id: u16,
    field2_0x4a: u8,
    field3_0x4b: u8,
    pub break_force: f32,
    pub break_distance: f32,
    pub break_on_body_modified: bool,
    field7_0x55: u8,
    field8_0x56: u8,
    field9_0x57: u8,
    pub break_on_shear_angle_deg: f32,
    field11_0x5c: u8,
    field12_0x5d: u8,
    field13_0x5e: u8,
    field14_0x5f: u8,
    pub body1_id: isize,
    pub body2_id: isize,
    pub offset_x: f32,
    pub offset_y: f32,
    pub ray_x: f32,
    pub ray_y: f32,
    pub surface_attachment_offset_x: f32,
    pub surface_attachment_offset_y: f32,
}
impl Component for LoadEntitiesComponent {
    const NAME: &'static str = "LoadEntitiesComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LoadEntitiesComponent {
    pub inherited_fields: ComponentData,
    pub entity_file: [u8; 24],
    pub count: [u8; 8],
    pub kill_entity: bool,
    field4_0x69: u8,
    field5_0x6a: u8,
    field6_0x6b: u8,
    pub timeout_frames: isize,
    pub m_timer_trigger_frame: isize,
    field9_0x74: u8,
    field10_0x75: u8,
    field11_0x76: u8,
    field12_0x77: u8,
}
impl Component for DebugFollowMouseComponent {
    const NAME: &'static str = "DebugFollowMouseComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DebugFollowMouseComponent {
    pub inherited_fields: ComponentData,
}
impl Component for ControllerGoombaAIComponent {
    const NAME: &'static str = "ControllerGoombaAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ControllerGoombaAIComponent {
    pub inherited_fields: ComponentData,
    pub m_changing_direction_counter: isize,
    pub auto_turn_around_enabled: bool,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub wait_to_turn_around: isize,
    pub wall_hit_wait: isize,
    pub check_wall_detection: bool,
    field9_0x59: u8,
    field10_0x5a: u8,
    field11_0x5b: u8,
    pub wall_detection_aabb_min_x: f32,
    pub wall_detection_aabb_max_x: f32,
    pub wall_detection_aabb_min_y: f32,
    pub wall_detection_aabb_max_y: f32,
    pub check_floor_detection: bool,
    field17_0x6d: u8,
    field18_0x6e: u8,
    field19_0x6f: u8,
    pub floor_detection_aabb_min_x: f32,
    pub floor_detection_aabb_max_x: f32,
    pub floor_detection_aabb_min_y: f32,
    pub floor_detection_aabb_max_y: f32,
}
impl Component for ManaReloaderComponent {
    const NAME: &'static str = "ManaReloaderComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ManaReloaderComponent {
    pub inherited_fields: ComponentData,
}
impl Component for DamageModelComponent {
    const NAME: &'static str = "DamageModelComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DamageModelComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    pub max_hp: f64,
    pub max_hp_cap: f64,
    pub max_hp_old: f64,
    pub damage_multipliers: [u8; 64],
    pub critical_damage_resistance: f32,
    pub invincibility_frames: isize,
    pub falling_damages: bool,
    field16_0xb1: u8,
    field17_0xb2: u8,
    field18_0xb3: u8,
    pub falling_damage_height_min: f32,
    pub falling_damage_height_max: f32,
    pub falling_damage_damage_min: f32,
    pub falling_damage_damage_max: f32,
    pub air_needed: bool,
    field24_0xc5: u8,
    field25_0xc6: u8,
    field26_0xc7: u8,
    pub air_in_lungs: f32,
    pub air_in_lungs_max: f32,
    pub air_lack_of_damage: f32,
    pub minimum_knockback_force: f32,
    pub materials_damage: bool,
    field32_0xd9: u8,
    field33_0xda: u8,
    field34_0xdb: u8,
    pub material_damage_min_cell_count: isize,
    pub materials_that_damage: [u8; 24],
    pub materials_how_much_damage: [u8; 24],
    pub materials_damage_proportional_to_maxhp: bool,
    pub physics_objects_damage: bool,
    pub materials_create_messages: bool,
    field41_0x113: u8,
    pub materials_that_create_messages: [u8; 24],
    pub ragdoll_filenames_file: [u8; 24],
    pub ragdoll_material: [u8; 24],
    pub ragdoll_offset_x: f32,
    pub ragdoll_offset_y: f32,
    pub ragdoll_fx_forced: [u8; 4],
    pub blood_material: [u8; 24],
    pub blood_spray_material: [u8; 24],
    pub blood_spray_create_some_cosmetic: bool,
    field51_0x199: u8,
    field52_0x19a: u8,
    field53_0x19b: u8,
    pub blood_multiplier: f32,
    pub ragdoll_blood_amount_absolute: isize,
    pub blood_sprite_directional: [u8; 24],
    pub blood_sprite_large: [u8; 24],
    pub healing_particle_effect_entity: [u8; 24],
    pub create_ragdoll: bool,
    pub ragdollify_child_entity_sprites: bool,
    field61_0x1ee: u8,
    field62_0x1ef: u8,
    pub ragdollify_root_angular_damping: f32,
    pub ragdollify_disisizeegrate_nonroot: bool,
    pub wait_for_kill_flag_on_death: bool,
    pub kill_now: bool,
    pub drop_items_on_death: bool,
    pub ui_report_damage: bool,
    pub ui_force_report_damage: bool,
    field70_0x1fa: u8,
    field71_0x1fb: u8,
    pub in_liquid_shooting_electrify_prob: isize,
    pub wet_status_effect_damage: f32,
    pub is_on_fire: bool,
    field75_0x205: u8,
    field76_0x206: u8,
    field77_0x207: u8,
    pub fire_probability_of_ignition: f32,
    pub fire_how_much_fire_generates: isize,
    pub fire_damage_ignited_amount: f32,
    pub fire_damage_amount: f32,
    pub m_is_on_fire: bool,
    field83_0x219: u8,
    field84_0x21a: u8,
    field85_0x21b: u8,
    pub m_fire_probability: isize,
    pub m_fire_frames_left: isize,
    pub m_fire_duration_frames: isize,
    pub m_fire_tried_igniting: bool,
    field90_0x229: u8,
    field91_0x22a: u8,
    field92_0x22b: u8,
    pub m_last_check_x: isize,
    pub m_last_check_y: isize,
    pub m_last_check_time: isize,
    pub m_last_material_damage_frame: isize,
    pub m_fall_is_on_ground: bool,
    field98_0x23d: u8,
    field99_0x23e: u8,
    field100_0x23f: u8,
    pub m_fall_highest_y: f32,
    pub m_fall_count: isize,
    pub m_air_are_we_in_water: bool,
    field104_0x249: u8,
    field105_0x24a: u8,
    field106_0x24b: u8,
    pub m_air_frames_not_in_water: isize,
    pub m_air_do_we_have: bool,
    field109_0x251: u8,
    field110_0x252: u8,
    field111_0x253: u8,
    pub m_total_cells: isize,
    pub m_liquid_count: isize,
    pub m_liquid_material_we_are_in: isize,
    pub m_damage_materials: [u8; 12],
    pub m_damage_materials_how_much: [u8; 12],
    pub m_collision_message_materials: [u8; 12],
    pub m_collision_message_material_counts_this_frame: [u8; 12],
    pub m_material_damage_this_frame: [u8; 12],
    pub m_fall_damage_this_frame: f32,
    pub m_electricity_damage_this_frame: f32,
    pub m_physics_damage_this_frame: f32,
    pub m_physics_damage_vec_this_frame: [u8; 8],
    pub m_physics_damage_last_frame: isize,
    pub m_physics_damage_entity: [u8; 4],
    pub m_physics_damage_telekinesis_caster_entity: [u8; 4],
    pub m_last_damage_frame: isize,
    pub m_hp_before_last_damage: f64,
    pub m_last_electricity_resistance_frame: isize,
    pub m_last_frame_reported_block: isize,
    pub m_last_max_hp_change_frame: isize,
    pub m_fire_damage_buffered: f32,
    pub m_fire_damage_buffered_next_delivery_frame: isize,
    field134_0x2dc: u8,
    field135_0x2dd: u8,
    field136_0x2de: u8,
    field137_0x2df: u8,
}
impl Component for BossHealthBarComponent {
    const NAME: &'static str = "BossHealthBarComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct BossHealthBarComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    pub gui_special_final_boss: bool,
    pub in_world: bool,
    field4_0x4b: u8,
    pub gui_max_distance_visible: f32,
    pub m_old_sprites_destroyed: bool,
    field7_0x51: u8,
    field8_0x52: u8,
    field9_0x53: u8,
    field10_0x54: u8,
    field11_0x55: u8,
    field12_0x56: u8,
    field13_0x57: u8,
}
impl Component for AudioListenerComponent {
    const NAME: &'static str = "AudioListenerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AudioListenerComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
}
impl Component for ItemAlchemyComponent {
    const NAME: &'static str = "ItemAlchemyComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemAlchemyComponent {
    pub inherited_fields: ComponentData,
    pub material_make_always_cast: isize,
    pub material_remove_shuffle: isize,
    pub material_animate_wand: isize,
    pub material_animate_wand_alt: isize,
    pub material_increase_uses_remaining: isize,
    pub material_sacrifice: isize,
}
impl Component for ShotEffectComponent {
    const NAME: &'static str = "ShotEffectComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ShotEffectComponent {
    pub inherited_fields: ComponentData,
    pub condition_effect: GameEffect,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub condition_status: [u8; 4],
    pub extra_modifier: [u8; 24],
}
impl Component for ProjectileComponent {
    const NAME: &'static str = "ProjectileComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ProjectileComponent {
    pub inherited_fields: ComponentData,
    pub projectile_type: [u8; 4],
    pub lifetime: isize,
    pub lifetime_randomness: isize,
    pub on_lifetime_out_explode: bool,
    pub collide_with_world: bool,
    field6_0x56: u8,
    field7_0x57: u8,
    pub config: [u8; 572],
    pub speed_min: f32,
    pub speed_max: f32,
    pub friction: f32,
    pub direction_random_rad: f32,
    pub direction_nonrandom_rad: f32,
    pub lob_min: f32,
    pub lob_max: f32,
    pub camera_shake_when_shot: f32,
    pub shoot_light_flash_radius: f32,
    pub shoot_light_flash_r: [u8; 4],
    pub shoot_light_flash_g: [u8; 4],
    pub shoot_light_flash_b: [u8; 4],
    pub create_shell_casing: bool,
    field22_0x2c5: u8,
    field23_0x2c6: u8,
    field24_0x2c7: u8,
    pub shell_casing_material: [u8; 24],
    pub shell_casing_offset: [u8; 8],
    pub muzzle_flash_file: [u8; 24],
    pub bounces_left: isize,
    pub bounce_energy: f32,
    pub bounce_always: bool,
    pub bounce_at_any_angle: bool,
    pub attach_to_parent_trigger: bool,
    field33_0x30b: u8,
    pub bounce_fx_file: [u8; 24],
    pub angular_velocity: f32,
    pub velocity_sets_rotation: bool,
    pub velocity_sets_scale: bool,
    field38_0x32a: u8,
    field39_0x32b: u8,
    pub velocity_sets_scale_coeff: f32,
    pub velocity_sets_y_flip: bool,
    field42_0x331: u8,
    field43_0x332: u8,
    field44_0x333: u8,
    pub velocity_updates_animation: f32,
    pub ground_penetration_coeff: f32,
    pub ground_penetration_max_durability_to_destroy: isize,
    pub go_through_this_material: [u8; 24],
    pub do_moveto_update: bool,
    field50_0x359: u8,
    field51_0x35a: u8,
    field52_0x35b: u8,
    pub on_death_duplicate_remaining: isize,
    pub on_death_gfx_leave_sprite: bool,
    pub on_death_explode: bool,
    pub on_death_emit_particle: bool,
    field57_0x363: u8,
    pub on_death_emit_particle_count: isize,
    pub die_on_liquid_collision: bool,
    pub die_on_low_velocity: bool,
    field61_0x36a: u8,
    field62_0x36b: u8,
    pub die_on_low_velocity_limit: f32,
    pub on_death_emit_particle_type: [u8; 24],
    pub on_death_particle_check_concrete: bool,
    pub ground_collision_fx: bool,
    pub explosion_dont_damage_shooter: bool,
    field68_0x38b: u8,
    pub config_explosion: [u8; 372],
    pub on_death_item_pickable_radius: f32,
    pub penetrate_world: bool,
    field72_0x505: u8,
    field73_0x506: u8,
    field74_0x507: u8,
    pub penetrate_world_velocity_coeff: f32,
    pub penetrate_entities: bool,
    pub on_collision_die: bool,
    pub on_collision_remove_projectile: bool,
    pub on_collision_spawn_entity: bool,
    pub spawn_entity: [u8; 24],
    pub spawn_entity_is_projectile: bool,
    field82_0x529: u8,
    field83_0x52a: u8,
    field84_0x52b: u8,
    pub physics_impulse_coeff: f32,
    pub damage_every_x_frames: isize,
    pub damage_scaled_by_speed: bool,
    field88_0x535: u8,
    field89_0x536: u8,
    field90_0x537: u8,
    pub damage_scale_max_speed: f32,
    pub ragdoll_fx_on_collision: [u8; 4],
    pub collide_with_entities: bool,
    field94_0x541: u8,
    field95_0x542: u8,
    field96_0x543: u8,
    pub collide_with_tag: [u8; 24],
    pub dont_collide_with_tag: [u8; 24],
    pub collide_with_shooter_frames: isize,
    pub friendly_fire: bool,
    field101_0x579: u8,
    field102_0x57a: u8,
    field103_0x57b: u8,
    pub damage: f32,
    pub damage_by_type: [u8; 64],
    pub damage_critical: [u8; 16],
    pub knockback_force: f32,
    pub ragdoll_force_multiplier: f32,
    pub hit_particle_force_multiplier: f32,
    pub blood_count_multiplier: f32,
    pub damage_game_effect_entities: [u8; 24],
    pub never_hit_player: bool,
    pub collect_materials_to_shooter: bool,
    field114_0x5fa: u8,
    field115_0x5fb: u8,
    pub m_who_shot: isize,
    pub m_who_shot_entity_type_i_d: [u8; 4],
    pub m_shooter_herd_id: isize,
    pub m_starting_lifetime: isize,
    pub m_entity_that_shot: isize,
    pub m_triggers: [u8; 36],
    pub play_damage_sounds: bool,
    field123_0x635: u8,
    field124_0x636: u8,
    field125_0x637: u8,
    pub m_last_frame_damaged: isize,
    pub m_damaged_entities: [u8; 12],
    pub m_initial_speed: f32,
    field129_0x64c: u8,
    field130_0x64d: u8,
    field131_0x64e: u8,
    field132_0x64f: u8,
}
impl Component for PlatformShooterPlayerComponent {
    const NAME: &'static str = "PlatformShooterPlayerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PlatformShooterPlayerComponent {
    pub inherited_fields: ComponentData,
    pub aiming_reticle_distance_from_character: f32,
    pub camera_max_distance_from_character: f32,
    pub alcohol_drunken_speed: f32,
    pub blood_fungi_drunken_speed: f32,
    pub blood_worm_drunken_speed: f32,
    pub eating_area_min: [u8; 8],
    pub eating_area_max: [u8; 8],
    pub eating_cells_per_frame: isize,
    pub eating_probability: isize,
    pub eating_delay_frames: isize,
    pub stoned_speed: f32,
    pub center_camera_on_this_entity: bool,
    pub move_camera_with_aim: bool,
    field14_0x7e: u8,
    field15_0x7f: u8,
    pub m_smoothed_camera_position: [u8; 8],
    pub m_smoothed_aiming_vector: [u8; 8],
    pub m_camera_recoil: f32,
    pub m_camera_recoil_target: f32,
    pub m_crouching: bool,
    field21_0x99: u8,
    field22_0x9a: u8,
    field23_0x9b: u8,
    pub m_camera_distance_lerped: f32,
    pub m_require_trigger_pull: bool,
    field26_0xa1: u8,
    field27_0xa2: u8,
    field28_0xa3: u8,
    pub m_warp_delay: isize,
    pub m_item_temporarily_hidden: isize,
    pub m_desired_camera_pos: [u8; 8],
    pub m_has_gamepad_controls_prev: bool,
    pub m_force_fire_on_next_update: bool,
    field34_0xb6: u8,
    field35_0xb7: u8,
    pub m_fast_movement_particles_alpha_smoothed: f32,
    field37_0xbc: u8,
    field38_0xbd: u8,
    field39_0xbe: u8,
    field40_0xbf: u8,
    pub m_tele_bolt_frames_during_last_second: u64,
    pub m_cam_correction_tele_smoothed: f32,
    pub m_cam_correction_gain_smoothed: [u8; 8],
    pub m_camera_error_prev: [u8; 1280],
    pub m_cam_error_averaged: [u8; 8],
    pub m_cam_moving_fast_prev: bool,
    field47_0x5dd: u8,
    field48_0x5de: u8,
    field49_0x5df: u8,
    pub m_cam_frame_started_moving_fast: isize,
    pub m_cam_frame_last_moving_fast_explosion: isize,
    pub m_cessation_do: bool,
    field53_0x5e9: u8,
    field54_0x5ea: u8,
    field55_0x5eb: u8,
    pub m_cessation_lifetime: isize,
}
impl Component for ItemComponent {
    const NAME: &'static str = "ItemComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemComponent {
    pub inherited_fields: ComponentData,
    pub item_name: [u8; 24],
    pub is_stackable: bool,
    pub is_consumable: bool,
    pub stats_count_as_item_pick_up: bool,
    pub auto_pickup: bool,
    pub permanently_attached: bool,
    field7_0x65: u8,
    field8_0x66: u8,
    field9_0x67: u8,
    pub uses_remaining: isize,
    pub is_identified: bool,
    pub is_frozen: bool,
    pub collect_nondefault_actions: bool,
    pub remove_on_death: bool,
    pub remove_on_death_if_empty: bool,
    pub remove_default_child_actions_on_death: bool,
    pub play_hover_animation: bool,
    pub play_spinning_animation: bool,
    pub is_equipable_forced: bool,
    pub play_pick_sound: bool,
    pub drinkable: bool,
    field22_0x77: u8,
    pub spawn_pos: [u8; 8],
    pub max_child_items: isize,
    pub ui_sprite: [u8; 24],
    pub ui_description: [u8; 24],
    pub preferred_inventory: [u8; 4],
    pub enable_orb_hacks: bool,
    pub is_all_spells_book: bool,
    pub always_use_item_name_in_ui: bool,
    field31_0xbb: u8,
    pub custom_pickup_string: [u8; 24],
    pub ui_display_description_on_pick_up_hisize: bool,
    field34_0xd5: u8,
    field35_0xd6: u8,
    field36_0xd7: u8,
    pub inventory_slot: [u8; 8],
    pub next_frame_pickable: isize,
    pub npc_next_frame_pickable: isize,
    pub is_pickable: bool,
    pub is_hittable_always: bool,
    field42_0xea: u8,
    field43_0xeb: u8,
    pub item_pickup_radius: f32,
    pub camera_max_distance: f32,
    pub camera_smooth_speed_multiplier: f32,
    pub has_been_picked_by_player: bool,
    field48_0xf9: u8,
    field49_0xfa: u8,
    field50_0xfb: u8,
    pub m_frame_picked_up: isize,
    pub m_item_uid: isize,
    pub m_is_identified: bool,
    field54_0x105: u8,
    field55_0x106: u8,
    field56_0x107: u8,
}
impl Component for AnimalAIComponent {
    const NAME: &'static str = "AnimalAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AnimalAIComponent {
    pub inherited_fields: ComponentData,
    pub ai_state: isize,
    pub ai_state_timer: isize,
    pub keep_state_alive_when_enabled: bool,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub preferred_job: [u8; 24],
    pub escape_if_damaged_probability: isize,
    pub attack_if_damaged_probability: isize,
    pub eye_offset_x: isize,
    pub eye_offset_y: isize,
    pub attack_only_if_attacked: bool,
    pub dont_counter_attack_own_herd: bool,
    field14_0x7e: u8,
    field15_0x7f: u8,
    pub creature_detection_range_x: f32,
    pub creature_detection_range_y: f32,
    pub creature_detection_angular_range_deg: f32,
    pub creature_detection_check_every_x_frames: isize,
    pub max_distance_to_cam_to_start_hunting: f32,
    pub pathfinding_max_depth_no_target: isize,
    pub pathfinding_max_depth_has_target: isize,
    pub aggressiveness_min: f32,
    pub aggressiveness_max: f32,
    pub tries_to_ranged_attack_friends: bool,
    pub attack_melee_enabled: bool,
    pub attack_dash_enabled: bool,
    pub attack_landing_ranged_enabled: bool,
    pub attack_ranged_enabled: bool,
    field30_0xa9: u8,
    field31_0xaa: u8,
    field32_0xab: u8,
    pub attack_knockback_multiplier: f32,
    pub is_static_turret: bool,
    field35_0xb1: u8,
    field36_0xb2: u8,
    field37_0xb3: u8,
    pub attack_melee_max_distance: isize,
    pub attack_melee_action_frame: isize,
    pub attack_melee_frames_between: isize,
    pub attack_melee_damage_min: f32,
    pub attack_melee_damage_max: f32,
    pub attack_melee_impulse_vector_x: f32,
    pub attack_melee_impulse_vector_y: f32,
    pub attack_melee_impulse_multiplier: f32,
    pub attack_melee_offset_x: f32,
    pub attack_melee_offset_y: f32,
    pub attack_melee_finish_enabled: bool,
    field49_0xdd: u8,
    field50_0xde: u8,
    field51_0xdf: u8,
    pub attack_melee_finish_config_explosion: [u8; 372],
    pub attack_melee_finish_action_frame: isize,
    pub attack_dash_distance: f32,
    pub attack_dash_frames_between: isize,
    pub attack_dash_damage: f32,
    pub attack_dash_speed: f32,
    pub attack_dash_lob: f32,
    pub attack_ranged_min_distance: f32,
    pub attack_ranged_max_distance: f32,
    pub attack_ranged_action_frame: isize,
    pub attack_ranged_frames_between: [u8; 12],
    pub attack_ranged_offset_x: f32,
    pub attack_ranged_offset_y: f32,
    pub attack_ranged_use_message: bool,
    pub attack_ranged_predict: bool,
    field67_0x28e: u8,
    field68_0x28f: u8,
    pub attack_ranged_entity_file: [u8; 24],
    pub attack_ranged_entity_count_min: isize,
    pub attack_ranged_entity_count_max: isize,
    pub attack_ranged_use_laser_sight: bool,
    pub attack_ranged_laser_sight_beam_kind: bool,
    pub attack_ranged_aim_rotation_enabled: bool,
    field75_0x2b3: u8,
    pub attack_ranged_aim_rotation_speed: f32,
    pub attack_ranged_aim_rotation_shooting_ok_angle_deg: f32,
    pub attack_ranged_state_duration_frames: isize,
    pub hide_from_prey: bool,
    field80_0x2c1: u8,
    field81_0x2c2: u8,
    field82_0x2c3: u8,
    pub hide_from_prey_target_distance: f32,
    pub hide_from_prey_time: isize,
    pub food_material: isize,
    pub food_particle_effect_material: isize,
    pub food_eating_create_particles: bool,
    field88_0x2d5: u8,
    field89_0x2d6: u8,
    field90_0x2d7: u8,
    pub eating_area_radius_x: isize,
    pub eating_area_radius_y: isize,
    pub mouth_offset_x: isize,
    pub mouth_offset_y: isize,
    pub defecates_and_pees: bool,
    field96_0x2e9: u8,
    field97_0x2ea: u8,
    field98_0x2eb: u8,
    pub butt_offset_x: isize,
    pub butt_offset_y: isize,
    pub pee_velocity_x: f32,
    pub pee_velocity_y: f32,
    pub needs_food: bool,
    pub sense_creatures: bool,
    pub sense_creatures_through_walls: bool,
    pub can_fly: bool,
    pub can_walk: bool,
    field108_0x301: u8,
    field109_0x302: u8,
    field110_0x303: u8,
    pub path_distance_to_target_node_to_turn_around: isize,
    pub path_cleanup_explosion_radius: f32,
    pub max_distance_to_move_from_home: f32,
    pub m_ai_state_stack: [u8; 12],
    pub m_ai_state_last_switch_frame: isize,
    pub m_ai_state_prev: isize,
    pub m_creature_detection_next_check: isize,
    pub m_greatest_threat: isize,
    pub m_greatest_prey: isize,
    pub m_selected_multi_attack: isize,
    pub m_has_found_prey: bool,
    pub m_has_been_attacked_by_player: bool,
    pub m_has_started_attacking: bool,
    field124_0x337: u8,
    pub m_nearby_food_count: isize,
    pub m_eat_next_frame: isize,
    pub m_eat_time: isize,
    pub m_frame_next_give_up: isize,
    pub m_last_frames_movement_area_min: [u8; 8],
    pub m_last_frames_movement_area_max: [u8; 8],
    pub m_food_material_id: isize,
    pub m_food_particle_effect_material_id: isize,
    pub m_aggression: [u8; 12],
    pub m_next_jump_lob: f32,
    pub m_next_jump_target: [u8; 8],
    pub m_next_jump_has_velocity: bool,
    field137_0x379: u8,
    field138_0x37a: u8,
    field139_0x37b: u8,
    pub m_last_frame_jumped: isize,
    pub m_frames_without_target: isize,
    pub m_last_frame_can_damage_own_herd: isize,
    pub m_home_position: [u8; 8],
    pub m_last_frame_attack_was_done: isize,
    pub m_next_frame_can_call_friend: isize,
    pub m_next_frame_respond_friend: isize,
    pub m_has_noticed_player: bool,
    field148_0x39d: u8,
    field149_0x39e: u8,
    field150_0x39f: u8,
    pub m_ranged_attack_current_aim_angle: f32,
    pub m_ranged_attack_next_frame: isize,
    pub m_melee_attack_next_frame: isize,
    pub m_next_melee_attack_damage: f32,
    pub m_melee_attacking: bool,
    field156_0x3b1: u8,
    field157_0x3b2: u8,
    field158_0x3b3: u8,
    pub m_melee_attack_dash_next_frame: isize,
    pub m_current_job: [u8; 36],
    field161_0x3dc: u8,
    field162_0x3dd: u8,
    field163_0x3de: u8,
    field164_0x3df: u8,
}
impl Component for OrbComponent {
    const NAME: &'static str = "OrbComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct OrbComponent {
    pub inherited_fields: ComponentData,
    pub orb_id: isize,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for LooseGroundComponent {
    const NAME: &'static str = "LooseGroundComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LooseGroundComponent {
    pub inherited_fields: ComponentData,
    pub probability: f32,
    pub max_durability: isize,
    pub max_distance: f32,
    pub max_angle: f32,
    pub min_radius: isize,
    pub max_radius: isize,
    pub chunk_probability: f32,
    pub chunk_max_angle: f32,
    pub chunk_count: isize,
    pub chunk_material: isize,
    pub m_chunk_count: isize,
    pub collapse_images: [u8; 24],
    field13_0x8c: u8,
    field14_0x8d: u8,
    field15_0x8e: u8,
    field16_0x8f: u8,
}
impl Component for TeleportProjectileComponent {
    const NAME: &'static str = "TeleportProjectileComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct TeleportProjectileComponent {
    pub inherited_fields: ComponentData,
    pub min_distance_from_wall: f32,
    pub actionable_lifetime: isize,
    pub reset_shooter_y_vel: bool,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub m_who_shot: isize,
}
impl Component for LifetimeComponent {
    const NAME: &'static str = "LifetimeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LifetimeComponent {
    pub inherited_fields: ComponentData,
    pub creation_frame: isize,
    pub lifetime: isize,
    pub randomize_lifetime: [u8; 8],
    pub fade_sprites: bool,
    pub kill_parent: bool,
    pub kill_all_parents: bool,
    field7_0x5b: u8,
    pub kill_frame: isize,
    pub serialize_duration: bool,
    field10_0x61: u8,
    field11_0x62: u8,
    field12_0x63: u8,
    pub kill_frame_serialized: isize,
    pub creation_frame_serialized: isize,
    field15_0x6c: u8,
    field16_0x6d: u8,
    field17_0x6e: u8,
    field18_0x6f: u8,
}
impl Component for ElectricitySourceComponent {
    const NAME: &'static str = "ElectricitySourceComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ElectricitySourceComponent {
    pub inherited_fields: ComponentData,
    pub radius: isize,
    pub emission_isizeerval_frames: isize,
    pub m_next_frame_emit_electricity: isize,
    field4_0x54: u8,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
}
impl Component for BiomeTrackerComponent {
    const NAME: &'static str = "BiomeTrackerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct BiomeTrackerComponent {
    pub inherited_fields: ComponentData,
    pub limit_to_every_n_frame: isize,
    pub unsafe_current_biome: [*mut u8; 4],
    pub current_biome_name: [u8; 24],
}
impl Component for PixelSceneComponent {
    const NAME: &'static str = "PixelSceneComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PixelSceneComponent {
    pub inherited_fields: ComponentData,
    pub pixel_scene: [u8; 24],
    pub pixel_scene_visual: [u8; 24],
    pub pixel_scene_background: [u8; 24],
    pub background_z_index: isize,
    pub offset_x: f32,
    pub offset_y: f32,
    pub skip_biome_checks: bool,
    pub skip_edge_textures: bool,
    field9_0x9e: u8,
    field10_0x9f: u8,
}
impl Component for CameraBoundComponent {
    const NAME: &'static str = "CameraBoundComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CameraBoundComponent {
    pub inherited_fields: ComponentData,
    pub enabled: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub distance: f32,
    pub distance_border: f32,
    pub max_count: isize,
    pub freeze_on_distance_kill: bool,
    pub freeze_on_max_count_kill: bool,
    field10_0x5a: u8,
    field11_0x5b: u8,
    field12_0x5c: u8,
    field13_0x5d: u8,
    field14_0x5e: u8,
    field15_0x5f: u8,
}
impl Component for LiquidDisplacerComponent {
    const NAME: &'static str = "LiquidDisplacerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LiquidDisplacerComponent {
    pub inherited_fields: ComponentData,
    pub radius: isize,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub m_prev_x: isize,
    pub m_prev_y: isize,
    field6_0x5c: u8,
    field7_0x5d: u8,
    field8_0x5e: u8,
    field9_0x5f: u8,
}
impl Component for GameEffectComponent {
    const NAME: &'static str = "GameEffectComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GameEffectComponent {
    pub inherited_fields: ComponentData,
    pub effect: GameEffect,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub custom_effect_id: [u8; 24],
    pub frames: isize,
    pub exclusivity_group: isize,
    pub report_block_msg: bool,
    pub disable_movement: bool,
    field10_0x6e: u8,
    field11_0x6f: u8,
    pub ragdoll_effect: [u8; 4],
    pub ragdoll_material: isize,
    pub ragdoll_effect_custom_entity_file: [u8; 24],
    pub ragdoll_fx_custom_entity_apply_only_to_largest_body: bool,
    field16_0x91: u8,
    field17_0x92: u8,
    field18_0x93: u8,
    pub polymorph_target: [u8; 24],
    pub m_serialized_data: [u8; 24],
    pub m_caster: isize,
    pub m_caster_herd_id: isize,
    pub teleportation_probability: isize,
    pub teleportation_delay_min_frames: isize,
    pub teleportation_radius_min: f32,
    pub teleportation_radius_max: f32,
    pub teleportations_num: isize,
    pub no_heal_max_hp_cap: f64,
    pub causing_status_effect: [u8; 4],
    pub caused_by_ingestion_status_effect: bool,
    pub caused_by_stains: bool,
    pub m_charm_disabled_camera_bound: bool,
    pub m_charm_enabled_teleporting: bool,
    pub m_invisible: bool,
    field35_0xf1: u8,
    field36_0xf2: u8,
    field37_0xf3: u8,
    pub m_counter: isize,
    pub m_cooldown: isize,
    pub m_is_extension: bool,
    pub m_is_spent: bool,
    field42_0xfe: u8,
    field43_0xff: u8,
}
impl Component for WalletComponent {
    const NAME: &'static str = "WalletComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WalletComponent {
    pub inherited_fields: ComponentData,
    pub money: i64,
    pub money_spent: i64,
    pub m_money_prev_frame: i64,
    pub m_has_reached_inf: bool,
    field5_0x61: u8,
    field6_0x62: u8,
    field7_0x63: u8,
    field8_0x64: u8,
    field9_0x65: u8,
    field10_0x66: u8,
    field11_0x67: u8,
}
impl Component for ItemStashComponent {
    const NAME: &'static str = "ItemStashComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemStashComponent {
    pub inherited_fields: ComponentData,
    pub throw_openable_cooldown_frames: isize,
    pub init_children: bool,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub m_next_frame_openable: isize,
    pub m_frame_opened: isize,
}
impl Component for MoveToSurfaceOnCreateComponent {
    const NAME: &'static str = "MoveToSurfaceOnCreateComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MoveToSurfaceOnCreateComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub lookup_radius: f32,
    pub offset_from_surface: f32,
    pub ray_count: isize,
    pub verlet_min_joisize_distance: f32,
    field9_0x5c: u8,
    field10_0x5d: u8,
    field11_0x5e: u8,
    field12_0x5f: u8,
}
impl Component for UIIconComponent {
    const NAME: &'static str = "UIIconComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct UIIconComponent {
    pub inherited_fields: ComponentData,
    pub icon_sprite_file: [u8; 24],
    field2_0x60: u8,
    field3_0x61: u8,
    field4_0x62: u8,
    field5_0x63: u8,
    field6_0x64: u8,
    field7_0x65: u8,
    field8_0x66: u8,
    field9_0x67: u8,
    field10_0x68: u8,
    field11_0x69: u8,
    field12_0x6a: u8,
    field13_0x6b: u8,
    field14_0x6c: u8,
    field15_0x6d: u8,
    field16_0x6e: u8,
    field17_0x6f: u8,
    field18_0x70: u8,
    field19_0x71: u8,
    field20_0x72: u8,
    field21_0x73: u8,
    field22_0x74: u8,
    field23_0x75: u8,
    field24_0x76: u8,
    field25_0x77: u8,
    pub description: [u8; 24],
    pub display_above_head: bool,
    pub display_in_hud: bool,
    pub is_perk: bool,
    field30_0x93: u8,
    field31_0x94: u8,
    field32_0x95: u8,
    field33_0x96: u8,
    field34_0x97: u8,
}
impl Component for DebugSpatialVisualizerComponent {
    const NAME: &'static str = "DebugSpatialVisualizerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DebugSpatialVisualizerComponent {
    pub inherited_fields: ComponentData,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub color: [u8; 4],
    field6_0x5c: u8,
    field7_0x5d: u8,
    field8_0x5e: u8,
    field9_0x5f: u8,
}
impl Component for HitboxComponent {
    const NAME: &'static str = "HitboxComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct HitboxComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    pub is_player: bool,
    pub is_enemy: bool,
    pub is_item: bool,
    pub aabb_min_x: f32,
    pub aabb_max_x: f32,
    pub aabb_min_y: f32,
    pub aabb_max_y: f32,
    pub offset: [u8; 8],
    pub damage_multiplier: f32,
}
impl Component for PathFindingGridMarkerComponent {
    const NAME: &'static str = "PathFindingGridMarkerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PathFindingGridMarkerComponent {
    pub inherited_fields: ComponentData,
    pub marker_work_flag: isize,
    pub marker_offset_x: f32,
    pub marker_offset_y: f32,
    pub player_marker_radius: f32,
    pub m_node: [u8; 8],
}
impl Component for GasBubbleComponent {
    const NAME: &'static str = "GasBubbleComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GasBubbleComponent {
    pub inherited_fields: ComponentData,
    pub acceleration: f32,
    pub max_speed: f32,
    pub m_velocity: f32,
    field4_0x54: u8,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
}
impl Component for WormAttractorComponent {
    const NAME: &'static str = "WormAttractorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WormAttractorComponent {
    pub inherited_fields: ComponentData,
    pub direction: isize,
    pub radius: f32,
}
impl Component for PhysicsPickUpComponent {
    const NAME: &'static str = "PhysicsPickUpComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsPickUpComponent {
    pub inherited_fields: ComponentData,
    pub transform: [u8; 32],
    pub original_left_joisize_pos: [u8; 8],
    pub original_right_joisize_pos: [u8; 8],
    pub pick_up_strength: f32,
    pub is_broken: bool,
    field6_0x7d: u8,
    field7_0x7e: u8,
    field8_0x7f: u8,
    pub left_joisize_pos: [u8; 8],
    pub right_joisize_pos: [u8; 8],
    pub left_joisize: [*mut u8; 4],
    pub right_joisize: [*mut u8; 4],
}
impl Component for AIAttackComponent {
    const NAME: &'static str = "AIAttackComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AIAttackComponent {
    pub inherited_fields: ComponentData,
    pub use_probability: isize,
    pub min_distance: f32,
    pub max_distance: f32,
    pub angular_range_deg: f32,
    pub state_duration_frames: isize,
    pub frames_between: isize,
    pub frames_between_global: isize,
    pub animation_name: [u8; 24],
    pub attack_landing_ranged_enabled: bool,
    field10_0x7d: u8,
    field11_0x7e: u8,
    field12_0x7f: u8,
    pub attack_ranged_action_frame: isize,
    pub attack_ranged_offset_x: f32,
    pub attack_ranged_offset_y: f32,
    pub attack_ranged_root_offset_x: f32,
    pub attack_ranged_root_offset_y: f32,
    pub attack_ranged_use_message: bool,
    pub attack_ranged_predict: bool,
    field20_0x96: u8,
    field21_0x97: u8,
    pub attack_ranged_entity_file: [u8; 24],
    pub attack_ranged_entity_count_min: isize,
    pub attack_ranged_entity_count_max: isize,
    pub attack_ranged_use_laser_sight: bool,
    pub attack_ranged_aim_rotation_enabled: bool,
    field27_0xba: u8,
    field28_0xbb: u8,
    pub attack_ranged_aim_rotation_speed: f32,
    pub attack_ranged_aim_rotation_shooting_ok_angle_deg: f32,
    pub m_ranged_attack_current_aim_angle: f32,
    pub m_next_frame_usable: isize,
    field33_0xcc: u8,
    field34_0xcd: u8,
    field35_0xce: u8,
    field36_0xcf: u8,
}
impl Component for LimbBossComponent {
    const NAME: &'static str = "LimbBossComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LimbBossComponent {
    pub inherited_fields: ComponentData,
    pub state: isize,
    pub m_state_prev: isize,
    pub m_move_to_position_x: f32,
    pub m_move_to_position_y: f32,
}
impl Component for LightComponent {
    const NAME: &'static str = "LightComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LightComponent {
    pub inherited_fields: ComponentData,
    pub update_properties: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub radius: f32,
    field6_0x50: u8,
    field7_0x51: u8,
    field8_0x52: u8,
    field9_0x53: u8,
    field10_0x54: u8,
    field11_0x55: u8,
    field12_0x56: u8,
    field13_0x57: u8,
    field14_0x58: u8,
    field15_0x59: u8,
    field16_0x5a: u8,
    field17_0x5b: u8,
    pub offset_x: f32,
    pub offset_y: f32,
    pub fade_out_time: f32,
    pub blinking_freq: f32,
    pub m_alpha: f32,
    pub m_sprite: [*mut u8; 4],
    field24_0x74: u8,
    field25_0x75: u8,
    field26_0x76: u8,
    field27_0x77: u8,
}
impl Component for FogOfWarRadiusComponent {
    const NAME: &'static str = "FogOfWarRadiusComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct FogOfWarRadiusComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for PlayerStatsComponent {
    const NAME: &'static str = "PlayerStatsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PlayerStatsComponent {
    pub inherited_fields: ComponentData,
    pub lives: isize,
    pub max_hp: f32,
    pub speed: f32,
    field4_0x54: u8,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
}
impl Component for LaserEmitterComponent {
    const NAME: &'static str = "LaserEmitterComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LaserEmitterComponent {
    pub inherited_fields: ComponentData,
    pub laser: [u8; 52],
    pub is_emitting: bool,
    field3_0x7d: u8,
    field4_0x7e: u8,
    field5_0x7f: u8,
    pub emit_until_frame: isize,
    pub laser_angle_add_rad: f32,
}
impl Component for VerletWeaponComponent {
    const NAME: &'static str = "VerletWeaponComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct VerletWeaponComponent {
    pub inherited_fields: ComponentData,
    pub damage_radius: f32,
    pub physics_force_radius: f32,
    pub damage_min_step: f32,
    pub damage_max: f32,
    pub damage_coeff: f32,
    pub impulse_coeff: f32,
    pub fade_duration_frames: isize,
    pub physics_impulse_coeff: f32,
    pub m_player_cooldown_end: isize,
    field10_0x6c: u8,
    field11_0x6d: u8,
    field12_0x6e: u8,
    field13_0x6f: u8,
}
impl Component for ElectricChargeComponent {
    const NAME: &'static str = "ElectricChargeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ElectricChargeComponent {
    pub inherited_fields: ComponentData,
    pub charge_time_frames: isize,
    pub fx_velocity_max: f32,
    pub electricity_emission_isizeerval_frames: isize,
    pub fx_emission_isizeerval_min: isize,
    pub fx_emission_isizeerval_max: isize,
    pub charge: isize,
}
impl Component for ConsumableTeleportComponent {
    const NAME: &'static str = "ConsumableTeleportComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ConsumableTeleportComponent {
    pub inherited_fields: ComponentData,
    pub create_other_end: bool,
    pub is_at_home: bool,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub collision_radius: f32,
    pub target_location: [u8; 8],
    pub target_id: usize,
    field8_0x5c: u8,
    field9_0x5d: u8,
    field10_0x5e: u8,
    field11_0x5f: u8,
    pub m_next_usable_frame: isize,
    pub m_has_other_end: bool,
    field14_0x65: u8,
    field15_0x66: u8,
    field16_0x67: u8,
}
impl Component for IKLimbAttackerComponent {
    const NAME: &'static str = "IKLimbAttackerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct IKLimbAttackerComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub leg_velocity_coeff: f32,
    pub targeting_radius: f32,
    pub targeting_raytrace: bool,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub target_entities_with_tag: [u8; 24],
    pub m_target: [u8; 8],
    pub m_target_entity: isize,
    pub m_state: [u8; 4],
    pub m_state_timer: f32,
    field13_0x84: u8,
    field14_0x85: u8,
    field15_0x86: u8,
    field16_0x87: u8,
}
impl Component for SetStartVelocityComponent {
    const NAME: &'static str = "SetStartVelocityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SetStartVelocityComponent {
    pub inherited_fields: ComponentData,
    pub velocity: [u8; 8],
    pub randomize_angle: [u8; 8],
    pub randomize_speed: [u8; 8],
}
impl Component for ElectricityComponent {
    const NAME: &'static str = "ElectricityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ElectricityComponent {
    pub inherited_fields: ComponentData,
    pub energy: isize,
    pub probability_to_heat: f32,
    pub speed: isize,
    pub splittings_min: isize,
    pub splittings_max: isize,
    pub splitting_energy_min: isize,
    pub splitting_energy_max: isize,
    pub hack_is_material_crack: bool,
    pub hack_crack_ice: bool,
    pub hack_is_set_fire: bool,
    field11_0x67: u8,
    pub m_splittings_left: isize,
    pub m_splitting_energy: isize,
    pub m_avg_dir: [u8; 8],
    pub m_prev_pos: [u8; 8],
    pub m_prev_material: isize,
    pub m_should_play_sound: bool,
    field18_0x85: u8,
    field19_0x86: u8,
    field20_0x87: u8,
}
impl Component for GunComponent {
    const NAME: &'static str = "GunComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GunComponent {
    pub inherited_fields: ComponentData,
    pub m_lua_manager: [*mut u8; 4],
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for PotionComponent {
    const NAME: &'static str = "PotionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PotionComponent {
    pub inherited_fields: ComponentData,
    pub spray_velocity_coeff: f32,
    pub spray_velocity_normalized_min: f32,
    pub body_colored: bool,
    pub throw_bunch: bool,
    field5_0x52: u8,
    field6_0x53: u8,
    pub throw_how_many: isize,
    pub dont_spray_static_materials: bool,
    pub dont_spray_just_leak_gas_materials: bool,
    pub never_color: bool,
    field11_0x5b: u8,
    pub custom_color_material: isize,
}
impl Component for TorchComponent {
    const NAME: &'static str = "TorchComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct TorchComponent {
    pub inherited_fields: ComponentData,
    pub probability_of_ignition_attempt: isize,
    pub suffocation_check_offset_y: f32,
    pub frames_suffocated_to_extinguish: isize,
    pub extinguishable: bool,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub fire_audio_weight: f32,
    pub m_flicker_offset: f32,
    pub m_frames_suffocated: isize,
    pub m_is_on: bool,
    pub m_fire_is_burning_prev: bool,
    field13_0x66: u8,
    field14_0x67: u8,
}
impl Component for SineWaveComponent {
    const NAME: &'static str = "SineWaveComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SineWaveComponent {
    pub inherited_fields: ComponentData,
    pub sinewave_freq: f32,
    pub sinewave_m: f32,
    pub lifetime: isize,
    field4_0x54: u8,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
}
impl Component for IKLimbsAnimatorComponent {
    const NAME: &'static str = "IKLimbsAnimatorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct IKLimbsAnimatorComponent {
    pub inherited_fields: ComponentData,
    pub future_state_samples: isize,
    pub ground_attachment_ray_length_coeff: f32,
    pub leg_velocity_coeff: f32,
    pub affect_flying: bool,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub large_movement_penalty_coeff: f32,
    pub no_ground_attachment_penalty_coeff: f32,
    pub ray_skip_material: isize,
    pub is_limp: bool,
    field12_0x65: u8,
    field13_0x66: u8,
    field14_0x67: u8,
    pub m_prev_body_position: [u8; 8],
    pub m_limb_states: [u8; 480],
    pub m_has_ground_attachment_on_any_leg: bool,
    field18_0x251: u8,
    field19_0x252: u8,
    field20_0x253: u8,
    field21_0x254: u8,
    field22_0x255: u8,
    field23_0x256: u8,
    field24_0x257: u8,
}
impl Component for CardinalMovementComponent {
    const NAME: &'static str = "CardinalMovementComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CardinalMovementComponent {
    pub inherited_fields: ComponentData,
    pub horizontal_movement: bool,
    pub vertical_movement: bool,
    pub isizeercardinal_movement: bool,
    field4_0x4b: u8,
    pub m_prev_pos: [u8; 8],
    field6_0x54: u8,
    field7_0x55: u8,
    field8_0x56: u8,
    field9_0x57: u8,
}
impl Component for WormComponent {
    const NAME: &'static str = "WormComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WormComponent {
    pub inherited_fields: ComponentData,
    pub speed: f32,
    pub acceleration: f32,
    pub ground_decceleration: [u8; 12],
    pub gravity: f32,
    pub tail_gravity: f32,
    pub part_distance: f32,
    pub ground_check_offset: isize,
    pub hitbox_radius: f32,
    pub bite_damage: f32,
    pub target_kill_radius: f32,
    pub target_kill_ragdoll_force: f32,
    pub jump_cam_shake: f32,
    pub jump_cam_shake_distance: f32,
    pub eat_anim_wait_mult: f32,
    pub ragdoll_filename: [u8; 24],
    pub is_water_worm: bool,
    field17_0xa1: u8,
    field18_0xa2: u8,
    field19_0xa3: u8,
    pub max_speed: f32,
    pub m_target_vec: [u8; 8],
    pub m_grav_velocity: f32,
    pub m_speed: f32,
    pub m_target_position: [u8; 8],
    pub m_target_speed: f32,
    pub m_on_ground_prev: bool,
    field27_0xc5: u8,
    field28_0xc6: u8,
    field29_0xc7: u8,
    pub m_material_id_prev: isize,
    pub m_frame_next_damage: isize,
    pub m_direction_adjust_speed: f32,
    pub m_prev_positions: [u8; 160],
    field34_0x174: u8,
    field35_0x175: u8,
    field36_0x176: u8,
    field37_0x177: u8,
}
impl Component for MusicEnergyAffectorComponent {
    const NAME: &'static str = "MusicEnergyAffectorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MusicEnergyAffectorComponent {
    pub inherited_fields: ComponentData,
    pub energy_target: f32,
    pub fade_range: f32,
    pub trigger_danger_music: bool,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub fog_of_war_threshold: isize,
    pub is_enemy: bool,
    field9_0x59: u8,
    field10_0x5a: u8,
    field11_0x5b: u8,
    pub energy_lerp_up_speed_multiplier: f32,
}
impl Component for ExplosionComponent {
    const NAME: &'static str = "ExplosionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ExplosionComponent {
    pub inherited_fields: ComponentData,
    pub trigger: [u8; 4],
    pub config_explosion: [u8; 372],
    pub timeout_frames: isize,
    pub timeout_frames_random: isize,
    pub kill_entity: bool,
    field6_0x1c9: u8,
    field7_0x1ca: u8,
    field8_0x1cb: u8,
    pub m_timer_trigger_frame: isize,
}
impl Component for PhysicsThrowableComponent {
    const NAME: &'static str = "PhysicsThrowableComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsThrowableComponent {
    pub inherited_fields: ComponentData,
    pub throw_force_coeff: f32,
    pub max_throw_speed: f32,
    pub min_torque: f32,
    pub max_torque: f32,
    pub tip_check_offset_min: f32,
    pub tip_check_offset_max: f32,
    pub tip_check_random_rotation_deg: f32,
    pub attach_min_speed: f32,
    pub attach_to_surfaces_knife_style: bool,
    field10_0x69: u8,
    field11_0x6a: u8,
    field12_0x6b: u8,
    field13_0x6c: u8,
    field14_0x6d: u8,
    field15_0x6e: u8,
    field16_0x6f: u8,
    pub m_has_joisize: bool,
    field18_0x71: u8,
    field19_0x72: u8,
    field20_0x73: u8,
    field21_0x74: u8,
    field22_0x75: u8,
    field23_0x76: u8,
    field24_0x77: u8,
}
impl Component for WormAIComponent {
    const NAME: &'static str = "WormAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WormAIComponent {
    pub inherited_fields: ComponentData,
    pub speed: f32,
    pub speed_hunt: f32,
    pub direction_adjust_speed: f32,
    pub direction_adjust_speed_hunt: f32,
    pub random_target_box_radius: f32,
    pub new_hunt_target_check_every: isize,
    pub new_random_target_check_every: isize,
    pub hunt_box_radius: f32,
    pub cocoon_food_required: isize,
    pub cocoon_entity: [u8; 24],
    pub give_up_area_radius: f32,
    pub give_up_time_frames: isize,
    pub m_random_target: [u8; 8],
    pub m_target_entity_id: isize,
    pub m_next_target_check_frame: isize,
    pub m_next_hunt_target_check_frame: isize,
    pub m_give_up_started: isize,
    pub m_give_up_area_min_x: isize,
    pub m_give_up_area_min_y: isize,
    pub m_give_up_area_max_x: isize,
    pub m_give_up_area_max_y: isize,
    pub debug_follow_mouse: bool,
    field23_0xb5: u8,
    field24_0xb6: u8,
    field25_0xb7: u8,
}
impl Component for FlyingComponent {
    const NAME: &'static str = "FlyingComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct FlyingComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub perlin_freq: f32,
    pub perlin_time_freq: f32,
    pub perlin_wind_x: f32,
    pub perlin_wind_y: f32,
    field9_0x5c: u8,
    field10_0x5d: u8,
    field11_0x5e: u8,
    field12_0x5f: u8,
}
impl Component for CharacterCollisionComponent {
    const NAME: &'static str = "CharacterCollisionComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct CharacterCollisionComponent {
    pub inherited_fields: ComponentData,
    pub getting_crushed_threshold: isize,
    pub moving_up_before_getting_crushed_threshold: isize,
    pub getting_crushed_counter: isize,
    pub stuck_in_ground_counter: isize,
    pub m_collided_horizontally: bool,
    field6_0x59: u8,
    field7_0x5a: u8,
    field8_0x5b: u8,
    field9_0x5c: u8,
    field10_0x5d: u8,
    field11_0x5e: u8,
    field12_0x5f: u8,
}
impl Component for IKLimbWalkerComponent {
    const NAME: &'static str = "IKLimbWalkerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct IKLimbWalkerComponent {
    pub inherited_fields: ComponentData,
    pub ground_attachment_min_spread: f32,
    pub ground_attachment_max_tries: isize,
    pub ground_attachment_max_angle: f32,
    pub ground_attachment_ray_length_coeff: f32,
    pub leg_velocity_coeff: f32,
    pub affect_flying: bool,
    field7_0x5d: u8,
    field8_0x5e: u8,
    field9_0x5f: u8,
    pub ray_skip_material: isize,
    pub m_target: [u8; 8],
    pub m_prev_target: [u8; 8],
    pub m_prev_center_position: [u8; 8],
    pub m_state: isize,
}
impl Component for SpriteAnimatorComponent {
    const NAME: &'static str = "SpriteAnimatorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SpriteAnimatorComponent {
    pub inherited_fields: ComponentData,
    pub target_sprite_comp_name: [u8; 24],
    pub rotate_to_surface_normal: bool,
    field3_0x61: u8,
    field4_0x62: u8,
    field5_0x63: u8,
    pub m_states: [u8; 12],
    pub m_cached_target_sprite_tag: [u8; 32],
    pub m_send_on_finished_message_name: [u8; 24],
}
impl Component for ItemChestComponent {
    const NAME: &'static str = "ItemChestComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemChestComponent {
    pub inherited_fields: ComponentData,
    pub item_count_min: isize,
    pub item_count_max: isize,
    pub level: isize,
    pub enemy_drop: bool,
    field5_0x55: u8,
    field6_0x56: u8,
    field7_0x57: u8,
    pub actions: [u8; 24],
    pub action_uses_remaining: [u8; 24],
    pub other_entities_to_spawn: [u8; 24],
    pub m_seed: [u8; 4],
    field12_0xa4: u8,
    field13_0xa5: u8,
    field14_0xa6: u8,
    field15_0xa7: u8,
}
impl Component for FishAIComponent {
    const NAME: &'static str = "FishAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct FishAIComponent {
    pub inherited_fields: ComponentData,
    pub direction: isize,
    pub speed: f32,
    pub aabb_min: [u8; 8],
    pub aabb_max: [u8; 8],
    pub velocity: [u8; 8],
    pub stuck_counter: isize,
    pub m_last_check_pos: [u8; 8],
    field8_0x74: u8,
    field9_0x75: u8,
    field10_0x76: u8,
    field11_0x77: u8,
}
impl Component for MaterialAreaCheckerComponent {
    const NAME: &'static str = "MaterialAreaCheckerComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MaterialAreaCheckerComponent {
    pub inherited_fields: ComponentData,
    pub update_every_x_frame: isize,
    pub look_for_failure: bool,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub area_aabb: [u8; 16],
    pub material: isize,
    pub material2: isize,
    pub count_min: isize,
    pub always_check_fullness: bool,
    pub kill_after_message: bool,
    field12_0x6e: u8,
    field13_0x6f: u8,
    pub m_position: isize,
    pub m_last_frame_checked: isize,
}
impl Component for DamageNearbyEntitiesComponent {
    const NAME: &'static str = "DamageNearbyEntitiesComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DamageNearbyEntitiesComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub damage_min: f32,
    pub damage_max: f32,
    pub target_vec_max_len: f32,
    pub knockback_multiplier: f32,
    pub time_between_damaging: isize,
    pub damage_type: [u8; 4],
    pub damage_description: [u8; 24],
    pub target_tag: [u8; 24],
    pub ragdoll_fx: [u8; 4],
    pub m_velocity: [u8; 8],
    pub m_next_damage_frame: isize,
    field13_0xa4: u8,
    field14_0xa5: u8,
    field15_0xa6: u8,
    field16_0xa7: u8,
}
impl Component for AIComponent {
    const NAME: &'static str = "AIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AIComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for RotateTowardsComponent {
    const NAME: &'static str = "RotateTowardsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct RotateTowardsComponent {
    pub inherited_fields: ComponentData,
    pub entity_with_tag: [u8; 24],
}
impl Component for PhysicsBody2Component {
    const NAME: &'static str = "PhysicsBody2Component";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsBody2Component {
    pub inherited_fields: ComponentData,
    pub m_body: [*mut u8; 4],
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub m_body_id: *mut B2Object,
    field7_0x54: u8,
    field8_0x55: u8,
    field9_0x56: u8,
    field10_0x57: u8,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub allow_sleep: bool,
    pub fixed_rotation: bool,
    pub is_bullet: bool,
    pub is_static: bool,
    pub buoyancy: f32,
    pub hax_fix_going_through_ground: bool,
    pub hax_fix_going_through_sand: bool,
    pub hax_wait_till_pixel_scenes_loaded: bool,
    pub go_through_sand: bool,
    pub auto_clean: bool,
    pub force_add_update_areas: bool,
    pub update_entity_transform: bool,
    pub kill_entity_if_body_destroyed: bool,
    pub kill_entity_after_initialized: bool,
    pub manual_init: bool,
    pub destroy_body_if_entity_destroyed: bool,
    field29_0x73: u8,
    pub root_offset_x: f32,
    pub root_offset_y: f32,
    pub init_offset_x: f32,
    pub init_offset_y: f32,
    pub m_active_state: bool,
    field35_0x85: u8,
    field36_0x86: u8,
    field37_0x87: u8,
    pub m_local_position: [u8; 8],
    pub m_pixel_count_orig: usize,
    pub m_initialized: bool,
    field41_0x95: u8,
    field42_0x96: u8,
    field43_0x97: u8,
    pub m_pixel_count: usize,
    pub m_refreshed: bool,
    field46_0x9d: u8,
    field47_0x9e: u8,
    field48_0x9f: u8,
}
impl Component for PhysicsRagdollComponent {
    const NAME: &'static str = "PhysicsRagdollComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsRagdollComponent {
    pub inherited_fields: ComponentData,
    pub filename: [u8; 24],
    pub filenames: [u8; 24],
    pub offset_x: f32,
    pub offset_y: f32,
    pub bodies: [*mut u8; 4],
    field6_0x84: u8,
    field7_0x85: u8,
    field8_0x86: u8,
    field9_0x87: u8,
}
impl Component for VerletWorldJoisizeComponent {
    const NAME: &'static str = "VerletWorldJoisizeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct VerletWorldJoisizeComponent {
    pub inherited_fields: ComponentData,
    pub world_position: [u8; 8],
    pub verlet_poisize_index: isize,
    pub m_updated: bool,
    field4_0x55: u8,
    field5_0x56: u8,
    field6_0x57: u8,
    pub m_cell: [*mut u8; 4],
    field8_0x5c: u8,
    field9_0x5d: u8,
    field10_0x5e: u8,
    field11_0x5f: u8,
}
impl Component for PositionSeedComponent {
    const NAME: &'static str = "PositionSeedComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PositionSeedComponent {
    pub inherited_fields: ComponentData,
    pub pos_x: f32,
    pub pos_y: f32,
}
impl Component for GameAreaEffectComponent {
    const NAME: &'static str = "GameAreaEffectComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct GameAreaEffectComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub collide_with_tag: [u8; 24],
    pub frame_length: isize,
    pub game_effect_entitities: [u8; 12],
    pub m_entities_applied_out_to: [u8; 12],
    pub m_entities_applied_frame: [u8; 12],
    field7_0x8c: u8,
    field8_0x8d: u8,
    field9_0x8e: u8,
    field10_0x8f: u8,
}
impl Component for PressurePlateComponent {
    const NAME: &'static str = "PressurePlateComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PressurePlateComponent {
    pub inherited_fields: ComponentData,
    pub m_next_frame: isize,
    pub check_every_x_frames: isize,
    pub state: isize,
    pub aabb_min: [u8; 8],
    pub aabb_max: [u8; 8],
    pub material_percent: f32,
}
impl Component for SpriteStainsComponent {
    const NAME: &'static str = "SpriteStainsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SpriteStainsComponent {
    pub inherited_fields: ComponentData,
    pub sprite_id: isize,
    pub fade_stains_towards_srite_top: bool,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
    pub stain_shaken_drop_chance_multiplier: [u8; 12],
    pub m_data: [*mut u8; 4],
    pub m_texture_handle: [u8; 4],
    pub m_state: [u8; 4],
}
impl Component for PixelSpriteComponent {
    const NAME: &'static str = "PixelSpriteComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PixelSpriteComponent {
    pub inherited_fields: ComponentData,
    pub image_file: [u8; 24],
    pub anchor_x: isize,
    pub anchor_y: isize,
    pub material: [u8; 24],
    pub diggable: bool,
    pub clean_overlapping_pixels: bool,
    pub kill_when_sprite_dies: bool,
    pub create_box2d_bodies: bool,
    pub m_pixel_sprite: [*mut u8; 4],
}
impl Component for SimplePhysicsComponent {
    const NAME: &'static str = "SimplePhysicsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SimplePhysicsComponent {
    pub inherited_fields: ComponentData,
    pub can_go_up: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub m_old_position: [u8; 8],
    field6_0x54: u8,
    field7_0x55: u8,
    field8_0x56: u8,
    field9_0x57: u8,
}
impl Component for BookComponent {
    const NAME: &'static str = "BookComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct BookComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_y: f32,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
}
impl Component for StreamingKeepAliveComponent {
    const NAME: &'static str = "StreamingKeepAliveComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct StreamingKeepAliveComponent {
    pub inherited_fields: ComponentData,
    pub _t_e_m_p_t_e_m_p_y: f32,
    pub _t_e_m_p_t_e_m_p_t_e_m_p: f32,
}
impl Component for PhysicsBodyCollisionDamageComponent {
    const NAME: &'static str = "PhysicsBodyCollisionDamageComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsBodyCollisionDamageComponent {
    pub inherited_fields: ComponentData,
    pub speed_threshold: f32,
    pub damage_multiplier: f32,
}
impl Component for FogOfWarRemoverComponent {
    const NAME: &'static str = "FogOfWarRemoverComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct FogOfWarRemoverComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    field2_0x4c: u8,
    field3_0x4d: u8,
    field4_0x4e: u8,
    field5_0x4f: u8,
}
impl Component for VariableStorageComponent {
    const NAME: &'static str = "VariableStorageComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct VariableStorageComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
    pub value_string: [u8; 24],
    pub value_isize: isize,
    pub value_bool: bool,
    field28_0x7d: u8,
    field29_0x7e: u8,
    field30_0x7f: u8,
    pub value_f32: f32,
    field32_0x84: u8,
    field33_0x85: u8,
    field34_0x86: u8,
    field35_0x87: u8,
}
impl Component for ExplodeOnDamageComponent {
    const NAME: &'static str = "ExplodeOnDamageComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ExplodeOnDamageComponent {
    pub inherited_fields: ComponentData,
    pub explode_on_death_percent: f32,
    pub explode_on_damage_percent: f32,
    pub physics_body_modified_death_probability: f32,
    pub physics_body_destruction_required: f32,
    pub config_explosion: [u8; 372],
    pub m_done: bool,
    field7_0x1cd: u8,
    field8_0x1ce: u8,
    field9_0x1cf: u8,
}
impl Component for MagicConvertMaterialComponent {
    const NAME: &'static str = "MagicConvertMaterialComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MagicConvertMaterialComponent {
    pub inherited_fields: ComponentData,
    pub radius: isize,
    pub min_radius: isize,
    pub is_circle: bool,
    field4_0x51: u8,
    field5_0x52: u8,
    field6_0x53: u8,
    pub steps_per_frame: isize,
    pub from_material: isize,
    pub from_material_tag: [u8; 24],
    pub from_any_material: bool,
    field11_0x75: u8,
    field12_0x76: u8,
    field13_0x77: u8,
    pub to_material: isize,
    pub clean_stains: bool,
    pub extinguish_fire: bool,
    field17_0x7e: u8,
    field18_0x7f: u8,
    pub fan_the_flames: isize,
    pub temperature_reaction_temp: isize,
    pub ignite_materials: isize,
    field22_0x8c: u8,
    pub kill_when_finished: bool,
    pub convert_entities: bool,
    pub stain_frozen: bool,
    pub reaction_audio_amount: f32,
    pub convert_same_material: bool,
    field28_0x95: u8,
    field29_0x96: u8,
    field30_0x97: u8,
    pub from_material_array: [u8; 24],
    pub to_material_array: [u8; 24],
    pub m_use_arrays: bool,
    field34_0xc9: u8,
    field35_0xca: u8,
    field36_0xcb: u8,
    pub m_from_material_array: [u8; 12],
    pub m_to_material_array: [u8; 12],
    pub m_radius: isize,
}
impl Component for AdvancedFishAIComponent {
    const NAME: &'static str = "AdvancedFishAIComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AdvancedFishAIComponent {
    pub inherited_fields: ComponentData,
    pub move_check_range_min: f32,
    pub move_check_range_max: f32,
    pub flock: bool,
    pub avoid_predators: bool,
    pub m_has_target_direction: bool,
    field6_0x53: u8,
    pub m_target_pos: [u8; 8],
    pub m_target_vec: [u8; 8],
    pub m_last_frames_movement_area_min: [u8; 8],
    pub m_last_frames_movement_area_max: [u8; 8],
    pub m_num_failed_target_searches: usize,
    pub m_next_frame_check_are_we_stuck: isize,
    pub m_next_frame_check_flock_wants: isize,
    pub m_next_frame_predator_avoidance: isize,
    pub m_scared: f32,
    pub m_wants_to_be_in_flock: bool,
    field17_0x89: u8,
    field18_0x8a: u8,
    field19_0x8b: u8,
    field20_0x8c: u8,
    field21_0x8d: u8,
    field22_0x8e: u8,
    field23_0x8f: u8,
}
impl Component for ControlsComponent {
    const NAME: &'static str = "ControlsComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ControlsComponent {
    pub inherited_fields: ComponentData,
    pub m_button_down_fire: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub m_button_frame_fire: isize,
    pub m_button_last_frame_fire: isize,
    pub m_button_down_fire2: bool,
    field8_0x55: u8,
    field9_0x56: u8,
    field10_0x57: u8,
    pub m_button_frame_fire2: isize,
    pub m_button_down_action: bool,
    field13_0x5d: u8,
    field14_0x5e: u8,
    field15_0x5f: u8,
    pub m_button_frame_action: isize,
    pub m_button_down_throw: bool,
    field18_0x65: u8,
    field19_0x66: u8,
    field20_0x67: u8,
    pub m_button_frame_throw: isize,
    pub m_button_down_interact: bool,
    field23_0x6d: u8,
    field24_0x6e: u8,
    field25_0x6f: u8,
    pub m_button_frame_interact: isize,
    pub m_button_down_left: bool,
    field28_0x75: u8,
    field29_0x76: u8,
    field30_0x77: u8,
    pub m_button_frame_left: isize,
    pub m_button_down_right: bool,
    field33_0x7d: u8,
    field34_0x7e: u8,
    field35_0x7f: u8,
    pub m_button_frame_right: isize,
    pub m_button_down_up: bool,
    field38_0x85: u8,
    field39_0x86: u8,
    field40_0x87: u8,
    pub m_button_frame_up: isize,
    pub m_button_down_down: bool,
    field43_0x8d: u8,
    field44_0x8e: u8,
    field45_0x8f: u8,
    pub m_button_frame_down: isize,
    pub m_button_down_jump: bool,
    field48_0x95: u8,
    field49_0x96: u8,
    field50_0x97: u8,
    pub m_button_frame_jump: isize,
    pub m_button_down_run: bool,
    field53_0x9d: u8,
    field54_0x9e: u8,
    field55_0x9f: u8,
    pub m_button_frame_run: isize,
    pub m_button_down_fly: bool,
    field58_0xa5: u8,
    field59_0xa6: u8,
    field60_0xa7: u8,
    pub m_button_frame_fly: isize,
    pub m_button_down_dig: bool,
    field63_0xad: u8,
    field64_0xae: u8,
    field65_0xaf: u8,
    pub m_button_frame_dig: isize,
    pub m_button_down_change_item_r: bool,
    field68_0xb5: u8,
    field69_0xb6: u8,
    field70_0xb7: u8,
    pub m_button_frame_change_item_r: isize,
    pub m_button_count_change_item_r: isize,
    pub m_button_down_change_item_l: bool,
    field74_0xc1: u8,
    field75_0xc2: u8,
    field76_0xc3: u8,
    pub m_button_frame_change_item_l: isize,
    pub m_button_count_change_item_l: isize,
    pub m_button_down_inventory: bool,
    field80_0xcd: u8,
    field81_0xce: u8,
    field82_0xcf: u8,
    pub m_button_frame_inventory: isize,
    pub m_button_down_holster_item: bool,
    field85_0xd5: u8,
    field86_0xd6: u8,
    field87_0xd7: u8,
    pub m_button_frame_holster_item: isize,
    pub m_button_down_drop_item: bool,
    field90_0xdd: u8,
    field91_0xde: u8,
    field92_0xdf: u8,
    pub m_button_frame_drop_item: isize,
    pub m_button_down_kick: bool,
    field95_0xe5: u8,
    field96_0xe6: u8,
    field97_0xe7: u8,
    pub m_button_frame_kick: isize,
    pub m_button_down_eat: bool,
    field100_0xed: u8,
    field101_0xee: u8,
    field102_0xef: u8,
    pub m_button_frame_eat: isize,
    pub m_button_down_left_click: bool,
    field105_0xf5: u8,
    field106_0xf6: u8,
    field107_0xf7: u8,
    pub m_button_frame_left_click: isize,
    pub m_button_down_right_click: bool,
    field110_0xfd: u8,
    field111_0xfe: u8,
    field112_0xff: u8,
    pub m_button_frame_right_click: isize,
    pub m_button_down_transform_left: bool,
    field115_0x105: u8,
    field116_0x106: u8,
    field117_0x107: u8,
    pub m_button_frame_transform_left: isize,
    pub m_button_down_transform_right: bool,
    field120_0x10d: u8,
    field121_0x10e: u8,
    field122_0x10f: u8,
    pub m_button_frame_transform_right: isize,
    pub m_button_down_transform_up: bool,
    field125_0x115: u8,
    field126_0x116: u8,
    field127_0x117: u8,
    pub m_button_frame_transform_up: isize,
    pub m_button_count_transform_up: isize,
    pub m_button_down_transform_down: bool,
    field131_0x121: u8,
    field132_0x122: u8,
    field133_0x123: u8,
    pub m_button_frame_transform_down: isize,
    pub m_button_count_transform_down: isize,
    pub m_flying_target_y: f32,
    pub m_aiming_vector: [u8; 8],
    pub m_aiming_vector_normalized: [u8; 8],
    pub m_aiming_vector_non_zero_latest: [u8; 8],
    pub m_gamepad_aiming_vector_raw: [u8; 8],
    pub m_jump_velocity: [u8; 8],
    pub m_mouse_position: [u8; 8],
    pub m_mouse_position_raw: [u8; 8],
    pub m_mouse_position_raw_prev: [u8; 8],
    pub m_mouse_delta: [u8; 8],
    pub m_gamepad_indirect_aiming: [u8; 8],
    pub m_game_pad_cursor_in_world: [u8; 8],
    pub m_button_down_delay_line_fire: usize,
    pub m_button_down_delay_line_fire2: usize,
    pub m_button_down_delay_line_right: usize,
    pub m_button_down_delay_line_left: usize,
    pub m_button_down_delay_line_up: usize,
    pub m_button_down_delay_line_down: usize,
    pub m_button_down_delay_line_kick: usize,
    pub m_button_down_delay_line_throw: usize,
    pub m_button_down_delay_line_jump: usize,
    pub m_button_down_delay_line_fly: usize,
    pub polymorph_hax: bool,
    field159_0x1b1: u8,
    field160_0x1b2: u8,
    field161_0x1b3: u8,
    pub polymorph_next_attack_frame: isize,
    pub input_latency_frames: [u8; 12],
    pub enabled: bool,
    pub gamepad_indirect_aiming_enabled: bool,
    pub gamepad_fire_on_thumbstick_extend: bool,
    field167_0x1c7: u8,
    pub gamepad_fire_on_thumbstick_extend_threshold: f32,
    field169_0x1cc: u8,
    field170_0x1cd: u8,
    field171_0x1ce: u8,
    field172_0x1cf: u8,
}
impl Component for WorldStateComponent {
    const NAME: &'static str = "WorldStateComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct WorldStateComponent {
    pub inherited_fields: ComponentData,
    pub is_initialized: bool,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    pub time_total: f32,
    pub time_dt: f32,
    pub day_count: isize,
    field12_0x5c: u8,
    field13_0x5d: u8,
    field14_0x5e: u8,
    field15_0x5f: u8,
    pub rain_target: f32,
    field17_0x64: u8,
    field18_0x65: u8,
    field19_0x66: u8,
    field20_0x67: u8,
    pub fog_target: f32,
    pub isizero_weather: bool,
    field23_0x6d: u8,
    field24_0x6e: u8,
    field25_0x6f: u8,
    field26_0x70: u8,
    field27_0x71: u8,
    field28_0x72: u8,
    field29_0x73: u8,
    pub wind_speed: f32,
    pub wind_speed_sin_t: f32,
    pub wind_speed_sin: f32,
    pub clouds_01_target: f32,
    pub clouds_02_target: f32,
    pub gradient_sky_alpha_target: f32,
    pub sky_sunset_alpha_target: f32,
    pub lightning_count: isize,
    pub player_spawn_location: [u8; 8],
    pub lua_globals: [u8; 8],
    pub pending_portals: [u8; 12],
    pub next_portal_id: usize,
    pub apparitions_per_level: [u8; 12],
    pub npc_parties: [u8; 12],
    pub session_stat_file: [u8; 24],
    pub orbs_found_thisrun: [u8; 12],
    pub flags: [u8; 12],
    pub changed_materials: [u8; 12],
    pub player_polymorph_count: isize,
    pub player_polymorph_random_count: isize,
    pub player_did_infinite_spell_count: isize,
    pub player_did_damage_over_1milj: isize,
    pub player_living_with_minus_hp: isize,
    pub global_genome_relations_modifier: f32,
    pub mods_have_been_active_during_this_run: bool,
    pub twitch_has_been_active_during_this_run: bool,
    field56_0x122: u8,
    field57_0x123: u8,
    pub next_cut_through_world_id: usize,
    pub cuts_through_world: [u8; 12],
    pub gore_multiplier: [u8; 12],
    pub trick_kill_gold_multiplier: [u8; 12],
    pub damage_flash_multiplier: [u8; 12],
    pub open_fog_of_war_everywhere: [u8; 8],
    pub consume_actions: [u8; 8],
    pub perk_infinite_spells: bool,
    pub perk_trick_kills_blood_money: bool,
    field67_0x16a: u8,
    field68_0x16b: u8,
    pub perk_hp_drop_chance: isize,
    pub perk_gold_is_forever: bool,
    pub perk_rats_player_friendly: bool,
    pub _e_v_e_r_y_t_h_i_n_g_t_o_g_o_l_d: bool,
    field73_0x173: u8,
    pub material_everything_to_gold: [u8; 24],
    pub material_everything_to_gold_static: [u8; 24],
    pub _i_n_f_i_n_i_t_e_g_o_l_d_h_a_p_p_e_n_i_n_g: bool,
    pub _e_n_d_i_n_g_h_a_p_p_i_n_e_s_s_h_a_p_p_e_n_i_n_g: bool,
    field78_0x1a6: u8,
    field79_0x1a7: u8,
    pub _e_n_d_i_n_g_h_a_p_p_i_n_e_s_s_f_r_a_m_e_s: isize,
    pub _e_n_d_i_n_g_h_a_p_p_i_n_e_s_s: bool,
    field82_0x1ad: u8,
    field83_0x1ae: u8,
    field84_0x1af: u8,
    pub m_flash_alpha: f32,
    pub _d_e_b_u_g_l_o_a_d_e_d_f_r_o_m_a_u_t_o_s_a_v_e: isize,
    pub _d_e_b_u_g_l_o_a_d_e_d_f_r_o_m_o_l_d_v_e_r_s_i_o_n: isize,
    pub rain_target_extra: f32,
    pub fog_target_extra: f32,
    pub perk_rats_player_friendly_prev: bool,
    field91_0x1c5: u8,
    field92_0x1c6: u8,
    field93_0x1c7: u8,
}
impl Component for AudioComponent {
    const NAME: &'static str = "AudioComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AudioComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    field9_0x50: u8,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    field17_0x58: u8,
    field18_0x59: u8,
    field19_0x5a: u8,
    field20_0x5b: u8,
    field21_0x5c: u8,
    field22_0x5d: u8,
    field23_0x5e: u8,
    field24_0x5f: u8,
    pub event_root: [u8; 24],
    pub audio_physics_material: [u8; 24],
    pub set_latest_event_position: bool,
    pub remove_latest_event_on_destroyed: bool,
    pub send_message_on_event_dead: bool,
    pub play_only_if_visible: bool,
    pub m_audio_physics_material: isize,
    pub m_latest_source: [u8; 4],
    field33_0x9c: u8,
    field34_0x9d: u8,
    field35_0x9e: u8,
    field36_0x9f: u8,
}
impl Component for DrugEffectModifierComponent {
    const NAME: &'static str = "DrugEffectModifierComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct DrugEffectModifierComponent {
    pub inherited_fields: ComponentData,
    pub fx_add: [u8; 28],
    pub fx_multiply: [u8; 28],
}
impl Component for HitEffectComponent {
    const NAME: &'static str = "HitEffectComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct HitEffectComponent {
    pub inherited_fields: ComponentData,
    pub condition_effect: GameEffect,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub condition_status: [u8; 4],
    pub effect_hit: [u8; 4],
    pub value: isize,
    pub value_string: [u8; 24],
}
impl Component for AbilityComponent {
    const NAME: &'static str = "AbilityComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AbilityComponent {
    pub inherited_fields: ComponentData,
    pub cooldown_frames: isize,
    pub entity_file: [u8; 24],
    pub sprite_file: [u8; 24],
    pub entity_count: isize,
    pub never_reload: bool,
    field6_0x81: u8,
    field7_0x82: u8,
    field8_0x83: u8,
    pub reload_time_frames: isize,
    field10_0x88: u8,
    field11_0x89: u8,
    field12_0x8a: u8,
    field13_0x8b: u8,
    pub mana_max: f32,
    pub mana_charge_speed: f32,
    pub rotate_in_hand: bool,
    field17_0x95: u8,
    field18_0x96: u8,
    field19_0x97: u8,
    pub rotate_in_hand_amount: f32,
    pub rotate_hand_amount: f32,
    pub fast_projectile: bool,
    field23_0xa1: u8,
    field24_0xa2: u8,
    field25_0xa3: u8,
    pub swim_propel_amount: f32,
    pub max_charged_actions: isize,
    pub charge_wait_frames: isize,
    pub item_recoil_recovery_speed: f32,
    pub item_recoil_max: f32,
    pub item_recoil_offset_coeff: f32,
    pub item_recoil_rotation_coeff: f32,
    pub base_item_file: [u8; 24],
    pub use_entity_file_as_projectile_info_proxy: bool,
    pub click_to_use: bool,
    field36_0xda: u8,
    field37_0xdb: u8,
    pub stat_times_player_has_shot: isize,
    pub stat_times_player_has_edited: isize,
    pub shooting_reduces_amount_in_inventory: bool,
    pub throw_as_item: bool,
    pub simulate_throw_as_item: bool,
    field43_0xe7: u8,
    pub max_amount_in_inventory: isize,
    pub amount_in_inventory: isize,
    pub drop_as_item_on_death: bool,
    field47_0xf1: u8,
    field48_0xf2: u8,
    field49_0xf3: u8,
    pub ui_name: [u8; 24],
    pub use_gun_script: bool,
    pub is_petris_gun: bool,
    field53_0x10e: u8,
    field54_0x10f: u8,
    pub gun_config: [u8; 20],
    pub gunaction_config: [u8; 572],
    pub gun_level: isize,
    pub add_these_child_actions: [u8; 24],
    pub current_slot_durability: isize,
    pub slot_consumption_function: [u8; 24],
    pub m_next_frame_usable: isize,
    pub m_cast_delay_start_frame: isize,
    pub m_ammo_left: isize,
    pub m_reload_frames_left: isize,
    pub m_reload_next_frame_usable: isize,
    pub m_charge_count: isize,
    pub m_next_charge_frame: isize,
    pub m_item_recoil: f32,
    pub m_is_initialized: bool,
    field70_0x3b9: u8,
    field71_0x3ba: u8,
    field72_0x3bb: u8,
    field73_0x3bc: u8,
    field74_0x3bd: u8,
    field75_0x3be: u8,
    field76_0x3bf: u8,
}
impl Component for PhysicsJoisizeComponent {
    const NAME: &'static str = "PhysicsJoisizeComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsJoisizeComponent {
    pub inherited_fields: ComponentData,
    pub nail_to_wall: bool,
    pub grid_joisize: bool,
    pub breakable: bool,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    pub body1_id: isize,
    pub body2_id: isize,
    pub pos_x: f32,
    pub pos_y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub m_motor_enabled: bool,
    field16_0x69: u8,
    field17_0x6a: u8,
    field18_0x6b: u8,
    pub m_motor_speed: f32,
    pub m_max_motor_torque: f32,
    pub m_joisize: [*mut u8; 4],
}
impl Component for ItemCostComponent {
    const NAME: &'static str = "ItemCostComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ItemCostComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    field5_0x4c: u8,
    field6_0x4d: u8,
    field7_0x4e: u8,
    field8_0x4f: u8,
    pub stealable: bool,
    field10_0x51: u8,
    field11_0x52: u8,
    field12_0x53: u8,
    field13_0x54: u8,
    field14_0x55: u8,
    field15_0x56: u8,
    field16_0x57: u8,
    pub m_ex_cost: i64,
}
impl Component for PhysicsJoisize2MutatorComponent {
    const NAME: &'static str = "PhysicsJoisize2MutatorComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct PhysicsJoisize2MutatorComponent {
    pub inherited_fields: ComponentData,
    pub joisize_id: u16,
    pub destroy: bool,
    field3_0x4b: u8,
    pub motor_speed: f32,
    pub motor_max_torque: f32,
    field6_0x54: u8,
    field7_0x55: u8,
    field8_0x56: u8,
    field9_0x57: u8,
    pub m_box2_d_joisize_id: u64,
    pub m_previous_motor_speed: f32,
    pub m_previous_motor_max_torque: f32,
}
impl Component for AreaDamageComponent {
    const NAME: &'static str = "AreaDamageComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct AreaDamageComponent {
    pub inherited_fields: ComponentData,
    pub aabb_min: [u8; 8],
    pub aabb_max: [u8; 8],
    pub circle_radius: f32,
    pub damage_type: [u8; 4],
    pub damage_per_frame: f32,
    pub update_every_n_frame: isize,
    pub entity_responsible: isize,
    pub death_cause: [u8; 24],
    pub entities_with_tag: [u8; 24],
    field10_0x9c: u8,
    field11_0x9d: u8,
    field12_0x9e: u8,
    field13_0x9f: u8,
}
impl Component for SpriteComponent {
    const NAME: &'static str = "SpriteComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct SpriteComponent {
    pub inherited_fields: ComponentData,
    pub image_file: [u8; 24],
    pub ui_is_parent: bool,
    pub is_text_sprite: bool,
    field4_0x62: u8,
    field5_0x63: u8,
    pub offset_x: f32,
    pub offset_y: f32,
    pub transform_offset: [u8; 8],
    pub offset_animator_offset: [u8; 8],
    pub alpha: f32,
    pub visible: bool,
    pub emissive: bool,
    pub additive: bool,
    pub fog_of_war_hole: bool,
    pub smooth_filtering: bool,
    field16_0x85: u8,
    field17_0x86: u8,
    field18_0x87: u8,
    pub rect_animation: [u8; 24],
    pub next_rect_animation: [u8; 24],
    field21_0xb8: u8,
    field22_0xb9: u8,
    field23_0xba: u8,
    field24_0xbb: u8,
    field25_0xbc: u8,
    field26_0xbd: u8,
    field27_0xbe: u8,
    field28_0xbf: u8,
    field29_0xc0: u8,
    field30_0xc1: u8,
    field31_0xc2: u8,
    field32_0xc3: u8,
    field33_0xc4: u8,
    field34_0xc5: u8,
    field35_0xc6: u8,
    field36_0xc7: u8,
    field37_0xc8: u8,
    field38_0xc9: u8,
    field39_0xca: u8,
    field40_0xcb: u8,
    field41_0xcc: u8,
    field42_0xcd: u8,
    field43_0xce: u8,
    field44_0xcf: u8,
    pub z_index: f32,
    pub update_transform: bool,
    pub update_transform_rotation: bool,
    pub kill_entity_after_finished: bool,
    pub has_special_scale: bool,
    pub special_scale_x: f32,
    pub special_scale_y: f32,
    pub never_ragdollify_on_death: bool,
    field53_0xe1: u8,
    field54_0xe2: u8,
    field55_0xe3: u8,
    pub m_sprite: [*mut u8; 4],
    pub m_render_list: [*mut u8; 4],
    pub m_render_list_handle: isize,
}
impl Component for MaterialInventoryComponent {
    const NAME: &'static str = "MaterialInventoryComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct MaterialInventoryComponent {
    pub inherited_fields: ComponentData,
    pub drop_as_item: bool,
    pub on_death_spill: bool,
    pub leak_gently: bool,
    field4_0x4b: u8,
    pub leak_on_damage_percent: f32,
    pub leak_pressure_min: f32,
    pub leak_pressure_max: f32,
    pub min_damage_to_leak: f32,
    pub b2_force_on_leak: f32,
    pub death_throw_particle_velocity_coeff: f32,
    pub kill_when_empty: bool,
    pub halftime_materials: bool,
    field13_0x66: u8,
    field14_0x67: u8,
    pub do_reactions: isize,
    pub do_reactions_explosions: bool,
    pub do_reactions_entities: bool,
    field18_0x6e: u8,
    field19_0x6f: u8,
    pub reaction_speed: isize,
    pub reactions_shaking_speeds_up: bool,
    field22_0x75: u8,
    field23_0x76: u8,
    field24_0x77: u8,
    pub max_capacity: f64,
    pub count_per_material_type: [u8; 12],
    pub audio_collision_size_modifier_amount: f32,
    pub is_death_handled: bool,
    field29_0x91: u8,
    field30_0x92: u8,
    field31_0x93: u8,
    pub last_frame_drank: isize,
    pub ex_position: [u8; 8],
    pub ex_angle: f32,
    field35_0xa4: u8,
    field36_0xa5: u8,
    field37_0xa6: u8,
    field38_0xa7: u8,
}
impl Component for ArcComponent {
    const NAME: &'static str = "ArcComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct ArcComponent {
    pub inherited_fields: ComponentData,
    field1_0x48: u8,
    field2_0x49: u8,
    field3_0x4a: u8,
    field4_0x4b: u8,
    pub material: isize,
    pub lifetime: isize,
    pub m_arc_target: isize,
}
impl Component for LevitationComponent {
    const NAME: &'static str = "LevitationComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct LevitationComponent {
    pub inherited_fields: ComponentData,
    pub radius: f32,
    pub entity_force: f32,
    pub box2d_force: f32,
    pub effect_lifetime_frames: isize,
}
impl Component for BossDragonComponent {
    const NAME: &'static str = "BossDragonComponent";
}
#[derive(Debug)]
#[repr(C)]
pub struct BossDragonComponent {
    pub inherited_fields: ComponentData,
    pub speed: f32,
    pub speed_hunt: f32,
    pub acceleration: f32,
    pub direction_adjust_speed: f32,
    pub direction_adjust_speed_hunt: f32,
    pub gravity: f32,
    pub tail_gravity: f32,
    pub part_distance: f32,
    pub ground_check_offset: isize,
    pub eat_ground_radius: f32,
    pub eat_ground: bool,
    field12_0x71: u8,
    field13_0x72: u8,
    field14_0x73: u8,
    pub hitbox_radius: f32,
    pub bite_damage: f32,
    pub target_kill_radius: f32,
    pub target_kill_ragdoll_force: f32,
    pub hunt_box_radius: f32,
    pub random_target_box_radius: f32,
    pub new_hunt_target_check_every: isize,
    pub new_random_target_check_every: isize,
    pub jump_cam_shake: f32,
    pub jump_cam_shake_distance: f32,
    pub eat_anim_wait_mult: f32,
    pub projectile_1: [u8; 24],
    pub projectile_1_count: isize,
    pub projectile_2: [u8; 24],
    pub projectile_2_count: isize,
    pub ragdoll_filename: [u8; 24],
    pub m_target_entity_id: isize,
    pub m_target_vec: [u8; 8],
    pub m_grav_velocity: f32,
    pub m_speed: f32,
    pub m_random_target: [u8; 8],
    pub m_last_living_target_pos: [u8; 8],
    pub m_next_target_check_frame: isize,
    pub m_next_hunt_target_check_frame: isize,
    pub m_on_ground_prev: bool,
    field40_0x11d: u8,
    field41_0x11e: u8,
    field42_0x11f: u8,
    pub m_material_id_prev: isize,
    pub m_phase: isize,
    pub m_next_phase_switch_time: isize,
    pub m_part_distance: f32,
    pub m_is_initialized: bool,
    field48_0x131: u8,
    field49_0x132: u8,
    field50_0x133: u8,
    field51_0x134: u8,
    field52_0x135: u8,
    field53_0x136: u8,
    field54_0x137: u8,
}
