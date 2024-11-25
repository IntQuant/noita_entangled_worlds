pub mod lua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityID(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentID(pub isize);

pub struct Obj(pub usize);

pub struct Color(pub u32);

noita_api_macro::generate_components!();

pub mod raw {
    use super::{Color, ComponentID, EntityID, Obj};
    use crate::lua::LuaPutValue;
    use std::borrow::Cow;

    use crate::lua::LuaState;

    noita_api_macro::generate_api!();

    fn component_get_value_base(
        component: ComponentID,
        field: &str,
        expected_results: i32,
    ) -> eyre::Result<()> {
        let lua = LuaState::current()?;
        lua.get_global(c"ComponentGetValue2");
        lua.push_integer(component.0);
        lua.push_string(field);
        lua.call(2, expected_results);
        Ok(())
    }

    pub(crate) fn component_get_value_number(
        component: ComponentID,
        field: &str,
    ) -> eyre::Result<f64> {
        component_get_value_base(component, field, 1)?;
        let lua = LuaState::current()?;
        let ret = lua.to_number(1);
        lua.pop_last();
        Ok(ret)
    }

    pub(crate) fn component_get_value_integer(
        component: ComponentID,
        field: &str,
    ) -> eyre::Result<i32> {
        component_get_value_base(component, field, 1)?;
        let lua = LuaState::current()?;
        let ret = lua.to_integer(1);
        lua.pop_last();
        Ok(ret as i32)
    }

    pub(crate) fn component_get_value_bool(
        component: ComponentID,
        field: &str,
    ) -> eyre::Result<bool> {
        component_get_value_base(component, field, 1)?;
        let lua = LuaState::current()?;
        let ret = lua.to_bool(1);
        lua.pop_last();
        Ok(ret)
    }
}
