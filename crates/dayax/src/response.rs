use std::collections::HashMap;

use axum::http::StatusCode;
use axum::response::AppendHeaders;
use axum::response::IntoResponse;
use axum::response::Redirect;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tracing::error;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DayaxResponse {
    Plain(String),
    Full(FullDayaxResponse),
}

impl IntoResponse for DayaxResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            DayaxResponse::Plain(text) => text.into_response(),
            DayaxResponse::Full(fdr) => fdr.into_response(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FullDayaxResponse {
    #[serde(alias = "statusCode")]
    status_code: Option<u16>,
    redirect: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<Value>,
}

impl IntoResponse for FullDayaxResponse {
    fn into_response(self) -> axum::response::Response {
        if let Some(redirect) = self.redirect {
            return Redirect::temporary(&redirect).into_response();
        }
        let headers = AppendHeaders(self.headers.unwrap_or_default());
        let status_code = StatusCode::from_u16(self.status_code.unwrap_or(200)).unwrap();

        let body = match self.body {
            None | Some(Value::Null) => Default::default(),
            Some(Value::Array(x)) => Json(x).into_response(),
            Some(Value::Object(x)) => Json(x).into_response(),
            Some(Value::String(x)) => x.into_response(),
            Some(body) => {
                error!(?body, "Invalid body type");
                (
                    http::status::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                )
                    .into_response()
            }
        };
        (status_code, headers, body).into_response()
    }
}
