mod request;

pub use request::HandlerRequest;

use mlua::LuaSerdeExt;
use mlua::RegistryKey;
use mlua::Value;
use serde::Serialize;
use std::ops::Deref;
use tokio::sync::Mutex;
use tracing::error;

use axum::response::IntoResponse;
use mlua::Function;
use mlua::Lua;

pub async fn request_handler(
    lua_mutex: &Mutex<Lua>,
    registry_key: impl AsRef<RegistryKey>,
    arguments: impl Serialize,
) -> impl IntoResponse {
    let result = {
        // Keep the lock only for this fn call
        let lua_lock = lua_mutex.lock().await;
        exec_lua_registry_callback(lua_lock.deref(), registry_key.as_ref(), arguments)
    };

    match result {
        Ok(x) => axum::Json(x).into_response(),
        Err(error) => {
            error!("{error}");
            (
                http::status::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response()
        }
    }
}

// Call is sync because !Send MutexGuard
fn exec_lua_registry_callback(
    lua: &Lua,
    registry_key: &RegistryKey,
    arguments: impl Serialize,
) -> eyre::Result<serde_json::Value> {
    let callback: Function = lua.registry_value(registry_key)?;
    let arguments = lua.to_value(&arguments)?;
    let value: Value = callback.call(arguments)?;
    let value = serde_json::to_value(value)?;
    Ok(value)
}
