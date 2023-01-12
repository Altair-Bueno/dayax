use axum::response::Response;
use mlua::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;

use axum::extract::MatchedPath;
use axum::extract::State;
use axum::response::IntoResponse;
use mlua::Function;
use mlua::Lua;

pub async fn request_handler(
    State(lua): State<Arc<Mutex<Lua>>>,
    matched: MatchedPath,
) -> impl IntoResponse {
    let result = exec_lua_handler(lua.as_ref(), matched.as_str()).await;
    match result {
        Ok(x) => x,
        Err(error) => {
            error!(?error);
            (
                http::status::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
            )
                .into_response()
        }
    }
}

async fn exec_lua_handler<'mutex, 'path>(
    lua: &'mutex Mutex<Lua>,
    matched: &'path str,
) -> Result<Response, mlua::Error> {
    let llua = lua.lock().await;
    let router = crate::runtime::get_dayax(&*llua)?;
    let callback: Function = router.get(matched)?;
    let arguments = llua.create_table()?;
    let value: Value = callback.call(arguments)?;

    let response = match value {
        Value::Nil => Response::default(),
        Value::String(x) => x.to_str()?.to_owned().into_response(),
        all @ Value::Table(_) => axum::Json(all).into_response(),
        _ => return Err(mlua::Error::RuntimeError("Unsuported return type".into())),
    };
    Ok(response)
}
