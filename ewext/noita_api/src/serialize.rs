use crate::lua::{LuaGetValue, LuaPutValue, LuaState};
use crate::{AbilityComponent, EntityID};
use eyre::{Context, OptionExt};
use std::backtrace::Backtrace;
pub fn serialize_entity(entity: EntityID) -> eyre::Result<Vec<u8>> {
    let lua = LuaState::current()?;
    lua.get_global(c"EwextSerialize");
    entity.put(lua);
    lua.call(1, 1i32).wrap_err(format!(
        "Failed to call EwextSerialize\n{}",
        Backtrace::force_capture()
    ))?;
    let res = lua.to_raw_string(-1);
    lua.pop_last_n(1i32);
    res
}

pub(crate) fn try_deserialize_entity(
    entity_data: &[u8],
    x: f32,
    y: f32,
) -> eyre::Result<Option<EntityID>> {
    let lua = LuaState::current()?;
    lua.get_global(c"EwextDeserialize");
    lua.push_raw_string(entity_data);
    x.put(lua);
    y.put(lua);
    lua.call(3, 1i32)
        .wrap_err("Failed to call EwextDeserialize")?;
    let res = LuaGetValue::get(lua, -1)?;
    lua.pop_last_n(1i32);
    Ok(res)
}

pub fn deserialize_entity(entity_data: &[u8], x: f32, y: f32) -> eyre::Result<EntityID> {
    let ent =
        try_deserialize_entity(entity_data, x, y)?.ok_or_eyre("Failed to deserialize entity")?;
    if let Some(ability) =
        ent.try_get_first_component_including_disabled::<AbilityComponent>(None)?
    {
        ability.set_m_next_frame_usable(0)?;
        ability.set_m_reload_next_frame_usable(0)?;
        ability.set_m_cast_delay_start_frame(0)?;
    }
    Ok(ent)
}
