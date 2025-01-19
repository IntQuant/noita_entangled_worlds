use crate::lua::{LuaGetValue, LuaPutValue};
use crate::serialize::deserialize_entity;
use eyre::{eyre, Context, OptionExt};
use shared::{GameEffectData, GameEffectEnum};
use std::collections::HashMap;
use std::{
    borrow::Cow,
    num::{NonZero, TryFromIntError},
    ops::Deref,
};
pub mod lua;
pub mod serialize;

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

impl EntityID {
    /// Returns true if entity is alive.
    ///
    /// Corresponds to EntityGetIsAlive from lua api.
    pub fn is_alive(self) -> bool {
        raw::entity_get_is_alive(self).unwrap_or(false)
    }

    pub fn name(self) -> eyre::Result<String> {
        raw::entity_get_name(self).map(|s| s.to_string())
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

    pub fn kill(self) {
        // Shouldn't ever error.
        let _ = raw::entity_kill(self);
    }

    pub fn set_position(self, x: f32, y: f32, r: Option<f32>) -> eyre::Result<()> {
        raw::entity_set_transform(
            self,
            x as f64,
            Some(y as f64),
            r.map(|a| a as f64),
            None,
            None,
        )
    }

    pub fn position(self) -> eyre::Result<(f32, f32)> {
        let (x, y, _, _, _) = raw::entity_get_transform(self)?;
        Ok((x as f32, y as f32))
    }

    pub fn rotation(self) -> eyre::Result<f32> {
        let (_, _, r, _, _) = raw::entity_get_transform(self)?;
        Ok(r as f32)
    }

    pub fn filename(self) -> eyre::Result<String> {
        raw::entity_get_filename(self).map(|x| x.to_string())
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

    pub fn remove_all_components_of_type<C: Component>(self) -> eyre::Result<()> {
        while let Some(c) = self.try_get_first_component::<C>(None)? {
            raw::entity_remove_component(self, c.into())?;
        }
        Ok(())
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
        Ok(
            raw::entity_get_component_including_disabled(self, C::NAME_STR.into(), tag)?
                .unwrap_or_default()
                .into_iter()
                .filter_map(|x| x.map(C::from)),
        )
    }

    pub fn add_component<C: Component>(self) -> eyre::Result<C> {
        raw::entity_add_component::<C>(self)?.ok_or_eyre("Couldn't create a component")
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

    pub fn children(self, tag: Option<Cow<'_, str>>) -> Vec<EntityID> {
        raw::entity_get_all_children(self, tag)
            .unwrap_or(None)
            .unwrap_or_default()
            .iter()
            .filter_map(|a| *a)
            .collect()
    }

    pub fn get_game_effects(self) -> Vec<(GameEffectData, EntityID)> {
        let mut effects = Vec::new();
        let mut name_to_n: HashMap<String, i32> = HashMap::default();
        for ent in self.children(None) {
            if ent.has_tag("projectile") {
                if let Ok(data) = serialize::serialize_entity(ent) {
                    let n = ent.filename().unwrap_or(String::new());
                    let num = name_to_n.entry(n.clone()).or_insert(0);
                    *num += 1;
                    effects.push((
                        GameEffectData::Projectile((format!("{}{}", n, num), data)),
                        ent,
                    ));
                }
            } else if let Ok(Some(effect)) =
                ent.try_get_first_component_including_disabled::<GameEffectComponent>(None)
            {
                let name = effect.effect().unwrap();

                if name == GameEffectEnum::Custom {
                    if let Ok(file) = ent.filename() {
                        if !file.is_empty() {
                            effects.push((GameEffectData::Custom(file), ent))
                        } else if let Ok(data) = serialize::serialize_entity(ent) {
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
                    }
                } else {
                    effects.push((GameEffectData::Normal(name), ent))
                }
            }
        }
        effects
    }

    pub fn set_game_effects(self, game_effect: &[GameEffectData]) {
        fn set_frames(ent: EntityID) -> eyre::Result<()> {
            if let Some(effect) =
                ent.try_get_first_component_including_disabled::<GameEffectComponent>(None)?
            {
                if effect.frames()? >= 0 {
                    effect.set_frames(i32::MAX)?;
                }
            }
            if let Some(life) =
                ent.try_get_first_component_including_disabled::<LifetimeComponent>(None)?
            {
                if life.lifetime()? >= 0 {
                    life.set_lifetime(i32::MAX)?;
                }
            }
            Ok(())
        }
        let local_effects = self.get_game_effects();
        for (i, (e1, ent)) in local_effects.iter().enumerate() {
            for (j, (e2, _)) in local_effects.iter().enumerate() {
                if i < j && e1 == e2 {
                    ent.kill()
                }
            }
        }
        let local_effects = self.get_game_effects();
        for effect in game_effect {
            if let Some(ent) =
                local_effects
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
                        if let Ok(Some(ent)) =
                            raw::entity_load(file.into(), Some(x as f64), Some(y as f64))
                        {
                            self.add_child(ent);
                            ent
                        } else {
                            continue;
                        }
                    }
                    GameEffectData::Projectile((_, data)) => {
                        let (x, y) = self.position().unwrap_or_default();
                        if let Ok(ent) = deserialize_entity(data, x, y) {
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
        let local_effects = self.get_game_effects();
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
    }
    pub fn add_child(self, child: EntityID) {
        let _ = raw::entity_add_child(self.0.get() as i32, child.0.get() as i32);
    }
    pub fn get_current_stains(self) -> eyre::Result<u64> {
        let mut current = 0;
        if let Ok(Some(status)) = self.try_get_first_component::<StatusEffectDataComponent>(None) {
            for (i, v) in status.stain_effects()?.iter().enumerate() {
                if *v >= 0.15 {
                    current += 1 << i
                }
            }
        }
        Ok(current)
    }

    pub fn set_current_stains(self, current_stains: u64) -> eyre::Result<()> {
        if let Ok(Some(status)) = self.try_get_first_component::<StatusEffectDataComponent>(None) {
            let file = raw::mod_text_file_get_content(
                "data/scripts/status_effects/status_list.lua".into(),
            )?;
            let to_id = file
                .lines()
                .filter(|l| {
                    !l.starts_with("-") && l.contains("id=\"") && !l.contains("BRAIN_DAMAGE")
                })
                .map(|l| {
                    l.split("\"")
                        .map(|a| a.to_string())
                        .collect::<Vec<String>>()[1]
                        .clone()
                })
                .collect::<Vec<String>>();
            for ((i, v), id) in status.stain_effects()?.iter().enumerate().zip(to_id) {
                if *v >= 0.15 && current_stains & (1 << i) == 0 {
                    raw::entity_remove_stain_status_effect(self.0.get() as i32, id.into(), None)?
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

    pub fn remove_component(self, component_id: ComponentID) -> eyre::Result<()> {
        raw::entity_remove_component(self, component_id)
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
}

impl StatusEffectDataComponent {
    pub fn stain_effects(self) -> eyre::Result<Vec<f32>> {
        let v: Vec<f32> = raw::component_get_value::<Vec<f32>>(self.0, "stain_effects")?;
        Ok(v[1..].to_vec())
    }
}

pub fn game_print(value: impl AsRef<str>) {
    let _ = raw::game_print(value.as_ref().into());
}

pub mod raw {
    use eyre::eyre;
    use eyre::Context;

    use super::{Color, ComponentID, EntityID, Obj, PhysData, PhysicsBodyID};
    use crate::lua::LuaGetValue;
    use crate::lua::LuaPutValue;
    use crate::Component;
    use std::borrow::Cow;
    use std::num::NonZero;

    use crate::lua::LuaState;

    noita_api_macro::generate_api!();

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
                Err(err) => {
                    lua.pop_last_n(6);
                    Err(err)
                }
            }
        }
    }

    pub fn entity_add_component<C: Component>(entity: EntityID) -> eyre::Result<Option<C>> {
        let lua = LuaState::current()?;
        lua.get_global(c"EntityAddComponent");
        lua.push_integer(entity.raw());
        lua.push_string(C::NAME_STR);
        lua.call(2, 1)
            .wrap_err("Failed to call EntityAddComponent")?;
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
