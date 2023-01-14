use http::StatusCode;
use mlua::LuaSerdeExt;
use mlua::RegistryKey;
use mlua::Value;
use std::ops::Deref;
use tokio::sync::Mutex;
use tracing::error;

use axum::http;

use axum::response::IntoResponse;
use mlua::Function;
use mlua::Lua;

use crate::request::DayaxRequest;
use crate::response::DayaxResponse;

pub async fn request_handler(
    lua_mutex: &Mutex<Lua>,
    registry_key: impl AsRef<RegistryKey>,
    arguments: DayaxRequest,
) -> impl IntoResponse {
    let result = {
        // Keep the lock only for this fn call
        let lua_lock = lua_mutex.lock().await;
        exec_lua_registry_callback(lua_lock.deref(), registry_key.as_ref(), arguments)
    };

    result.map_err(|error| {
        error!("{error}");
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
    })
}

// Call is sync because !Send MutexGuard
fn exec_lua_registry_callback(
    lua: &Lua,
    registry_key: &RegistryKey,
    arguments: DayaxRequest,
) -> eyre::Result<DayaxResponse> {
    let callback: Function = lua.registry_value(registry_key)?;
    let arguments = lua.to_value(&arguments)?;
    let value: Value = callback.call(arguments)?;
    let value = serde_json::to_value(value)?;
    Ok(serde_json::from_value(value)?)
}
