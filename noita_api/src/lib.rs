use crate::lua::{LuaGetValue, LuaPutValue};
use crate::serialize::deserialize_entity;
use base64::Engine;
use eyre::{Context, OptionExt, eyre};
use rustc_hash::{FxBuildHasher, FxHashMap};
use shared::des::Gid;
use shared::{GameEffectData, GameEffectEnum};
use smallvec::SmallVec;
use std::num::NonZeroIsize;
use std::sync::LazyLock;
use std::{
    borrow::Cow,
    num::{NonZero, TryFromIntError},
    ops::Deref,
};
pub mod lua;
pub mod serialize;
pub use noita_api_macro::add_lua_fn;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityID(pub NonZero<isize>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentID(pub NonZero<isize>);

pub struct Obj(pub usize);

pub struct Color(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsBodyID(pub i32);

pub trait Component: From<ComponentID> + Into<ComponentID> + Deref<Target = ComponentID> {
    const NAME_STR: &'static str;
}

noita_api_macro::generate_components!();

static TO_ID: LazyLock<Vec<String>> = LazyLock::new(|| {
    let file = raw::mod_text_file_get_content("data/scripts/status_effects/status_list.lua".into())
        .unwrap();
    file.lines()
        .filter(|l| !l.starts_with("-") && l.contains("id=\"") && !l.contains("BRAIN_DAMAGE"))
        .map(|l| l.split("\"").map(|a| a.to_string()).nth(1).unwrap())
        .collect::<Vec<String>>()
});
impl GameEffectComponent {
    pub fn m_serialized_data(self) -> eyre::Result<Cow<'static, str>> {
        raw::component_get_value::<Cow<str>>(*self, "mSerializedData")
    }
}
impl TelekinesisComponent {
    pub fn set_m_state(self, value: isize) -> eyre::Result<()> {
        raw::component_set_value(*self, "mState", value)
    }
}
impl EntityID {
    /// Returns true if entity is alive.
    ///
    /// Corresponds to EntityGetIsAlive from lua api.
    pub fn is_alive(self) -> bool {
        raw::entity_get_is_alive(self).unwrap_or(false)
    }

    pub fn name(self) -> eyre::Result<Cow<'static, str>> {
        raw::entity_get_name(self)
    }

    pub fn handle_poly(&self) -> eyre::Result<Option<Gid>> {
        for ent in self.children(None) {
            if let Ok(Some(effect)) =
                ent.try_get_first_component_including_disabled::<GameEffectComponent>(None)
            {
                let name = effect.effect()?;
                match name {
                    GameEffectEnum::Polymorph
                    | GameEffectEnum::PolymorphRandom
                    | GameEffectEnum::PolymorphUnstable
                    | GameEffectEnum::PolymorphCessation => {
                        if let Ok(data) = effect.m_serialized_data() {
                            if data.is_empty() {
                                return Ok(None);
                            }
                            if let Ok(data) =
                                base64::engine::general_purpose::STANDARD.decode(data.as_bytes())
                            {
                                let data = unsafe { str::from_utf8_unchecked(&data) };
                                if let Some((_, data)) = data.split_once("ew_gid_lid") {
                                    let mut gid = String::new();
                                    let mut found = false;
                                    for c in data.chars() {
                                        if c.is_numeric() {
                                            found = true;
                                            gid.push(c)
                                        } else if found {
                                            break;
                                        }
                                    }
                                    return Ok(Some(Gid(gid.parse::<u64>()?)));
                                }
                            }
                        }
                        return Ok(None);
                    }
                    _ => {}
                }
            }
        }
        Ok(None)
    }

    pub fn add_tag(self, tag: impl AsRef<str>) -> eyre::Result<()> {
        raw::entity_add_tag(self, tag.as_ref().into())
    }

    /// Returns true if entity has a tag.
    ///
    /// Corresponds to EntityGetTag from lua api.
    pub fn has_tag(self, tag: impl AsRef<str>) -> bool {
        raw::entity_has_tag(self, tag.as_ref().into()).unwrap_or(false)
    }

    pub fn remove_tag(self, tag: impl AsRef<str>) -> eyre::Result<()> {
        raw::entity_remove_tag(self, tag.as_ref().into())
    }

    pub fn root(self) -> eyre::Result<Option<EntityID>> {
        raw::entity_get_root_entity(self)
    }

    pub fn check_all_phys_init(self) -> eyre::Result<bool> {
        for phys_c in self.iter_all_components_of_type::<PhysicsBody2Component>(None)? {
            if !phys_c.m_initialized()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn get_physics_body_ids(self) -> eyre::Result<Vec<PhysicsBodyID>> {
        raw::physics_body_id_get_from_entity(self, None)
    }

    pub fn set_static(self, is_static: bool) -> eyre::Result<()> {
        raw::physics_set_static(self, is_static)
    }

    pub fn create(name: Option<Cow<'_, str>>) -> eyre::Result<Self> {
        match raw::entity_create_new(name) {
            Ok(Some(n)) => Ok(n),
            Err(s) => Err(s),
            _ => Err(eyre!("ent not found")),
        }
    }

    pub fn kill(self) {
        // Shouldn't ever error.
        if self.is_alive()
            && self
                .try_get_first_component_including_disabled::<CellEaterComponent>(None)
                .ok()
                .map(|a| a.is_none())
                .unwrap_or(true)
            && self.check_all_phys_init().unwrap_or(false)
        {
            let body_id = self.get_physics_body_ids().unwrap_or_default();
            if !body_id.is_empty()
                && let Ok(iter) = EntityID::get_with_tag("ew_peer")
            {
                for com in iter
                    .filter_map(|e| {
                        e.try_get_first_component_including_disabled::<TelekinesisComponent>(None)
                            .ok()
                    })
                    .flatten()
                {
                    if body_id.contains(&com.get_body_id()) {
                        let _ = com.set_m_state(0);
                    }
                }
            }
        }
        let _ = raw::entity_kill(self);
    }
    pub fn get_with_tag(tag: &str) -> eyre::Result<impl Iterator<Item = EntityID>> {
        match raw::entity_get_with_tag(tag.into()) {
            Ok(v) => Ok(v.into_iter().flatten()),
            Err(s) => Err(s),
        }
    }

    pub fn set_position(self, x: f64, y: f64, r: Option<f64>) -> eyre::Result<()> {
        raw::entity_set_transform(self, x, Some(y), r, None, None)
    }

    pub fn set_rotation(self, r: f64) -> eyre::Result<()> {
        let (x, y) = self.position()?;
        raw::entity_set_transform(self, x, Some(y), Some(r), None, None)
    }

    pub fn transform(self) -> eyre::Result<(f64, f64, f64, f64, f64)> {
        raw::entity_get_transform(self)
    }

    pub fn position(self) -> eyre::Result<(f64, f64)> {
        let (x, y, _, _, _) = raw::entity_get_transform(self)?;
        Ok((x, y))
    }

    pub fn rotation(self) -> eyre::Result<f64> {
        let (_, _, r, _, _) = raw::entity_get_transform(self)?;
        Ok(r)
    }

    pub fn filename(self) -> eyre::Result<Cow<'static, str>> {
        raw::entity_get_filename(self)
    }

    pub fn parent(self) -> eyre::Result<EntityID> {
        Ok(raw::entity_get_parent(self)?.unwrap_or(self))
    }

    /// Returns the first component of this type if an entity has it.
    pub fn try_get_first_component<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<Option<C>> {
        raw::entity_get_first_component(self, C::NAME_STR.into(), tag)
            .map(|x| x.flatten().map(Into::into))
            .wrap_err_with(|| eyre!("Failed to get first component {} for {self:?}", C::NAME_STR))
    }

    pub fn try_get_first_component_including_disabled<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<Option<C>> {
        raw::entity_get_first_component_including_disabled(self, C::NAME_STR.into(), tag)
            .map(|x| x.flatten().map(Into::into))
            .wrap_err_with(|| eyre!("Failed to get first component {} for {self:?}", C::NAME_STR))
    }

    /// Returns the first component of this type if an entity has it.
    pub fn get_first_component<C: Component>(self, tag: Option<Cow<'_, str>>) -> eyre::Result<C> {
        self.try_get_first_component(tag)?
            .ok_or_else(|| eyre!("Entity {self:?} has no component {}", C::NAME_STR))
    }

    pub fn get_first_component_including_disabled<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<C> {
        self.try_get_first_component_including_disabled(tag)?
            .ok_or_else(|| eyre!("Entity {self:?} has no component {}", C::NAME_STR))
    }

    pub fn remove_all_components_of_type<C: Component>(
        self,
        tags: Option<Cow<str>>,
    ) -> eyre::Result<bool> {
        let mut is_some = false;
        while let Some(c) = self.try_get_first_component_including_disabled::<C>(tags.clone())? {
            is_some = true;
            raw::entity_remove_component(self, *c)?;
        }
        Ok(is_some)
    }

    pub fn iter_all_components_of_type<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<impl Iterator<Item = C>> {
        Ok(raw::entity_get_component(self, C::NAME_STR.into(), tag)?
            .unwrap_or_default()
            .into_iter()
            .filter_map(|x| x.map(C::from)))
    }

    pub fn iter_all_components_of_type_including_disabled<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<impl Iterator<Item = C>> {
        Ok(self
            .iter_all_components_of_type_including_disabled_raw::<C>(tag)?
            .into_iter()
            .filter_map(|x| x.map(C::from)))
    }

    pub fn iter_all_components_of_type_including_disabled_raw<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<Vec<Option<ComponentID>>> {
        Ok(
            raw::entity_get_component_including_disabled(self, C::NAME_STR.into(), tag)?
                .unwrap_or_default(),
        )
    }

    pub fn add_component<C: Component>(self) -> eyre::Result<C> {
        raw::entity_add_component::<C>(self)?.ok_or_eyre("Couldn't create a component")
    }

    pub fn get_var(self, name: &str) -> Option<VariableStorageComponent> {
        self.iter_all_components_of_type_including_disabled::<VariableStorageComponent>(None)
            .map(|mut i| i.find(|var| var.name().unwrap_or("".into()) == name))
            .unwrap_or(None)
    }

    pub fn get_var_or_default(self, name: &str) -> eyre::Result<VariableStorageComponent> {
        if let Some(var) = self.get_var(name) {
            Ok(var)
        } else {
            let var = self.add_component::<VariableStorageComponent>()?;
            var.set_name(name.into())?;
            Ok(var)
        }
    }

    pub fn add_lua_init_component<C: Component>(self, file: &str) -> eyre::Result<C> {
        raw::entity_add_lua_init_component::<C>(self, file)?
            .ok_or_eyre("Couldn't create a component")
    }

    pub fn load(
        filename: impl AsRef<str>,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
    ) -> eyre::Result<Self> {
        raw::entity_load(filename.as_ref().into(), pos_x, pos_y)?
            .ok_or_else(|| eyre!("Failed to spawn entity from filename {}", filename.as_ref()))
    }

    pub fn max_in_use() -> eyre::Result<Self> {
        Ok(Self::try_from(raw::entities_get_max_id()? as isize)?)
    }

    /// Returns id+1
    pub fn next(self) -> eyre::Result<Self> {
        Ok(Self(NonZero::try_from(isize::from(self.0) + 1)?))
    }

    pub fn raw(self) -> isize {
        isize::from(self.0)
    }

    pub fn children(self, tag: Option<Cow<'_, str>>) -> impl Iterator<Item = EntityID> {
        raw::entity_get_all_children(self, tag)
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .flatten()
    }

    pub fn get_game_effects(self) -> eyre::Result<Vec<(GameEffectData, EntityID)>> {
        let mut effects = Vec::new();
        let mut name_to_n: FxHashMap<Cow<'static, str>, i32> = FxHashMap::default();
        for ent in self.children(None) {
            if ent.has_tag("projectile") {
                if let Ok(data) = serialize::serialize_entity(ent) {
                    let n = ent.filename()?;
                    let num = name_to_n.entry(n.clone()).or_insert(0);
                    *num += 1;
                    effects.push((GameEffectData::Projectile((format!("{n}{num}"), data)), ent));
                }
            } else if let Ok(Some(effect)) =
                ent.try_get_first_component_including_disabled::<GameEffectComponent>(None)
            {
                let name = effect.effect()?;
                match name {
                    GameEffectEnum::Custom => {
                        if let Ok(file) = ent.filename()
                            && !file.is_empty()
                        {
                            effects.push((GameEffectData::Custom(file.to_string()), ent))
                        } /* else if let Ok(data) = serialize::serialize_entity(ent) {
                        let n = ent.filename().unwrap_or(String::new());
                        effects.push((GameEffectData::Projectile((n, data)), ent))
                        }
                        } else if let Ok(data) = serialize::serialize_entity(ent) {
                        let n = ent.filename().unwrap_or(String::new());
                        let num = name_to_n.entry(n.clone()).or_insert(0);
                         *num += 1;
                        effects.push((
                        GameEffectData::Projectile((format!("{}{}", n, num), data)),
                        ent,
                        ))
                        }*/
                    }
                    GameEffectEnum::Polymorph
                    | GameEffectEnum::PolymorphRandom
                    | GameEffectEnum::PolymorphUnstable
                    | GameEffectEnum::PolymorphCessation => {}
                    _ => effects.push((GameEffectData::Normal(name), ent)),
                }
            }
        }
        Ok(effects)
    }
    pub fn set_game_effects(self, game_effect: &[GameEffectData]) -> eyre::Result<()> {
        fn set_frames(ent: EntityID) -> eyre::Result<()> {
            if let Some(effect) =
                ent.try_get_first_component_including_disabled::<GameEffectComponent>(None)?
                && effect.frames()? >= 0
            {
                effect.set_frames(i32::MAX)?;
            }
            if let Some(life) =
                ent.try_get_first_component_including_disabled::<LifetimeComponent>(None)?
                && life.lifetime()? >= 0
            {
                life.set_lifetime(i32::MAX)?;
            }
            Ok(())
        }
        let local_effects = self.get_game_effects()?;
        for (i, (e1, ent)) in local_effects.iter().enumerate() {
            if let GameEffectData::Normal(e1) = e1
                && (*e1 == GameEffectEnum::Polymorph
                    || *e1 == GameEffectEnum::PolymorphRandom
                    || *e1 == GameEffectEnum::PolymorphUnstable
                    || *e1 == GameEffectEnum::PolymorphCessation)
            {
                ent.kill();
                continue;
            }
            for (j, (e2, _)) in local_effects.iter().enumerate() {
                if i < j && e1 == e2 {
                    ent.kill()
                }
            }
        }
        let local_effects = self.get_game_effects()?;
        for effect in game_effect {
            if let Some(ent) = local_effects
                .iter()
                .find_map(|(e, ent)| if e == effect { Some(ent) } else { None })
            {
                let _ = set_frames(*ent);
            } else {
                let ent = match effect {
                    GameEffectData::Normal(e) => {
                        let e: &str = e.into();
                        if let Ok(ent) = NonZero::try_from(
                            raw::get_game_effect_load_to(self, e.into(), true)
                                .unwrap_or_default()
                                .1 as isize,
                        ) {
                            EntityID(ent)
                        } else {
                            continue;
                        }
                    }
                    GameEffectData::Custom(file) => {
                        let (x, y) = self.position().unwrap_or_default();
                        if let Ok(ent) = EntityID::load(file, Some(x), Some(y)) {
                            self.add_child(ent);
                            ent
                        } else {
                            continue;
                        }
                    }
                    GameEffectData::Projectile((_, data)) => {
                        let (x, y) = self.position().unwrap_or_default();
                        if let Ok(ent) = deserialize_entity(data, x as f32, y as f32) {
                            self.add_child(ent);
                            ent
                        } else {
                            continue;
                        }
                    }
                };
                let _ = set_frames(ent);
            }
        }
        let local_effects = self.get_game_effects()?;
        for (effect, ent) in local_effects {
            if game_effect.iter().all(|e| *e != effect) {
                ent.kill()
            }
        }
        if let Ok(damage) = self.get_first_component::<DamageModelComponent>(None) {
            if game_effect
                .iter()
                .any(|e| e == &GameEffectData::Normal(GameEffectEnum::OnFire))
            {
                let _ = damage.set_m_fire_probability(100);
                let _ = damage.set_m_fire_probability(1600);
                let _ = damage.set_m_fire_probability(1600);
            } else {
                let _ = damage.set_m_fire_probability(0);
                let _ = damage.set_m_fire_probability(0);
                let _ = damage.set_m_fire_probability(0);
            }
        }
        Ok(())
    }
    pub fn tags(self) -> eyre::Result<Cow<'static, str>> {
        Ok(raw::entity_get_tags(self)?.unwrap_or_default())
    }
    pub fn inflict_damage(
        self,
        damage: f64,
        damage_type: DamageType,
        message: &str,
        entity: Option<EntityID>,
    ) -> eyre::Result<()> {
        raw::entity_inflict_damage(
            self.raw() as i32,
            damage,
            damage_type.to_str(),
            message.into(),
            "NONE".into(),
            0.0,
            0.0,
            entity.map(|e| e.raw() as i32),
            None,
            None,
            None,
        )
    }
    pub fn add_child(self, child: EntityID) {
        let _ = raw::entity_add_child(self.0.get() as i32, child.0.get() as i32);
    }
    pub fn get_current_stains(self) -> eyre::Result<u64> {
        let mut current = 0;
        if let Ok(Some(status)) = self.try_get_first_component::<StatusEffectDataComponent>(None) {
            for (i, v) in status.stain_effects()?.enumerate() {
                if v >= 0.15 {
                    current += 1 << i
                }
            }
        }
        Ok(current)
    }

    pub fn set_current_stains(self, current_stains: u64) -> eyre::Result<()> {
        if let Ok(Some(status)) = self.try_get_first_component::<StatusEffectDataComponent>(None) {
            for ((i, v), id) in status.stain_effects()?.enumerate().zip(TO_ID.iter()) {
                if v >= 0.15 && current_stains & (1 << i) == 0 {
                    self.remove_stain(id)?
                }
            }
        }
        Ok(())
    }

    pub fn set_components_with_tag_enabled(
        self,
        tag: Cow<'_, str>,
        enabled: bool,
    ) -> eyre::Result<()> {
        raw::entity_set_components_with_tag_enabled(self, tag, enabled)
    }

    pub fn set_component_enabled(self, com: ComponentID, enabled: bool) -> eyre::Result<()> {
        raw::entity_set_component_is_enabled(self, com, enabled)
    }

    pub fn remove_component(self, component_id: ComponentID) -> eyre::Result<()> {
        raw::entity_remove_component(self, component_id)
    }
    pub fn shoot_projectile(
        self,
        pos_x: f64,
        pos_y: f64,
        target_x: f64,
        target_y: f64,
        proj: EntityID,
    ) -> eyre::Result<()> {
        raw::game_shoot_projectile(
            self.raw() as i32,
            pos_x,
            pos_y,
            target_x,
            target_y,
            proj.raw() as i32,
            None,
            None,
        )
    }
    pub fn get_all_components(self) -> eyre::Result<Vec<i32>> {
        raw::entity_get_all_components(self)
    }
    pub fn get_hotspot(self, hotspot: &str) -> eyre::Result<(f64, f64)> {
        raw::entity_get_hotspot(self.raw() as i32, hotspot.into(), true, None)
    }
    pub fn get_closest_with_tag(x: f64, y: f64, tag: &str) -> eyre::Result<Self> {
        match raw::entity_get_closest_with_tag(x, y, tag.into()) {
            Ok(Some(n)) => Ok(n),
            Err(s) => Err(s),
            _ => Err(eyre!("ent not found")),
        }
    }
    pub fn get_in_radius_with_tag(
        x: f64,
        y: f64,
        r: f64,
        tag: &str,
    ) -> eyre::Result<Vec<Option<EntityID>>> {
        raw::entity_get_in_radius_with_tag(x, y, r, tag.into())
    }
    pub fn remove_stain(self, id: &str) -> eyre::Result<()> {
        raw::entity_remove_stain_status_effect(self.0.get() as i32, id.into(), None)
    }
}
impl PhysicsBodyID {
    pub fn set_transform(
        self,
        x: f64,
        y: f64,
        r: f64,
        vx: f64,
        vy: f64,
        av: f64,
    ) -> eyre::Result<()> {
        raw::physics_body_id_set_transform(self, x, y, r, vx, vy, av)
    }
    pub fn get_transform(self) -> eyre::Result<Option<PhysData>> {
        raw::physics_body_id_get_transform(self)
    }
}

pub enum DamageType {
    None,
    DamageMelee,
    DamageProjectile,
    DamageExplosion,
    DamageBite,
    DamageFire,
    DamageMaterial,
    DamageFall,
    DamageElectricity,
    DamageDrowning,
    DamagePhysicsBodyDamaged,
    DamageDrill,
    DamageSlice,
    DamageIce,
    DamageHealing,
    DamagePhysicsHit,
    DamageRadioActive,
    DamagePoison,
    DamageMaterialWithFlash,
    DamageOvereating,
    DamageCurse,
    DamageHoly,
}
impl DamageType {
    fn to_str(&self) -> Cow<'static, str> {
        match self {
            DamageType::None => "NONE".into(),
            DamageType::DamageMelee => "DAMAGE_MELEE".into(),
            DamageType::DamageProjectile => "DAMAGE_PROJECTILE".into(),
            DamageType::DamageExplosion => "DAMAGE_EXPLOSION".into(),
            DamageType::DamageBite => "DAMAGE_BITE".into(),
            DamageType::DamageFire => "DAMAGE_FIRE".into(),
            DamageType::DamageMaterial => "DAMAGE_MATERIAL".into(),
            DamageType::DamageFall => "DAMAGE_FALL".into(),
            DamageType::DamageElectricity => "DAMAGE_ELECTRICITY".into(),
            DamageType::DamageDrowning => "DAMAGE_DROWNING".into(),
            DamageType::DamagePhysicsBodyDamaged => "DAMAGE_PHYSICS_BODY_DAMAGED".into(),
            DamageType::DamageDrill => "DAMAGE_DRILL".into(),
            DamageType::DamageSlice => "DAMAGE_SLICE".into(),
            DamageType::DamageIce => "DAMAGE_ICE".into(),
            DamageType::DamageHealing => "DAMAGE_HEALING".into(),
            DamageType::DamagePhysicsHit => "DAMAGE_PHYSIICS_HIT".into(),
            DamageType::DamageRadioActive => "DAMAGE_RADIOACTIVE".into(),
            DamageType::DamagePoison => "DAMAGE_POISON".into(),
            DamageType::DamageMaterialWithFlash => "DAMAGE_MATERIAL_WITH_FLASH".into(),
            DamageType::DamageOvereating => "DAMAGE_OVEREATING".into(),
            DamageType::DamageCurse => "DAMAGE_CURSE".into(),
            DamageType::DamageHoly => "DAMAGE_HOLY".into(),
        }
    }
}

impl TryFrom<isize> for EntityID {
    type Error = TryFromIntError;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        NonZero::<isize>::try_from(value).map(Self)
    }
}

impl ComponentID {
    pub fn add_tag(self, tag: impl AsRef<str>) -> eyre::Result<()> {
        raw::component_add_tag(self, tag.as_ref().into())
    }

    pub fn has_tag(self, tag: impl AsRef<str>) -> bool {
        raw::component_has_tag(self, tag.as_ref().into()).unwrap_or(false)
    }

    pub fn remove_tag(self, tag: impl AsRef<str>) -> eyre::Result<()> {
        raw::component_remove_tag(self, tag.as_ref().into())
    }

    pub fn object_set_value<T>(self, object: &str, key: &str, value: T) -> eyre::Result<()>
    where
        T: LuaPutValue,
    {
        raw::component_object_set_value::<T>(self, object, key, value)?;
        Ok(())
    }

    pub fn object_get_value<T>(self, object: &str, key: &str) -> eyre::Result<T>
    where
        T: LuaGetValue,
    {
        raw::component_object_get_value::<T>(self, object, key)
    }
    pub fn get_type(self) -> eyre::Result<Cow<'static, str>> {
        raw::component_get_type_name(self)
    }
    pub fn is_enabled(self) -> eyre::Result<bool> {
        raw::component_get_is_enabled(self)
    }
    pub fn get_tags(self) -> eyre::Result<Cow<'static, str>> {
        match raw::component_get_tags(self) {
            Ok(Some(s)) => Ok(s),
            Ok(None) => Err(eyre!("no string found")),
            Err(s) => Err(s),
        }
    }
}

impl StatusEffectDataComponent {
    pub fn stain_effects(self) -> eyre::Result<impl Iterator<Item = f32>> {
        let v: Vec<f32> = raw::component_get_value(self.0, "stain_effects")?;
        let out = v.into_iter();
        Ok(out.skip(1))
    }
}

impl TelekinesisComponent {
    pub fn get_body_id(self) -> PhysicsBodyID {
        raw::component_get_value_old::<PhysicsBodyID>(*self, "mBodyID").unwrap_or(PhysicsBodyID(0))
    }
}

pub fn game_print(value: impl AsRef<str>) {
    let _ = raw::game_print(value.as_ref().into());
}

pub fn print(value: impl AsRef<str>) {
    let _ = raw::print(value.as_ref());
}

pub mod raw {
    use eyre::Context;
    use eyre::eyre;

    use super::{Color, ComponentID, EntityID, Obj, PhysData, PhysicsBodyID};
    use crate::Component;
    use crate::lua::LuaGetValue;
    use crate::lua::LuaPutValue;
    use std::borrow::Cow;
    use std::num::NonZero;

    use crate::lua::LuaState;

    noita_api_macro::generate_api!();

    pub(crate) fn print(value: &str) -> eyre::Result<()> {
        let lua = LuaState::current()?;
        lua.get_global(c"print");
        lua.push_string(value);
        lua.call(1, 0)
            .wrap_err("Failed to call ComponentGetValue2")?;
        Ok(())
    }
    pub(crate) fn component_get_value<T>(component: ComponentID, field: &str) -> eyre::Result<T>
    where
        T: LuaGetValue,
    {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentGetValue2");
        lua.push_integer(component.0.into());
        lua.push_string(field);
        lua.call(2, T::size_on_stack())
            .wrap_err("Failed to call ComponentGetValue2")?;
        let ret = T::get(lua, -1);
        lua.pop_last_n(T::size_on_stack());
        ret.wrap_err_with(|| eyre!("Getting {field} for {component:?}"))
    }

    pub(crate) fn component_get_value_old<T>(component: ComponentID, field: &str) -> eyre::Result<T>
    where
        T: LuaGetValue,
    {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentGetValue");
        lua.push_integer(component.0.into());
        lua.push_string(field);
        lua.call(2, T::size_on_stack())
            .wrap_err("Failed to call ComponentGetValue")?;
        let ret = T::get(lua, -1);
        lua.pop_last_n(T::size_on_stack());
        ret.wrap_err_with(|| eyre!("Getting {field} for {component:?}"))
    }

    pub(crate) fn component_object_get_value<T>(
        component: ComponentID,
        object: &str,
        field: &str,
    ) -> eyre::Result<T>
    where
        T: LuaGetValue,
    {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentObjectGetValue2");
        lua.push_integer(component.0.into());
        lua.push_string(object);
        lua.push_string(field);
        lua.call(3, T::size_on_stack())
            .wrap_err("Failed to call ComponentObjectGetValue2")?;
        let ret = T::get(lua, -1);
        lua.pop_last_n(T::size_on_stack());
        ret.wrap_err_with(|| eyre!("Getting {field} from {object} for {component:?}"))
    }

    pub(crate) fn component_set_value<T>(
        component: ComponentID,
        field: &str,
        value: T,
    ) -> eyre::Result<()>
    where
        T: LuaPutValue,
    {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentSetValue2");
        lua.push_integer(component.0.into());
        lua.push_string(field);
        value.put(lua);
        lua.call((2 + T::SIZE_ON_STACK).try_into()?, 0)
            .wrap_err("Failed to call ComponentSetValue2")?;
        Ok(())
    }

    pub(crate) fn component_object_set_value<T>(
        component: ComponentID,
        object: &str,
        field: &str,
        value: T,
    ) -> eyre::Result<()>
    where
        T: LuaPutValue,
    {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentObjectSetValue2");
        lua.push_integer(component.0.into());
        lua.push_string(object);
        lua.push_string(field);
        value.put(lua);
        lua.call((3 + T::SIZE_ON_STACK).try_into()?, 0)
            .wrap_err("Failed to call ComponentObjectSetValue2")?;
        Ok(())
    }

    pub fn physics_body_id_get_transform(body: PhysicsBodyID) -> eyre::Result<Option<PhysData>> {
        let lua = LuaState::current()?;
        lua.get_global(c"PhysicsBodyIDGetTransform");
        lua.push_integer(body.0 as isize);
        lua.call(1, 6)
            .wrap_err("Failed to call PhysicsBodyIDGetTransform")?;
        if lua.is_nil_or_none(-1) {
            Ok(None)
        } else {
            match LuaGetValue::get(lua, -1) {
                Ok(ret) => {
                    let ret: (f32, f32, f32, f32, f32, f32) = ret;
                    lua.pop_last_n(6);
                    Ok(Some(PhysData {
                        x: ret.0,
                        y: ret.1,
                        angle: ret.2,
                        vx: ret.3,
                        vy: ret.4,
                        av: ret.5,
                    }))
                }
                Err(_) => {
                    lua.pop_last_n(6);
                    Ok(None)
                }
            }
        }
    }

    pub fn entity_add_component<C: Component>(entity: EntityID) -> eyre::Result<Option<C>> {
        let lua = LuaState::current()?;
        lua.get_global(c"EntityAddComponent2");
        lua.push_integer(entity.raw());
        lua.push_string(C::NAME_STR);
        lua.call(2, 1)
            .wrap_err("Failed to call EntityAddComponent2")?;
        let c = lua.to_integer(-1);
        lua.pop_last_n(1);
        Ok(NonZero::new(c).map(ComponentID).map(C::from))
    }

    pub fn entity_add_lua_init_component<C: Component>(
        entity: EntityID,
        file: &str,
    ) -> eyre::Result<Option<C>> {
        let lua = LuaState::current()?;
        lua.get_global(c"EwextAddInitLuaComponent");
        lua.push_integer(entity.raw());
        lua.push_string(file);
        lua.call(2, 1)
            .wrap_err("Failed to call EntityAddComponent2")?;
        let c = lua.to_integer(-1);
        lua.pop_last_n(1);
        Ok(NonZero::new(c).map(ComponentID).map(C::from))
    }
}
pub struct PhysData {
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub vx: f32,
    pub vy: f32,
    pub av: f32,
}

#[derive(PartialEq)]
pub enum CachedTag {
    EwClient,
    PitcheckB,
    SeedD,
    BossWizard,
    CardAction,
    BossCentipede,
    BossCentipedeActive,
    DesTag,
    BossDragon,
    PolymorphableNOT,
    EggItem,
}
//const TAG_LEN: usize = 11;
impl CachedTag {
    /*const fn iter() -> [CachedTag; 1] {
        [CachedTag::EwClient]
    }*/
    const fn to_tag(&self) -> &'static str {
        match self {
            Self::EwClient => "ew_client",
            Self::PitcheckB => "pitcheck_b",
            Self::SeedD => "seed_d",
            Self::BossWizard => "boss_wizard",
            Self::CardAction => "card_action",
            Self::BossCentipede => "boss_centipede",
            Self::BossCentipedeActive => "boss_centipede_active",
            Self::DesTag => "ew_des",
            Self::BossDragon => "boss_dragon",
            Self::PolymorphableNOT => "polymorphable_NOT",
            Self::EggItem => "egg_item",
        }
    }
    pub const fn from_tag(s: &'static str) -> Self {
        match s.as_bytes() {
            b"ew_client" => Self::EwClient,
            b"pitcheck_b" => Self::PitcheckB,
            b"seed_d" => Self::SeedD,
            b"boss_wizard" => Self::BossWizard,
            b"card_action" => Self::CardAction,
            b"boss_centipede" => Self::BossCentipede,
            b"boss_centipede_active" => Self::BossCentipedeActive,
            b"ew_des" => Self::DesTag,
            b"boss_dragon" => Self::BossDragon,
            b"polymorphable_NOT" => Self::PolymorphableNOT,
            b"egg_item" => Self::EggItem,
            _ => unreachable!(),
        }
    }
}
#[derive(Hash, Eq, PartialEq)]
pub enum CachedComponent {
    PhysicsBody2Component,
    VariableStorageComponent,
    AnimalAIComponent,
    ItemCostComponent,
    LaserEmitterComponent,
    CharacterDataComponent,
    WormComponent,
    VelocityComponent,
    BossDragonComponent,
    LuaComponent,
    BossHealthBarComponent,
    DamageModelComponent,
    ItemComponent,
    StreamingKeepAliveComponent,
    SpriteComponent,
    CameraBoundComponent,
    GhostComponent,
    PhysicsBodyComponent,
    Inventory2Component,
    AIAttackComponent,
    IKLimbWalkerComponent,
    IKLimbAttackerComponent,
    IKLimbsAnimatorComponent,
    CharacterPlatformingComponent,
    PhysicsAIComponent,
    AdvancedFishAIComponent,
    LifetimeComponent,
    ExplodeOnDamageComponent,
    ItemPickUpperComponent,
    AudioComponent,
    AbilityComponent,
    StatusEffectDataComponent,
}
const COMP_LEN: usize = 32;
impl CachedComponent {
    const fn from_component<C: Component>() -> Self {
        match C::NAME_STR.as_bytes() {
            b"PhysicsBody2Component" => Self::PhysicsBody2Component,
            b"VariableStorageComponent" => Self::VariableStorageComponent,
            b"AnimalAIComponent" => Self::AnimalAIComponent,
            b"ItemCostComponent" => Self::ItemCostComponent,
            b"LaserEmitterComponent" => Self::LaserEmitterComponent,
            b"CharacterDataComponent" => Self::CharacterDataComponent,
            b"WormComponent" => Self::WormComponent,
            b"VelocityComponent" => Self::VelocityComponent,
            b"BossDragonComponent" => Self::BossDragonComponent,
            b"LuaComponent" => Self::LuaComponent,
            b"BossHealthBarComponent" => Self::BossHealthBarComponent,
            b"DamageModelComponent" => Self::DamageModelComponent,
            b"ItemComponent" => Self::ItemComponent,
            b"StreamingKeepAliveComponent" => Self::StreamingKeepAliveComponent,
            b"SpriteComponent" => Self::SpriteComponent,
            b"CameraBoundComponent" => Self::CameraBoundComponent,
            b"GhostComponent" => Self::GhostComponent,
            b"PhysicsBodyComponent" => Self::PhysicsBodyComponent,
            b"Inventory2Component" => Self::Inventory2Component,
            b"AIAttackComponent" => Self::AIAttackComponent,
            b"IKLimbWalkerComponent" => Self::IKLimbWalkerComponent,
            b"IKLimbAttackerComponent" => Self::IKLimbAttackerComponent,
            b"IKLimbsAnimatorComponent" => Self::IKLimbsAnimatorComponent,
            b"CharacterPlatformingComponent" => Self::CharacterPlatformingComponent,
            b"PhysicsAIComponent" => Self::PhysicsAIComponent,
            b"AdvancedFishAIComponent" => Self::AdvancedFishAIComponent,
            b"LifetimeComponent" => Self::LifetimeComponent,
            b"ExplodeOnDamageComponent" => Self::ExplodeOnDamageComponent,
            b"ItemPickUpperComponent" => Self::ItemPickUpperComponent,
            b"AudioComponent" => Self::AudioComponent,
            b"AbilityComponent" => Self::AbilityComponent,
            b"StatusEffectDataComponent" => Self::StatusEffectDataComponent,
            _ => unreachable!(),
        }
    }
    fn from_component_non_const(s: &str) -> Option<Self> {
        Some(match s {
            "PhysicsBody2Component" => Self::PhysicsBody2Component,
            "VariableStorageComponent" => Self::VariableStorageComponent,
            "AnimalAIComponent" => Self::AnimalAIComponent,
            "ItemCostComponent" => Self::ItemCostComponent,
            "LaserEmitterComponent" => Self::LaserEmitterComponent,
            "CharacterDataComponent" => Self::CharacterDataComponent,
            "WormComponent" => Self::WormComponent,
            "VelocityComponent" => Self::VelocityComponent,
            "BossDragonComponent" => Self::BossDragonComponent,
            "LuaComponent" => Self::LuaComponent,
            "BossHealthBarComponent" => Self::BossHealthBarComponent,
            "DamageModelComponent" => Self::DamageModelComponent,
            "ItemComponent" => Self::ItemComponent,
            "StreamingKeepAliveComponent" => Self::StreamingKeepAliveComponent,
            "SpriteComponent" => Self::SpriteComponent,
            "CameraBoundComponent" => Self::CameraBoundComponent,
            "GhostComponent" => Self::GhostComponent,
            "PhysicsBodyComponent" => Self::PhysicsBodyComponent,
            "Inventory2Component" => Self::Inventory2Component,
            "AIAttackComponent" => Self::AIAttackComponent,
            "IKLimbWalkerComponent" => Self::IKLimbWalkerComponent,
            "IKLimbAttackerComponent" => Self::IKLimbAttackerComponent,
            "IKLimbsAnimatorComponent" => Self::IKLimbsAnimatorComponent,
            "CharacterPlatformingComponent" => Self::CharacterPlatformingComponent,
            "PhysicsAIComponent" => Self::PhysicsAIComponent,
            "AdvancedFishAIComponent" => Self::AdvancedFishAIComponent,
            "LifetimeComponent" => Self::LifetimeComponent,
            "ExplodeOnDamageComponent" => Self::ExplodeOnDamageComponent,
            "ItemPickUpperComponent" => Self::ItemPickUpperComponent,
            "AudioComponent" => Self::AudioComponent,
            "AbilityComponent" => Self::AbilityComponent,
            "StatusEffectDataComponent" => Self::StatusEffectDataComponent,
            _ => return None,
        })
    }
}
#[derive(PartialEq, Copy, Clone)]
pub enum VarName {
    None,
    SunbabyEssencesList,
    Rolling,
    EwWasStealable,
    EwRng,
    ThrowTime,
    GhostId,
    EwGidLid,
    Active,
    EwHasStarted,
    Unknown,
}
impl VarName {
    pub const fn from_str(s: &str) -> Self {
        match s.as_bytes() {
            b"sunbaby_essences_list" => Self::SunbabyEssencesList,
            b"rolling" => Self::Rolling,
            b"ew_was_stealable" => Self::EwWasStealable,
            b"ew_rng" => Self::EwRng,
            b"throw_time" => Self::ThrowTime,
            b"ghost_id" => Self::GhostId,
            b"ew_gid_lid" => Self::EwGidLid,
            b"active" => Self::Active,
            b"ew_has_started" => Self::EwHasStarted,
            _ => unreachable!(),
        }
    }
    pub const fn from_str_non_const(s: &str) -> Self {
        match s.as_bytes() {
            b"sunbaby_essences_list" => Self::SunbabyEssencesList,
            b"rolling" => Self::Rolling,
            b"ew_was_stealable" => Self::EwWasStealable,
            b"ew_rng" => Self::EwRng,
            b"throw_time" => Self::ThrowTime,
            b"ghost_id" => Self::GhostId,
            b"ew_gid_lid" => Self::EwGidLid,
            b"active" => Self::Active,
            b"ew_has_started" => Self::EwHasStarted,
            _ if s.is_empty() => Self::None,
            _ => Self::Unknown,
        }
    }
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::SunbabyEssencesList => "sunbaby_essences_list",
            Self::Rolling => "rolling",
            Self::EwWasStealable => "ew_was_stealable",
            Self::EwRng => "ew_rng",
            Self::ThrowTime => "throw_time",
            Self::GhostId => "ghost_id",
            Self::EwGidLid => "ew_gid_lid",
            Self::Active => "active",
            Self::EwHasStarted => "ew_has_started",
            Self::Unknown => unreachable!(),
        }
    }
}
#[derive(PartialEq, Clone, Copy)]
pub enum ComponentTag {
    DisabledAtStart,
    SunbabySprite,
    EwSyncedVar,
    Disabled,
    Activate,
    EnabledAtStart,
    ShopCost,
    Character,
    EwDesLua,
    None,
}
impl ComponentTag {
    pub const fn from_str(s: &str) -> Self {
        match s.as_bytes() {
            b"disabled_at_start" => Self::DisabledAtStart,
            b"sunbaby_sprite" => Self::SunbabySprite,
            b"ew_synced_var" => Self::EwSyncedVar,
            b"disabled" => Self::Disabled,
            b"activate" => Self::Activate,
            b"enabled_at_start" => Self::EnabledAtStart,
            b"shop_cost" => Self::ShopCost,
            b"character" => Self::Character,
            b"ew_des_lua" => Self::EwDesLua,
            _ => unreachable!(),
        }
    }
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::DisabledAtStart => "disabled_at_start",
            Self::SunbabySprite => "sunbaby_sprite",
            Self::EwSyncedVar => "ew_synced_var",
            Self::Disabled => "disabled",
            Self::Activate => "activate",
            Self::EnabledAtStart => "enabled_at_start",
            Self::ShopCost => "shop_cost",
            Self::Character => "character",
            Self::EwDesLua => "ew_des_lua",
            Self::None => "",
        }
    }
}
//const COMP_TAG_LEN: usize = 9;
struct ComponentData {
    id: ComponentID,
    enabled: bool,
    name: VarName,
    tags: Tags<u16>, //[bool; COMP_TAG_LEN],
}

#[derive(Default)]
struct Tags<T: Default>(T);
impl Tags<u16> {
    fn get(&self, n: u16) -> bool {
        self.0 & (1 << n) != 0
    }
    fn set(&mut self, n: u16) {
        self.0 |= 1 << n
    }
    fn clear(&mut self, n: u16) {
        self.0 &= !(1 << n)
    }
}

impl ComponentData {
    fn new(id: ComponentID, is_var: bool) -> Self {
        let enabled = id.is_enabled().unwrap_or_default();
        let name = if is_var {
            VarName::from_str_non_const(
                &VariableStorageComponent::from(id)
                    .name()
                    .unwrap_or_default(),
            )
        } else {
            VarName::None
        };
        let ent_tags = format!(",{},", id.get_tags().unwrap_or_default());
        let mut tags = Tags(0);
        macro_rules! push_tag {
            ($($e: expr),*) => {
                $(
                    if ent_tags.contains(&format!(",{},",const {$e.to_str()})){
                        tags.set(const{$e as u16});
                    }
                )*
            };
        }
        push_tag!(
            ComponentTag::DisabledAtStart,
            ComponentTag::SunbabySprite,
            ComponentTag::EwSyncedVar,
            ComponentTag::Disabled,
            ComponentTag::Activate,
            ComponentTag::EnabledAtStart,
            ComponentTag::ShopCost,
            ComponentTag::Character,
            ComponentTag::EwDesLua
        );
        ComponentData {
            id,
            enabled,
            name,
            tags,
        }
    }
    fn new_with_name(id: ComponentID, name: VarName) -> Self {
        let mut c = Self::new(id, true);
        c.name = name;
        c
    }
}
const COMP_VEC_LEN: usize = 1;
#[derive(Default)]
struct EntityData {
    tags: Tags<u16>, //[bool; TAG_LEN],
    phys_init: bool,
    components: [SmallVec<[ComponentData; COMP_VEC_LEN]>; COMP_LEN],
}
pub struct EntityManager {
    cache: FxHashMap<EntityID, EntityData>,
    current_entity: EntityID,
    current_data: EntityData,
    has_ran: bool,
    pub files: Option<FxHashMap<Cow<'static, str>, Vec<String>>>,
    frame_num: i32,
    camera_pos: (f64, f64),
    use_cache: bool,
}
impl Default for EntityManager {
    fn default() -> Self {
        Self {
            cache: FxHashMap::with_capacity_and_hasher(1024, FxBuildHasher),
            current_entity: EntityID(NonZeroIsize::new(-1).unwrap()),
            current_data: Default::default(),
            has_ran: false,
            files: Some(FxHashMap::with_capacity_and_hasher(512, FxBuildHasher)),
            frame_num: -1,
            camera_pos: (0.0, 0.0),
            use_cache: false,
        }
    }
}
impl EntityData {
    fn new(ent: EntityID) -> eyre::Result<Self> {
        let ent_tags = format!(",{},", ent.tags()?,);
        let mut tags = Tags(0);
        macro_rules! push_tag {
            ($($e: expr),*) => {
                $(
                    if ent_tags.contains(&format!(",{},",const {$e.to_tag()})) {
                        tags.set(const{$e as u16});
                    }
                )*
            };
        }
        push_tag!(
            CachedTag::EwClient,
            CachedTag::PitcheckB,
            CachedTag::SeedD,
            CachedTag::BossWizard,
            CachedTag::CardAction,
            CachedTag::BossCentipede,
            CachedTag::BossCentipedeActive,
            CachedTag::DesTag,
            CachedTag::BossDragon,
            CachedTag::PolymorphableNOT,
            CachedTag::EggItem
        );
        let coms = ent.get_all_components()?;
        let mut components: [SmallVec<[ComponentData; COMP_VEC_LEN]>; COMP_LEN] =
            std::array::from_fn(|_| SmallVec::new());
        for c in coms {
            let c = ComponentID((c as isize).try_into()?);
            let name = c.get_type()?;
            if let Some(com) = CachedComponent::from_component_non_const(&name) {
                let is_var = com == CachedComponent::VariableStorageComponent;
                components[com as usize].push(ComponentData::new(c, is_var))
            }
        }
        Ok(EntityData {
            tags,
            components,
            phys_init: false,
        })
    }
    fn has_tag(&self, tag: CachedTag) -> bool {
        self.tags.get(tag as u16)
    }
    fn add_tag(&mut self, tag: CachedTag) {
        self.tags.set(tag as u16)
    }
    fn remove_tag(&mut self, tag: CachedTag) {
        self.tags.clear(tag as u16)
    }
}
impl EntityManager {
    pub fn set_cache(&mut self, cache: bool) {
        self.use_cache = cache;
    }
    pub fn init_frame_num(&mut self) -> eyre::Result<()> {
        if self.frame_num == -1 {
            self.frame_num = raw::game_get_frame_num()?;
        } else {
            self.frame_num += 1;
        }
        Ok(())
    }
    pub fn frame_num(&self) -> i32 {
        self.frame_num
    }
    pub fn init_pos(&mut self) -> eyre::Result<()> {
        self.camera_pos = raw::game_get_camera_pos()?;
        Ok(())
    }
    pub fn camera_pos(&self) -> (f64, f64) {
        self.camera_pos
    }
    pub fn set_current_entity(&mut self, ent: EntityID) -> eyre::Result<()> {
        if self.use_cache {
            self.current_entity = ent;
            return Ok(());
        }
        if self.current_entity == ent {
            return Ok(());
        }
        if self.has_ran {
            let old_ent = std::mem::replace(&mut self.current_entity, ent);
            let data = if let Some(data) = self.cache.remove(&ent) {
                data
            } else {
                EntityData::new(ent)?
            };
            let old_data = std::mem::replace(&mut self.current_data, data);
            self.cache.insert(old_ent, old_data);
        } else {
            self.current_entity = ent;
            self.current_data = EntityData::new(ent)?;
            self.has_ran = true;
        }
        Ok(())
    }
    #[inline]
    pub fn entity(&self) -> EntityID {
        self.current_entity
    }
    pub fn remove_current(&mut self) {
        self.has_ran = false;
    }
    pub fn remove_ent(&mut self, ent: &EntityID) {
        if &self.current_entity == ent || self.use_cache {
            self.has_ran = false;
        } else {
            self.cache.remove(ent);
        }
    }
    pub fn add_tag(&mut self, tag: CachedTag) -> eyre::Result<()> {
        self.current_entity.add_tag(tag.to_tag())?;
        if self.use_cache {
            return Ok(());
        }
        self.current_data.add_tag(tag);
        Ok(())
    }
    pub fn has_tag(&self, tag: CachedTag) -> bool {
        if self.use_cache {
            return self.entity().has_tag(tag.to_tag());
        }
        self.current_data.has_tag(tag)
    }
    pub fn remove_tag(&mut self, tag: CachedTag) -> eyre::Result<()> {
        self.current_entity.remove_tag(tag.to_tag())?;
        if self.use_cache {
            return Ok(());
        }
        self.current_data.remove_tag(tag);
        Ok(())
    }
    pub fn check_all_phys_init(&mut self) -> eyre::Result<bool> {
        if self.use_cache {
            return self.entity().check_all_phys_init();
        }
        if self.current_data.phys_init {
            return Ok(true);
        }
        for phys_c in self.iter_mut_all_components_of_type::<PhysicsBody2Component>() {
            if !PhysicsBody2Component::from(phys_c.id).m_initialized()? {
                return Ok(false);
            }
        }
        self.current_data.phys_init = true;
        Ok(true)
    }
    pub fn try_get_first_component<C: Component>(&self, tag: ComponentTag) -> Option<C> {
        if self.use_cache {
            return self
                .entity()
                .try_get_first_component::<C>(if matches!(tag, ComponentTag::None) {
                    None
                } else {
                    Some(tag.to_str().into())
                })
                .unwrap_or(None);
        }
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter()
            .find(|c| c.enabled && (tag == ComponentTag::None || c.tags.get(tag as u16)))
            .map(|com| C::from(com.id))
    }
    pub fn try_get_first_component_including_disabled<C: Component>(
        &self,
        tag: ComponentTag,
    ) -> Option<C> {
        if self.use_cache {
            return self
                .entity()
                .try_get_first_component_including_disabled::<C>(
                    if matches!(tag, ComponentTag::None) {
                        None
                    } else {
                        Some(tag.to_str().into())
                    },
                )
                .unwrap_or(None);
        }
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter()
            .find(|c| tag == ComponentTag::None || c.tags.get(tag as u16))
            .map(|c| C::from(c.id))
    }
    pub fn get_first_component<C: Component>(&self, tag: ComponentTag) -> eyre::Result<C> {
        if self.use_cache {
            return self
                .entity()
                .get_first_component::<C>(if matches!(tag, ComponentTag::None) {
                    None
                } else {
                    Some(tag.to_str().into())
                });
        }
        if let Some(coms) = self.current_data.components
            [const { CachedComponent::from_component::<C>() as usize }]
        .iter()
        .find(|c| c.enabled && (tag == ComponentTag::None || c.tags.get(tag as u16)))
        .map(|com| C::from(com.id))
        {
            Ok(coms)
        } else {
            Err(eyre!("no comp found"))
        }
    }
    pub fn get_first_component_including_disabled<C: Component>(
        &self,
        tag: ComponentTag,
    ) -> eyre::Result<C> {
        if self.use_cache {
            return self.entity().get_first_component_including_disabled::<C>(
                if matches!(tag, ComponentTag::None) {
                    None
                } else {
                    Some(tag.to_str().into())
                },
            );
        }
        if let Some(coms) = self.current_data.components
            [const { CachedComponent::from_component::<C>() as usize }]
        .iter()
        .find(|c| tag == ComponentTag::None || c.tags.get(tag as u16))
        {
            Ok(C::from(coms.id))
        } else {
            Err(eyre!("no comp found"))
        }
    }
    pub fn remove_all_components_of_type<C: Component>(
        &mut self,
        tags: ComponentTag,
    ) -> eyre::Result<bool> {
        if self.use_cache {
            return self.entity().remove_all_components_of_type::<C>(
                if matches!(tags, ComponentTag::None) {
                    None
                } else {
                    Some(tags.to_str().into())
                },
            );
        }
        let mut is_some = false;
        let vec = std::mem::take(
            &mut self.current_data.components[const { CachedComponent::from_component::<C>() as usize }],
        );
        for com in vec.into_iter() {
            if tags == ComponentTag::None || com.tags.get(tags as u16) {
                is_some = true;
                self.current_entity.remove_component(com.id)?;
            } else {
                self.current_data.components
                    [const { CachedComponent::from_component::<C>() as usize }].push(com);
            }
        }
        Ok(is_some)
    }
    pub fn iter_all_components_of_type<C: Component>(
        &self,
        tag: ComponentTag,
    ) -> impl Iterator<Item = C> {
        if self.use_cache {
            return self
                .entity()
                .iter_all_components_of_type::<C>(if matches!(tag, ComponentTag::None) {
                    None
                } else {
                    Some(tag.to_str().into())
                })
                .map(|i| i.collect::<Vec<C>>())
                .unwrap_or_default()
                .into_iter();
        }
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter()
            .filter(move |c| c.enabled && (tag == ComponentTag::None || c.tags.get(tag as u16)))
            .map(|c| C::from(c.id))
            .collect::<Vec<C>>() //TODO fix collect allocation
            .into_iter()
    }
    fn iter_mut_all_components_of_type<C: Component>(
        &mut self,
    ) -> impl Iterator<Item = &mut ComponentData> {
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter_mut()
            .filter(|c| c.enabled)
    }
    pub fn iter_all_components_of_type_including_disabled<C: Component>(
        &self,
        tag: ComponentTag,
    ) -> impl Iterator<Item = C> {
        if self.use_cache {
            return self
                .entity()
                .iter_all_components_of_type_including_disabled::<C>(
                    if matches!(tag, ComponentTag::None) {
                        None
                    } else {
                        Some(tag.to_str().into())
                    },
                )
                .map(|i| i.collect::<Vec<C>>())
                .unwrap_or_default()
                .into_iter();
        }
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter()
            .filter(move |c| tag == ComponentTag::None || c.tags.get(tag as u16))
            .map(|c| C::from(c.id))
            .collect::<Vec<C>>() //TODO fix collect allocation
            .into_iter()
    }
    fn iter_all_components_of_type_including_disabled_raw<C: Component>(
        &self,
    ) -> impl Iterator<Item = &ComponentData> {
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .iter()
    }
    pub fn add_component<C: Component>(&mut self) -> eyre::Result<C> {
        if self.use_cache {
            return self.entity().add_component();
        }
        let c = self.current_entity.add_component::<C>()?;
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .push(ComponentData::new(
                *c,
                C::NAME_STR == "VariableStorageComponent",
            ));
        Ok(c)
    }
    fn add_component_var<C: Component>(&mut self, name: VarName) -> eyre::Result<C> {
        let c = self.current_entity.add_component::<C>()?;
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .push(ComponentData::new_with_name(*c, name));
        Ok(c)
    }
    pub fn get_var(&self, name: VarName) -> Option<VariableStorageComponent> {
        if self.use_cache {
            return self.entity().get_var(name.to_str());
        }
        let mut i =
            self.iter_all_components_of_type_including_disabled_raw::<VariableStorageComponent>();
        i.find_map(|var| {
            if var.name == name {
                Some(VariableStorageComponent::from(var.id))
            } else {
                None
            }
        })
    }
    pub fn get_var_unknown(&self, name: &str) -> Option<VariableStorageComponent> {
        if self.use_cache {
            return self.entity().get_var(name);
        }
        let mut i =
            self.iter_all_components_of_type_including_disabled_raw::<VariableStorageComponent>();
        i.find_map(|var| {
            if var.name == VarName::Unknown {
                let var = VariableStorageComponent::from(var.id);
                if var.name().unwrap_or_default() == name {
                    Some(var)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
    pub fn get_var_or_default(&mut self, name: VarName) -> eyre::Result<VariableStorageComponent> {
        if self.use_cache {
            return self.entity().get_var_or_default(name.to_str());
        }
        if let Some(var) = self.get_var(name) {
            Ok(var)
        } else {
            let var = self.add_component_var::<VariableStorageComponent>(name)?;
            var.set_name(name.to_str().into())?;
            Ok(var)
        }
    }
    pub fn get_var_or_default_unknown(
        &mut self,
        name: &str,
    ) -> eyre::Result<VariableStorageComponent> {
        if self.use_cache {
            return self.entity().get_var_or_default(name);
        }
        if let Some(var) = self.get_var_unknown(name) {
            Ok(var)
        } else {
            let var = self.add_component_var::<VariableStorageComponent>(VarName::Unknown)?;
            var.set_name(name.into())?;
            Ok(var)
        }
    }
    pub fn add_lua_init_component<C: Component>(&mut self, file: &str) -> eyre::Result<C> {
        if self.use_cache {
            return self.entity().add_lua_init_component(file);
        }
        let c = self.current_entity.add_lua_init_component::<C>(file)?;
        self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
            .push(ComponentData::new(*c, false));
        Ok(c)
    }
    pub fn set_components_with_tag_enabled(
        &mut self,
        tag: ComponentTag,
        enabled: bool,
    ) -> eyre::Result<()> {
        if self.use_cache {
            return self
                .entity()
                .set_components_with_tag_enabled(tag.to_str().into(), enabled);
        }
        let mut some = false;
        for c in self.current_data.components.iter_mut().flatten() {
            if c.tags.get(tag as u16) {
                some = true;
                c.enabled = enabled
            }
        }
        if some {
            self.current_entity
                .set_components_with_tag_enabled(tag.to_str().into(), enabled)?
        }
        Ok(())
    }
    pub fn set_component_enabled<C: Component>(
        &mut self,
        com: C,
        enabled: bool,
    ) -> eyre::Result<()> {
        if self.use_cache {
            return self.entity().set_component_enabled(*com, enabled);
        }
        let id = *com;
        if let Some(n) = self.current_data.components
            [const { CachedComponent::from_component::<C>() as usize }]
        .iter_mut()
        .find(|c| c.id == id)
            && n.enabled != enabled
        {
            n.enabled = enabled;
            self.current_entity.set_component_enabled(id, enabled)?;
        }
        Ok(())
    }
    pub fn remove_component<C: Component>(&mut self, component: C) -> eyre::Result<()> {
        if self.use_cache {
            return self.entity().remove_component(*component);
        }
        let id = *component;
        if let Some(n) = self.current_data.components
            [const { CachedComponent::from_component::<C>() as usize }]
        .iter()
        .position(|c| c.id == id)
        {
            self.current_data.components[const { CachedComponent::from_component::<C>() as usize }]
                .remove(n);
        }
        self.current_entity.remove_component(id)
    }
    pub fn get_current_stains(&self) -> eyre::Result<u64> {
        if self.use_cache {
            return self.entity().get_current_stains();
        }
        let mut current = 0;
        if let Some(status) =
            self.try_get_first_component::<StatusEffectDataComponent>(ComponentTag::None)
        {
            for (i, v) in status.stain_effects()?.enumerate() {
                if v >= 0.15 {
                    current += 1 << i
                }
            }
        }
        Ok(current)
    }
    pub fn set_current_stains(&self, current_stains: u64) -> eyre::Result<()> {
        if self.use_cache {
            return self.entity().set_current_stains(current_stains);
        }
        if let Some(status) =
            self.try_get_first_component::<StatusEffectDataComponent>(ComponentTag::None)
        {
            for ((i, v), id) in status.stain_effects()?.enumerate().zip(TO_ID.iter()) {
                if v >= 0.15 && current_stains & (1 << i) == 0 {
                    self.current_entity.remove_stain(id)?
                }
            }
        }
        Ok(())
    }
}
pub fn get_file<'a>(
    files: &'a mut Option<FxHashMap<Cow<'static, str>, Vec<String>>>,
    file: Cow<'static, str>,
) -> eyre::Result<&'a [String]> {
    match files.as_mut().unwrap().entry(file) {
        std::collections::hash_map::Entry::Occupied(entry) => Ok(entry.into_mut()),
        std::collections::hash_map::Entry::Vacant(entry) => {
            let content = raw::mod_text_file_get_content(entry.key().clone())?;
            let mut split = content.split("name=\"");
            split.next();
            let split = split.map(|piece| piece.split_once("\"").unwrap().0.to_string());
            Ok(entry.insert(split.collect::<Vec<String>>()))
        }
    }
}
