use mlua::Lua;
use std::fmt::Display;

use crate::userdata::Dayax;

pub const GLOBAL_DAYAX: &str = "dayax";
pub const LUA_REGISTRY_PREFIX: &str = "DAYAX";
pub const KEY_SEPARATOR: &str = "+";

pub fn get_registry_request_key(method: impl Display, path: impl Display) -> String {
    format!(
        "{LUA_REGISTRY_PREFIX}{KEY_SEPARATOR}request{KEY_SEPARATOR}{method}{KEY_SEPARATOR}{path}"
    )
}

pub fn start() -> mlua::Result<Lua> {
    let lua = Lua::new();
    let dayax = Dayax::new();
    lua.globals().set(GLOBAL_DAYAX, dayax)?;
    Ok(lua)
}

pub fn get_dayax(lua: &Lua) -> mlua::Result<Dayax> {
    let globals = lua.globals();
    globals.get(GLOBAL_DAYAX)
}
