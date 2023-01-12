use mlua::Lua;
use mlua::Table;

const GLOBAL_DAYAX: &str = "dayax";

pub fn start() -> mlua::Result<Lua> {
    let lua = Lua::new();
    let dayax = lua.create_table()?;
    lua.globals().set(GLOBAL_DAYAX, dayax)?;
    Ok(lua)
}

pub fn get_dayax(lua: &Lua) -> mlua::Result<Table> {
    let globals = lua.globals();
    globals.get(GLOBAL_DAYAX)
}
