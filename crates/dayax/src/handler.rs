use mlua::LuaSerdeExt;
use mlua::RegistryKey;
use mlua::Value;
use std::ops::Deref;
use tokio::sync::Mutex;
use tracing::error;

use std::collections::HashMap;

use axum::async_trait;
use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::extract::Query;
use axum::http;
use axum::http::Request;
use http::Method;
use http::Uri;
use serde::Serialize;

use axum::response::IntoResponse;
use mlua::Function;
use mlua::Lua;

#[derive(Debug, Serialize)]
pub struct DayaxRequest {
    method: String,
    #[serde(rename = "searchParams")]
    search_params: HashMap<String, String>,
    uri: String,
    path: HashMap<String, String>,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for DayaxRequest
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = String;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let method = Method::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Query(search_params) = Query::from_request_parts(&mut parts, state)
            .await
            .map_err(|x| x.to_string())?;
        let uri = Uri::from_request_parts(&mut parts, state)
            .await
            .unwrap()
            .to_string();
        let Path(path) = Path::from_request_parts(&mut parts, state).await.unwrap();
        // TODO extract everything
        let _req = Request::from_parts(parts, body);
        Ok(DayaxRequest {
            method,
            search_params,
            uri,
            path,
        })
    }
}

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
