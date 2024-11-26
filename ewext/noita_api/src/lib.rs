use std::{borrow::Cow, num::NonZero};

use eyre::{eyre, Context};

pub mod lua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityID(pub NonZero<isize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentID(pub NonZero<isize>);

pub struct Obj(pub usize);

pub struct Color(pub u32);

pub trait Component: From<ComponentID> {
    const NAME_STR: &'static str;
}

noita_api_macro::generate_components!();

impl EntityID {
    pub fn try_get_first_component<C: Component>(
        self,
        tag: Option<Cow<'_, str>>,
    ) -> eyre::Result<Option<C>> {
        raw::entity_get_first_component(self, C::NAME_STR.into(), tag)
            .map(|x| x.flatten().map(Into::into))
            .wrap_err_with(|| eyre!("Failed to get first component {} for {self:?}", C::NAME_STR))
    }
    pub fn get_first_component<C: Component>(self, tag: Option<Cow<'_, str>>) -> eyre::Result<C> {
        self.try_get_first_component(tag)?
            .ok_or_else(|| eyre!("Entity {self:?} has no component {}", C::NAME_STR))
    }
}

pub mod raw {
    use eyre::eyre;
    use eyre::Context;

    use super::{Color, ComponentID, EntityID, Obj};
    use crate::lua::LuaGetValue;
    use crate::lua::LuaPutValue;
    use std::borrow::Cow;

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
        lua.call(2, T::size_on_stack());
        let ret = T::get(lua, -1);
        lua.pop_last_n(T::size_on_stack());
        ret.wrap_err_with(|| eyre!("Getting {field} for {component:?}"))
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
        lua.call(2 + T::size_on_stack(), 0);
        Ok(())
    }
}
