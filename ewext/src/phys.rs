use noita_api::{
    lua::{LuaGetValue, LuaState},
    Component, ComponentID, PhysicsBody2Component, PhysicsBodyComponent,
};

pub(crate) trait PhysComponent: Component {
    fn is_initialized(&self) -> eyre::Result<bool>;
}

impl PhysComponent for PhysicsBodyComponent {
    fn is_initialized(&self) -> eyre::Result<bool> {
        Ok(true)
    }
}
impl PhysComponent for PhysicsBody2Component {
    fn is_initialized(&self) -> eyre::Result<bool> {
        self.m_initialized()
    }
}

pub(crate) fn get_phys_transform(
    c: impl PhysComponent,
) -> eyre::Result<Option<(f32, f32, f32, f32, f32, f32)>> {
    if !c.is_initialized()? {
        return Ok(None);
    }
    let lua = LuaState::current()?;
    lua.get_global(c"EwextPhysBodyGetTransform");
    lua.push_integer(Into::<ComponentID>::into(c).0.into());
    lua.call(1, 6);
    if lua.is_nil_or_none(-1) {
        Ok(None)
    } else {
        match LuaGetValue::get(lua, -1) {
            Ok(ret) => {
                lua.pop_last_n(6);
                Ok(Some(ret))
            }
            Err(err) => {
                lua.pop_last_n(6);
                Err(err)
            }
        }
    }
}

pub(crate) fn set_phys_transform(
    c: impl PhysComponent,
    x: f32,
    y: f32,
    r: f32,
    vx: f32,
    vy: f32,
    vr: f32,
) -> eyre::Result<()> {
    if !c.is_initialized()? {
        return Ok(());
    }
    let lua = LuaState::current()?;
    lua.get_global(c"EwextPhysBodyGetTransform");
    lua.push_integer(Into::<ComponentID>::into(c).0.into());
    lua.push_number(x as f64);
    lua.push_number(y as f64);
    lua.push_number(r as f64);
    lua.push_number(vx as f64);
    lua.push_number(vy as f64);
    lua.push_number(vr as f64);
    lua.call(7, 0);
    Ok(())
}
