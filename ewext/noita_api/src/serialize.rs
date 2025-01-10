use eyre::{Context, OptionExt};
use crate::EntityID;
use crate::lua::{LuaGetValue, LuaPutValue, LuaState};
pub fn serialize_entity(entity: EntityID) -> eyre::Result<Vec<u8>> {
    let lua = LuaState::current()?;
    lua.get_global(c"EwextSerialize");
    entity.put(lua);
    lua.call(1, 1i32)
        .wrap_err("Failed to call EwextSerialize")?;
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
    try_deserialize_entity(entity_data, x, y)?.ok_or_eyre("Failed to deserialize entity")
}