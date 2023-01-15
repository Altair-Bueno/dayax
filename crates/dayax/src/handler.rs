use http::StatusCode;
use mlua::LuaSerdeExt;
use mlua::RegistryKey;
use mlua::Value;
use std::sync::Arc;
use tracing::debug;
use tracing::error;

use axum::http;

use axum::response::IntoResponse;
use mlua::Function;

use crate::request::DayaxRequest;
use crate::response::DayaxResponse;
use crate::DayaxState;

#[derive(Debug, Clone)]
pub struct DayaxRequestHandler {
    callback_registry_key: Arc<RegistryKey>,
    state: DayaxState,
}

impl DayaxRequestHandler {
    pub fn new(callback_registry_key: Arc<RegistryKey>, state: DayaxState) -> Self {
        Self {
            callback_registry_key,
            state,
        }
    }
    pub async fn handle(self, request: DayaxRequest) -> impl IntoResponse {
        // Keep the lock only for lua code. Transform into response outside the
        // async block
        let result: Result<DayaxResponse, eyre::ErrReport> = async move {
            let lua = self.state.lock().await;
            let callback: Function = lua.registry_value(&self.callback_registry_key)?;
            let arguments = lua.to_value(&request)?;
            let value: Value = callback.call(arguments)?;
            let value = serde_json::to_value(value)?;
            Ok(serde_json::from_value(value)?)
        }
        .await;
        result.map_err(|error| {
            error!("{error}");
            debug!(?error);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        })
    }
}
