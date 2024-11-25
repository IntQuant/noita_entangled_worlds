use std::num::NonZero;

pub mod lua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityID(pub NonZero<isize>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentID(pub NonZero<isize>);

pub struct Obj(pub usize);

pub struct Color(pub u32);

noita_api_macro::generate_components!();

pub mod raw {
    use eyre::Ok;

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
        ret
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
